# SDKs

Python has no external runtime dependency. Install `pip install ./sdk/python` or put `sdk/python` on `PYTHONPATH` and construct `SamiClient(base_url, token)`. TypeScript uses standard fetch: build `sdk/typescript` with `npm ci && npm run build`, then construct `new SamiClient(token, baseUrl)`. Rust uses `sami-client` from `sdk/rust`; construct `SamiClient::new(base, token)`. Constructor argument order is intentionally documented because Python/TypeScript differ.

All clients provide decisions, sessions, generic governed resources, source lifecycle, receipt inspection/replay, action proposal/approval/execute/reconcile, feedback, research and scoped key operations. They URI-encode IDs, surface status/details, use caller credentials and never automatically retry a mutation. Execute requires a stable caller idempotency key. Read current revisions before updating records. For extended native routes use the Python/TypeScript `request` method, CLI request or an HTTP client. SSE is a direct HTTP stream, not a fabricated chat session.

See `examples/README.md` for tested generated workflows. A timeout may occur after a durable commit. Inspect the resource, keep the same payload/key, and reconcile effects before retrying. Generic mutations do not yet provide a universal idempotency ledger; resource CAS and the action-specific effect state machine have distinct contracts.

Do not put keys in URLs, committed example files or log output. Use HTTPS for remote hosts and a secret provider. Browser credentials stay in memory by default. Optional session storage remains script-accessible and is cleared on disconnect.

All clients reject cleartext remote API origins and disable redirects to avoid forwarding credentials or repeating writes. Localhost HTTP is supported for development.
