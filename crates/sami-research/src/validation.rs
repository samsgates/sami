use crate::{array, digest, invalid, text, Result};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

fn condition(v: &Value, depth: usize, budget: &mut usize, errors: &mut Vec<String>, path: &str) {
    if *budget == 0 || depth > 24 {
        errors.push(format!("{path}: condition budget/depth exhausted"));
        return;
    }
    *budget -= 1;
    if v.is_boolean() {
        return;
    }
    if let Some(items) = v.get("all").or_else(|| v.get("any")) {
        if let Some(items) = items.as_array() {
            if items.len() > 64 {
                errors.push(format!("{path}: boolean group exceeds 64"));
            }
            for (i, item) in items.iter().enumerate().take(64) {
                condition(item, depth + 1, budget, errors, &format!("{path}/{i}"));
            }
        } else {
            errors.push(format!("{path}: all/any must be array"));
        }
        return;
    }
    if let Some(inner) = v.get("not") {
        condition(inner, depth + 1, budget, errors, path);
        return;
    }
    if v.get("always").and_then(Value::as_bool).is_some() {
        return;
    }
    if v.get("field").and_then(Value::as_str).is_none() {
        errors.push(format!("{path}: field missing"));
    }
    let op = v.get("op").and_then(Value::as_str).unwrap_or("eq");
    if !matches!(
        op,
        "eq" | "ne"
            | "gt"
            | "ge"
            | "gte"
            | "lt"
            | "le"
            | "lte"
            | "in"
            | "contains"
            | "starts_with"
            | "exists"
    ) {
        errors.push(format!("{path}: unsupported rule operator {op}"));
    }
    if op == "in" && !v.get("value").is_some_and(Value::is_array) {
        errors.push(format!("{path}: in requires array"));
    }
}
pub fn pack(input: &Value) -> Result<Value> {
    let pack = input.get("pack").unwrap_or(input);
    if !pack.is_object() {
        return Err(invalid("pack must be object"));
    }
    let mut errors = Vec::new();
    let mut warnings: Vec<String> = Vec::new();
    for key in ["id", "version", "language"] {
        if !pack
            .get(key)
            .and_then(Value::as_str)
            .is_some_and(|s| !s.is_empty() && s.len() <= 128)
        {
            errors.push(format!("{key}: nonempty bounded string required"));
        }
    }
    if let Some(version) = pack.get("version").and_then(Value::as_str) {
        let parts = version.split('.').collect::<Vec<_>>();
        if parts.len() != 3 || parts.iter().any(|p| p.parse::<u64>().is_err()) {
            errors.push("version: expected numeric MAJOR.MINOR.PATCH".into());
        }
    }
    if !pack.get("limits").is_some_and(Value::is_object) {
        errors.push("limits: object required".into());
    }
    if !pack.get("ontology").is_some_and(Value::is_object) {
        errors.push("ontology: object required".into());
    }
    let tasks = pack.get("tasks").and_then(Value::as_array);
    let mut task_ids = BTreeSet::new();
    let mut budget = 20_000;
    if let Some(tasks) = tasks {
        if tasks.is_empty() || tasks.len() > 128 {
            errors.push("tasks: expected 1..128".into());
        }
        for task in tasks.iter().take(128) {
            let id = task.get("id").and_then(Value::as_str).unwrap_or("");
            if id.is_empty() || !task_ids.insert(id) {
                errors.push(format!("task id missing or duplicate: {id}"));
            }
            let kind = task.get("type").and_then(Value::as_str).unwrap_or("");
            if !matches!(
                kind,
                "service_triage"
                    | "warranty_prequalification"
                    | "boolean"
                    | "choice"
                    | "multi_choice"
                    | "score"
                    | "rank"
                    | "extract"
                    | "classification"
                    | "action"
            ) {
                errors.push(format!("task {id}: unsupported type {kind}"));
            }
            if let Some(c) = task.get("condition") {
                condition(c, 0, &mut budget, &mut errors, &format!("task/{id}"));
            }
            if matches!(
                kind,
                "choice" | "multi_choice" | "rank" | "classification" | "action"
            ) {
                if let Some(candidates) = task.get("candidates").and_then(Value::as_array) {
                    if candidates.is_empty() || candidates.len() > 128 {
                        errors.push(format!("task {id}: candidate count invalid"));
                    }
                    let mut values = BTreeSet::new();
                    for c in candidates.iter().take(128) {
                        if let Some(value) = c.get("value") {
                            if !values.insert(value.to_string()) {
                                errors.push(format!("task {id}: duplicate candidate"));
                            }
                        } else {
                            errors.push(format!("task {id}: candidate value missing"));
                        }
                        if let Some(when) = c.get("when") {
                            condition(
                                when,
                                0,
                                &mut budget,
                                &mut errors,
                                &format!("task/{id}/candidate"),
                            );
                        }
                    }
                } else {
                    errors.push(format!("task {id}: candidates required"));
                }
            }
            if kind == "score" {
                let min = task.get("min").and_then(Value::as_f64);
                let max = task.get("max").and_then(Value::as_f64);
                if !matches!((min,max),(Some(a),Some(b)) if a<b) {
                    errors.push(format!("task {id}: invalid score range"));
                }
            }
        }
    } else {
        errors.push("tasks: array required".into());
    }
    let mut rule_ids = BTreeSet::new();
    if let Some(rules) = pack.get("rules").and_then(Value::as_array) {
        if rules.len() > 5000 {
            errors.push("rules: limit is 5000".into());
        }
        for rule in rules.iter().take(5000) {
            let id = rule.get("id").and_then(Value::as_str).unwrap_or("");
            if id.is_empty() || !rule_ids.insert(id) {
                errors.push(format!("rule id missing/duplicate: {id}"));
            }
            let effect = rule.get("effect").and_then(Value::as_str).unwrap_or("set");
            if !matches!(effect, "deny" | "allow" | "require_review" | "set") {
                errors.push(format!("rule {id}: invalid effect"));
            }
            if let Some(when) = rule.get("when") {
                condition(when, 0, &mut budget, &mut errors, &format!("rule/{id}"));
            }
            if let Some(set) = rule.get("set").and_then(Value::as_object) {
                for field in set.keys() {
                    if ["tenant", "permission", "tool", "authority", "acl"]
                        .iter()
                        .any(|p| field.starts_with(p))
                    {
                        errors.push(format!("rule {id}: forbidden authority update {field}"));
                    }
                }
            }
        }
    } else {
        errors.push("rules: array required".into());
    }
    if let Some(responses) = pack.get("responses").and_then(Value::as_object) {
        if responses.len() > 256 {
            errors.push("responses: at most 256 actions".into());
        }
        for (action, patterns) in responses {
            if let Some(patterns) = patterns.as_array() {
                if patterns.is_empty() || patterns.len() > 128 {
                    errors.push(format!("response {action}: invalid pattern count"));
                }
                for p in patterns.iter().take(128) {
                    if !p.as_str().is_some_and(|s| !s.is_empty() && s.len() <= 8192) {
                        errors.push(format!("response {action}: nonempty bounded text required"));
                    }
                }
            } else {
                errors.push(format!("response {action}: array required"));
            }
        }
    } else {
        errors.push("responses: object required".into());
    }
    if let Some(workflows) = pack.get("workflows").and_then(Value::as_object) {
        if workflows.len() > 64 {
            errors.push("workflows: at most 64".into());
        }
        for (id, w) in workflows.iter().take(64) {
            let actions = w.get("actions").and_then(Value::as_array);
            if let Some(actions) = actions {
                if actions.is_empty() || actions.len() > 128 {
                    errors.push(format!("workflow {id}: invalid action count"));
                }
                let mut ids = BTreeSet::new();
                for a in actions.iter().take(128) {
                    let action = a.get("id").and_then(Value::as_str).unwrap_or("");
                    if action.is_empty() || !ids.insert(action) {
                        errors.push(format!("workflow {id}: missing/duplicate action"));
                    }
                    if let Some(pre) = a.get("preconditions") {
                        condition(
                            pre,
                            0,
                            &mut budget,
                            &mut errors,
                            &format!("workflow/{id}/{action}"),
                        );
                    }
                    if !a.get("effects").is_some_and(Value::is_object) {
                        errors.push(format!("workflow {id}/{action}: effects object required"));
                    }
                }
            } else {
                errors.push(format!("workflow {id}: actions required"));
            }
        }
    } else {
        errors.push("workflows: object required".into());
    }
    let license = pack
        .get("license")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty());
    if license.is_none() {
        warnings.push(
            "license: declaration missing; publication eligibility cannot be established".into(),
        );
    }
    if !pack.get("manifest").is_some_and(Value::is_object) {
        warnings.push("manifest missing; publish through runtime bundle verification".into());
    }
    let license_entries = pack.get("licenses").and_then(Value::as_array);
    let rights = license_entries.is_some_and(|items| {
        !items.is_empty()
            && items.iter().all(|v| {
                v.get("license")
                    .and_then(Value::as_str)
                    .is_some_and(|s| !s.is_empty())
                    && v.get("permitted").and_then(Value::as_bool) == Some(true)
            })
    });
    if !rights {
        warnings.push("content license permissions missing or unconfirmed".into());
    }
    Ok(
        json!({"valid":errors.is_empty(),"publication_eligible":errors.is_empty()&&warnings.is_empty(),"errors":errors,"warnings":warnings,"task_count":task_ids.len(),"rule_count":rule_ids.len(),"pack_sha256":digest(pack)?,"signature_verified":false,"data_classification":pack.get("data_classification").unwrap_or(&Value::Null),"semantic_correctness_verified":false}),
    )
}

fn safe_path(path: &str) -> bool {
    !path.is_empty()
        && path.len() <= 256
        && !path.starts_with('/')
        && !path.contains('\\')
        && !path
            .split('/')
            .any(|p| p == ".." || p == "." || p.is_empty())
        && !path.contains(':')
}
pub fn bundle(input: &Value) -> Result<Value> {
    let manifest = input
        .get("manifest")
        .ok_or_else(|| invalid("manifest required"))?;
    let now = input
        .get("now")
        .and_then(Value::as_i64)
        .ok_or_else(|| invalid("now epoch seconds required"))?;
    let mut errors = Vec::new();
    for key in ["id", "version", "issuer", "purpose"] {
        if manifest
            .get(key)
            .and_then(Value::as_str)
            .is_none_or(|s| s.is_empty())
        {
            errors.push(format!("manifest.{key} required"));
        }
    }
    if manifest.get("format_version").and_then(Value::as_u64) != Some(1) {
        errors.push("unsupported manifest format_version".into());
    }
    let created = manifest.get("created_at").and_then(Value::as_i64);
    let expires = manifest.get("expires_at").and_then(Value::as_i64);
    if !matches!((created,expires),(Some(c),Some(e))if c<=now&&e>now&&e>c) {
        errors.push("manifest not yet valid, expired or invalid timestamps".into());
    }
    let expected = array(manifest, "artifacts", 256)?;
    let actual = array(input, "artifacts", 256)?;
    if expected.is_empty() {
        errors.push("manifest has no artifacts".into());
    }
    let mut actual_map = BTreeMap::new();
    for artifact in actual {
        let path = text(artifact, "path")?;
        if !safe_path(path) {
            errors.push(format!("unsafe artifact path: {path}"));
        }
        let content = text(artifact, "content")?;
        if actual_map.insert(path, content).is_some() {
            errors.push(format!("duplicate supplied artifact: {path}"));
        }
    }
    let mut names = BTreeSet::new();
    for artifact in expected {
        let path = text(artifact, "path")?;
        let hash = text(artifact, "sha256")?;
        if !safe_path(path) || !names.insert(path) {
            errors.push(format!("unsafe/duplicate manifest artifact: {path}"));
        }
        if hash.len() != 64 || hex::decode(hash).is_err() {
            errors.push(format!("invalid hash for {path}"));
        }
        if artifact.get("license").and_then(Value::as_str).is_none()
            || artifact.get("permitted").and_then(Value::as_bool) != Some(true)
        {
            errors.push(format!("license permission missing: {path}"));
        }
        if let Some(content) = actual_map.get(path) {
            if hex::encode(Sha256::digest(content.as_bytes())) != hash {
                errors.push(format!("checksum mismatch: {path}"));
            }
        } else {
            errors.push(format!("missing artifact: {path}"));
        }
    }
    for path in actual_map.keys() {
        if !names.contains(path) {
            errors.push(format!("undeclared artifact: {path}"));
        }
    }
    let checksum_valid = errors.is_empty();
    Ok(
        json!({"metadata_and_checksums_valid":checksum_valid,"errors":errors,"manifest_sha256":digest(manifest)?,"signature_verified":false,"production_activation_allowed":false,"signature_contract":"runtime must verify a trusted HMAC/Ed25519 signature over canonical manifest, issuer, audience and expiry before activation; a body flag is never proof"}),
    )
}

pub fn splits(input: &Value) -> Result<Value> {
    let partitions = input
        .get("partitions")
        .and_then(Value::as_object)
        .ok_or_else(|| invalid("partitions object required"))?;
    if partitions.len() > 8 {
        return Err(invalid("at most 8 partitions"));
    }
    let grouping = match input.get("group_keys") {
        Some(v) => v
            .as_array()
            .filter(|a| a.len() <= 8)
            .ok_or_else(|| invalid("group_keys must have at most 8 entries"))?
            .iter()
            .map(|v| {
                v.as_str()
                    .map(str::to_owned)
                    .ok_or_else(|| invalid("group keys must be strings"))
            })
            .collect::<Result<Vec<_>>>()?,
        None => vec!["source_family".into()],
    };
    let mut ids: BTreeMap<String, String> = BTreeMap::new();
    let mut hashes: BTreeMap<String, String> = BTreeMap::new();
    let mut groups: BTreeMap<String, String> = BTreeMap::new();
    let mut errors = Vec::new();
    let mut counts = BTreeMap::new();
    let mut times: BTreeMap<String, (i64, i64)> = BTreeMap::new();
    for (name, cases) in partitions {
        let cases = cases
            .as_array()
            .filter(|a| a.len() <= 20_000)
            .ok_or_else(|| invalid("partition must be array with at most 20000 cases"))?;
        if cases.is_empty() {
            errors.push(format!("empty partition:{name}"));
        }
        counts.insert(name.clone(), cases.len());
        for case in cases {
            let id = text(case, "id")?;
            if let Some(previous) = ids.insert(id.into(), name.clone()) {
                errors.push(format!("case id {id} duplicated in {previous}/{name}"));
            }
            let hash = match case.get("content_sha256").and_then(Value::as_str) {
                Some(h) => h.to_owned(),
                None => digest(
                    case.get("x")
                        .ok_or_else(|| invalid("split case needs x or content_sha256"))?,
                )?,
            };
            if let Some(previous) = hashes.insert(hash, name.clone()) {
                if previous != *name {
                    errors.push(format!(
                        "identical feature content across {previous}/{name}"
                    ));
                }
            }
            for key in &grouping {
                let value = case
                    .get(key)
                    .and_then(Value::as_str)
                    .filter(|s| !s.is_empty())
                    .ok_or_else(|| invalid(format!("case {id}: group field {key} required")))?;
                let group = format!("{key}:{value}");
                if let Some(previous) = groups.insert(group.clone(), name.clone()) {
                    if previous != *name {
                        errors.push(format!("group {group} spans {previous}/{name}"));
                    }
                }
            }
            if let Some(time) = case.get("event_time").and_then(Value::as_i64) {
                times
                    .entry(name.clone())
                    .and_modify(|r| {
                        r.0 = r.0.min(time);
                        r.1 = r.1.max(time);
                    })
                    .or_insert((time, time));
            } else if input.get("time_ordered").and_then(Value::as_bool) == Some(true) {
                errors.push(format!("case {id}: event_time required"));
            }
        }
    }
    if input.get("time_ordered").and_then(Value::as_bool) == Some(true) {
        let order = ["train", "validation", "calibration", "locked_transfer_test"];
        let mut previous = None;
        for name in order {
            if let Some(&(min, max)) = times.get(name) {
                if previous.is_some_and(|t| t >= min) {
                    errors.push(format!("temporal overlap before {name}"));
                }
                previous = Some(max);
            }
        }
    }
    let locked = partitions.contains_key("locked_transfer_test");
    if !locked {
        errors.push("locked_transfer_test partition required for transfer validation".into());
    }
    Ok(
        json!({"valid":errors.is_empty(),"errors":errors,"counts":counts,"group_keys":grouping,"locked_transfer_test_present":locked,"test_content_returned":false,"scope":"detects declared id/content/group/time leakage; does not detect paraphrase leakage, hidden training contamination or certify real customer representativeness"}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn expiry_checksum_and_path_traversal_are_enforced() {
        let content = "abc";
        let h = hex::encode(Sha256::digest(content.as_bytes()));
        let v = json!({"now":100,"manifest":{"format_version":1,"id":"p","version":"1","issuer":"local","purpose":"research","created_at":90,"expires_at":200,"artifacts":[{"path":"pack.json","sha256":h,"license":"Apache-2.0","permitted":true}]},"artifacts":[{"path":"pack.json","content":content}]});
        assert_eq!(
            bundle(&v).expect("bundle")["metadata_and_checksums_valid"],
            true
        );
        let mut expired = v.clone();
        expired["now"] = json!(201);
        assert_eq!(
            bundle(&expired).expect("expired")["metadata_and_checksums_valid"],
            false
        );
        let mut malicious = v;
        malicious["manifest"]["artifacts"][0]["path"] = json!("../secret");
        assert_eq!(
            bundle(&malicious).expect("path")["metadata_and_checksums_valid"],
            false
        );
    }
    #[test]
    fn split_family_overlap_is_rejected() {
        let v = json!({"partitions":{"train":[{"id":"1","x":{"v":1},"source_family":"a"}],"validation":[{"id":"2","x":{"v":2},"source_family":"a"}],"locked_transfer_test":[{"id":"3","x":{"v":3},"source_family":"b"}]}});
        assert_eq!(splits(&v).expect("split")["valid"], false);
    }
    #[test]
    fn typed_pack_validation_reports_errors_without_promotion() {
        let r = pack(&json!({"pack":{"id":"demo"}})).expect("validate");
        assert_eq!(r["valid"], false);
        assert_eq!(r["publication_eligible"], false);
        assert!(r["errors"].as_array().expect("errors").len() > 4);
    }
}
