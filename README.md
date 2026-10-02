# SAMI

**Correctable memory. Accountable decisions. Controlled actions.**

SAMI is a native, CPU-first operational intelligence runtime and workbench. Its first domain is industrial service triage and warranty prequalification. Approved evidence, temporal claims, typed tasks, policy checks and durable action state belong to the runtime. Language is a presentation layer; it never grants permissions.

This is an implementation of the bounded SAMI v2 design, with executable native examples and research routines. A runnable system is not evidence of customer fit, a 99.9% service level, a passed independent security assessment or superiority over frontier models. The [PRD](docs/PRD.md) defines those separate gates. Included industrial examples are generated demonstration data.

## What is in the repository

- Rust API/runtime, durable PostgreSQL or local SQLite storage, scoped credentials and controlled execution.
- Native parsing, evidence-driven decisions, rules, templates and the versioned [industrial domain pack](packs/industrial-service/pack.json).
- Correctable source/claim lifecycle, historical decision receipts and replay, administrative-action approval and reconciliation.
- A React/TypeScript console for cases, evidence, pack authoring, decisions, actions, research, feedback, audit, credentials and runtime controls.
- Bounded calibration, evaluation, drift, conformal prediction, causal diagnostics, safe bandits, program synthesis, DSL, sequence and semantic research operations.
- Python, TypeScript and Rust SDKs; Compose, nonroot images, Kubernetes deployment recipes and CI quality/security gates.

## Start with Docker Compose

Prerequisites: Docker Engine or Docker Desktop with Compose v2, and OpenSSL. From the repository root:

```sh
sh deploy/scripts/init-env.sh
docker compose up --build -d
docker compose ps
curl --fail http://127.0.0.1:8088/health
```

Open `http://127.0.0.1:8088`. In **Connect credential**, enter the bootstrap credential stored in your private `.env`, or a scoped key/OIDC bearer token obtained from your administrator. The console stores it only in memory unless you explicitly select tab session storage. No local user password or hardcoded login exists.

The database and API have no public Compose ports. The console binds to loopback by default and proxies `/v1` on the same origin. Configure trusted TLS and authentication before exposing it beyond your machine. See [deployment](docs/deployment.md) for infrastructure, OIDC and key handling.

## Develop locally

Requires Rust 1.86 or a compatible newer toolchain, Node.js 22, npm and Python 3.10+. Install/build the console:

```sh
cd console
npm ci
npm run build
cd ..
```

Create your private environment with `sh deploy/scripts/init-env.sh`, then export its variables in your shell. For a local SQLite runtime also set:

```sh
export SAMI_DATABASE_URL='sqlite://sami.db?mode=rwc'
export SAMI_ENV=development
export SAMI_DATA_DIR='./data'
export SAMI_CONSOLE_DIR='./console/dist'
cargo run --locked -- init --seed-demo
cargo run --locked -- serve --host 127.0.0.1 --port 8080
```

Use `sami --help` / `cargo run --locked -- --help` for the CLI command and option contract supplied by this version. A separate terminal can run `cd console && npm run dev` for the Vite development proxy at port 5173. For the native server open `http://127.0.0.1:8080/console/`. The production console is compiled static content, not the Vite development server.

## First supported workflow

Read the [user guide](docs/user-guide.md) and inspect the [generated fixtures](packs/industrial-service/fixtures.json). Establish a source with legitimate rights and scope; review/publish its claims; use an approved domain pack; run `service_triage` or `warranty_prequalification` with a confirmed asset and an as-of date. A missing or contradictory record is a review condition, not a fabricated denial.

An approved administrative tool can create an internal ticket after an explicit proposal, evidence checks and human approval. Physical machinery control, hazardous repair instructions and financial payouts are excluded. Research candidates and feedback do not automatically become production policies.

## Verify and contribute

```sh
sh deploy/scripts/check.sh
```

See [developer guide](docs/developer-guide.md), [API guide](docs/api.md), [SDK guide](docs/sdk.md), [operations](docs/operations.md), [evaluation](docs/evaluation.md), [CONTRIBUTING](CONTRIBUTING.md) and [SECURITY](SECURITY.md). CI runs formatting, Clippy, tests, frontend/SDK builds, dependency audits, an SBOM and a container vulnerability gate. A scan configuration is not a passed scan; review the actual job results.

Apache-2.0 applies to repository code. Customer documents, connectors, domain data and third-party artifacts retain their own licenses and permitted-use restrictions. No customer data is authorized for shared/global learning merely by installing SAMI.

## Included scope and evidence

Read [PRD coverage](docs/requirement-status.md) and [verification results](docs/verification.md) before deployment. They identify implemented behavior, bounded research, infrastructure obligations and remaining release gates for every numbered requirement. This source release does not assert complete PRD GA acceptance.

Run `docker compose -f compose.yml -f compose.dev.yml up --build -d` for opt-in generated examples. The base production profile initializes empty authoritative data. Use [examples](examples/README.md) for Python, TypeScript, Rust, HTTP, ingestion, workflows, receipts, bundles and research.
