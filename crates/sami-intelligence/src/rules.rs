use crate::{
    util::{at, bounded, number},
    EngineError,
};
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Truth {
    True,
    False,
    Unknown,
}
impl Truth {
    fn not(self) -> Self {
        match self {
            Self::True => Self::False,
            Self::False => Self::True,
            Self::Unknown => Self::Unknown,
        }
    }
}

pub(crate) fn condition(
    condition: &Value,
    fields: &Value,
    remaining: &mut usize,
) -> Result<Truth, EngineError> {
    if *remaining == 0 {
        return Err(EngineError::new(
            "budget_exhausted",
            "Rule condition budget exhausted",
        ));
    }
    *remaining -= 1;
    if let Some(b) = condition.as_bool() {
        return Ok(if b { Truth::True } else { Truth::False });
    }
    let object = condition
        .as_object()
        .ok_or_else(|| EngineError::new("invalid_rule", "Conditions must be boolean or objects"))?;
    if let Some(v) = object.get("all").or_else(|| object.get("any")) {
        let list = v
            .as_array()
            .ok_or_else(|| EngineError::new("invalid_rule", "all/any expects an array"))?;
        let all = object.contains_key("all");
        let mut unknown = false;
        for clause in list {
            match condition_fn(clause, fields, remaining)? {
                Truth::False if all => return Ok(Truth::False),
                Truth::True if !all => return Ok(Truth::True),
                Truth::Unknown => unknown = true,
                _ => {}
            }
        }
        return Ok(if unknown {
            Truth::Unknown
        } else if all {
            Truth::True
        } else {
            Truth::False
        });
    }
    if let Some(v) = object.get("not") {
        return Ok(condition_fn(v, fields, remaining)?.not());
    }
    if let Some(v) = object.get("always").and_then(Value::as_bool) {
        return Ok(if v { Truth::True } else { Truth::False });
    }
    let path = object
        .get("field")
        .and_then(Value::as_str)
        .ok_or_else(|| EngineError::new("invalid_rule", "Predicate requires a field path"))?;
    let op = object.get("op").and_then(Value::as_str).unwrap_or("eq");
    let found = at(fields, path);
    if op == "exists" {
        return Ok(if found.is_some() {
            Truth::True
        } else {
            Truth::False
        });
    }
    let Some(found) = found else {
        return Ok(Truth::Unknown);
    };
    let expected = object.get("value").unwrap_or(&Value::Null);
    let result = match op {
        "eq" => found == expected,
        "ne" => found != expected,
        "gt" | "ge" | "gte" | "lt" | "le" | "lte" => {
            let actual_unit = found.get("unit").filter(|v| !v.is_null());
            let expected_unit = expected
                .get("unit")
                .or_else(|| object.get("unit"))
                .filter(|v| !v.is_null());
            if actual_unit != expected_unit {
                return Ok(Truth::Unknown);
            }
            let (Some(a), Some(b)) = (number(found), number(expected)) else {
                return Ok(Truth::Unknown);
            };
            match op {
                "gt" => a > b,
                "ge" | "gte" => a >= b,
                "lt" => a < b,
                _ => a <= b,
            }
        }
        "in" => expected
            .as_array()
            .ok_or_else(|| EngineError::new("invalid_rule", "in expects array value"))?
            .contains(found),
        "contains" => {
            if let (Some(a), Some(b)) = (found.as_str(), expected.as_str()) {
                a.to_lowercase().contains(&b.to_lowercase())
            } else if let Some(a) = found.as_array() {
                a.contains(expected)
            } else {
                return Ok(Truth::Unknown);
            }
        }
        "starts_with" => match (found.as_str(), expected.as_str()) {
            (Some(a), Some(b)) => a.starts_with(b),
            _ => return Ok(Truth::Unknown),
        },
        _ => {
            return Err(EngineError::new(
                "unsupported_operator",
                format!("Unsupported rule operator {op}"),
            ))
        }
    };
    Ok(if result { Truth::True } else { Truth::False })
}
fn condition_fn(v: &Value, fields: &Value, remaining: &mut usize) -> Result<Truth, EngineError> {
    condition(v, fields, remaining)
}

/// Deny overrides every allow. Equal-priority incompatible state updates conflict;
/// higher-priority updates supersede lower-priority preferences.
pub fn evaluate_rules(rules: &Value, fields: &Value, budget: usize) -> Result<Value, EngineError> {
    bounded(rules, 2_000_000)?;
    bounded(fields, 1_000_000)?;
    let rules = rules
        .as_array()
        .ok_or_else(|| EngineError::new("invalid_rules", "rules must be an array"))?;
    if rules.len() > 5_000 || budget > 100_000 {
        return Err(EngineError::new(
            "resource_limit",
            "Rule count/budget exceeds limit",
        ));
    }
    let mut remaining = budget;
    let mut denied = false;
    let mut allowed = false;
    let mut review = false;
    let mut matched = Vec::new();
    let mut unknown = Vec::new();
    let mut writes = BTreeMap::<String, (i64, Value)>::new();
    let mut conflicts = Vec::new();
    for rule in rules {
        let id = rule
            .get("id")
            .and_then(Value::as_str)
            .ok_or_else(|| EngineError::new("invalid_rule", "Rule must have an id"))?;
        let when = rule.get("when").unwrap_or(&Value::Bool(true));
        match condition(when, fields, &mut remaining)? {
            Truth::False => continue,
            Truth::Unknown => {
                unknown.push(id.to_string());
                continue;
            }
            Truth::True => {}
        }
        let effect = rule.get("effect").and_then(Value::as_str).unwrap_or("set");
        let priority = rule.get("priority").and_then(Value::as_i64).unwrap_or(0);
        match effect {
            "deny" => denied = true,
            "allow" => allowed = true,
            "require_review" => review = true,
            "set" => {}
            _ => {
                return Err(EngineError::new(
                    "invalid_rule",
                    "Rule effect must be deny, allow, set or require_review",
                ))
            }
        }
        if let Some(updates) = rule.get("set").and_then(Value::as_object) {
            for (key, value) in updates {
                if key.starts_with("permission")
                    || key.starts_with("tenant")
                    || key.starts_with("tool")
                {
                    return Err(EngineError::new(
                        "forbidden_rule_update",
                        "Decision rules cannot update authority",
                    ));
                }
                match writes.get(key) {
                    Some((old_priority, old)) if *old_priority == priority && old != value => {
                        conflicts
                            .push(json!({"field":key,"priority":priority,"values":[old,value]}))
                    }
                    Some((old_priority, _)) if *old_priority > priority => {}
                    _ => {
                        writes.insert(key.clone(), (priority, value.clone()));
                    }
                }
            }
        }
        matched.push(json!({"id":id,"effect":effect,"priority":priority}));
    }
    conflicts.retain(|conflict| {
        conflict
            .get("field")
            .and_then(Value::as_str)
            .and_then(|key| writes.get(key))
            .is_some_and(|(priority, _)| {
                conflict.get("priority").and_then(Value::as_i64) == Some(*priority)
            })
    });
    let updates: Map<_, _> = writes.into_iter().map(|(key, (_, v))| (key, v)).collect();
    let status = if denied {
        "policy_denied"
    } else if !conflicts.is_empty() {
        "policy_conflict"
    } else if review {
        "needs_human"
    } else {
        "resolved"
    };
    Ok(
        json!({"status":status,"denied":denied,"allowed":allowed && !denied && conflicts.is_empty() && !review,"requires_review":review,"updates":updates,"matched":matched,"unknown_conditions":unknown,"conflicts":conflicts,"conditions_evaluated":budget-remaining,"probabilities":null}),
    )
}
