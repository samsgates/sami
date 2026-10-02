# Industrial service native pack

`pack.json` is a runnable, generated domain configuration. `fixtures.json` contains
two generated assets, two approved local sources, fourteen sourced claims, and
three complete operational example inputs. These are demo fixtures, not evidence
of customer quality or production release qualification.

The supported native tasks are `service_triage`, `warranty_prequalification`,
`financial_objection` (boolean), `objection_type` (choice), `case_flags`
(multi-choice), `simulation_readiness` (score), `next_step_rank` (rank),
`extract_case_fields` (extract), and `next_action` (finite action recommendation).
The score task is explicitly a supplied simulation value, not a purchase forecast.
All probabilities/calibration references remain null: these deterministic tasks
have no trained, independently validated probabilistic artifact.

## Operational call

Load the pack and fixture JSON. Add `fixture.sources` to an example input, then
call `decide(example.task, input, fixture.claims, pack)`. The root API performs
tenant/ACL/purpose filtering and an epoch-snapshot transaction around this call.
The kernel independently rejects unapproved claims, outdated source revisions,
expired authority where a source lease exists, invalid temporal intervals, missing
or cyclic derivation parents, ambiguous asset assertions, and conflicting values.

Date inputs are ISO dates/timestamps. `as_of` specifies real-world applicability;
`known_at` specifies recorded knowledge time. A missing `known_at` defaults to
`as_of`. Intervals are `[from,to)`. Warranty duration uses calendar months with
an exclusive expiry date and an inclusive operating-hours maximum. The generated
policy is limited to product PX320 in jurisdiction IN; a different pack/policy
requires authorized publication and separate evaluation.

No decision from this crate executes or authorizes tools. `authorized_effects` is
always empty. Warranty results always require the designated final reviewer.
Service triage is categorical routing, not mechanical diagnosis or repair advice.

## Other pure interfaces

- `parse(message,state,pack)`: returns unconfirmed fields, concept/intent support,
  original text offsets, alternative parses and ambiguity reasons. `state.assets`
  overrides the pack's asset lookup. A relative newer/older reference requires
  exactly two known assets with distinct years. Locale-dependent slash dates are
  intentionally not silently resolved.
- `search(query,documents,limit)`: documents have `id`, `content` (or `text`),
  optional `title`, `semantic_aliases`, and source metadata. Results include BM25,
  normalized BM25 and sparse approved-alias cosine components. Fusion weights are
  authored ranking defaults, not calibrated relevance probabilities. Caller must
  prefilter access; this pure function does not own identity or storage.
- `evaluate_rules(rules,fields,budget)`: three-valued conditions use `all`, `any`,
  `not`, or `{field,op,value}`. Missing operands remain unknown, including under
  negation. Effects are allow/deny/set/require_review. Deny overrides every allow;
  equal-priority incompatible updates conflict. The budget counts condition nodes.
- `plan(goal,state,registry,max_nodes)`: registry shape is the pack's
  `workflows.service_case`: `{actions:[{id,preconditions,effects,cost}],goals:{...}}`.
  Uniform-cost search returns a bounded process simulation and commit-required
  steps, never performs an effect. This does not infer physical consequences.
- `graph_query(query,claims,max_hops)`: query includes `start`, optional `target`,
  `predicate`, `inverse`, and `as_of`. Returns source-linked relation paths;
  temporal claims are omitted when no query time is supplied. Caller must supply
  only authorized evidence. Path tracing does not prove external truth.
- `compose(plan,pack,previous)`: plan has `action`, authorized `slots`, optional
  `hidden_slots`, and `max_characters`. Every placeholder must be supplied;
  restricted or missing slots fail closed. Repetition control selects another
  approved template without modifying protected numbers or identifiers.
- `roleplay(message,state,persona)`: use `pack.roleplay_persona`. This finite sales
  simulation reveals at most one configured pain point per turn and refuses
  postponed-information behavior when configured. Trust is labeled simulated and
  never presented as measured psychology of a real customer.

Run `cargo test -p sami-intelligence` for integrated parsing, temporal/source
failure, policy, retry-independent planning, slot-disclosure and task-shape tests.
