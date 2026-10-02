//! Pure bounded native intelligence. The caller owns authentication, tenant filtering,
//! durable state, source epochs and external effects. This crate never executes tools.
mod decision;
mod language;
mod parsing;
mod planning;
mod retrieval;
mod rules;
mod util;

pub use decision::decide;
pub use language::{compose, roleplay};
pub use parsing::parse;
pub use planning::{graph_query, plan};
pub use retrieval::search;
pub use rules::evaluate_rules;

use serde::Serialize;
use std::fmt;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct EngineError {
    pub code: String,
    pub message: String,
}
impl EngineError {
    pub(crate) fn new(code: &str, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }
}
impl fmt::Display for EngineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}
impl std::error::Error for EngineError {}

/// A bundled demo is generated fixture data, never a customer evaluation.
pub fn default_pack() -> serde_json::Value {
    serde_json::from_str(include_str!("../../../packs/industrial-service/pack.json"))
        .expect("the bundled pack is validated by integration tests")
}
