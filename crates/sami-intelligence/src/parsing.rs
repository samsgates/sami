use crate::util::{date, tokens};
use chrono::Duration;
use regex::Regex;
use serde_json::{json, Map, Value};
use std::collections::BTreeSet;

fn negated(text: &str, start: usize) -> bool {
    let Some(before) = text.get(..start) else {
        return false;
    };
    let clause = before
        .rsplit(['.', ';', '!', '?', '\n'])
        .next()
        .unwrap_or(before);
    let words = tokens(clause);
    let words = words.iter().rev().take(6).collect::<Vec<_>>();
    // Contrast closes the scope of a preceding negation. This is deliberately
    // a bounded grammar rather than an unrestricted semantic parser.
    for word in words {
        if matches!(word.as_str(), "but" | "however" | "instead") {
            return false;
        }
        if matches!(
            word.as_str(),
            "not" | "no" | "never" | "without" | "isn" | "isnt"
        ) {
            return true;
        }
    }
    false
}

/// Parse supported domain expressions into proposals, never authoritative claims.
pub fn parse(message: &str, state: &Value, pack: &Value) -> Value {
    if message.len() > 65_536 {
        return json!({"status":"budget_exhausted", "fields":{}, "ambiguities":["message_byte_limit"], "intent_candidates":[]});
    }
    // The shipped grammar is English. Latin-script names are preserved, but
    // unsupported scripts cannot inherit English keyword/negation semantics.
    if message.chars().any(|c|matches!(c as u32,0x0400..=0x052f|0x0600..=0x06ff|0x0900..=0x097f|0x3040..=0x30ff|0x4e00..=0x9fff|0xac00..=0xd7af)) {
        return json!({"status":"unsupported","reason":"unsupported_language_script","language":"unsupported","fields":{},"concepts":[],"negated_concepts":[],"intent_candidates":[],"ambiguities":[],"evidence":[],"assertion_status":"unconfirmed","probabilities":null});
    }
    let lower = message.to_lowercase();
    let mut fields = Map::new();
    let mut ambiguities = Vec::<Value>::new();
    let mut evidence = Vec::<Value>::new();
    let mut negated_concepts = BTreeSet::new();
    let mut concepts = BTreeSet::new();
    if let Some(list) = pack.pointer("/ontology/concepts").and_then(Value::as_array) {
        for concept in list.iter().take(500) {
            let id = concept
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or("unknown");
            for phrase in concept
                .get("phrases")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .take(100)
            {
                let Some(phrase) = phrase.as_str() else {
                    continue;
                };
                let Ok(pattern) = Regex::new(&format!(r"(?i)\b{}\b", regex::escape(phrase))) else {
                    continue;
                };
                for found in pattern.find_iter(message) {
                    let is_negated = negated(message, found.start());
                    if is_negated {
                        negated_concepts.insert(id.to_owned());
                    } else {
                        concepts.insert(id.to_owned());
                    }
                    evidence.push(json!({"field":"concept", "value":id, "span":[found.start(),found.end()], "text":found.as_str(), "negated":is_negated}));
                }
            }
        }
    }
    let issue_ids: Vec<_> = concepts
        .iter()
        .filter(|id| id.starts_with("issue."))
        .cloned()
        .collect();
    match issue_ids.len() {
        0 => {}
        1 => {
            fields.insert("issue".into(), json!(issue_ids[0]));
        }
        _ => ambiguities.push(json!({"field":"issue", "candidates":issue_ids})),
    }

    let mut serials = BTreeSet::new();
    let serial_pattern =
        Regex::new(r"(?i)\b(?:serial(?:\s+number)?|s/n)\s*[:#]?\s*([a-z0-9][a-z0-9_-]{1,40})")
            .expect("fixed regex");
    for cap in serial_pattern.captures_iter(message) {
        if let Some(found) = cap.get(1) {
            if !negated(message, found.start()) {
                serials.insert(found.as_str().to_uppercase());
                evidence.push(json!({"field":"serial_number", "value":found.as_str().to_uppercase(), "span":[found.start(),found.end()]}));
            }
        }
    }
    match serials.len() {
        0 => {}
        1 => {
            fields.insert("serial_number".into(), json!(serials.iter().next()));
        }
        _ => ambiguities.push(json!({"field":"serial_number", "candidates":serials})),
    }

    let quantity =
        Regex::new(r"(?i)\b([0-9][0-9,]*(?:\.[0-9]+)?)\s*(hours?|hrs?|h|minutes?|mins?|days?)\b")
            .expect("fixed regex");
    let mut hours = Vec::<Value>::new();
    for cap in quantity.captures_iter(message) {
        let Some(raw) = cap.get(1) else {
            continue;
        };
        if negated(message, raw.start()) {
            continue;
        }
        let Some(value) = raw
            .as_str()
            .replace(',', "")
            .parse::<f64>()
            .ok()
            .filter(|v| v.is_finite())
        else {
            continue;
        };
        let unit = cap
            .get(2)
            .map(|m| m.as_str().to_lowercase())
            .unwrap_or_default();
        if unit.starts_with("day") {
            evidence.push(json!({"field":"duration_days", "value":value, "span":[raw.start(),cap.get(0).map(|m|m.end()).unwrap_or(raw.end())]}));
            continue;
        }
        let normalized = if unit.starts_with("min") {
            value / 60.0
        } else {
            value
        };
        hours.push(json!(normalized));
        evidence.push(json!({"field":"operating_hours", "value":normalized, "unit":"hour", "raw":raw.as_str(), "span":[raw.start(),cap.get(0).map(|m|m.end()).unwrap_or(raw.end())]}));
    }
    hours.dedup();
    match hours.len() {
        0 => {}
        1 => {
            fields.insert("operating_hours".into(), hours[0].clone());
        }
        _ => ambiguities.push(json!({"field":"operating_hours", "candidates":hours})),
    }

    let iso_date = Regex::new(r"\b\d{4}-\d{2}-\d{2}\b").expect("fixed regex");
    let mut dates = Vec::new();
    for found in iso_date.find_iter(message) {
        if negated(message, found.start()) {
            continue;
        }
        if date(found.as_str()).is_some() {
            dates.push(found.as_str().to_string());
            evidence.push(
                json!({"field":"date", "value":found.as_str(), "span":[found.start(),found.end()]}),
            );
        } else {
            ambiguities.push(
                json!({"field":"date", "reason":"invalid_calendar_date", "text":found.as_str()}),
            );
        }
    }
    let local_date = Regex::new(r"\b\d{1,2}/\d{1,2}/\d{2,4}\b").expect("fixed regex");
    for found in local_date.find_iter(message) {
        ambiguities
            .push(json!({"field":"date", "reason":"locale_ambiguous_date", "text":found.as_str()}));
    }
    if lower.contains("yesterday") || lower.contains("today") {
        if let Some(base) = state.get("as_of").and_then(Value::as_str).and_then(date) {
            let d = if lower.contains("yesterday") {
                base - Duration::days(1)
            } else {
                base
            };
            dates.push(d.to_string());
        } else {
            ambiguities.push(json!({"field":"date", "reason":"relative_date_requires_as_of"}));
        }
    }
    dates.sort();
    dates.dedup();
    match dates.len() {
        0 => {}
        1 => {
            fields.insert("date".into(), json!(dates[0]));
        }
        _ => ambiguities.push(json!({"field":"date", "candidates":dates})),
    }

    let assets: Vec<&Value> = state
        .get("assets")
        .or_else(|| pack.get("assets"))
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .take(10_000)
        .collect();
    let mut matches = Vec::<Value>::new();
    if let Some(serial) = fields.get("serial_number").and_then(Value::as_str) {
        for asset in &assets {
            if asset
                .get("serial_number")
                .and_then(Value::as_str)
                .is_some_and(|s| s.eq_ignore_ascii_case(serial))
            {
                matches.push((*asset).clone());
            }
        }
        if matches.is_empty() {
            ambiguities
                .push(json!({"field":"asset_id", "reason":"unknown_serial", "value":serial}));
        }
    } else {
        for asset in &assets {
            let mut hit = false;
            for alias in asset
                .get("aliases")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                if let Some(alias) = alias.as_str() {
                    if let Ok(pattern) = Regex::new(&format!(r"(?i)\b{}\b", regex::escape(alias))) {
                        hit |= pattern
                            .find_iter(message)
                            .any(|m| !negated(message, m.start()));
                    }
                }
            }
            if hit {
                matches.push((*asset).clone());
            }
        }
        if lower.contains("newer one") || lower.contains("older one") {
            if assets.len() == 2 {
                let mut ordered = assets.clone();
                ordered.sort_by_key(|a| a.get("year").and_then(Value::as_u64).unwrap_or(0));
                if ordered[0].get("year") != ordered[1].get("year") {
                    matches = vec![if lower.contains("newer one") {
                        ordered[1].clone()
                    } else {
                        ordered[0].clone()
                    }];
                } else {
                    ambiguities.push(
                        json!({"field":"asset_id", "reason":"relative_asset_year_ambiguous"}),
                    );
                }
            } else {
                ambiguities.push(json!({"field":"asset_id", "reason":"relative_asset_requires_two_known_assets"}));
            }
        }
    }
    if matches.len() == 1 {
        if let Some(id) = matches[0].get("id") {
            fields.insert("asset_id".into(), id.clone());
        }
    } else if matches.len() > 1 {
        ambiguities.push(json!({"field":"asset_id", "candidates":matches.iter().filter_map(|a|a.get("id")).collect::<Vec<_>>()}));
    } else if !ambiguities
        .iter()
        .any(|a| a.get("field").and_then(Value::as_str) == Some("asset_id"))
    {
        if let Some(id) = state.get("asset_id") {
            fields.insert("asset_id".into(), id.clone());
        }
    }

    let mut intents = Vec::new();
    for intent in pack
        .pointer("/ontology/intents")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .take(100)
    {
        let matched: Vec<_> = intent
            .get("concepts")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .filter(|id| concepts.contains(*id))
            .collect();
        if !matched.is_empty() {
            intents.push(json!({"intent":intent["id"], "matched_concepts":matched, "score_kind":"lexical_support", "score":matched.len()}));
        }
    }
    if intents.is_empty() && !issue_ids.is_empty() {
        intents.push(json!({"intent":"service_triage", "score_kind":"lexical_support", "score":issue_ids.len()}));
    }
    let status = if !ambiguities.is_empty() {
        "ambiguous"
    } else if intents.is_empty()
        && fields.is_empty()
        && concepts.is_empty()
        && negated_concepts.is_empty()
    {
        "unsupported"
    } else {
        "parsed"
    };
    json!({"status":status, "message":message, "normalization":"casefolded_lookup_original_offsets_preserved", "fields":fields, "assertion_status":"unconfirmed", "concepts":concepts, "negated_concepts":negated_concepts, "intent_candidates":intents, "ambiguities":ambiguities, "evidence":evidence, "probabilities":null})
}
