# Operations and recovery

## Health, logs and bounded capacity

`/health` checks process availability. `/ready` verifies the authoritative store connection. Readiness does not assert that a tenant has an approved pack or that customer release gates passed. `/metrics` requires a bearer with `metrics.read`; it exposes process request/decision counters. Tracing logs HTTP request processing and sanitized errors. This version does not supply a full OpenTelemetry collector, production dashboard or measured SLO.

Serving supports at most 5,000 sources and 10,000 claims per native snapshot. Repair also caps active decisions/actions at 10,000 and refuses publication if it cannot repair safely. Admission permits four bounded CPU tasks. Kernel calls have a 500 ms request timeout; research calls five seconds. Timed-out blocking work retains its admission permit until its own bounded computation ends. Do not describe this as a hard process-kill sandbox. Large tenants need tested indexing/partitioning before changing these bounds.

## Correct evidence and contain an incident

Read current source/claim/control revisions before changing them. Sources remain candidates until an authorized publication with a reason. Publication/retraction changes an atomic tenant epoch, disputes affected claims, invalidates current decisions and blocks queued approvals. A receipt's historical computation can remain valid while its action eligibility is invalidated. Completed remote effects cannot be retroactively erased.

Read `controls/runtime` and preserve its revision. Set `effects_enabled:false` to suspend new effects, `learning_enabled:false` to suspend promotion, or `sources_enabled:false` to deny current source serving. Revoke affected API credentials; revocation serializes with dispatch. Disable an OIDC mapping or registered connector. Reconcile uncertain external operations and record reviewed follow-up. There is no universal rollback for external systems.

## Effects and uncertainty

The internal ticket adapter writes ticket, succeeded state and signed effect receipt in one transaction. HTTP tools require allowlisted HTTPS, schema/scopes, declared destination idempotency and fresh approval. Requests use `sami_<action_id>` as the destination operation key. `resource_version` is sent as `If-Match` when registered in the schema. A source/actor/policy check is serialized at local dispatch; a remote conditional-write race is only closed when the destination enforces its own precondition/fence.

A timeout/invalid response enters `unknown_completion`. Reconciliation uses a configured HTTPS endpoint and the same connector credential; it must return authoritative `confirmed:true` evidence before success. An inconclusive query leaves an uncertain state. Never blindly resend a remote write. Declare the destination's idempotency retention and reconciliation semantics in its connector contract. No vendor-specific adapter or compensation guarantee is claimed.

## Deletion and retention

Source deletion publishes a tombstone/epoch fence, invalidates future authority and purges active source/claim versions and replay payloads. To avoid leaving private free-text derivatives with incomplete lineage, it also conservatively purges the tenant's decision/action/ticket/session/workflow/feedback/procedure/evaluation derivative payloads. This is broader than deleting one source; assess that impact before invoking deletion. Minimal signed receipt/audit envelopes remain. Replay returns `not_replayable_due_to_deletion` when sensitive snapshots are purged. Existing tombstones prevent ingestion/bundle resurrection of the source ID.

The operator owns retention of downloaded exports, disconnected devices, old backups and object storage. This version has no automatic universal deletion of copies outside its database, no fully deployed OCR sandbox and no automatic legal-compliance certification. Configure backup expiry (PRD proposed default 30 days), backup protection and independent deletion-ledger retention for your environment.

## Backup and restore

For a portable encrypted backup:

```sh
sami backup --output sami-backup.json --tombstones-output latest-tombstones.json
```

Both paths must be new files and are written privately. Backup captures a consistent database snapshot, encrypted record history, minimal audit chain and hashed credentials. It signs the manifest and encrypts the backup with AES-256-GCM. The portable path caps each table at 100,000 rows; larger stores use database-native backup tooling. Preserve encryption/signing keys separately in secure secret storage.

Restore into a new empty database with the same trusted keys, using the newest independently retained deletion ledger, which may be newer than the backup:

```sh
sami restore --input sami-backup.json --tombstones-file latest-tombstones.json
sami doctor
```

A ledger is a JSON array of `{ "tenant":"TENANT", "source_id":"ID" }`. The restore transaction applies it before returning readiness eligibility. It refuses nonempty databases, invalid signatures/checksums and incompatible formats. An old backup's own deletion list cannot capture deletions that occurred later. Keep the newest ledger outside the backup rotation. Never infer that an empty supplied ledger proves no later deletion occurred.

`deploy/scripts/backup-postgres.sh` creates a database-native custom dump through Compose; encrypt it and restore into an isolated instance with `pg_restore`. It does not replace independently retained keys, artifact storage or deletion ledgers. Perform a restore drill and check approved source scope, epoch history, deleted content, credential revocation, replay availability, queued approvals and uncertain remote outcomes before re-enabling traffic. RPO/RTO targets in the PRD have not been measured here.

## Version and key changes

Current schema is v1, initialized through idempotent table creation. Future structural changes require explicit versioned forward/recovery migrations. Record history pins replay inputs; pack publication invalidates queued authority. A changed durable workflow definition currently requires a new run rather than silently migrating an old one. Reconstruct candidate rollback from an authorized historical record and publish it as a new reviewed revision.

This source version does not implement transparent encryption-key rotation or a multi-key receipt trust store. Preserve old key material and test a controlled migration; simply replacing the encryption environment variable makes old records unreadable. A new signing key cannot verify old receipts through the runtime's current verifier without an explicit trust-store migration. Public Ed25519 verification is available in `crypto::verify_public` for independently pinned historical keys.
