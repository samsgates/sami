//! Bounded offline experiments. No function performs network calls, file writes or
//! production mutations. Returned candidates require the runtime's quarantine and approval.
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

mod calibration;
mod causal;
mod ingestion;
mod learning;
mod program;
mod semantic;
mod sequence;
mod validation;

#[derive(Debug, thiserror::Error)]
pub enum ResearchError {
    #[error("invalid input: {0}")]
    Invalid(String),
    #[error("resource limit: {0}")]
    Limit(String),
    #[error("unsupported operation: {0}")]
    Unsupported(String),
}

pub type Result<T> = std::result::Result<T, ResearchError>;
pub(crate) fn invalid(message: impl Into<String>) -> ResearchError {
    ResearchError::Invalid(message.into())
}
pub(crate) fn array<'a>(value: &'a Value, name: &str, max: usize) -> Result<&'a Vec<Value>> {
    let items = value
        .get(name)
        .and_then(Value::as_array)
        .ok_or_else(|| invalid(format!("{name} must be an array")))?;
    if items.len() > max {
        return Err(ResearchError::Limit(format!("{name} exceeds {max} items")));
    }
    Ok(items)
}
pub(crate) fn number(value: &Value, name: &str) -> Result<f64> {
    let x = value
        .get(name)
        .and_then(Value::as_f64)
        .ok_or_else(|| invalid(format!("{name} must be finite numeric")))?;
    if !x.is_finite() {
        return Err(invalid(format!("{name} must be finite")));
    }
    Ok(x)
}
pub(crate) fn text<'a>(value: &'a Value, name: &str) -> Result<&'a str> {
    value
        .get(name)
        .and_then(Value::as_str)
        .ok_or_else(|| invalid(format!("{name} must be a string")))
}
pub(crate) fn bounded_usize(
    input: &Value,
    name: &str,
    default: usize,
    maximum: usize,
) -> Result<usize> {
    let n = match input.get(name) {
        None => default,
        Some(v) => v
            .as_u64()
            .ok_or_else(|| invalid(format!("{name} must be an unsigned integer")))?
            as usize,
    };
    if n == 0 || n > maximum {
        return Err(ResearchError::Limit(format!(
            "{name} must be in 1..={maximum}"
        )));
    }
    Ok(n)
}
pub(crate) fn digest(value: &Value) -> Result<String> {
    let bytes = serde_json::to_vec(value).map_err(|e| invalid(e.to_string()))?;
    Ok(hex::encode(Sha256::digest(bytes)))
}

/// Each operation has a closed JSON contract; arbitrary code or question text is never executed.
pub fn dispatch(operation: &str, input: &Value) -> Result<Value> {
    if !input.is_object() {
        return Err(invalid("input must be a JSON object"));
    }
    let bytes = serde_json::to_vec(input).map_err(|e| invalid(e.to_string()))?;
    if bytes.len() > 2 * 1024 * 1024 {
        return Err(ResearchError::Limit("JSON input exceeds 2 MiB".into()));
    }
    match operation {
        "causal.analyze" => causal::analyze(input),
        "calibration.fit" => calibration::fit(input),
        "calibration.predict" => calibration::predict(input),
        "evaluation.metrics" => calibration::metrics(input),
        "evaluation.splits" => validation::splits(input),
        "drift.evaluate" => calibration::drift(input),
        "conformal.fit" => calibration::conformal_fit(input),
        "conformal.predict" => calibration::conformal_predict(input),
        "bandit.select" => learning::select(input),
        "bandit.update" => learning::update(input),
        "feedback.validate" => learning::feedback(input),
        "dsl.execute" => program::execute(input),
        "synthesis.search" => program::synthesize(input),
        "sequence.build" => sequence::build(input),
        "sequence.query" => sequence::query(input),
        "semantic.ppmi" => semantic::ppmi(input),
        "semantic.lsa" => semantic::lsa(input),
        "semantic.signature" => semantic::signature(input),
        "semantic.similarity" => semantic::similarity(input),
        "ingestion.extract" => ingestion::extract(input),
        "pack.validate" => validation::pack(input),
        "bundle.validate" => validation::bundle(input),
        "capabilities" => Ok(
            json!({"operations":["causal.analyze","calibration.fit","calibration.predict","evaluation.metrics","evaluation.splits","drift.evaluate","conformal.fit","conformal.predict","bandit.select","bandit.update","feedback.validate","dsl.execute","synthesis.search","sequence.build","sequence.query","semantic.ppmi","semantic.lsa","semantic.signature","semantic.similarity","ingestion.extract","pack.validate","bundle.validate"], "side_effects":false,"production_promotion":false,"general_intelligence_claim":false}),
        ),
        _ => Err(ResearchError::Unsupported(operation.into())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn closed_dispatch_rejects_unknown_and_non_objects() {
        assert!(dispatch("shell", &json!({})).is_err());
        assert!(dispatch("capabilities", &json!([])).is_err());
    }
}
