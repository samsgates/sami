# Contributing

Start with the [PRD](docs/PRD.md), [implementation contract](IMPLEMENTATION_CONTRACT.md) and [developer guide](docs/developer-guide.md). Work within declared native task semantics and authority boundaries.

1. Describe the observable problem and supported scope. Link the relevant requirement ID and a reproducible example.
2. Add regression coverage for changed behavior, including invalid input and the relevant tenant/permission/freshness boundary.
3. Keep persistent state, knowledge publication, control-plane changes and external effects separate. Do not create silent model fallbacks or derive permission from generated text.
4. Run `sh deploy/scripts/check.sh`. Supply the actual output of meaningful checks and any unverified environment requirements.
5. Document API/schema changes, data migrations, deployment effects and compatibility. Retain snapshot/replay semantics and explain how deleted material is handled.
6. Submit a focused change with problem, behavior, validation and limits. Production claims require evidence; generated fixtures must be labeled synthetic.

Never commit keys, private source documents, personal information, local databases, evaluation answers from customer data or live connector credentials. New third-party datasets need explicit provenance and permitted-use records. Security issues follow [SECURITY.md](SECURITY.md), not a public reproduction containing private records.

Maintain Rust formatting and Clippy without request-path panics; strict TypeScript; bounded parser/DSL behavior; parameterized SQL; explicit errors; and understandable user-facing state. New optional services or bespoke indexing must earn their operational cost through measurement.
