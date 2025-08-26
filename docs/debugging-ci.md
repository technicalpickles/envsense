# Debugging CI Issues

This guide helps debug baseline test failures that occur in CI but not locally.

## Quick Start

### Option 1: VS Code Dev Container (Recommended)

1. **Prerequisites**: Docker Desktop + VS Code with "Dev Containers" extension
2. **Open**: `Ctrl+Shift+P` → "Dev Containers: Reopen in Container"
3. **Test**: Run `./scripts/test-ci-environment.sh` inside the container

### Option 2: Docker Command Line

```bash
# Build and run CI simulation
./scripts/run-in-docker.sh

# Or run specific commands
docker run --rm envsense-ci-test ./scripts/compare-baseline.sh cursor
```

### Option 3: Manual Environment Setup

```bash
# Set CI environment variables
export GITHUB_ACTIONS=true
export CI=true
export RUNNER_OS=Linux

# Run tests
./scripts/compare-baseline.sh --debug
```

## Debugging Workflow

### 1. Compare Environments

**Local Environment:**
```bash
./scripts/compare-environments.sh > local-output.txt
```

**CI Environment (in container):**
```bash
./scripts/compare-environments.sh > ci-output.txt
```

**Compare:**
```bash
diff local-output.txt ci-output.txt
```

### 2. Test Specific Scenarios

```bash
# List available scenarios
./scripts/compare-baseline.sh --list

# Test problematic scenario with debug output
./scripts/compare-baseline.sh --debug cursor

# Test multiple scenarios
./scripts/compare-baseline.sh cursor plain_tty github_actions
```

### 3. Analyze Differences

Common differences to check:

#### CI Detection
```bash
# Check if CI is being detected when it shouldn't be
./target/debug/envsense info --json | jq '.facets.ci'

# Expected for most scenarios: {"is_ci": false}
# If you see GitHub Actions detection, the environment isn't isolated
```

#### TTY Detection
```bash
# Check TTY status
./target/debug/envsense info --json | jq '.traits | {is_tty_stdin, is_tty_stdout, is_tty_stderr}'

# Should match the ENVSENSE_TTY_* overrides in the .env files
```

#### Environment Variable Isolation
```bash
# Check if CI vars are leaking through
env | grep -E "(GITHUB|CI|GITLAB)"

# The baseline script should clear these with env -i
```

## Common Issues and Solutions

### Issue 1: CI Detection in Non-CI Scenarios

**Symptom:**
```
Expected: contexts=[], ci_id=none
Actual:   contexts=[ci], ci_id=github_actions
```

**Cause:** GitHub Actions environment variables are leaking through `env -i`

**Debug:**
```bash
# Test environment isolation
./scripts/compare-baseline.sh --debug plain_tty 2>&1 | grep "GITHUB\|CI"
```

**Solution:** Check if `env -i` is working correctly in the baseline script

### Issue 2: TTY Detection Differences

**Symptom:**
```
Expected: is_tty_stdin=true, is_tty_stdout=false
Actual:   is_tty_stdin=false, is_tty_stdout=false
```

**Cause:** Container environment has different TTY behavior

**Debug:**
```bash
# Check if TTY overrides are being loaded
./scripts/compare-baseline.sh --debug plain_tty 2>&1 | grep "ENVSENSE_TTY"
```

**Solution:** Verify TTY overrides in `tests/snapshots/*.env` files

### Issue 3: Binary Not Found

**Symptom:**
```
ERROR cursor (failed to run envsense, exit code: 127)
```

**Cause:** Binary not built or not in expected location

**Debug:**
```bash
# Check binary
ls -la target/debug/envsense
file target/debug/envsense

# Rebuild if needed
cargo build
```

## Environment Files

Each scenario has an `.env` file that controls the test environment:

```bash
# Example: tests/snapshots/plain_tty.env
TERM=xterm-256color
ENVSENSE_TTY_STDIN=true
ENVSENSE_TTY_STDOUT=false
ENVSENSE_TTY_STDERR=false
ENVSENSE_COLOR_LEVEL=none
ENVSENSE_SUPPORTS_HYPERLINKS=false
```

These override runtime detection to make tests deterministic.

## Baseline Files

Each scenario has a `.json` file with expected output:

```bash
# Example: tests/snapshots/plain_tty.json
{
  "contexts": [],
  "facets": {
    "ci": {"is_ci": false}
  },
  "traits": {
    "is_tty_stdin": true,
    "is_tty_stdout": false,
    ...
  }
}
```

## Updating Baselines

If the output legitimately changed:

```bash
# Update all baselines
./scripts/compare-baseline.sh --update

# Update specific scenario
./scripts/compare-baseline.sh --update cursor

# Review changes before committing
git diff tests/snapshots/
```

## Advanced Debugging

### Trace Environment Loading

```bash
# Add debug output to load_env function
# Edit scripts/compare-baseline.sh and add:
echo "DEBUG: About to run envsense with env:" >&2
env | sort >&2
```

### Test Individual Components

```bash
# Test CI detection directly
cargo test detectors::ci::tests --nocapture

# Test terminal detection
cargo test detectors::terminal::tests --nocapture

# Test with specific environment
GITHUB_ACTIONS=true cargo test detectors::ci::tests::detects_github_actions
```

### Manual Environment Testing

```bash
# Simulate exact CI environment
env -i \
  PATH="$PATH" \
  HOME="$HOME" \
  TMPDIR="/tmp" \
  USER="$USER" \
  GITHUB_ACTIONS="true" \
  CI="true" \
  bash -c './target/debug/envsense info --json'
```

## Getting Help

1. **Check logs**: Look at the full CI output with debug information
2. **Compare environments**: Use the comparison scripts to see differences
3. **Test locally**: Use the devcontainer to reproduce the issue
4. **Isolate the problem**: Test individual scenarios and components

The enhanced debugging tools should help identify the root cause of any CI vs local differences.
