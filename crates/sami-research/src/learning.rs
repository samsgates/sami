use crate::{array, bounded_usize, invalid, number, text, Result};
use serde_json::{json, Value};
use std::collections::BTreeSet;

#[derive(Clone)]
struct Arm {
    id: String,
    successes: f64,
    failures: f64,
}
fn arms(input: &Value) -> Result<Vec<Arm>> {
    let values = array(input, "arms", 64)?;
    if values.is_empty() {
        return Err(invalid("at least one finite action is required"));
    }
    let mut ids = BTreeSet::new();
    let mut arms = Vec::new();
    for a in values {
        let id = text(a, "id")?;
        if id.is_empty() || id.len() > 128 || !ids.insert(id.to_owned()) {
            return Err(invalid(
                "action ids must be unique nonempty strings of at most 128 bytes",
            ));
        }
        if a.get("risk").and_then(Value::as_str) != Some("low") {
            return Err(invalid(
                "research bandits accept only explicitly low-risk actions",
            ));
        }
        let s = number(a, "successes")?;
        let f = number(a, "failures")?;
        if !(0.0..=1_000_000.0).contains(&s) || !(0.0..=1_000_000.0).contains(&f) {
            return Err(invalid(
                "successes and failures must be bounded nonnegative statistics",
            ));
        }
        arms.push(Arm {
            id: id.into(),
            successes: s,
            failures: f,
        });
    }
    Ok(arms)
}
// Reproducible research PRNG. It must never be used for secrets, signatures or approvals.
struct Rng(u64);
impl Rng {
    fn new(seed: u64) -> Self {
        Self(if seed == 0 { 0x9e3779b97f4a7c15 } else { seed })
    }
    fn uniform(&mut self) -> f64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        ((x >> 11) as f64 + 0.5) / (1u64 << 53) as f64
    }
    fn normal(&mut self) -> f64 {
        (-2.0 * self.uniform().ln()).sqrt() * (2.0 * std::f64::consts::PI * self.uniform()).cos()
    }
    fn gamma(&mut self, shape: f64) -> Result<f64> {
        if shape < 1.0 {
            return Ok(self.gamma(shape + 1.0)? * self.uniform().powf(1.0 / shape));
        }
        let d = shape - 1.0 / 3.0;
        let c = 1.0 / (9.0 * d).sqrt();
        for _ in 0..1000 {
            let x = self.normal();
            let t = 1.0 + c * x;
            if t <= 0.0 {
                continue;
            }
            let v = t.powi(3);
            let u = self.uniform();
            if u < 1.0 - 0.0331 * x.powi(4) || u.ln() < 0.5 * x * x + d * (1.0 - v + v.ln()) {
                return Ok(d * v);
            }
        }
        Err(invalid("bounded gamma sampler exhausted"))
    }
    fn beta(&mut self, a: f64, b: f64) -> Result<f64> {
        let x = self.gamma(a)?;
        let y = self.gamma(b)?;
        Ok(x / (x + y))
    }
}
fn seed(input: &Value) -> Result<u64> {
    input
        .get("seed")
        .and_then(Value::as_u64)
        .ok_or_else(|| invalid("explicit integer seed required for reproducible research"))
}
fn best(values: &[f64]) -> usize {
    let mut i = 0;
    for j in 1..values.len() {
        if values[j] > values[i] {
            i = j;
        }
    }
    i
}
pub fn select(input: &Value) -> Result<Value> {
    let arms = arms(input)?;
    let context = text(input, "context_id")?;
    let policy = text(input, "policy_version")?;
    if context.is_empty() || context.len() > 128 || policy.is_empty() {
        return Err(invalid("context_id and policy_version required"));
    }
    let method = input.get("method").and_then(Value::as_str).unwrap_or("ucb");
    let mut rng = Rng::new(seed(input)?);
    let means = arms
        .iter()
        .map(|a| (a.successes + 1.0) / (a.successes + a.failures + 2.0))
        .collect::<Vec<_>>();
    let (chosen, propensity, estimator, statistics) = match method {
        "ucb" => {
            let total = arms.iter().map(|a| a.successes + a.failures).sum::<f64>() + 1.0;
            let scores = arms
                .iter()
                .zip(&means)
                .map(|(a, &m)| {
                    if a.successes + a.failures == 0.0 {
                        f64::MAX
                    } else {
                        m + (2.0 * total.ln() / (a.successes + a.failures)).sqrt()
                    }
                })
                .collect::<Vec<_>>();
            (best(&scores), 1.0, "exact_deterministic", scores)
        }
        "epsilon_greedy" => {
            let e = input.get("epsilon").and_then(Value::as_f64).unwrap_or(0.1);
            if !(0.0..=1.0).contains(&e) {
                return Err(invalid("epsilon must be in [0,1]"));
            }
            let greedy = best(&means);
            let chosen = if rng.uniform() < e {
                (rng.uniform() * arms.len() as f64) as usize
            } else {
                greedy
            };
            let propensity = e / arms.len() as f64 + if chosen == greedy { 1.0 - e } else { 0.0 };
            (chosen, propensity, "exact_epsilon_greedy", means.clone())
        }
        "thompson" => {
            let mc = bounded_usize(input, "propensity_samples", 512, 4096)?;
            let mut counts = vec![0usize; arms.len()];
            for _ in 0..mc {
                let draws = arms
                    .iter()
                    .map(|a| rng.beta(a.successes + 1.0, a.failures + 1.0))
                    .collect::<Result<Vec<_>>>()?;
                counts[best(&draws)] += 1;
            }
            let draws = arms
                .iter()
                .map(|a| rng.beta(a.successes + 1.0, a.failures + 1.0))
                .collect::<Result<Vec<_>>>()?;
            let i = best(&draws);
            (
                i,
                (counts[i] as f64 + 0.5) / (mc as f64 + 0.5 * arms.len() as f64),
                "monte_carlo_smoothed_estimate_not_exact",
                draws,
            )
        }
        _ => return Err(invalid("method must be ucb, thompson or epsilon_greedy")),
    };
    let scores = arms
        .iter()
        .zip(statistics)
        .map(|(a, s)| json!({"id":a.id,"score":s}))
        .collect::<Vec<_>>();
    Ok(
        json!({"action_id":arms[chosen].id,"context_id":context,"policy_version":policy,"method":method,"seed":input["seed"],"propensity":propensity,"propensity_estimator":estimator,"scores":scores,"risk":"low","execution_authorized":false,"context_model":"caller-supplied discrete context stratum; no cross-context generalization","off_policy_evaluation_warning":"deterministic UCB has no support for unchosen actions; approximate Thompson propensities do not establish unbiased IPS"}),
    )
}
pub fn update(input: &Value) -> Result<Value> {
    let arms = arms(input)?;
    let action = text(input, "action_id")?;
    let context = text(input, "context_id")?;
    let event = text(input, "outcome_id")?;
    if input.get("verified_outcome").and_then(Value::as_bool) != Some(true)
        || input.get("consent").and_then(Value::as_bool) != Some(true)
    {
        return Err(invalid("verified outcome and consent required"));
    }
    if input.get("decision_policy_version") != input.get("current_policy_version")
        || input
            .get("current_policy_version")
            .and_then(Value::as_str)
            .is_none()
    {
        return Err(invalid("outcome policy epoch differs from current policy"));
    }
    let reward = number(input, "reward")?;
    if !(0.0..=1.0).contains(&reward) {
        return Err(invalid("reward must be in [0,1]"));
    }
    let mut ids = array(input, "seen_outcome_ids", 10_000)?
        .iter()
        .map(|v| {
            v.as_str()
                .map(str::to_owned)
                .ok_or_else(|| invalid("seen_outcome_ids must be strings"))
        })
        .collect::<Result<Vec<_>>>()?;
    if ids.iter().any(|id| id == event) {
        return Err(invalid("duplicate outcome rejected"));
    }
    if event.is_empty() {
        return Err(invalid("outcome_id required"));
    }
    if !arms.iter().any(|a| a.id == action) {
        return Err(invalid("unknown selected action"));
    }
    let updated=arms.iter().map(|a|json!({"id":a.id,"risk":"low","successes":a.successes+if a.id==action{reward}else{0.0},"failures":a.failures+if a.id==action{1.0-reward}else{0.0}})).collect::<Vec<_>>();
    ids.push(event.into());
    Ok(
        json!({"arms":updated,"context_id":context,"seen_outcome_ids":ids,"status":"quarantined_candidate","production_promoted":false,"reward_model":"fractional Bernoulli pseudo-count update; posterior interpretation requires suitable reward assumptions"}),
    )
}
pub fn feedback(input: &Value) -> Result<Value> {
    let votes = array(input, "events", 1000)?;
    let required = bounded_usize(input, "min_independent_actors", 3, 100)?;
    let mut actors = BTreeSet::new();
    let mut ids = BTreeSet::new();
    let mut families = BTreeSet::new();
    let mut reasons = Vec::new();
    let policy = text(input, "policy_version")?;
    let mut positive = 0;
    let mut negative = 0;
    for event in votes {
        let id = text(event, "id")?;
        let actor = text(event, "actor_id")?;
        let family = text(event, "source_family")?;
        if !ids.insert(id.to_owned()) {
            reasons.push(format!("duplicate_event:{id}"));
            continue;
        }
        if event.get("authorized").and_then(Value::as_bool) != Some(true)
            || event.get("consent").and_then(Value::as_bool) != Some(true)
        {
            reasons.push(format!("untrusted_event:{id}"));
            continue;
        }
        if event.get("policy_version").and_then(Value::as_str) != Some(policy) {
            reasons.push(format!("stale_policy:{id}"));
            continue;
        }
        if actor.is_empty() || family.is_empty() {
            return Err(invalid("actor_id and source_family cannot be empty"));
        }
        if !actors.insert(actor.to_owned()) {
            reasons.push(format!("duplicate_actor:{actor}"));
            continue;
        }
        families.insert(family.to_owned());
        let reward = number(event, "reward")?;
        if !(0.0..=1.0).contains(&reward) {
            return Err(invalid("event reward outside [0,1]"));
        }
        if reward >= 0.5 {
            positive += 1;
        } else {
            negative += 1;
        }
    }
    if actors.len() < required {
        reasons.push("insufficient_independent_actors".into());
    }
    if families.len() < 2 {
        reasons.push("insufficient_source_families".into());
    }
    if negative > 0 {
        reasons.push("conflicting_feedback_requires_review".into());
    }
    let permitted = input
        .get("candidate")
        .and_then(|v| v.get("kind"))
        .and_then(Value::as_str)
        .is_some_and(|k| {
            matches!(
                k,
                "response_strategy" | "retrieval_weight" | "fact_correction"
            )
        });
    if !permitted {
        reasons.push("candidate_kind_cannot_modify_authority_or_policy".into());
    }
    Ok(
        json!({"status":"quarantined","eligible_for_review":reasons.is_empty()&&positive>=required,"independent_actors":actors.len(),"source_families":families.len(),"positive":positive,"negative":negative,"reasons":reasons,"production_promoted":false,"authority_input":"actor/source identity and consent must be supplied by an authenticated upstream; body assertions alone are not authentication"}),
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> Value {
        json!({"arms":[{"id":"clarify","risk":"low","successes":20,"failures":1},{"id":"summarize","risk":"low","successes":1,"failures":20}],"context_id":"billing","policy_version":"1","seed":42})
    }
    #[test]
    fn bandits_reproducible_and_propensity_is_recorded() {
        let mut v = fixture();
        v["method"] = json!("thompson");
        assert_eq!(select(&v).expect("a"), select(&v).expect("b"));
        let p = select(&v).expect("selected")["propensity"]
            .as_f64()
            .expect("p");
        assert!(p > 0.0 && p <= 1.0);
    }
    #[test]
    fn expensive_actions_are_rejected() {
        let mut v = fixture();
        v["arms"][0]["risk"] = json!("high");
        assert!(select(&v).is_err());
    }
    #[test]
    fn exact_epsilon_greedy_distribution() {
        let mut v = fixture();
        v["method"] = json!("epsilon_greedy");
        v["epsilon"] = json!(0.0);
        let r = select(&v).expect("select");
        assert_eq!(r["action_id"], "clarify");
        assert_eq!(r["propensity"], 1.0);
    }
    #[test]
    fn duplicate_rewards_and_stale_policies_are_rejected() {
        let mut v = fixture();
        v["action_id"] = json!("clarify");
        v["outcome_id"] = json!("e1");
        v["seen_outcome_ids"] = json!(["e1"]);
        v["reward"] = json!(1);
        v["consent"] = json!(true);
        v["verified_outcome"] = json!(true);
        v["decision_policy_version"] = json!("1");
        v["current_policy_version"] = json!("1");
        assert!(update(&v).is_err());
        v["seen_outcome_ids"] = json!([]);
        assert_eq!(update(&v).expect("update")["production_promoted"], false);
        v["current_policy_version"] = json!("2");
        assert!(update(&v).is_err());
    }
}
