#!/usr/bin/env sh
set -eu
umask 077
cd "$(dirname "$0")/../.."
if [ -e .env ]; then
  echo '.env already exists; refusing to overwrite secrets.' >&2
  exit 1
fi
command -v openssl >/dev/null 2>&1 || { echo 'openssl is required to generate secrets.' >&2; exit 1; }
task_postgres_password=$(openssl rand -hex 24)
task_encryption_key=$(openssl rand -hex 32)
task_signing_key=$(openssl rand -hex 32)
task_bootstrap_key=$(openssl rand -hex 32)
cat > .env <<EOF
SAMI_POSTGRES_PASSWORD=$task_postgres_password
SAMI_ENCRYPTION_KEY=$task_encryption_key
SAMI_SIGNING_KEY=$task_signing_key
SAMI_BOOTSTRAP_KEY=$task_bootstrap_key
SAMI_ENV=production
SAMI_CONSOLE_PORT=8088
RUST_LOG=sami=info,tower_http=info
EOF
echo 'Created .env with mode 600. Keep it private and back up encryption/signing keys separately.'
