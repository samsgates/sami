#!/usr/bin/env sh
set -eu
cd "$(dirname "$0")/../.."
python3 deploy/scripts/check-assets.py
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
(cd console && npm ci --no-audit --no-fund && npm run test && npm run build)
(cd sdk/typescript && npm ci --no-audit --no-fund && npm test)
(cd sdk/python && python3 -m unittest discover -s tests -v)
(cd sdk/rust && cargo fmt -- --check && cargo test --locked)
echo 'Local automated checks passed. Customer, load, independent security and research gates remain separate.'
