# SAMI

**Correctable memory. Accountable decisions. Controlled actions.**

SAMI is a native, CPU-first operational intelligence runtime and workbench. Its first domain is industrial service triage and warranty prequalification. Approved evidence, temporal claims, typed tasks, policy checks and durable action state belong to the runtime. Language is a presentation layer; it never grants permissions.

This repository contains a bounded implementation with executable native examples and research routines. A runnable system is not evidence of customer fit, a 99.9% service level, a passed independent security assessment or superiority over frontier models.

## Repository contents

- Rust API/runtime with PostgreSQL or local SQLite storage, scoped credentials and controlled execution.
- Native parsing, evidence-driven decisions, rules, templates and the versioned [industrial domain pack](packs/industrial-service/pack.json).
- Correctable source/claim lifecycle, historical decision receipts and replay, administrative-action approval and reconciliation.
- React/TypeScript console for cases, evidence, pack authoring, decisions, actions, research, feedback, audit, credentials and runtime controls.
- Bounded calibration, evaluation, drift, conformal prediction, causal diagnostics, safe bandits, program synthesis, DSL, sequence and semantic research operations.
- Python, TypeScript and Rust SDKs; Compose, nonroot images, Kubernetes deployment recipes and CI quality/security gates.

## Prerequisites

Choose either the Docker path or the local development path.

**Docker Compose:** Docker Engine or Docker Desktop with Compose v2, and OpenSSL.

**Local development:** Rust 1.86 or newer with Cargo, Node.js 22 and npm, and OpenSSL. Python 3.10+ is needed for Python SDK tests and Python examples. A local PostgreSQL server is optional; SQLite is sufficient for a single-user development instance.

Clone the repository and run commands below from its root:

```sh
git clone https://github.com/samsgates/sami.git
cd sami
```

## Run with Docker Compose

### Local demo

This profile seeds generated demonstration data and enables development behavior. Never use it with customer or production data.

```sh
sh deploy/scripts/init-env.sh
docker compose -f compose.yml -f compose.dev.yml up --build -d
docker compose -f compose.yml -f compose.dev.yml ps
curl --fail http://127.0.0.1:8088/health
```

Open [http://127.0.0.1:8088](http://127.0.0.1:8088). In **Connect credential**, enter `SAMI_BOOTSTRAP_KEY` from your private `.env` file, or use an appropriately scoped credential. Treat the bootstrap key as a secret and do not paste it into source files, issue trackers or chat. The console keeps credentials in memory unless tab session storage is explicitly selected. SAMI does not provide a local password login.

The generated demo pack and fixtures are synthetic. See [the user guide](docs/user-guide.md) for a guided case, evidence review and internal-ticket workflow. The demo creates an internal SAMI ticket, not a ticket in an external vendor system.

Stop the demo while preserving its data with:

```sh
docker compose -f compose.yml -f compose.dev.yml down
```

`docker compose down -v` removes the database and runtime volumes permanently. Do not use it if you need the stored data.

### Empty Compose deployment

For a fresh installation with no generated sample records, initialize secrets and start the base profile:

```sh
sh deploy/scripts/init-env.sh
docker compose up --build -d
docker compose ps
curl --fail http://127.0.0.1:8088/health
```

Open [http://127.0.0.1:8088](http://127.0.0.1:8088). The base profile uses production mode and does not seed demo data. Store `.env` securely; the initializer refuses to overwrite an existing file. The database and API have no public host ports. The console binds to loopback by default and proxies API requests on the same origin. This is a local deployment recipe, not a claim of production readiness. Configure trusted TLS, identity, backups, monitoring and reviewed infrastructure before serving users. Read [deployment](docs/deployment.md) and [operations](docs/operations.md) before deploying.

## Develop locally

### Install and initialize

Build and install the console dependencies:

```sh
cd console
npm ci
cd ..
```

Configure a development runtime using SQLite. Development mode creates local encryption and signing keys under `./data` if they are not explicitly set. Keep that directory private and backed up; losing those keys makes encrypted local records unreadable.

```sh
export SAMI_ENV=development
export SAMI_DATABASE_URL='sqlite://sami.db?mode=rwc'
export SAMI_DATA_DIR='./data'
mkdir -p "$SAMI_DATA_DIR"
umask 077
if [ ! -f "$SAMI_DATA_DIR/bootstrap.key" ]; then openssl rand -hex 32 > "$SAMI_DATA_DIR/bootstrap.key"; fi
export SAMI_BOOTSTRAP_KEY="$(cat "$SAMI_DATA_DIR/bootstrap.key")"
cargo run --locked -- init --seed-demo
```

The bootstrap credential is available in the current shell as `$SAMI_BOOTSTRAP_KEY`. Keep it secret. The demo seeder is disabled in production mode and must only target disposable generated data.

### Start the API and console

Start the API in the first terminal, from the repository root:

```sh
export SAMI_ENV=development
export SAMI_DATABASE_URL='sqlite://sami.db?mode=rwc'
export SAMI_DATA_DIR='./data'
export SAMI_BOOTSTRAP_KEY="$(cat "$SAMI_DATA_DIR/bootstrap.key")"
cargo run --locked -- serve --host 127.0.0.1 --port 8080
```

The generated bootstrap credential is stored in the ignored `data/bootstrap.key` file so both terminals can load the same value. Keep the file private. In a second terminal, run the Vite console:

```sh
cd console
npm run dev
```

Open [http://127.0.0.1:5173](http://127.0.0.1:5173). Vite proxies `/v1`, `/health` and `/ready` to the API at port 8080. Alternatively, build the static console and open the API-served console at [http://127.0.0.1:8080/console/](http://127.0.0.1:8080/console/):

```sh
cd console
npm run build
cd ..
export SAMI_CONSOLE_DIR='./console/dist'
```

Set `SAMI_CONSOLE_DIR` before starting the API for the compiled console. The Vite development server and the compiled static console are separate ways to serve the UI.

### Run a generated example

With the local seeded API running, set `SAMI_API_KEY` in your shell to the generated demo credential, then run an example:

```sh
export SAMI_API_URL='http://127.0.0.1:8080/v1'
# Set SAMI_API_KEY to your private local demo credential; do not commit it.
python3 examples/complete_case.py
```

This example runs a generated service case, verifies receipt replay and exercises approval/idempotency for an internal ticket. It is not a customer-policy evaluation. More Python, TypeScript, Rust, HTTP, ingestion, workflow, bundle and research examples are documented in [examples/README.md](examples/README.md).

## CLI and common tasks

The CLI reads runtime configuration from environment variables. Get the exact options for this checkout with:

```sh
cargo run --locked -- --help
cargo run --locked -- serve --help
```

Available command families include `serve`, `init`, `worker`, `doctor`, `evaluate`, `ingest`, `replay`, `provision-key`, `validate`, `publish`, `export`, `backup` and `restore`. Commands that modify data or invoke effects require the relevant credentials and scopes. Review [API](docs/api.md), [SDK](docs/sdk.md), [developer guide](docs/developer-guide.md) and [operations](docs/operations.md) before integrating them. Workers should use a dedicated least-privilege `SAMI_WORKER_KEY`, never a general bootstrap key.

## Test and verify

Run the repository's local automated check sequence:

```sh
sh deploy/scripts/check.sh
```

It checks repository assets, Rust formatting and Clippy, Rust workspace tests, console tests/build, TypeScript SDK tests, Python SDK tests and Rust SDK tests. It installs locked JavaScript dependencies with `npm ci`. It does not establish customer acceptance, production load/SLOs, an independent security assessment or a completed container scan. Inspect [verification results](docs/verification.md) for checks actually run and outstanding release gates.

Useful focused checks include:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
(cd console && npm ci && npm test && npm run build)
cargo test --locked -p sami-research --test operation_contracts
```

For PostgreSQL integration testing, use a fresh isolated database and the `SAMI_TEST_POSTGRES_URL` procedure in [verification](docs/verification.md). Do not point tests at a production database.

## Operating and security guidelines

- Keep `.env`, API keys, signing/encryption keys, source documents, local databases and customer evaluation data out of version control and logs. `.env` is ignored by Git; `.env.example` contains placeholders only.
- Do not expose the loopback Compose console directly to a network. Put a reviewed TLS reverse proxy and approved identity configuration in front of an externally reachable deployment.
- Use scoped credentials for people and workers. Exchange the bootstrap credential for scoped keys and revoke credentials when no longer needed. Tenant identity and permissions come from validated credentials, not request-body fields.
- Preserve `SAMI_DATA_DIR` and its key files across restarts. Back up encryption/signing keys separately and test restoration on an isolated installation before upgrades.
- Use only data with documented rights, purpose and access scope. Customer data is not automatically eligible for shared/global learning. Do not combine the demo overlay with customer data.
- Treat missing, stale or conflicting evidence as a review condition. Generated wording and user assertions do not establish facts or grant authority.
- Require explicit human review for supported administrative actions. SAMI does not enable physical equipment control, hazardous repair instructions or financial payouts.
- Research candidates and feedback do not automatically become production policies. Retain versioned packs and evaluate changes before publication.

See [SECURITY](SECURITY.md), [deployment](docs/deployment.md), [offline use](docs/offline.md), [user guide](docs/user-guide.md) and [CONTRIBUTING](CONTRIBUTING.md) for detailed procedures. Apache-2.0 applies to repository code; customer documents, connectors, domain data and third-party artifacts retain their own licenses and use restrictions.

## Scope and evidence

The supported release is a bounded native runtime and experimental lab, not a general intelligence engine. Customer data acquisition, validated risk/coverage, integration economics, independent security, production load/HA and comparative model benchmarks require separate acceptance evidence. See [requirement coverage](docs/requirement-status.md), [evaluation](docs/evaluation.md) and [verification results](docs/verification.md) for the implemented behavior, bounded research and remaining obligations.
