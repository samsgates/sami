# Developer guide

## Ownership and authority

The runtime separates ingress interpretation, approved evidence, decision semantics, policy, durable execution and wording. `crates/sami-intelligence` implements bounded native operations; `crates/sami-research` provides pure bounded experiments; root `src/` owns API/authentication/storage/receipt/action integration. `console/` contains the real fetch workbench. `sdk/` contains independent TypeScript, Python and Rust clients. The domain pack is under `packs/industrial-service/`.

Documents and connector output are untrusted data. Only authenticated authorized control-plane operations can publish rules, tasks, workflows, tools or policy. A parsed instruction cannot create permission. The tenant comes from the validated credential, never a request-body tenant ID.

## Local cycle

Use Rust 1.86+, Node 22 and Python 3.10+. Run the native API against SQLite or PostgreSQL using the variables in [deployment](deployment.md). Build the console with `npm ci && npm run build`; its Vite development proxy sends `/v1`, `/health` and `/ready` to `http://127.0.0.1:8080`. The production API can serve the compiled bundle through `SAMI_CONSOLE_DIR`; Compose uses Caddy to serve static assets and proxy API calls on one origin.

Use `sh deploy/scripts/check.sh` for the local automated check sequence. Python/HTTP tests bind only a local loopback ephemeral port. Sandbox restrictions may require permission for loopback or package downloads; they are environment constraints rather than failed product behavior. Test outputs and dependency versions belong in the release evidence.

## Domain-pack authoring

Start from the complete generated industrial pack rather than an arbitrary ad hoc JSON shape. Tasks have registered IDs and explicit supported task types, schemas, mandatory fields/claims and output semantics. Rules use named priorities/effects and bounded typed conditions. Responses protect factual slots. Workflows declare allowed transitions/conditions/effects and costs. Source applicability checks include valid time, serial/product/jurisdiction and approved revision.

To add a task, define its label semantics, supported population, required evidence, costs, abstention conditions and locked evaluation set first. Implement or configure its known task type. Test missing input, conflicting sources, negation, wrong units, new entity, expired policy and unfamiliar requests. A free-form task question is not an implicitly trained model.

Save a pack as a candidate, validate via `POST /v1/research/pack.validate`, review the result, then publish with `POST /v1/packs/{id}/publish`. Successful schema validation does not establish decision quality or permission to deploy. Capture approver, version, change rationale, risk/coverage result and rollback artifact.

## Data and concurrency

Keep current record `revision` on updates and handle a conflict by reading current state, comparing changes and submitting a reviewed new version. Do not blindly overwrite a concurrent correction. Authoritative state transitions and outbox/event intent must be transactional. Derived indexes/caches are reconstructable and watermarked.

Claims need source revision, locator, authority, scope, units, valid/system time and dependencies. Store absent/unknown/disputed state explicitly; lack of a record is not proof of ineligibility unless a source completeness contract exists. Corrections increment source/policy eligibility epochs before asynchronous recomputation. Decisions and approvals check current epochs so backlog cannot authorize stale effects.

Historical replay uses recorded observations and pinned artifacts. Calling today's connector is a new evaluation. Deletion can make exact replay unavailable; the API must report that rather than resurrect erased content.

## External connector implementation

Register tool schema, scopes, destination, effect class, idempotency retention, timeout, reconciliation and compensation limits. Validate arguments and authorization at proposal and refresh authority/current external preconditions at commit. Destination-side conditional writes or fencing are necessary for strong protection against a remote race. Unsupported guarantees must be exposed and execution constrained.

For a possible effect followed by a timeout, persist `unknown_completion` and reconcile by operation ID/business key. Do not blindly replay. An internal ticket is a real durable SAMI record but does not stand in for a supported external vendor adapter.

Source connectors must preserve upstream per-record ACLs and group mappings, source revision/cursor, deletion and revocation events, permission-check timestamp/max-age, right-to-use records and watermarks. Restrict network destinations, prevent SSRF, isolate parsers, cap size/CPU and make failures visible.

## Console and SDK contracts

The console uses no simulated backend. Its API base is `/v1`; credentials are supplied by a human/identity provider and absent by default. Generic administration accepts `GET /admin/{kind}`, `GET /admin/{kind}/{id}` and `POST /admin/{kind}`; source, pack, receipt and action transitions use their dedicated routes. Keep protected transitions out of generic raw storage paths.

The SDKs make one request per explicit call, do not automatically retry effects, URI-encode resource IDs and require an execution idempotency key. Keep error status/details/request IDs; avoid printing keys or entire private payloads in diagnostics. Change all three clients and examples when the shared API contract changes.

## Quality and release

Run formatting, lint, core/native/research and integration tests, SDK tests, console build, dependency audits, container scans and parser/DSL fuzzing. Evaluate source applicability and risk/coverage independently from response fluency. A pleasing template or an accurate toy benchmark cannot waive a critical authorization, tenant, deletion or recovery failure. Customer validation, actual SLOs, independent security assessment and cross-domain research claims remain PRD release gates.

The release contract check verifies all numbered PRD IDs and routes are documented, and the console embeds the same approved demo pack/lab examples as the source. Update both JSON copies together; CI rejects drift.
