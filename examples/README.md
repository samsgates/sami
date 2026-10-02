# Runnable examples

Use a separate development database seeded with `sami init --seed-demo`. Export a generated demo installation's credential as `SAMI_API_KEY`; `SAMI_API_URL` defaults to `http://127.0.0.1:8080/v1`. These examples never need a paid model API.

| Example | Run | Actual behavior |
|---|---|---|
| Python case → receipt → approved ticket | `python3 examples/complete_case.py` | Creates one real internal ticket; stable-key retry returns its existing result. |
| TypeScript SDK | `cd sdk/typescript && npm ci && npm run build`, then `node examples/client.mjs` from root | Calls warranty prequalification, prints the typed human-review result. |
| Native Rust library | `cargo run --locked --example native` | Executes offline against explicitly supplied generated authorized-source premises. Pure library callers must apply real tenant/source authorization themselves. |
| Durable process | `python3 examples/workflow.py` | Creates and confirms generated steps through durable CAS state. No external tool effects. |
| REST client | `examples/http/native.http` | Real authenticated native decision, search, DSL and paginated audit requests. |
| Research operation contracts | `cargo test --locked -p sami-research --test operation_contracts` | Executes every shipped bounded lab input. |
| Leakage-controlled toy comparison | `cargo run --locked -p sami-research --example benchmark` | Freezes an induced program before eight generated transfer cases; compares constant, lexical, nearest-case, explicit-policy and induction baselines. |
| Local text-PDF parser | `python3 examples/research/pdf_extract.py --help` | Extracts quarantined page/line segments. Requires the parser requirements in an isolated worker. No OCR/network fallback. |

Research inputs are in `examples/research/operations.json`. In the console, choose an operation to populate its real executable input. Outputs such as fitted calibration models or sequence indexes are supplied to the matching follow-up operation; see the contract tests for chaining.

The TypeScript example command assumes its SDK has already been built. The HTTP file's process-environment placeholder follows VS Code REST Client syntax; adapt token storage to another client's secret mechanism. No real secret is included.

To reproduce native scenarios, run `cargo test --workspace --locked`; for PostgreSQL use the explicit ignored test with a fresh isolated `SAMI_TEST_POSTGRES_URL`. All shipped policy and case data are generated, Apache-2.0 demonstration data. The hours exercise omits real warranty evidence and cannot establish customer accuracy or LLM superiority.

Run one stdin research operation with `cargo run --locked -p sami-research --example research -- OPERATION < input.json`, or use `sami evaluate --operation OPERATION --input input.json`.

The [offline device tutorial](../docs/offline.md) covers key provisioning, export/import, expiry and reconnection limits.
