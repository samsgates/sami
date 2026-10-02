use crate::{
    compose, evaluate_rules, parse,
    rules::{condition, Truth},
    util::{at, bounded, date, in_interval, number, timestamp},
    EngineError,
};
use chrono::Months;
use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, BTreeSet};

fn accepted_claims<'a>(input: &Value, claims: &'a [Value]) -> (Vec<&'a Value>, Vec<Value>) {
    let now = input.get("as_of").and_then(Value::as_str).unwrap_or("");
    let known = input.get("known_at").and_then(Value::as_str).unwrap_or(now);
    let sources: Vec<&Value> = input
        .get("sources")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .collect();
    let mut accepted = Vec::new();
    let mut excluded = Vec::new();
    for claim in claims {
        let id = claim
            .get("id")
            .and_then(Value::as_str)
            .unwrap_or("missing_id");
        let reason = if claim.get("status").and_then(Value::as_str) != Some("approved") {
            Some("claim_not_approved")
        } else if !in_interval(claim, "valid_from", "valid_to", now) {
            Some("claim_not_applicable")
        } else if !in_interval(claim, "system_from", "system_to", known) {
            Some("claim_not_known_at_snapshot")
        } else {
            let source = sources
                .iter()
                .find(|s| s.get("id") == claim.get("source_id"));
            match source {
                None => Some("source_verification_unavailable"),
                Some(source)
                    if !matches!(
                        source.get("status").and_then(Value::as_str),
                        Some("approved" | "published")
                    ) =>
                {
                    Some("source_not_approved")
                }
                Some(source) if source.get("revision") != claim.get("source_revision") => {
                    Some("source_revision_mismatch")
                }
                Some(source) if !in_interval(source, "valid_from", "valid_to", now) => {
                    Some("source_not_applicable")
                }
                Some(source) => {
                    if let Some(max_age) = source.get("max_age_seconds").and_then(Value::as_i64) {
                        match (
                            timestamp(known),
                            source
                                .get("permission_checked_at")
                                .and_then(Value::as_str)
                                .and_then(timestamp),
                        ) {
                            (Some(t), Some(checked))
                                if max_age >= 0
                                    && t.signed_duration_since(checked).num_seconds() >= 0
                                    && t.signed_duration_since(checked).num_seconds()
                                        <= max_age =>
                            {
                                None
                            }
                            _ => Some("source_authority_stale"),
                        }
                    } else {
                        None
                    }
                }
            }
        };
        if let Some(reason) = reason {
            excluded.push(json!({"id":id,"reason":reason}));
        } else {
            accepted.push(claim);
        }
    }
    // A cyclic or missing derivation cannot masquerade as verified support.
    let index: BTreeMap<&str, &Value> = accepted
        .iter()
        .filter_map(|c| Some((c.get("id")?.as_str()?, *c)))
        .collect();
    fn supported(
        id: &str,
        index: &BTreeMap<&str, &Value>,
        visiting: &mut BTreeSet<String>,
        remaining: &mut usize,
    ) -> bool {
        if *remaining == 0 || !visiting.insert(id.to_owned()) {
            return false;
        }
        *remaining -= 1;
        let Some(c) = index.get(id) else {
            return false;
        };
        let result = c
            .get("depends_on")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .all(|v| {
                v.as_str()
                    .is_some_and(|parent| supported(parent, index, visiting, remaining))
            });
        visiting.remove(id);
        result
    }
    accepted.retain(|c| {
        let id = c.get("id").and_then(Value::as_str).unwrap_or("");
        let mut remaining = 1_000;
        let valid = supported(id, &index, &mut BTreeSet::new(), &mut remaining);
        if !valid {
            excluded.push(json!({"id":id,"reason":"missing_cyclic_or_unbounded_derivation"}));
        }
        valid
    });
    (accepted, excluded)
}

fn normalized_claim_value(claim: &Value) -> Option<Value> {
    let value = claim.get("value")?.clone();
    if claim.get("predicate").and_then(Value::as_str) == Some("operating_hours") {
        let n = number(&value)?;
        if n < 0.0 {
            return None;
        }
        let normalized = match claim.get("unit").and_then(Value::as_str) {
            Some("hour" | "hours" | "h" | "hr" | "hrs") => n,
            Some("minute" | "minutes" | "min") => n / 60.0,
            _ => return None,
        };
        return Some(json!(normalized));
    }
    Some(value)
}

fn task<'a>(task_id: &str, pack: &'a Value) -> Result<&'a Value, EngineError> {
    let list = pack
        .get("tasks")
        .and_then(Value::as_array)
        .ok_or_else(|| EngineError::new("invalid_pack", "Pack requires tasks array"))?;
    list.iter()
        .find(|t| {
            t.get("id").and_then(Value::as_str) == Some(task_id)
                || t.get("aliases")
                    .and_then(Value::as_array)
                    .is_some_and(|a| a.iter().any(|v| v.as_str() == Some(task_id)))
        })
        .ok_or_else(|| {
            EngineError::new(
                "unsupported_task",
                format!("Task {task_id} is not registered in the selected pack"),
            )
        })
}

/// Registered deterministic/classical decisions. No action is executed and no
/// unvalidated probability/confidence is fabricated. The caller supplies only
/// authorized claims/sources and commits under its epoch transaction contract.
pub fn decide(
    task_id: &str,
    input: &Value,
    claims: &[Value],
    pack: &Value,
) -> Result<Value, EngineError> {
    bounded(input, 2_000_000)?;
    bounded(pack, 4_000_000)?;
    if claims.len() > 100_000 {
        return Err(EngineError::new(
            "resource_limit",
            "Too many candidate claims",
        ));
    }
    let spec = task(task_id, pack)?;
    let kind = spec
        .get("type")
        .or_else(|| spec.get("kind"))
        .and_then(Value::as_str)
        .unwrap_or("");
    let message = input.get("message").and_then(Value::as_str).unwrap_or("");
    let parsing = parse(message, input, pack);
    let mut fields = parsing
        .get("fields")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    fields.insert("message".into(), json!(message));
    fields.insert("concepts".into(), parsing["concepts"].clone());
    if let Some(explicit) = input.get("fields").and_then(Value::as_object) {
        fields.extend(explicit.clone());
    }
    for key in ["asset_id", "as_of", "jurisdiction", "action"] {
        if let Some(value) = input.get(key).filter(|v| !v.is_null()) {
            fields.insert(key.into(), value.clone());
        }
    }
    let mut remaining = spec
        .get("condition_budget")
        .and_then(Value::as_u64)
        .unwrap_or(1_000)
        .min(10_000) as usize;
    let rules = evaluate_rules(
        pack.get("rules").unwrap_or(&json!([])),
        &Value::Object(fields.clone()),
        remaining,
    )?;
    if matches!(
        rules["status"].as_str(),
        Some("policy_denied" | "policy_conflict")
    ) {
        return Ok(
            json!({"task":task_id,"status":rules["status"],"value":null,"rules":rules,"probabilities":null,"calibration_ref":null,"authorized_effects":[],"response_plan":{"action":"policy_denied","slots":{}},"interpretation":parsing}),
        );
    }
    if let Some(updates) = rules.get("updates").and_then(Value::as_object) {
        fields.extend(updates.clone());
    }
    let (approved, excluded) = accepted_claims(input, claims);
    let asset = fields.get("asset_id").and_then(Value::as_str).unwrap_or("");
    let mut facts = Map::new();
    let mut evidence_ids = Vec::new();
    let mut conflicts = Vec::<Value>::new();
    let mut pending_claims = Vec::new();
    // Scope-sensitive claims enter only the selected asset, or explicitly global
    // knowledge whose qualifiers can be checked against confirmed asset facts.
    for claim in &approved {
        if claim.get("subject").and_then(Value::as_str) != Some(asset) {
            continue;
        }
        let Some(predicate) = claim.get("predicate").and_then(Value::as_str) else {
            continue;
        };
        let Some(value) = normalized_claim_value(claim) else {
            pending_claims.push(json!({"id":claim["id"],"reason":"invalid_value_or_unit"}));
            continue;
        };
        if let Some(old) = facts.get(predicate) {
            if old != &value {
                conflicts.push(
                    json!({"predicate":predicate,"values":[old,value],"claim_id":claim["id"]}),
                );
            }
        } else {
            facts.insert(predicate.into(), value);
        }
        evidence_ids.push(claim["id"].clone());
    }
    let mut global_claims = Vec::new();
    for claim in &approved {
        let subject = claim.get("subject").and_then(Value::as_str).unwrap_or("");
        if subject == asset {
            continue;
        }
        if subject != "global" && subject != "policy" {
            continue;
        }
        let qualifier = claim.get("qualifiers").unwrap_or(&Value::Null);
        let mut matches = true;
        for (key, fact_key) in [("product", "product"), ("jurisdiction", "jurisdiction")] {
            if let Some(required) = qualifier.get(key).filter(|v| !v.is_null()) {
                let actual = facts.get(fact_key).or_else(|| fields.get(fact_key));
                matches &= actual == Some(required);
            }
        }
        if let Some(serial_range) = qualifier.get("serial_range").and_then(Value::as_array) {
            let serial = facts
                .get("serial_number")
                .and_then(Value::as_str)
                .and_then(|s| s.parse::<u64>().ok());
            matches &= match (
                serial,
                serial_range.first().and_then(Value::as_u64),
                serial_range.get(1).and_then(Value::as_u64),
            ) {
                (Some(s), Some(a), Some(b)) => s >= a && s <= b,
                _ => false,
            };
        }
        if matches {
            global_claims.push(*claim);
        }
    }
    let f = Value::Object(fields.clone());
    let mut status = "resolved".to_string();
    let mut missing = Vec::<String>::new();
    let mut response_action = "decision_result".to_string();
    let mut slots = Map::new();
    let mut checks = Vec::new();
    let mut ranking = Vec::new();
    let value = if kind == "service_triage" || kind == "warranty_prequalification" {
        for critical in ["serial_number", "operating_hours"] {
            if let (Some(asserted), Some(confirmed)) = (fields.get(critical), facts.get(critical)) {
                let agrees = match (number(asserted), number(confirmed)) {
                    (Some(a), Some(b)) => (a - b).abs() < 1e-9,
                    _ => asserted == confirmed,
                };
                if !agrees {
                    conflicts.push(json!({"predicate":critical,"reason":"unconfirmed_assertion_disagrees_with_approved_record","asserted":asserted,"confirmed":confirmed}));
                }
            }
        }
        if let Some(interpreted) = parsing.pointer("/fields/asset_id").and_then(Value::as_str) {
            if !asset.is_empty() && interpreted != asset {
                conflicts.push(json!({"predicate":"asset_id","reason":"input_asset_disagrees_with_message","asserted":interpreted,"confirmed":asset}));
            }
        }
        if asset.is_empty() {
            missing.push("asset_id".into());
        }
        if input
            .get("as_of")
            .and_then(Value::as_str)
            .and_then(date)
            .is_none()
        {
            missing.push("as_of".into());
        }
        for required in spec
            .get("required_claims")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
        {
            if !facts.contains_key(required) {
                missing.push(required.into());
            }
        }
        if !conflicts.is_empty() {
            status = "conflicting_evidence".into();
            response_action = "conflicting_evidence".into();
            Value::Null
        } else if !missing.is_empty() {
            status = "insufficient_evidence".into();
            response_action = "missing_evidence".into();
            Value::Null
        } else if kind == "service_triage" {
            if parsing["status"] == "ambiguous"
                && !input.pointer("/fields/issue").is_some_and(Value::is_string)
            {
                status = "ambiguous".into();
                response_action = "clarify_issue".into();
                Value::Null
            } else {
                let issue = fields.get("issue").and_then(Value::as_str).unwrap_or("");
                let route = spec
                    .get("routes")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                    .find(|r| r.get("issue").and_then(Value::as_str) == Some(issue));
                match route {
                    None => {
                        status = "unsupported".into();
                        response_action = "unsupported".into();
                        Value::Null
                    }
                    Some(route) => {
                        let required = route
                            .get("bulletin_required")
                            .and_then(Value::as_bool)
                            .unwrap_or(false);
                        let bulletins: Vec<_> = global_claims
                            .iter()
                            .filter(|c| {
                                c.get("predicate").and_then(Value::as_str)
                                    == Some("service_bulletin")
                                    && c.pointer("/value/issue").and_then(Value::as_str)
                                        == Some(issue)
                            })
                            .collect();
                        let bulletin = bulletins.first().copied();
                        let bulletin_conflict = bulletins.iter().any(|candidate| {
                            bulletin
                                .is_some_and(|first| candidate.get("value") != first.get("value"))
                        });
                        if bulletin_conflict {
                            conflicts.push(json!({"predicate":"service_bulletin","reason":"incompatible_applicable_bulletins","claim_ids":bulletins.iter().map(|c|c.get("id")).collect::<Vec<_>>()}));
                            status = "conflicting_evidence".into();
                            response_action = "conflicting_evidence".into();
                            Value::Null
                        } else if required && bulletin.is_none() {
                            missing.push("applicable_service_bulletin".into());
                            status = "insufficient_evidence".into();
                            response_action = "missing_evidence".into();
                            Value::Null
                        } else {
                            if let Some(bulletin) = bulletin {
                                evidence_ids.push(bulletin["id"].clone());
                                checks.push(json!({"rule":"approved_bulletin_applicability","result":"pass","claim_id":bulletin["id"]}));
                            }
                            let queue =
                                route.get("queue").cloned().unwrap_or(json!("human_review"));
                            slots.insert("queue".into(), queue.clone());
                            slots.insert("asset_id".into(), json!(asset));
                            response_action = "service_route".into();
                            json!({"issue":issue,"queue":queue,"asset_id":asset,"mechanical_diagnosis":false})
                        }
                    }
                }
            }
        } else {
            let policy = spec.get("policy").ok_or_else(|| {
                EngineError::new(
                    "invalid_pack",
                    "Warranty task requires approved pack policy",
                )
            })?;
            for key in ["product", "jurisdiction"] {
                if let Some(required) = policy.get(key) {
                    if facts.get(key) != Some(required) {
                        return Ok(
                            json!({"task":task_id,"status":"unsupported","value":null,"reason":"policy_applicability_mismatch","field":key,"required":required,"observed":facts.get(key),"probabilities":null,"confidence":null,"authorized_effects":[],"interpretation":parsing,"evidence":{"claim_ids":evidence_ids,"excluded":excluded},"response_plan":{"action":"unsupported","slots":{}}}),
                        );
                    }
                }
            }
            let start = facts
                .get("in_service_date")
                .and_then(Value::as_str)
                .and_then(date);
            let now = input.get("as_of").and_then(Value::as_str).and_then(date);
            let hours = facts.get("operating_hours").and_then(number);
            let months = policy
                .get("max_months")
                .and_then(Value::as_u64)
                .filter(|n| *n <= 1200)
                .ok_or_else(|| {
                    EngineError::new(
                        "invalid_pack",
                        "Warranty max_months must be bounded integer",
                    )
                })? as u32;
            let maximum = policy
                .get("max_operating_hours")
                .and_then(number)
                .filter(|v| *v >= 0.0)
                .ok_or_else(|| {
                    EngineError::new(
                        "invalid_pack",
                        "Warranty requires nonnegative operating hours",
                    )
                })?;
            match (start, now, hours) {
                (Some(start), Some(now), Some(hours)) if now >= start => {
                    let expiry =
                        start
                            .checked_add_months(Months::new(months))
                            .ok_or_else(|| {
                                EngineError::new(
                                    "invalid_date",
                                    "Warranty calendar addition overflow",
                                )
                            })?;
                    let date_pass = now < expiry;
                    let hour_pass = hours <= maximum;
                    let proof_pass =
                        facts.get("proof_of_service").and_then(Value::as_bool) == Some(true);
                    checks.push(json!({"rule":"calendar_coverage","result":if date_pass {"pass"}else{"fail"},"in_service_date":start.to_string(),"expiry_exclusive":expiry.to_string(),"as_of":now.to_string()}));
                    checks.push(json!({"rule":"operating_hours","result":if hour_pass {"pass"}else{"fail"},"value":hours,"maximum_inclusive":maximum,"unit":"hour"}));
                    checks.push(json!({"rule":"proof_of_service","result":if proof_pass {"pass"}else{"review"}}));
                    status = "needs_human".into();
                    response_action = "warranty_prequalification".into();
                    let conclusion = if !date_pass || !hour_pass {
                        "potentially_ineligible"
                    } else if !proof_pass {
                        "review_required"
                    } else {
                        "potentially_eligible"
                    };
                    slots.insert("asset_id".into(), json!(asset));
                    slots.insert("conclusion".into(), json!(conclusion));
                    slots.insert("expiry_date".into(), json!(expiry.to_string()));
                    json!({"prequalification":conclusion,"asset_id":asset,"final_approval":false,"requires_human":true,"policy_version":pack["version"]})
                }
                _ => {
                    status = "insufficient_evidence".into();
                    response_action = "missing_evidence".into();
                    missing.push("valid_in_service_date_and_operating_hours".into());
                    Value::Null
                }
            }
        }
    } else {
        for field in spec
            .get("required_fields")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
        {
            if at(&f, field).is_none() {
                missing.push(field.into());
            }
        }
        if !missing.is_empty() {
            status = "insufficient_evidence".into();
            Value::Null
        } else if spec.get("requires_interpretation").and_then(Value::as_bool) == Some(true)
            && matches!(
                parsing["status"].as_str(),
                Some("unsupported" | "ambiguous" | "budget_exhausted")
            )
        {
            status = parsing["status"].as_str().unwrap_or("unsupported").into();
            response_action = "unsupported".into();
            Value::Null
        } else {
            match kind {
                "boolean" => match condition(
                    spec.get("condition").ok_or_else(|| {
                        EngineError::new("invalid_task", "Boolean task requires condition")
                    })?,
                    &f,
                    &mut remaining,
                )? {
                    Truth::True => json!(true),
                    Truth::False => json!(false),
                    Truth::Unknown => {
                        status = "insufficient_evidence".into();
                        Value::Null
                    }
                },
                "choice" | "multi_choice" | "action" => {
                    let candidates = spec
                        .get("candidates")
                        .and_then(Value::as_array)
                        .ok_or_else(|| {
                            EngineError::new(
                                "invalid_task",
                                "Choice/action task requires finite candidates",
                            )
                        })?;
                    let mut selected = Vec::new();
                    let mut unknown = Vec::new();
                    if candidates.len() > 500 {
                        return Err(EngineError::new(
                            "resource_limit",
                            "Choice candidates exceed bound",
                        ));
                    }
                    for c in candidates.iter().take(500) {
                        let output = c.get("value").ok_or_else(|| {
                            EngineError::new("invalid_task", "Candidate missing value")
                        })?;
                        match condition(
                            c.get("when").unwrap_or(&Value::Bool(true)),
                            &f,
                            &mut remaining,
                        )? {
                            Truth::True => selected.push(output.clone()),
                            Truth::Unknown => unknown.push(output.clone()),
                            Truth::False => {}
                        }
                    }
                    if !unknown.is_empty() {
                        status = "insufficient_evidence".into();
                        checks.push(json!({"unresolved_candidates":unknown}));
                    }
                    if kind == "multi_choice" {
                        json!(selected)
                    } else if selected.len() == 1 && unknown.is_empty() {
                        selected[0].clone()
                    } else {
                        if status == "resolved" {
                            status = if selected.is_empty() {
                                "unsupported"
                            } else {
                                "ambiguous"
                            }
                            .into();
                        }
                        checks.push(json!({"matched_candidates":selected}));
                        Value::Null
                    }
                }
                "score" => {
                    let field = spec.get("field").and_then(Value::as_str).ok_or_else(|| {
                        EngineError::new("invalid_task", "Score task requires field")
                    })?;
                    let n = at(&f, field).and_then(number);
                    let min = spec
                        .get("min")
                        .and_then(number)
                        .unwrap_or(f64::NEG_INFINITY);
                    let max = spec.get("max").and_then(number).unwrap_or(f64::INFINITY);
                    match n {
                        Some(n) if n >= min && n <= max => {
                            json!({"value":n,"unit":spec.get("unit"),"interval":null,"score_kind":"observed_or_authored_value"})
                        }
                        _ => {
                            status = "insufficient_evidence".into();
                            Value::Null
                        }
                    }
                }
                "rank" => {
                    for c in spec
                        .get("candidates")
                        .and_then(Value::as_array)
                        .ok_or_else(|| {
                            EngineError::new("invalid_task", "Rank task requires candidates")
                        })?
                        .iter()
                        .take(500)
                    {
                        let feasible = condition(
                            c.get("when").unwrap_or(&Value::Bool(true)),
                            &f,
                            &mut remaining,
                        )?;
                        if feasible == Truth::Unknown {
                            status = "insufficient_evidence".into();
                            continue;
                        }
                        if feasible != Truth::True {
                            continue;
                        }
                        let mut score = c.get("base_score").and_then(number).unwrap_or(0.0);
                        for term in c
                            .get("terms")
                            .and_then(Value::as_array)
                            .into_iter()
                            .flatten()
                        {
                            let term_truth = condition(&term["when"], &f, &mut remaining)?;
                            if term_truth == Truth::Unknown {
                                status = "insufficient_evidence".into();
                            }
                            if term_truth == Truth::True {
                                score += term.get("weight").and_then(number).ok_or_else(|| {
                                    EngineError::new("invalid_task", "Rank weight must be finite")
                                })?;
                            }
                        }
                        if !score.is_finite() {
                            return Err(EngineError::new("invalid_task", "Rank score overflow"));
                        }
                        ranking.push(json!({"value":c["value"],"score":score,"score_kind":"authored_utility_not_probability"}));
                    }
                    ranking.sort_by(|a, b| {
                        b["score"]
                            .as_f64()
                            .unwrap_or(0.0)
                            .total_cmp(&a["score"].as_f64().unwrap_or(0.0))
                    });
                    if ranking.is_empty() {
                        status = "insufficient_evidence".into();
                    }
                    json!(ranking)
                }
                "extract" => {
                    let mut extracted = Map::new();
                    for field in spec
                        .get("fields")
                        .and_then(Value::as_array)
                        .ok_or_else(|| {
                            EngineError::new("invalid_task", "Extract task requires fields")
                        })?
                        .iter()
                        .filter_map(Value::as_str)
                    {
                        if let Some(v) = at(&f, field) {
                            extracted.insert(field.into(), v.clone());
                        } else {
                            missing.push(field.into());
                        }
                    }
                    if !missing.is_empty() {
                        status = "insufficient_evidence".into();
                    }
                    Value::Object(extracted)
                }
                _ => {
                    return Err(EngineError::new(
                        "unsupported_task_type",
                        format!("Unsupported registered task type {kind}"),
                    ))
                }
            }
        }
    };
    if rules["requires_review"] == true && status == "resolved" {
        status = "needs_human".into();
    }
    if !missing.is_empty() {
        slots.insert("missing".into(), json!(missing.join(", ")));
    }
    if kind != "service_triage" && kind != "warranty_prequalification" {
        slots.insert("result".into(), json!(crate::util::string(&value)));
    }
    evidence_ids.sort_by_key(Value::to_string);
    evidence_ids.dedup();
    let response_plan = json!({"action":response_action,"slots":slots,"authorized_fact_ids":evidence_ids,"max_characters":2_000});
    let response = compose(&response_plan, pack, &[]).ok();
    Ok(
        json!({"task":task_id,"task_type":kind,"status":status,"value":value,"probabilities":null,"calibration_ref":null,"confidence":null,"score_semantics":"deterministic_rule_or_authored_utility","authorized_effects":[],"checks":checks,"rules":rules,"fields":fields,"interpretation":parsing,"evidence":{"claim_ids":evidence_ids,"missing":missing,"conflicts":conflicts,"excluded":excluded,"invalid_values":pending_claims,"source_verification":"caller_authorized_sources_plus_kernel_revision_check","as_of":input.get("as_of"),"known_at":input.get("known_at"),"complete":missing.is_empty()&&conflicts.is_empty()},"response_plan":response_plan,"response":response,"execution_profile":"native_strict","final_authority":"caller_control_plane"}),
    )
}
