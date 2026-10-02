# Security policy

This project is pre-release software for bounded supervised workflows. Public release, deployment or a successful build does not certify its security. An independent review, realistic connector tests and documented risk acceptance are required before sensitive production use.

## Report a vulnerability

For a GitHub-hosted copy, use the repository's **Security → Report a vulnerability** private reporting feature when enabled. If no private channel is configured, contact the repository owner through a private channel already established with them. Do not publish real credentials, private records or an exploit affecting a live customer as a public issue. The maintainer must configure a security contact before distributing a production release; this file does not invent an email address or response-time promise.

Include the affected version, deployment mode, minimal synthetic reproduction, prerequisite privileges, expected/actual authority or data access, and likely impact. Scope includes tenant escape, unsafe tool authorization, stale approvals, poisoned-source promotion, data resurrection, replay tampering, parser/DSL escape, credential leakage and supply-chain defects.

## Deployment controls

Use TLS outside local loopback. Keep API/database/metrics private; provide a scoped OIDC/key credential; separate publisher and reviewer roles; secure encryption and Ed25519 signing keys; configure source rights/ACL propagation and freshness; restrict connector destinations/egress; test deletion, restore and emergency controls. OIDC must verify issuer, audience, expiry and signature; do not accept an unverified JWT payload as identity.

Bootstrap credentials are installation secrets. Exchange them for scoped operational credentials and restrict/remove bootstrap use according to the runtime's supported rotation procedure. Never use example placeholders or one key for both encryption and signing. Exported evidence and new-key creation results can contain sensitive material; govern those files.

## Incident containment

Disable effects and learning promotion; revoke affected credentials and source/bundle eligibility; preserve privacy-minimized audit metadata; identify impacted receipts/cases; reconcile uncertain external effects; repair/rollback approved artifacts; restore only after tombstone/epoch checks; rerun the relevant adversarial and regression tests. See [operations](docs/operations.md).

No automated scan establishes that all vulnerabilities are absent. Unresolved critical failures block release. Review high-severity findings and either fix them or formally restrict scope with an accountable risk decision; never hide a failed security job by removing its gate.
