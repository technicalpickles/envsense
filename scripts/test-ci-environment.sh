#!/bin/bash
set -euo pipefail

# Script to test envsense in a CI-like environment
# This helps reproduce issues that occur in GitHub Actions

echo "=== Testing envsense in CI-like environment ==="
echo ""

echo "Current environment variables:"
echo "GITHUB_ACTIONS=${GITHUB_ACTIONS:-<not set>}"
echo "CI=${CI:-<not set>}"
echo "RUNNER_OS=${RUNNER_OS:-<not set>}"
echo ""

echo "Building envsense..."
cargo build
echo ""

echo "Running basic envsense info:"
./target/debug/envsense info --json | jq .
echo ""

echo "Testing baseline comparison (all scenarios):"
./scripts/compare-baseline.sh
echo ""

echo "Testing individual scenarios:"
for scenario in cursor github_actions gitlab_ci plain_tty; do
    echo "Testing scenario: $scenario"
    ./scripts/compare-baseline.sh --quiet "$scenario" || echo "FAILED: $scenario"
done
echo ""

echo "=== CI environment test complete ==="
