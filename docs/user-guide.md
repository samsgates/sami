# User guide

## Connect and understand the scope

Open the console on your trusted deployment origin. **Connect credential** accepts a scoped API key or a bearer token issued by the configured OIDC provider. SAMI does not run a local password login. The credential is kept in memory by default. Optional tab session storage survives refresh, but remains accessible to scripts on that origin. Disconnect clears it. Authorization is checked by the server for every request; the green connection label means a credential was supplied, not that every scope has been verified.

The first workflows are service routing and warranty prequalification. A qualified human owns mechanical diagnosis, repair instructions, final coverage and financial decisions. The native parser supports its configured English ontology; use explicit structured fields when wording or identity is ambiguous. A missing fact, stale source or policy conflict must be resolved rather than guessed.

## Prepare evidence

1. In **Evidence & memory → Sources**, load existing records or create a candidate. Set its content, source family, effective dates, purpose, license/rights and ACL labels. A locally managed source can be marked `local_authority`; only an authorized owner should do this. An upstream connector instead needs permission/freshness information.
2. Review extraction and applicability. Approved records must refer to the correct product, serial scope, region and source revision. In **Claims**, inspect subject/predicate/value, units, source locator, dependencies and temporal interval.
3. Select the source, enter a review reason and **Publish source**. Publication is distinct from saving a candidate. Source publication and claim promotion require the appropriate server scopes.
4. Correct a source by editing the returned current record/revision, saving the revision and publishing after review. **Retract source** invalidates its future dependent eligibility. The runtime preserves historical receipts; already completed external effects require follow-up.
5. For deletion, review the selected ID and impact, select the acknowledgment checkbox, then **Delete selected source**. Serving exclusion, physical purge, backup retention and disconnected copies have separate contracts. Confirm actual completion through API/audit and deletion tests.

Generic record editors preserve backend fields. Keep the returned numeric `revision` for optimistic concurrency. Changing a claim to `retracted` is a governed write, not removal of history. Do not mark a claim approved merely to make a demonstration succeed.

## Run a case

In **Case workbench**, enter the customer description, confirmed entity ID, as-of date, available identifiers, operating hours and jurisdiction. Select the registered task and approved domain pack. **Evaluate case** sends a real decision request. User assertions do not automatically replace approved entitlement evidence.

Generated fixtures include `asset:unit_42` / serial `1200` and `asset:unit_84` / serial `1600`, product `PX320`, jurisdiction `IN`. Inspect `packs/industrial-service/fixtures.json` for complete applicable sample claims; these examples are not real warranty policies. Choose a date supported by their effective intervals.

Inspect the result's typed status, value, missing evidence, conflicts and checks. `insufficient_evidence`, `conflicting_evidence`, `stale_evidence` and `unsupported` are useful outcomes, not reasons to invent an answer. **Inspect evidence receipt** reads the recorded source/task/policy state. A receipt establishes what was checked relative to its inputs; it does not prove that external documents are true.

For multi-turn state, expand **Persistent conversation**, create a session and send a description. Reuse its ID for subsequent turns. Explicit revisions guard concurrent updates. Conversation never bypasses the separate action-authority checks.

## Review an administrative action

1. In **Action approvals**, enter the recorded decision ID, registered tool ID and exact arguments. The built-in `internal.ticket` tool uses string fields `title`, `description`, `queue`; it creates a SAMI internal ticket rather than pretending to integrate with an external service vendor.
2. **Validate proposal** creates durable proposed action state. Read that state and inspect scope, policy/source dependencies and changed fields.
3. An authorized reviewer supplies the reason and **Approve for 5 minutes**. A stale or changed action requires new approval.
4. Execute with a stable idempotency key. Retain the same key for a deliberate retry of the same action; never reuse it for a changed payload.
5. A timeout is ambiguous: read the action and **Reconcile outcome** before resending. The UI intentionally does not retry writes automatically. An uncertain remote effect must not be reported as success.

Use the **Action queue** to inspect current records. Final warranty payment and physical equipment control are not registered supported effects.

## Author, evaluate and administer

**Rules & domain packs** stores candidate tasks/rules/workflows and complete packs. A standalone rule does not automatically modify the active pack. Edit the complete pack, validate it with `pack.validate`, evaluate held-out cases, then use **Publish domain pack** with an authorized review reason. Retain a compatible prior version for active workflows/replay.

**Decisions & receipts** supports explicit registered task JSON and pinned replay. **Research & evaluation** exposes bounded operations with runnable generated examples. Do not interpret a four-item calibration or synthesis demo as deployment evidence. **Feedback** records attributable outcomes in quarantine; positive feedback does not prove correctness.

**Access & operations** shows health/readiness, runtime controls, key metadata and audit. Creating a key returns its secret once—store it securely. Read current controls before editing them; preserve the revision. Set `effects_enabled:false` to stop new tool effects and `learning_enabled:false` to stop promotion. Changing these flags needs administrator authority and must be audited.

## Offline use and export

A local/edge host can run the native runtime without a remote language API. Approved bundle scope, freshness and authority expiry still apply. External connector writes remain drafts when a destination or authority is unreachable. Reconnection requires conflict and outcome reconciliation. Never describe disconnected data as guaranteed current.

**Export JSON** saves the actual selected result, including evidence or new key material if present. Check data classification before sharing. Do not export private source records or credentials to public reports.
