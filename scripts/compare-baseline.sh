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
DEBUG_BASELINE="${DEBUG_BASELINE:-true}"  # Default to true for CI debugging, but allow env override
FAILED_SCENARIOS=()
SPECIFIC_SCENARIOS=()

usage() {
    echo "Usage: $0 [OPTIONS] [SCENARIO...]"
    echo ""
    echo "OPTIONS:"
    echo "  -u, --update      Update baseline JSON files with current output"
    echo "  -v, --verbose     Show detailed diff output"
    echo "  -d, --debug       Show debug information (environment vars, etc.)"
    echo "  -q, --quiet       Disable debug output (debug is on by default)"
    echo "  -l, --list        List available scenarios and exit"
    echo "  -h, --help        Show this help message"
    echo ""
    echo "ARGUMENTS:"
    echo "  SCENARIO          Run specific scenario(s) only (e.g., cursor, plain_tty)"
    echo "                    If no scenarios specified, runs all scenarios"
    echo ""
    echo "EXAMPLES:"
    echo "  $0                # Compare all scenarios against baselines"
    echo "  $0 cursor         # Compare only the cursor scenario"
    echo "  $0 cursor plain_tty # Compare cursor and plain_tty scenarios"
    echo "  $0 --update       # Update all baselines with current output"
    echo "  $0 --verbose      # Show detailed diffs for failures"
    echo "  $0 --debug cursor # Show debug info for cursor scenario only"
    echo "  $0 --quiet        # Run without debug output"
    echo ""
    echo "ENVIRONMENT:"
    echo "  DEBUG_BASELINE=false  # Disable debug mode via environment variable"
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
        -d|--debug)
            DEBUG_BASELINE=true
            shift
            ;;
        -q|--quiet)
            DEBUG_BASELINE=false
            shift
            ;;
        -l|--list)
            echo "Available scenarios:"
            for env_file in "$SNAPSHOTS_DIR"/*.env; do
                if [[ -f "$env_file" ]]; then
                    scenario=$(basename "$env_file" .env)
                    echo "  $scenario"
                fi
            done
            exit 0
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        -*)
            echo "Unknown option $1"
            usage
            exit 1
            ;;
        *)
            # This is a scenario name
            SPECIFIC_SCENARIOS+=("$1")
            shift
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
    local debug_mode="${DEBUG_BASELINE:-false}"
    
    if [[ "$debug_mode" == "true" ]]; then
        echo "DEBUG: Loading environment from $env_file" >&2
        echo "DEBUG: Original CI-related env vars:" >&2
        env | grep -E "(GITHUB|CI|GITLAB)" | head -5 >&2 || echo "DEBUG: No CI vars found" >&2
    fi
    
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
        
        # Debug output if requested
        if [[ '$debug_mode' == 'true' ]]; then
            echo 'DEBUG: Loaded environment variables:' >&2
            env | grep -E '(ENVSENSE|TERM|CI|GITHUB|GITLAB)' | sort >&2 || echo 'DEBUG: No relevant env vars' >&2
            echo 'DEBUG: Running envsense binary: $ENVSENSE_BIN' >&2
            echo 'DEBUG: Binary exists:' \$(test -f '$ENVSENSE_BIN' && echo 'yes' || echo 'no') >&2
            echo 'DEBUG: Binary executable:' \$(test -x '$ENVSENSE_BIN' && echo 'yes' || echo 'no') >&2
        fi
        
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
    local debug_mode="${DEBUG_BASELINE:-false}"
    if [[ "$debug_mode" == "true" ]]; then
        echo "DEBUG: Generating output for $scenario..." >&2
        
        # Handle errors gracefully
        set +e  # Temporarily disable exit on error
        load_env "$env_file" > "$temp_output" 2>"$temp_output.stderr"
        local exit_code=$?
        set -e  # Re-enable exit on error
        
        if [[ $exit_code -ne 0 ]]; then
            echo -e "${RED}ERROR${NC} $scenario (failed to run envsense, exit code: $exit_code)"
            if [[ $exit_code -eq 124 ]]; then
                echo "DEBUG: Command timed out after 30 seconds" >&2
            fi
            if [[ -f "$temp_output.stderr" ]]; then
                echo "DEBUG: stderr output:" >&2
                cat "$temp_output.stderr" >&2
            fi
            if [[ -f "$temp_output" ]]; then
                echo "DEBUG: partial stdout output:" >&2
                head -10 "$temp_output" >&2
            fi
            FAILED_SCENARIOS+=("$scenario")
            return 1
        fi
        echo "DEBUG: Generated output for $scenario ($(wc -l < "$temp_output" 2>/dev/null || echo "?") lines)" >&2
    else
        set +e  # Temporarily disable exit on error
        load_env "$env_file" > "$temp_output" 2>/dev/null
        local exit_code=$?
        set -e  # Re-enable exit on error
        
        if [[ $exit_code -ne 0 ]]; then
            echo -e "${RED}ERROR${NC} $scenario (failed to run envsense, exit code: $exit_code)"
            FAILED_SCENARIOS+=("$scenario")
            return 1
        fi
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
                local expected_ci=$(jq -r '.facets.ci_id // "none"' "$baseline_file" 2>/dev/null || echo "ERROR")
                local actual_ci=$(jq -r '.facets.ci_id // "none"' "$temp_output" 2>/dev/null || echo "ERROR")
                local expected_interactive=$(jq -r '.traits.is_interactive' "$baseline_file" 2>/dev/null || echo "ERROR")
                local actual_interactive=$(jq -r '.traits.is_interactive' "$temp_output" 2>/dev/null || echo "ERROR")
                local expected_tty_stdin=$(jq -r '.traits.is_tty_stdin' "$baseline_file" 2>/dev/null || echo "ERROR")
                local actual_tty_stdin=$(jq -r '.traits.is_tty_stdin' "$temp_output" 2>/dev/null || echo "ERROR")
                local expected_tty_stdout=$(jq -r '.traits.is_tty_stdout' "$baseline_file" 2>/dev/null || echo "ERROR")
                local actual_tty_stdout=$(jq -r '.traits.is_tty_stdout' "$temp_output" 2>/dev/null || echo "ERROR")
                
                echo -e "  Expected: contexts=[${expected_contexts}], ide_id=${expected_ide}, ci_id=${expected_ci}"
                echo -e "  Actual:   contexts=[${actual_contexts}], ide_id=${actual_ide}, ci_id=${actual_ci}"
                
                # Show CI differences if they exist
                if [[ "$expected_ci" != "$actual_ci" ]]; then
                    echo -e "  ${YELLOW}CI detection differences:${NC}"
                    echo -e "    Expected CI: $expected_ci"
                    echo -e "    Actual CI:   $actual_ci"
                    if [[ "$DEBUG_BASELINE" == "true" ]]; then
                        echo -e "    ${YELLOW}CI facet details:${NC}"
                        echo -e "    Expected: $(jq -c '.facets.ci // {}' "$baseline_file" 2>/dev/null)"
                        echo -e "    Actual:   $(jq -c '.facets.ci // {}' "$temp_output" 2>/dev/null)"
                    fi
                fi
                
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
                echo -e "${YELLOW}=== DETAILED DIFF for $scenario ===${NC}"
                echo "Expected output ($baseline_file):"
                cat "$baseline_file" | head -20
                echo ""
                echo "Actual output ($temp_output):"
                cat "$temp_output" | head -20
                echo ""
                echo "Unified diff:"
                diff -u "$baseline_file" "$temp_output" || true
                echo -e "${YELLOW}=== END DIFF ===${NC}"
                echo ""
            else
                echo -e "  ${YELLOW}Run with --verbose to see full diff${NC}"
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
if [[ "$DEBUG_BASELINE" == "true" ]]; then
    echo "DEBUG: Running in debug mode"
    echo "DEBUG: Current environment CI-related variables:"
    env | grep -E "(GITHUB|CI|GITLAB)" | head -10 || echo "DEBUG: No CI variables found"
    echo ""
fi
echo "NOTE: TTY detection may differ between environments due to how the script"
echo "      runs commands. Use --update if TTY differences are expected."
echo ""

# Find scenarios to run
if [[ ${#SPECIFIC_SCENARIOS[@]} -eq 0 ]]; then
    # No specific scenarios provided, run all
    echo "Running all scenarios..."
    for env_file in "$SNAPSHOTS_DIR"/*.env; do
        if [[ -f "$env_file" ]]; then
            scenario=$(basename "$env_file" .env)
            compare_baseline "$scenario"
        fi
    done
else
    # Run specific scenarios
    echo "Running specific scenarios: ${SPECIFIC_SCENARIOS[*]}"
    for scenario in "${SPECIFIC_SCENARIOS[@]}"; do
        env_file="$SNAPSHOTS_DIR/${scenario}.env"
        if [[ ! -f "$env_file" ]]; then
            echo -e "${RED}ERROR${NC}: Scenario '$scenario' not found (no $env_file)"
            FAILED_SCENARIOS+=("$scenario")
            continue
        fi
        compare_baseline "$scenario"
    done
fi

echo ""

# Report results
if [[ ${#FAILED_SCENARIOS[@]} -eq 0 ]]; then
    echo -e "${GREEN}All baselines match!${NC}"
    exit 0
else
    echo -e "${RED}${#FAILED_SCENARIOS[@]} scenario(s) failed:${NC}"
    for scenario in "${FAILED_SCENARIOS[@]}"; do
        echo -e "  - $scenario"
    done
    echo ""
    echo -e "${YELLOW}Common fixes:${NC}"
    echo "  • If TTY differences are expected in CI, the overrides should handle this"
    echo "  • If new functionality changed output, update baselines with: $0 --update"
    echo "  • For detailed diffs, run: $0 --verbose"
    echo ""
    echo -e "${YELLOW}Debugging tips:${NC}"
    echo "  • Check /tmp/envsense_*.json files for actual output"
    echo "  • Compare with tests/snapshots/*.json for expected output"
    echo "  • TTY detection is now controlled by ENVSENSE_TTY_* environment variables"
    exit 1
fi