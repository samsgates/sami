# API guide

This is the implemented API contract for v0.2.0. The [OpenAPI descriptor](openapi.json) inventories all routes and documents shared envelopes. Per-domain schemas are in the reviewed pack, tool registry and operation fixtures; arbitrary `Record` input is still validated by the server. `/v1/openapi` serves the same descriptor to authenticated clients.

Use an `Authorization: Bearer <scoped credential>` header. Tenant comes from the authenticated identity. Keys and OIDC subject mappings are revocable; body tenant IDs never grant authority. `*` is an installation administrator. A resource scope alone does not bypass current source ACL, purpose or owner checks. Reviewers who need to inspect another actor's case also need the explicit `data.read_all` capability and appropriate source groups. HTTP bodies are limited to 4 MiB. The default credential rate limit is 600 authenticated requests per minute.

## Routes and authority

| Route | Required scope |
|---|---|
| `GET /v1/capabilities` | authenticated |
| `GET /v1/me` | authenticated |
| `GET /v1/openapi` | authenticated |
| `GET /v1/admin/{kind}` | admin.{kind}.read |
| `POST /v1/admin/{kind}` | admin.{kind}.write |
| `GET /v1/admin/{kind}/{id}` | admin.{kind}.read |
| `GET /v1/admin/{kind}/{id}/history` | admin.{kind}.read |
| `DELETE /v1/sources/{id}` | sources.delete |
| `POST /v1/sources/{id}/publish` | sources.publish |
| `POST /v1/sources/{id}/retract` | sources.retract |
| `POST /v1/sources/{id}/permissions` | sources.permissions |
| `POST /v1/claims/candidates` | admin.claims.write |
| `POST /v1/packs/{id}/publish` | packs.publish |
| `POST /v1/decisions` | decision.invoke |
| `POST /v1/decide` | decision.invoke |
| `POST /v1/respond` | session.write + decision.invoke |
| `POST /v1/chat/completions` | session.write + decision.invoke |
| `GET /v1/receipts/{id}` | receipt.read |
| `POST /v1/receipts/{id}/replay` | receipt.read + receipt.replay |
| `POST /v1/actions/propose` | action.propose + tool scopes |
| `POST /v1/actions/{id}/approve` | action.approve |
| `POST /v1/actions/{id}/execute` | action.execute + tool scopes |
| `POST /v1/actions/{id}/reconcile` | action.reconcile |
| `GET /v1/actions/{id}` | action.read |
| `POST /v1/feedback` | feedback.write |
| `POST /v1/feedback/{id}/promote` | learning.promote |
| `POST /v1/sessions` | session.write |
| `GET /v1/sessions/{id}` | session.read |
| `POST /v1/knowledge/search` | knowledge.search |
| `POST /v1/memory/query` | knowledge.search |
| `POST /v1/memory/write` | admin.claims.write |
| `POST /v1/graph/query` | knowledge.search |
| `POST /v1/research/{operation}` | research.execute |
| `GET /v1/keys` | keys.manage |
| `POST /v1/keys` | keys.manage |
| `POST /v1/keys/{id}/revoke` | keys.manage |
| `GET /v1/audit` | audit.read |
| `GET /v1/events` | audit.read |
| `GET /v1/export` | data.export |
| `POST /v1/bundles/export` | bundle.export |
| `POST /v1/bundles/import` | bundle.import |
| `POST /v1/workflows/{id}/run` | workflow.execute |
| `POST /v1/workflow-runs/{id}/advance` | workflow.execute |
| `POST /v1/workflow-runs/{id}/cancel` | workflow.execute |
| `POST /v1/roleplay` | simulation.invoke |
| `POST /v1/policy/diff` | evaluation.execute |
| `GET /health` | admin.{kind}.read |
| `GET /ready` | admin.{kind}.read |
| `GET /metrics` | admin.{kind}.read |

## Evidence and records

List supported kinds from `src/service.rs::KINDS`. `GET /v1/admin/{kind}?limit=100&offset=0` returns `items`, `limit`, `offset`, `next_offset`; pages are capped at 500. ACL filtering may shorten a page, so use offsets rather than interpreting an empty filtered page as the end of the underlying collection. POSTing a new record uses no revision or revision 0. Updating an existing record requires its exact returned numeric revision. Preserve that revision; a stale update returns 409. Dedicated lifecycle records such as actions and receipts cannot be replaced through the generic editor. Historical source-dependent data is also checked against current ACLs.

A candidate source example:

```json
{"id":"reviewed-source-1","title":"Service notice","content":"Reviewed domain content","license":"authorized customer use","source_family":"manufacturer.manual","purpose":["service_prequalification"],"acl":["service.read"],"local_authority":true}
```

`local_authority` is an operator assertion that SAMI owns current source permission, not a convenience bypass for an upstream document. An upstream source needs a fresh permission timestamp and a maximum age no greater than 300 seconds. Source save produces candidate status. Publication requires a reason and authorized scopes:

```json
{"reason":"Reviewed against original notice","claims":[{"id":"claim-1","subject":"asset:example","predicate":"operating_hours","value":900,"unit":"h","locator":"table 2 row 3","valid_from":"2026-01-01"}]}
```

Updating a pre-existing claim during publication also requires its current revision. A claim inherits the published source revision, purpose and ACL. Do not reuse another source's claim identity. Retraction/correction conservatively invalidates dependent eligibility. Deletion physically purges private derivatives in the active store, including other tenant-derived free text where lineage cannot be established precisely. See [operations](operations.md) for backups and disconnected copies.

## Decision, conversation and receipt

```json
{"task_id":"service_triage","pack_id":"industrial-service","input":{"message":"Serial 1600 is overheating","asset_id":"asset:unit_84","as_of":"2026-10-02","known_at":"2026-10-02T06:00:00Z"}}
```

The output includes typed `status`/`value`, evidence diagnostics, checks, a reviewed `response`, `decision_id`, `receipt_id`, source revisions/epochs and eligibility. Rule outcomes have no invented predictive probability. A successful HTTP request can still return an abstention status; applications must inspect that status before proposing an effect. As-of time and known-at time are distinct. Never submit example asset data for a real customer.

Create a session with `POST /v1/sessions {}`. Send `POST /v1/respond` with `session_id`, `message`, optional `fields`, `task_id`, `pack_id` and `revision`. The returned revision should be included in the next turn. Session fields are unconfirmed assertions; an authoritative asset claim still controls warranty checks. A session supports at most 200 turns.

`GET /v1/receipts/{id}` verifies and inspects a signed historical envelope. POST `/replay` recomputes the pinned structured result without executing tools. Current eligibility and historical integrity are separate. Deleted private payloads return a typed not-replayable result; a signature does not certify external truth.

## Action lifecycle

Propose an action with `decision_id`, `tool_id` and typed `arguments`. The generated tool `internal.ticket` accepts string `title`, `description`, `queue`. A reviewer approves with `reason` and optional `expires_in_seconds` (30..3600). Execute with a required stable `Idempotency-Key` of 8..255 characters. The key is immutable for that action; a changed retry key returns 409. Internal ticket creation and successful outcome are one transaction. For the HTTP adapter, destination-enforced idempotency and precondition support are required parts of the operator's connector contract. No automatic resend follows an ambiguous write.

Tool schema validation currently supports typed top-level objects, required properties, enum, unknown-field rejection and string length. It does not claim full recursive JSON Schema validation. Register only schemas within this supported subset. The HTTP adapter requires an exact allowlisted HTTPS host, public DNS addresses, no redirects, an environment-bound bearer credential and a reconciliation endpoint. It has not been certified against a real third-party vendor in this package.

Execution checks current actor and approver authority, source/policy revisions, approval expiry, control flags and external preconditions. Remote timeouts enter `unknown_completion`; reconcile before retrying. Current source changes block queued effects, while already sent remote writes can require manual follow-up.

## Workflow and research

Start an approved workflow with optional structured `state` and `goal`. Advance a run with current `revision`, `reason` and exact planned `step`, or a state-machine `transition`. Optional `wait_seconds` (1..86400) persists a timer. `report_failure:true` persists bounded retry/failure state. The worker releases due timer waits. A confirmation changes simulated/process state and does not execute an external tool. Cancellation is durable; a changed definition requires a new run.

`/v1/research/{operation}` accepts the matching object in `examples/research/operations.json`. The closed operation registry exposes calibration, metrics/splits, drift/conformal, safe bandits, feedback validation, bounded DSL/synthesis, sequence/semantic indexes, causal diagnostics, extraction, pack/bundle validation. These operations cannot perform network/file/tool effects. Results remain experimental until domain validation and review; fitted artifacts do not automatically replace production decisions.

## Compatibility, streams and errors

`/v1/chat/completions` is a native compatibility shape, with reviewed wording and a bounded single content event plus `[DONE]` for `stream:true`. It is not an arbitrary hosted language model. Unsupported generation fields are rejected and token usage is null. `/v1/events` returns a bounded audit page (up to 1000); resume using `Last-Event-ID` or `after`. It is not a continuously pushed socket for all intermediate decision/action states.

Errors use `{"error":{"code":"…","message":"…","request_id":"…"}}`. 401 means missing authentication; 403 denied or unavailable authority; 404 hides unavailable resources; 409 requires reading current state or a new approval; 429 requires backoff; 503 requires recovery of a dependency/budget. SDKs expose status, code and request ID and do not blindly retry mutations. Effect retries have durable action idempotency. Other mutations use resource revision checks; this version does not have a universal mutation idempotency ledger.
