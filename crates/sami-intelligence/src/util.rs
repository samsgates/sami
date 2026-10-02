use crate::EngineError;
use chrono::{DateTime, NaiveDate, Utc};
use serde_json::Value;

pub(crate) fn at<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
    let mut current = value;
    for component in path.split('.') {
        current = current.get(component)?;
    }
    if current.is_null() {
        None
    } else {
        Some(current)
    }
}
pub(crate) fn string(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        _ => value.to_string(),
    }
}
pub(crate) fn date(s: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(s.get(..10)?, "%Y-%m-%d").ok()
}
pub(crate) fn timestamp(s: &str) -> Option<DateTime<Utc>> {
    if let Ok(v) = DateTime::parse_from_rfc3339(s) {
        return Some(v.with_timezone(&Utc));
    }
    date(s)?.and_hms_opt(0, 0, 0).map(|v| v.and_utc())
}
pub(crate) fn in_interval(value: &Value, start: &str, end: &str, now: &str) -> bool {
    let Some(point) = timestamp(now) else {
        return false;
    };
    for (key, lower) in [(start, true), (end, false)] {
        if let Some(v) = value.get(key).filter(|v| !v.is_null()) {
            let Some(bound) = v.as_str().and_then(timestamp) else {
                return false;
            };
            if (lower && point < bound) || (!lower && point >= bound) {
                return false;
            }
        }
    }
    true
}
pub(crate) fn bounded(value: &Value, max_bytes: usize) -> Result<(), EngineError> {
    if serde_json::to_vec(value)
        .map_err(|e| EngineError::new("invalid_json", e.to_string()))?
        .len()
        > max_bytes
    {
        Err(EngineError::new(
            "resource_limit",
            "JSON input exceeds the kernel byte budget",
        ))
    } else {
        Ok(())
    }
}
pub(crate) fn tokens(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_lowercase())
        .collect()
}
pub(crate) fn number(value: &Value) -> Option<f64> {
    value
        .as_f64()
        .or_else(|| value.get("value").and_then(Value::as_f64))
        .filter(|n| n.is_finite())
}
