# Implementation contract

The corrected SAMI v2.0 PRD is the design source. This repository implements bounded native operational intelligence and research tooling. External customer validation, security assessment, benchmark targets and production certification are release gates, not software features to fabricate.

## Shared interfaces

`sami-intelligence` exports pure JSON interfaces returning `Result<Value, EngineError>` unless noted:

- `decide(task_id: &str, input: &Value, claims: &[Value], pack: &Value)`
- `parse(message: &str, state: &Value, pack: &Value) -> Value`
- `compose(plan: &Value, pack: &Value, previous: &[String]) -> Result<String, EngineError>`
- `evaluate_rules(rules: &Value, fields: &Value, budget: usize)`
- `plan(goal: &str, state: &Value, actions: &Value, max_nodes: usize)`
- `search(query: &str, documents: &[Value], limit: usize) -> Value`
- `roleplay(message: &str, state: &Value, persona: &Value) -> Value`

`sami-research` exports `pub fn dispatch(operation: &str, input: &Value) -> Result<Value, ResearchError>` for calibration, learning, synthesis, DSL execution, sequence indexing, semantic statistics, ingestion extraction, pack validation and bundle validation. Public functions should accept bounded JSON and reject unsupported/invalid operations.

API authorization extracts tenant from bearer credential or validated OIDC subject. Never body supplied tenant. All records are namespaced `(tenant, kind, id)`. `Source` JSON fields: `id`, `revision`, `status`, `epoch`, `content`, `acl` (scope labels), `purpose`, `license`, `source_family`, `permission_checked_at`, `max_age_seconds`, `valid_from`, `valid_to`. Source creator can set local-authority to make a user-managed source authoritative without needing an upstream connector.

`Claim` JSON fields: `id`, `subject`, `predicate`, `value`, `value_type`, `unit`, `source_id`, `source_revision`, `source_family`, `locator`, `status`, `valid_from`, `valid_to`, `system_from`, `system_to`, `depends_on` (claim ids), `authority`, `qualifiers` (serial/product/jurisdiction), `acl`, `purpose`.

Default domain pack: `id=industrial-service`, `version=1.0.0`, `language=en`, `tasks` array (`service_triage`, `warranty_prequalification`, other typed tasks), `rules` array, `responses` object, `workflows` object, `ontology`, `limits`. Agent producing this pack must include usable example assets, claims and source fixtures.

All experiments must distinguish generated demo data from locked real customer evaluation. Never report production or research gates as passed without actual evidence.

## File ownership

Root: Cargo root, `src/`, backend integration/security/storage/actions, integration tests and final ZIP.
Intelligence agent: `crates/sami-intelligence/`, `packs/`, native kernel tests only.
Research agent: `crates/sami-research/`, `evaluation/`, extraction/learning/synthesis examples only.
Console/deployment agent: `console/`, `sdk/`, `docs/` (except copied PRD and requirements status), `deploy/`, `.github/`, root README/license/contributing/security, scripts for deployment only. Do not modify root Cargo or backend files.
