#!/usr/bin/env sh
set -eu
umask 077
cd "$(dirname "$0")/../.."
task_backup_directory=${1:-backups}
mkdir -p "$task_backup_directory"
task_backup_path="$task_backup_directory/sami-$(date -u +%Y%m%dT%H%M%SZ).dump"
docker compose exec -T database pg_dump -U sami -d sami --format=custom > "$task_backup_path"
test -s "$task_backup_path"
echo "Database snapshot written to $task_backup_path. Encrypt this file and back up artifact storage plus key material separately."
