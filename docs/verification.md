# Verification for source v0.2.0

The recorded local verification completed on 2 October 2026 on macOS arm64 with Rust 1.86, Node 22.23.3 and Python 3.11. [Machine-readable summary](verification/summary.json) and redacted raw logs are included. These are finite generated-fixture/software results, not customer release acceptance.

| Completed check | Actual result |
|---|---|
| Rust workspace contracts | 70 passed: 4 runtime units, 14 API/storage integration, 20 native, 29 research, 3 research operation contracts. |
| PostgreSQL integration | 1 passed against a fresh isolated local PostgreSQL database; not merely SQLite mocks. |
| Console | 3 tests passed, TypeScript check and Vite production build passed. |
| TypeScript SDK | 3 passed, including a real localhost redirect rejection test. |
| Python SDK | 4 passed against a local HTTP fixture, including redirect rejection. |
| Rust SDK | 2 passed and SDK formatting passed. |
| Total distinct automated tests | **83 passed**. The PostgreSQL test is deliberately ignored in the default workspace command and was explicitly run separately. Repeated checks are not counted again. |
| Quality/build | Rustfmt, workspace all-target Clippy with warnings denied, optimized workspace build. |
| Deployment configuration | All three Compose combinations parse successfully. |
| End-to-end examples | Python real decision → signed receipt replay → once-per-action internal ticket; TypeScript warranty human-review result; durable workflow confirmation; pure native example. |
| Browser check | Built console rendered an actual resolved service route and readable response from the optimized local API. Synthetic asset/input only. |
| Research harness | Five real baselines evaluated on a frozen generated hours dataset; errors and narrow assumptions retained in generated-benchmark.json. |
| Release contracts | All 102 unique numbered PRD requirements and all 49 API operations inventoried; embedded pack/lab examples match source. |

## Dependency evidence

The locked console and TypeScript SDK npm audit reports each show zero reported vulnerabilities. The Rust SDK audit passes without exceptions. The runtime audit passes with a specific inactive optional dependency exception: SQLx's meta-package lockfile contains MySQL/RSA although this runtime enables only PostgreSQL/SQLite. The raw lockfile advisory is **RUSTSEC-2023-0071**. `cargo tree --locked -i rsa --target all` reports no activated RSA dependency. `deploy/scripts/audit-rust.sh` first checks that fact, fails on inspection errors or any activated RSA path, and only then applies the advisory exception. Enabling MySQL/RSA invalidates this exception and requires reassessment. This is not a claim that the raw meta-lock contains no advisory.

Advisory databases were fetched during this verification. Results can change as new advisories appear; rerun CI before release. The audit reports and guard are inspectable. Unused JWT PEM dependencies and an unmaintained HTML hashing dependency were removed; JSON/JWK OIDC and bounded HTML tokenization remain supported. The console uses Vite 6.4.3/Vitest 4.1.11. Its `.npmrc` excludes unused optional test/browser peer graph resolution to avoid npm 10's optional peer error; the actual installed dependency graph is locked and audited.

## Explicit outstanding checks

Docker CLI is installed but its daemon was unavailable, so **local image build, image vulnerability scan and SBOM creation were not completed**. CI defines those gates; configuration is not a passed run. Kubernetes recipes have not been applied to a cluster. Production TLS/secrets/retention, signing/encryption-key rotation, network enforcement, HA, destination precondition/idempotency retention and identity-provider acceptance require the target environment.

Customer locked-case volume, severe-error bound, useful coverage, human-review cost, performance/availability, independent security/accessibility/fuzzing, onboarding economics and frontier-model comparison remain unproven. Research primitives, calibration/synthesis/causal/index/bandit examples are bounded experiments. Their existence does not activate validated live models or establish general intelligence. Every remaining engineering obligation is listed in requirement-status.md.

## Archive verification

A new temporary extraction was tested from source, with no packaged dependencies or binaries. The workspace's 70 tests, all three SDK suites, console tests, optimized Rust workspace build and clean Vite build passed using locked cached dependencies. These repeat runs are not added to the 83 distinct-test count. The ZIP passed CRC integrity checking, safe-path validation, required-file inventory and SHA-256 checks for every inventoried file. `MANIFEST.sha256` includes source, lockfiles, docs, fixtures and verification reports; it does not hash itself. The final archive includes these fresh-extraction logs and excludes runtime keys, databases, `.env`, dependency folders and build outputs.
