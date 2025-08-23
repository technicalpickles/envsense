#!/bin/bash
set -euo pipefail

# Baseline comparison script for envsense
# Validates that current JSON output matches established baselines

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
SNAPSHOTS_DIR="$PROJECT_ROOT/tests/snapshots"
ENVSENSE_BIN="$PROJECT_ROOT/target/debug/envsense"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Options
UPDATE_BASELINES=false
VERBOSE=false
FAILED_SCENARIOS=()

usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "OPTIONS:"
    echo "  -u, --update      Update baseline JSON files with current output"
    echo "  -v, --verbose     Show detailed diff output"
    echo "  -h, --help        Show this help message"
    echo ""
    echo "EXAMPLES:"
    echo "  $0                # Compare all scenarios against baselines"
    echo "  $0 --update       # Update all baselines with current output"
    echo "  $0 --verbose      # Show detailed diffs for failures"
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -u|--update)
            UPDATE_BASELINES=true
            shift
            ;;
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            echo "Unknown option $1"
            usage
            exit 1
            ;;
    esac
done

# Build the project if binary doesn't exist or is older than source
if [[ ! -f "$ENVSENSE_BIN" ]] || [[ "$PROJECT_ROOT/src" -nt "$ENVSENSE_BIN" ]]; then
    echo "Building envsense..."
    cd "$PROJECT_ROOT"
    cargo build --quiet
fi

# Function to load environment from .env file
load_env() {
    local env_file="$1"
    
    # Preserve essential system variables but clear envsense-detectable ones
    env -i \
        PATH="$PATH" \
        HOME="$HOME" \
        TMPDIR="$TMPDIR" \
        USER="$USER" \
        bash -c "
        set -a
        # Load the env file, ignoring comments and empty lines
        while IFS= read -r line || [[ -n \$line ]]; do
            if [[ \$line =~ ^[[:space:]]*# ]] || [[ -z \$line ]]; then
                continue
            fi
            export \"\$line\"
        done < '$env_file'
        # Run envsense with the loaded environment
        exec '$ENVSENSE_BIN' info --json
    "
}

# Function to compare JSON output
compare_baseline() {
    local scenario="$1"
    local env_file="$SNAPSHOTS_DIR/${scenario}.env"
    local baseline_file="$SNAPSHOTS_DIR/${scenario}.json"
    local temp_output="/tmp/envsense_${scenario}.json"
    
    if [[ ! -f "$env_file" ]]; then
        echo -e "${YELLOW}SKIP${NC} $scenario (no .env file)"
        return 0
    fi
    
    if [[ ! -f "$baseline_file" ]]; then
        echo -e "${YELLOW}SKIP${NC} $scenario (no baseline .json file)"
        return 0
    fi
    
    # Generate current output
    if ! load_env "$env_file" > "$temp_output" 2>/dev/null; then
        echo -e "${RED}ERROR${NC} $scenario (failed to run envsense)"
        FAILED_SCENARIOS+=("$scenario")
        return 1
    fi
    
    # Compare with baseline
    if cmp -s "$temp_output" "$baseline_file"; then
        echo -e "${GREEN}PASS${NC} $scenario"
    else
        if [[ "$UPDATE_BASELINES" == "true" ]]; then
            cp "$temp_output" "$baseline_file"
            echo -e "${YELLOW}UPDATED${NC} $scenario"
        else
            echo -e "${RED}FAIL${NC} $scenario"
            FAILED_SCENARIOS+=("$scenario")
            
            # Always show a brief summary of what's different
            if [[ "$JQ_AVAILABLE" == "true" ]]; then
                local expected_contexts=$(jq -r '.contexts // [] | join(",")' "$baseline_file" 2>/dev/null || echo "ERROR")
                local actual_contexts=$(jq -r '.contexts // [] | join(",")' "$temp_output" 2>/dev/null || echo "ERROR")
                local expected_ide=$(jq -r '.facets.ide_id // "none"' "$baseline_file" 2>/dev/null || echo "ERROR")
                local actual_ide=$(jq -r '.facets.ide_id // "none"' "$temp_output" 2>/dev/null || echo "ERROR")
                local expected_interactive=$(jq -r '.traits.is_interactive' "$baseline_file" 2>/dev/null || echo "ERROR")
                local actual_interactive=$(jq -r '.traits.is_interactive' "$temp_output" 2>/dev/null || echo "ERROR")
                local expected_tty_stdin=$(jq -r '.traits.is_tty_stdin' "$baseline_file" 2>/dev/null || echo "ERROR")
                local actual_tty_stdin=$(jq -r '.traits.is_tty_stdin' "$temp_output" 2>/dev/null || echo "ERROR")
                local expected_tty_stdout=$(jq -r '.traits.is_tty_stdout' "$baseline_file" 2>/dev/null || echo "ERROR")
                local actual_tty_stdout=$(jq -r '.traits.is_tty_stdout' "$temp_output" 2>/dev/null || echo "ERROR")
                
                echo -e "  Expected: contexts=[${expected_contexts}], ide_id=${expected_ide}"
                echo -e "  Actual:   contexts=[${actual_contexts}], ide_id=${actual_ide}"
                
                # Show TTY differences if they exist
                if [[ "$expected_interactive" != "$actual_interactive" ]] || 
                   [[ "$expected_tty_stdin" != "$actual_tty_stdin" ]] || 
                   [[ "$expected_tty_stdout" != "$actual_tty_stdout" ]]; then
                    echo -e "  ${YELLOW}TTY differences detected:${NC}"
                    echo -e "    interactive: $expected_interactive → $actual_interactive"
                    echo -e "    tty_stdin:   $expected_tty_stdin → $actual_tty_stdin"  
                    echo -e "    tty_stdout:  $expected_tty_stdout → $actual_tty_stdout"
                fi
            else
                echo -e "  ${YELLOW}Install 'jq' for detailed diff reporting${NC}"
                echo -e "  Files differ: $baseline_file vs current output"
            fi
            
            if [[ "$VERBOSE" == "true" ]]; then
                echo ""
                echo "Full expected output ($baseline_file):"
                cat "$baseline_file" | head -15
                echo "..."
                echo ""
                echo "Full actual output ($temp_output):"
                cat "$temp_output" | head -15
                echo "..."
                echo ""
                echo "JSON diff:"
                diff "$baseline_file" "$temp_output" || true
                echo ""
            fi
        fi
    fi
    
    rm -f "$temp_output"
}

# Check if jq is available for better error reporting
JQ_AVAILABLE=false
if command -v jq >/dev/null 2>&1; then
    JQ_AVAILABLE=true
fi

echo "Comparing baselines in $SNAPSHOTS_DIR..."
echo ""
echo "NOTE: TTY detection may differ between environments due to how the script"
echo "      runs commands. Use --update if TTY differences are expected."
echo ""

# Find all .env files and run comparisons
for env_file in "$SNAPSHOTS_DIR"/*.env; do
    if [[ -f "$env_file" ]]; then
        scenario=$(basename "$env_file" .env)
        compare_baseline "$scenario"
    fi
done

echo ""

# Report results
if [[ ${#FAILED_SCENARIOS[@]} -eq 0 ]]; then
    echo -e "${GREEN}All baselines match!${NC}"
    exit 0
else
    echo -e "${RED}Failed scenarios: ${FAILED_SCENARIOS[*]}${NC}"
    echo ""
    echo "To update baselines, run:"
    echo "  $0 --update"
    exit 1
fi