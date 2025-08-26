#!/bin/bash
set -euo pipefail

# Script to compare envsense output between different environments
# Useful for debugging CI vs local differences

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo "=== Environment Comparison ==="
echo ""

echo "Current environment:"
echo "OS: $(uname -s)"
echo "Architecture: $(uname -m)"
echo "CI: ${CI:-false}"
echo "GITHUB_ACTIONS: ${GITHUB_ACTIONS:-false}"
echo ""

echo "Building envsense..."
cd "$PROJECT_ROOT"
cargo build --quiet
echo ""

echo "=== Full envsense output ==="
./target/debug/envsense info --json | jq .
echo ""

echo "=== CI-related output ==="
echo "Contexts:"
./target/debug/envsense info --json | jq -r '.contexts[]?' || echo "  (none)"
echo ""

echo "CI Facet:"
./target/debug/envsense info --json | jq '.facets.ci' 2>/dev/null || echo "  (none)"
echo ""

echo "CI Traits:"
./target/debug/envsense info --json | jq 'to_entries | map(select(.key | startswith("ci_"))) | from_entries' 2>/dev/null || echo "  (none)"
echo ""

echo "=== Environment Variables (CI-related) ==="
env | grep -E "(GITHUB|CI|GITLAB)" | sort || echo "  (none found)"
echo ""

echo "=== TTY Status ==="
echo "stdin is TTY: $(test -t 0 && echo 'yes' || echo 'no')"
echo "stdout is TTY: $(test -t 1 && echo 'yes' || echo 'no')"
echo "stderr is TTY: $(test -t 2 && echo 'yes' || echo 'no')"
echo ""

echo "=== Test a specific baseline scenario ==="
echo "Testing plain_tty scenario:"
if [[ -f "$PROJECT_ROOT/tests/snapshots/plain_tty.env" ]]; then
    echo "Environment file contents:"
    cat "$PROJECT_ROOT/tests/snapshots/plain_tty.env"
    echo ""
    
    echo "Expected output:"
    jq -c . "$PROJECT_ROOT/tests/snapshots/plain_tty.json" 2>/dev/null || echo "  (file not found)"
    echo ""
    
    echo "Actual output with environment:"
    env -i \
        PATH="$PATH" \
        HOME="$HOME" \
        TMPDIR="${TMPDIR:-/tmp}" \
        USER="${USER:-root}" \
        bash -c "
        set -a
        while IFS= read -r line || [[ -n \$line ]]; do
            if [[ \$line =~ ^[[:space:]]*# ]] || [[ -z \$line ]]; then
                continue
            fi
            export \"\$line\"
        done < '$PROJECT_ROOT/tests/snapshots/plain_tty.env'
        '$PROJECT_ROOT/target/debug/envsense' info --json
    " | jq -c . 2>/dev/null || echo "  (failed to generate)"
else
    echo "  plain_tty.env not found"
fi

echo ""
echo "=== Comparison complete ==="
