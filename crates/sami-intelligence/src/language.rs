use crate::{
    util::{at, bounded, string},
    EngineError,
};
use regex::Regex;
use serde_json::{json, Value};
use std::collections::BTreeSet;

pub fn compose(plan: &Value, pack: &Value, previous: &[String]) -> Result<String, EngineError> {
    bounded(plan, 1_000_000)?;
    let action = plan
        .get("action")
        .and_then(Value::as_str)
        .ok_or_else(|| EngineError::new("invalid_plan", "Response plan requires action"))?;
    let entry = pack
        .get("responses")
        .and_then(|v| v.get(action))
        .ok_or_else(|| {
            EngineError::new(
                "unsupported_response",
                format!("No approved response grammar for {action}"),
            )
        })?;
    let templates: Vec<&str> = if let Some(s) = entry.as_str() {
        vec![s]
    } else {
        entry
            .as_array()
            .ok_or_else(|| {
                EngineError::new("invalid_pack", "Response entry must be string or array")
            })?
            .iter()
            .filter_map(Value::as_str)
            .collect()
    };
    if templates.is_empty() {
        return Err(EngineError::new("invalid_pack", "No response templates"));
    }
    let slots = plan
        .get("slots")
        .or_else(|| plan.get("fields"))
        .unwrap_or(&Value::Null);
    let placeholder = Regex::new(r"\{([a-zA-Z_][a-zA-Z0-9_.]*)\}").expect("fixed regex");
    let hidden: BTreeSet<&str> = plan
        .get("hidden_slots")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .collect();
    let mut candidates = Vec::new();
    for (index, template) in templates.into_iter().enumerate() {
        let mut missing = None;
        let result = placeholder
            .replace_all(template, |cap: &regex::Captures| {
                let key = &cap[1];
                if hidden.contains(key) {
                    missing = Some(format!("Restricted slot {key}"));
                    return String::new();
                }
                match at(slots, key) {
                    Some(value) => string(value),
                    None => {
                        missing = Some(format!("Missing required slot {key}"));
                        String::new()
                    }
                }
            })
            .into_owned();
        if missing.is_some() {
            continue;
        }
        let penalty = previous
            .iter()
            .rev()
            .take(20)
            .filter(|p| p.as_str() == result)
            .count();
        candidates.push((penalty, index, result));
    }
    candidates.sort_by_key(|(penalty, index, _)| (*penalty, *index));
    let Some((_, _, response)) = candidates.into_iter().next() else {
        return Err(EngineError::new(
            "missing_response_slot",
            "No template can be realized from authorized slots",
        ));
    };
    let limit = plan
        .get("max_characters")
        .and_then(Value::as_u64)
        .unwrap_or(2_000)
        .min(20_000) as usize;
    if response.chars().count() > limit {
        return Err(EngineError::new(
            "response_limit",
            "Approved response exceeds declared character budget",
        ));
    }
    Ok(response)
}

/// Finite sales simulation. Persona variables are explicitly simulated; hidden
/// facts are revealed by a configured predicate and at most one per response.
pub fn roleplay(message: &str, state: &Value, persona: &Value) -> Value {
    if message.len() > 65_536 {
        return json!({"status":"budget_exhausted","state":state,"response":"Please shorten the simulation input.","simulation":true});
    }
    let lower = message.to_lowercase();
    let mut next = state.clone();
    if !next.is_object() {
        next = json!({});
    }
    let mut revealed: BTreeSet<String> = state
        .get("revealed_pain_points")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect();
    let trust = state
        .get("simulated_trust")
        .or_else(|| persona.get("initial_trust"))
        .and_then(Value::as_f64)
        .unwrap_or(0.5)
        .clamp(0.0, 1.0);
    let positive = ["understand", "evidence", "payback", "maintenance", "budget"]
        .iter()
        .any(|p| lower.contains(p));
    let pressure = ["must buy", "guaranteed", "just sign"]
        .iter()
        .any(|p| lower.contains(p));
    let updated_trust = (trust + if positive { 0.05 } else { 0.0 }
        - if pressure { 0.1 } else { 0.0 })
    .clamp(0.0, 1.0);
    let mut newly_revealed = Vec::new();
    let postpone = ["tomorrow", "next week", "later", "send logs"]
        .iter()
        .any(|p| lower.contains(p));
    let response = if postpone
        && !persona
            .get("allow_postponement")
            .and_then(Value::as_bool)
            .unwrap_or(false)
    {
        "Let us work with the information available in this meeting. What can you establish now, and what remains uncertain?".to_string()
    } else {
        let generic = ["problem", "pain point", "challenge", "concern"]
            .iter()
            .any(|p| lower.contains(p));
        let eligible = persona
            .get("pain_points")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .find(|pain| {
                let id = pain.get("id").and_then(Value::as_str).unwrap_or("");
                let topics = pain
                    .get("reveal_topics")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                    .filter_map(Value::as_str)
                    .any(|p| lower.contains(&p.to_lowercase()));
                let threshold = pain
                    .get("minimum_trust")
                    .and_then(Value::as_f64)
                    .unwrap_or(0.0);
                !id.is_empty()
                    && !revealed.contains(id)
                    && updated_trust >= threshold
                    && (topics
                        || (generic
                            && pain
                                .get("allow_generic")
                                .and_then(Value::as_bool)
                                .unwrap_or(false)))
            });
        if let Some(pain) = eligible {
            let id = pain["id"].as_str().unwrap_or("");
            revealed.insert(id.into());
            newly_revealed.push(id.to_string());
            pain.get("text")
                .and_then(Value::as_str)
                .unwrap_or("Please ask a more specific question.")
                .to_string()
        } else if ["price", "expensive", "cost"]
            .iter()
            .any(|p| lower.contains(p))
        {
            "The investment matters to us. What assumptions and payback period support your proposal?".to_string()
        } else if pressure {
            "I need evidence and a clear account of the risks before making a commitment."
                .to_string()
        } else {
            "What specific operational issue would you like to understand, and how would your proposal address it?".to_string()
        }
    };
    next["revealed_pain_points"] = json!(revealed);
    next["simulated_trust"] = json!(updated_trust);
    next["simulation"] = json!(true);
    json!({"status":"simulated","response":response,"state":next,"revealed_this_turn":newly_revealed,"simulation":true,"probabilities":null,"real_customer_psychology":false})
}
