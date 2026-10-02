//! Explicit assumption-based causal estimation for a bounded reviewed dataset.
use crate::{array, invalid, text, Result};
use serde_json::{json, Value};
use std::collections::BTreeMap;
pub fn analyze(input: &Value) -> Result<Value> {
    let design = text(input, "design")?;
    if !["randomized", "backdoor_adjustment"].contains(&design) {
        return Ok(
            json!({"status":"not_identifiable","reason":"unsupported_identification_design","external_experiments":false}),
        );
    }
    let treatment = text(input, "treatment")?;
    let outcome = text(input, "outcome")?;
    let adjustment: Vec<String> = serde_json::from_value(
        input
            .get("adjustment")
            .cloned()
            .unwrap_or_else(|| json!([])),
    )
    .map_err(|_| invalid("adjustment must list field names"))?;
    if adjustment.len() > 5 {
        return Err(invalid("at most five declared adjustment variables"));
    }
    for name in ["consistency", "exchangeability", "positivity"] {
        if input["assumptions"][name] != true {
            return Ok(
                json!({"status":"not_identifiable","reason":format!("assumption_not_declared:{name}"),"external_experiments":false}),
            );
        }
    }
    let rows = array(input, "rows", 1000)?;
    if rows.len() < 10 {
        return Ok(
            json!({"status":"not_identifiable","reason":"insufficient_sample","external_experiments":false}),
        );
    }
    let mut groups: BTreeMap<String, [Vec<f64>; 2]> = BTreeMap::new();
    for row in rows {
        let t = row[treatment]
            .as_bool()
            .ok_or_else(|| invalid("binary boolean treatment required"))? as usize;
        let y = row[outcome]
            .as_f64()
            .filter(|v| v.is_finite())
            .ok_or_else(|| invalid("finite numeric outcome required"))?;
        let key: Vec<_> = adjustment
            .iter()
            .map(|k| {
                row.get(k)
                    .cloned()
                    .ok_or_else(|| invalid("adjustment value missing"))
            })
            .collect::<Result<_>>()?;
        let key = serde_json::to_string(&key).map_err(|_| invalid("invalid stratum"))?;
        groups.entry(key).or_default()[t].push(y);
    }
    if groups.len() > 100 {
        return Err(invalid("stratum capacity exceeded"));
    }
    let mut ate = 0.0;
    let mut variance = 0.0;
    let mut strata = Vec::new();
    for (key, arms) in &groups {
        if arms.iter().any(|a| a.len() < 2) {
            return Ok(
                json!({"status":"not_identifiable","reason":"insufficient_overlap_within_stratum","stratum":key,"external_experiments":false}),
            );
        }
        let means: [f64; 2] =
            std::array::from_fn(|i| arms[i].iter().sum::<f64>() / arms[i].len() as f64);
        let weight = (arms[0].len() + arms[1].len()) as f64 / rows.len() as f64;
        let delta = means[1] - means[0];
        ate += weight * delta;
        for i in 0..2 {
            let n = arms[i].len() as f64;
            let v = arms[i].iter().map(|y| (y - means[i]).powi(2)).sum::<f64>() / (n - 1.0);
            variance += weight.powi(2) * v / n;
        }
        strata.push(json!({"stratum":key,"n_control":arms[0].len(),"n_treated":arms[1].len(),"weight":weight,"difference":delta}));
    }
    let confounding_bias = input["sensitivity_bias"].as_f64().unwrap_or(0.0);
    if !confounding_bias.is_finite() || confounding_bias < 0.0 {
        return Err(invalid("sensitivity_bias must be finite and nonnegative"));
    }
    if !ate.is_finite() || !variance.is_finite() {
        return Err(invalid("numeric scale exceeds estimator capacity"));
    }
    let half = 1.96 * variance.sqrt();
    Ok(
        json!({"status":"assumption_based_estimate","estimand":"average_treatment_effect_in_declared_sample_population","design":design,"effect":ate,"normal_approximation_interval_95":[ate-half,ate+half],"sensitivity_interval":[ate-confounding_bias,ate+confounding_bias],"strata":strata,"assumptions_verified":false,"unmeasured_confounding_excluded":false,"intervention_authorized":false,"external_experiments":false,"limits":"No arbitrary graph identification; no causal truth inferred from text frequency; small/clustering/non-normal samples require specialist review."}),
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn overlap_and_assumptions_are_explicit() {
        let rows: Vec<_> = (0..12)
            .map(|i| json!({"T":i%2==1,"Y":if i%2==1{7}else{2},"Z":"same"}))
            .collect();
        let mut input = json!({"design":"backdoor_adjustment","treatment":"T","outcome":"Y","adjustment":["Z"],"assumptions":{"consistency":true,"exchangeability":true,"positivity":true},"rows":rows});
        let out = analyze(&input).unwrap();
        assert_eq!(out["effect"], 5.0);
        assert_eq!(out["intervention_authorized"], false);
        input["assumptions"]["exchangeability"] = json!(false);
        assert_eq!(analyze(&input).unwrap()["status"], "not_identifiable");
    }
}
