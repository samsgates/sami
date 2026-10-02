#!/usr/bin/env sh
set -eu
cd "$(dirname "$0")/../.."
# SQLx's meta-package lock includes unused MySQL optional dependencies. The
# runtime enables SQLite/Postgres only. Fail if RSA becomes an activated dependency.
task_rsa_tree=$(cargo tree --locked -i rsa --target all)
if [ -n "$task_rsa_tree" ]; then
  echo 'RSA is now activated. Remove the unused-dependency exception and reassess RUSTSEC-2023-0071.' >&2
  exit 1
fi
cargo audit --deny warnings --ignore RUSTSEC-2023-0071
