use crate::{
    rules::{condition, Truth},
    util::{bounded, in_interval},
    EngineError,
};
use serde_json::{json, Value};
use std::collections::{BTreeSet, VecDeque};

fn set_field(state: &mut Value, path: &str, value: Value) -> Result<(), EngineError> {
    let mut cursor = state;
    let mut parts = path.split('.').peekable();
    while let Some(part) = parts.next() {
        if part.is_empty() {
            return Err(EngineError::new("invalid_action", "Empty effect path"));
        }
        if !cursor.is_object() {
            return Err(EngineError::new(
                "invalid_action",
                "Effect path crosses a non-object",
            ));
        }
        if parts.peek().is_none() {
            cursor
                .as_object_mut()
                .expect("checked")
                .insert(part.into(), value);
            return Ok(());
        }
        cursor = cursor
            .as_object_mut()
            .expect("checked")
            .entry(part.to_string())
            .or_insert(json!({}));
    }
    Ok(())
}

/// Bounded uniform-cost search over registered typed effects. Plans are proposals,
/// never authorization or execution. `actions` is `{actions:[...], goals:{...}}`.
pub fn plan(
    goal: &str,
    state: &Value,
    actions: &Value,
    max_nodes: usize,
) -> Result<Value, EngineError> {
    bounded(state, 1_000_000)?;
    bounded(actions, 2_000_000)?;
    if max_nodes == 0 || max_nodes > 10_000 {
        return Err(EngineError::new(
            "invalid_budget",
            "Planner max_nodes must be 1..10000",
        ));
    }
    let list = actions
        .get("actions")
        .unwrap_or(actions)
        .as_array()
        .ok_or_else(|| {
            EngineError::new(
                "invalid_actions",
                "actions must be an array or registry object",
            )
        })?;
    if list.len() > 500 {
        return Err(EngineError::new(
            "resource_limit",
            "Too many planner actions",
        ));
    }
    let goal_condition = actions
        .get("goals")
        .and_then(|g| g.get(goal))
        .cloned()
        .unwrap_or_else(|| json!({"field":goal,"op":"eq","value":true}));
    let mut frontier = vec![(0.0f64, state.clone(), Vec::<Value>::new())];
    let mut visited = BTreeSet::new();
    let mut expanded = 0;
    while !frontier.is_empty() {
        frontier.sort_by(|a, b| b.0.total_cmp(&a.0));
        let (cost, current, path) = frontier.pop().expect("nonempty");
        let key = serde_json::to_string(&current)
            .map_err(|e| EngineError::new("invalid_state", e.to_string()))?;
        if !visited.insert(key) {
            continue;
        }
        if expanded >= max_nodes {
            return Ok(
                json!({"status":"budget_exhausted","goal":goal,"expanded":expanded,"plan":[],"authorized_effects":[]}),
            );
        }
        expanded += 1;
        let mut condition_budget = 10_000;
        if condition(&goal_condition, &current, &mut condition_budget)? == Truth::True {
            return Ok(
                json!({"status":"planned","goal":goal,"steps":path,"cost":cost,"predicted_state":current,"expanded":expanded,"authorized_effects":[],"semantics":"registered_process_simulation"}),
            );
        }
        for action in list {
            let id = action
                .get("id")
                .and_then(Value::as_str)
                .ok_or_else(|| EngineError::new("invalid_action", "Action requires id"))?;
            if condition(
                action.get("preconditions").unwrap_or(&Value::Bool(true)),
                &current,
                &mut condition_budget,
            )? != Truth::True
            {
                continue;
            }
            let effects = action
                .get("effects")
                .and_then(Value::as_object)
                .ok_or_else(|| {
                    EngineError::new("invalid_action", "Action requires effects object")
                })?;
            let step_cost = action.get("cost").and_then(Value::as_f64).unwrap_or(1.0);
            if !step_cost.is_finite() || step_cost < 0.0 {
                return Err(EngineError::new(
                    "invalid_action",
                    "Action cost must be finite and nonnegative",
                ));
            }
            let mut next = current.clone();
            for (key, value) in effects {
                if key.starts_with("permission")
                    || key.starts_with("tenant")
                    || key.starts_with("policy")
                    || key.starts_with("tool")
                {
                    return Err(EngineError::new(
                        "forbidden_effect",
                        "Planner actions cannot create authority",
                    ));
                }
                set_field(&mut next, key, value.clone())?;
            }
            if next == current {
                continue;
            }
            let mut next_path = path.clone();
            next_path.push(json!({"id":id,"tool":action.get("tool"),"effects":effects,"cost":step_cost,"requires_commit_authorization":true}));
            if frontier.len() >= max_nodes.saturating_mul(5) {
                return Ok(
                    json!({"status":"budget_exhausted","goal":goal,"expanded":expanded,"plan":[],"reason":"frontier_budget","authorized_effects":[]}),
                );
            }
            let next_cost = cost + step_cost;
            if !next_cost.is_finite() {
                return Err(EngineError::new("invalid_action", "Plan cost overflow"));
            }
            frontier.push((next_cost, next, next_path));
        }
    }
    Ok(
        json!({"status":"no_plan","goal":goal,"expanded":expanded,"steps":[],"authorized_effects":[]}),
    )
}

/// Inspect bounded paths over caller-authorized approved relation claims.
pub fn graph_query(query: &Value, claims: &[Value], max_hops: usize) -> Result<Value, EngineError> {
    bounded(query, 100_000)?;
    if max_hops > 10 || claims.len() > 100_000 {
        return Err(EngineError::new("resource_limit", "Graph budget exceeded"));
    }
    let start = query
        .get("start")
        .and_then(Value::as_str)
        .ok_or_else(|| EngineError::new("invalid_query", "Graph query requires start"))?;
    let predicate = query.get("predicate").and_then(Value::as_str);
    let target = query.get("target").and_then(Value::as_str);
    let inverse = query
        .get("inverse")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let as_of = query.get("as_of").and_then(Value::as_str);
    let mut queue = VecDeque::from([(start.to_owned(), Vec::<Value>::new())]);
    let mut seen = BTreeSet::from([start.to_owned()]);
    let mut paths = Vec::new();
    while let Some((node, path)) = queue.pop_front() {
        if path.len() >= max_hops {
            continue;
        }
        for claim in claims {
            if claim.get("status").and_then(Value::as_str) != Some("approved") {
                continue;
            }
            if let Some(now) = as_of {
                if !in_interval(claim, "valid_from", "valid_to", now) {
                    continue;
                }
            } else if claim.get("valid_from").is_some_and(|v| !v.is_null())
                || claim.get("valid_to").is_some_and(|v| !v.is_null())
            {
                continue;
            }
            if predicate.is_some_and(|p| claim.get("predicate").and_then(Value::as_str) != Some(p))
            {
                continue;
            }
            let (Some(subject), Some(object)) = (
                claim.get("subject").and_then(Value::as_str),
                claim.get("value").and_then(Value::as_str),
            ) else {
                continue;
            };
            let destination = if subject == node {
                Some(object)
            } else if inverse && object == node {
                Some(subject)
            } else {
                None
            };
            let Some(destination) = destination else {
                continue;
            };
            let mut next = path.clone();
            next.push(json!({"claim_id":claim.get("id"),"from":node,"predicate":claim.get("predicate"),"to":destination,"source_id":claim.get("source_id"),"source_revision":claim.get("source_revision")}));
            if target.is_none() || target == Some(destination) {
                paths.push(json!({"node":destination,"hops":next.len(),"steps":next}));
            }
            if seen.insert(destination.into()) {
                queue.push_back((destination.into(), next));
            }
            if seen.len() > 10_000 || paths.len() > 10_000 {
                return Ok(
                    json!({"status":"budget_exhausted","paths":paths,"proof_of_external_truth":false}),
                );
            }
        }
    }
    Ok(
        json!({"status":"resolved","paths":paths,"visited":seen.len(),"proof_of_external_truth":false,"authorization":"caller_prefilter_required","temporal_filter":as_of}),
    )
}
