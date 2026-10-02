# Deployment

## Choose a profile

Use `compose.yml` for an empty production configuration backed by PostgreSQL. Run `sh deploy/scripts/init-env.sh`, then `docker compose up --build -d`. Open `http://127.0.0.1:8088` for a local deployment. The API and database have private Compose networks and no host ports. Production traffic must terminate at a trusted TLS reverse proxy. The loopback binding is intentional; change it only as part of your reviewed infrastructure configuration.

For generated examples use a separate installation: `docker compose -f compose.yml -f compose.dev.yml up --build -d`. It seeds the industrial pack and sample records, runs a reconciliation worker and sets development mode. The seeder preserves an already seeded demo; it refuses to replace a customer pack. Never use this overlay with a customer production database. A `--seed-demo` operation is disabled when `SAMI_ENV=production`.

For a production worker first provision a dedicated key with `action.reconcile`, appropriate source groups, and optionally `workflow.execute`. Store it as `SAMI_WORKER_KEY`, then use `docker compose -f compose.yml -f compose.worker.yml up -d`. Workers reconcile uncertain effects and release due timer waits; they do not execute arbitrary actions or retry remote writes automatically.

## Bare-metal setup

Build with Rust 1.86 and Node 22:

```sh
cargo build --release --locked
cd console
npm ci
npm run build
cd ..
```

For local generated examples:

```sh
export SAMI_ENV=development
export SAMI_DATA_DIR=./data
export SAMI_CONSOLE_DIR=./console/dist
# Set a securely generated installation key; do not use documentation placeholders.
export SAMI_BOOTSTRAP_KEY=$(openssl rand -hex 32)
cargo run --locked -- init --seed-demo
cargo run --locked -- serve --host 127.0.0.1 --port 8080
```

Open `http://127.0.0.1:8080/console/` and enter the bootstrap key from your private shell environment. An `init` without a configured bootstrap key returns a generated installation credential once. Preserve the development `.encryption.key` and `.signing.key` files separately; deleting them makes existing encrypted records unreadable.

For production, configure all keys explicitly, set a PostgreSQL URL, run `sami init` and deploy `sami serve --host 0.0.0.0 --port 8080` behind TLS. Prefer a managed PostgreSQL service with tested backup/restore. SQLite uses WAL and one writer connection for edge/local operation; this is not a high-concurrency cluster design.

| Environment variable | Contract |
|---|---|
| `SAMI_ENV` | `production` requires explicit encryption/signing keys and disables generated demo seeding. |
| `SAMI_DATABASE_URL` | `postgres://…` or `sqlite://…?mode=rwc`; keep credentials in secret storage. |
| `SAMI_ENCRYPTION_KEY` | Separate 64-character hexadecimal AES-256 key. |
| `SAMI_SIGNING_KEY` | Separate 64-character hexadecimal Ed25519 private seed. |
| `SAMI_BOOTSTRAP_KEY` | At least 32 characters; initial owner of the `default` tenant. Exchange for scoped keys. |
| `SAMI_DATA_DIR` | Writable runtime directory; development-only keys are generated with mode 600. |
| `SAMI_CONSOLE_DIR` | Compiled static console directory. |
| `SAMI_OIDC_ISSUER`, `SAMI_OIDC_AUDIENCE`, `SAMI_OIDC_JWKS_URL` | All are required for OIDC; JWKS uses HTTPS. RS256/ES256 tokens require provisioned subject mappings. |
| `SAMI_CONNECTOR_HOSTS` | Comma-separated exact HTTPS hosts permitted for HTTP tools. Private/reserved IPs and redirects are rejected. |
| `SAMI_CONNECTOR_SECRET_<NAME>` | Connector bearer credential referenced by the registered tool; never put the secret in source text or tool arguments. |
| `SAMI_WORKER_KEY` | Dedicated current scoped worker credential. |
| `SAMI_DEVICE_ID` | Required exact target identity for edge bundle import. |
| `SAMI_BUNDLE_TRUST_KEYS` | Comma-separated pinned issuer Ed25519 public keys for edge bundle import. |
| `SAMI_BUNDLE_ENCRYPTION_KEY` | Per-device 64-hex decryption secret, distinct from edge database/signing keys. |
| `RUST_LOG` | Default structured logs; avoid debug output with customer data. |

## Identity and tenants

Use `sami provision-key --tenant TENANT --principal SUBJECT --scopes scope1,scope2 --groups group1` on the trusted deployment host to provision another tenant. CLI database access is privileged operator access. API key creation remains inside the caller's tenant and cannot grant unowned scopes/groups. API keys are stored as SHA-256 commitments of random 256-bit credentials and can be revoked.

For OIDC, an owner provisions an `identities` resource with `subject`, `scopes`, `groups`, and optional `disabled:true`. The ID is derived from configured issuer plus subject. Tenant and scopes come from this authenticated mapping, not token body tenant claims. The console accepts a short-lived bearer obtained through your identity provider's approved login; this version does not implement a password server or a full browser PKCE login flow. JWKS cache lasts five minutes; coordinate signing-key rotation with token lifetimes/cache refresh.

A reviewer key usually needs `decision.invoke`, `knowledge.search`, `receipt.read`, `receipt.replay`, `session.read`, `session.write`, and the appropriate `service.read` group. Administrative and effect scopes are separate. `*` is an installation owner role and includes purpose/ACL administration; do not distribute it to ordinary users.

## Kubernetes and release

`deploy/kubernetes/` supplies deployment, services, probes, PVC, ingress and ingress NetworkPolicies. Configure a real image registry/digest, `sami-secrets`, an external PostgreSQL URL, storage class, ingress hostname/TLS secret and reviewed egress rules. Placeholder images/hosts are deployment inputs, not actual public deployments. The recipe has not been applied to a cluster in this package's verification run. Production availability, key rotation, network policy enforcement and failover need infrastructure acceptance.

CI builds source, checks PostgreSQL, audits dependencies, creates an image SBOM and gates container findings. Docker Engine was unavailable during local package verification, so container build/scan is an explicit outstanding check. `docs/verification.md` records actual completed checks. Register a private vulnerability-reporting contact and validate the supported connector against its destination before a sensitive production rollout.

See [offline device tutorial](offline.md) for independent device keys and pinned public trust.
