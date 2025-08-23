#!/usr/bin/env bash
set -euo pipefail

# Determine repo root and binary path
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BIN_PATH="$ROOT_DIR/target/debug/envsense"
JQ_BIN=${JQ:-jq}

if [ ! -x "$BIN_PATH" ]; then
  echo "envsense binary not found at $BIN_PATH" >&2
  echo "build the project first (e.g., cargo build)" >&2
  exit 1
fi

# Iterate over all snapshot env files
for envfile in "$ROOT_DIR/tests/snapshots"/*.env; do
  name=$(basename "$envfile" .env)
  snapshot="$ROOT_DIR/tests/snapshots/$name.json"
  tmp_actual=$(mktemp)
  tmp_expected=$(mktemp)

  # Build environment command array starting with env -i to clear env
  env_cmd=(env -i)
  while IFS='=' read -r key value; do
    [ -z "$key" ] && continue
    env_cmd+=("$key=$value")
  done < "$envfile"

  if [[ "$name" == "tmux" || "$name" == "plain_tty" ]]; then
    if [[ "$OSTYPE" == "darwin"* ]]; then
      "${env_cmd[@]}" script -q /dev/null "$BIN_PATH" info --json 2>&1 | "$JQ_BIN" -S . > "$tmp_actual"
    else
      "${env_cmd[@]}" script -qec "$BIN_PATH info --json" /dev/null 2>&1 | "$JQ_BIN" -S . > "$tmp_actual"
    fi
  else
    printf '' | "${env_cmd[@]}" "$BIN_PATH" info --json 2>&1 | "$JQ_BIN" -S . > "$tmp_actual"
  fi

  "$JQ_BIN" -S . "$snapshot" > "$tmp_expected"
  if ! diff -u "$tmp_expected" "$tmp_actual"; then
    echo "Snapshot drift detected for $name" >&2
    rm -f "$tmp_actual" "$tmp_expected"
    exit 1
  fi
  rm -f "$tmp_actual" "$tmp_expected"
done

echo "All snapshots match"
