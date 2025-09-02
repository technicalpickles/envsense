#!/usr/bin/env bash

# validate-comprehensive-testing.sh
#
# Purpose: Comprehensive validation of envsense CLI functionality in real environments
# Created: 2024-01-15
# Used for: Validating Task 5.5 implementation and ongoing quality assurance
#
# This script validates that all CLI functionality works correctly in real environments
# and can be used for regression testing and quality assurance.

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Test counter
TESTS_PASSED=0
TESTS_FAILED=0

# Test function
run_test() {
    local test_name="$1"
    local test_command="$2"
    local expected_exit_code="${3:-0}"
    
    log_info "Running test: $test_name"
    
    if eval "$test_command" >/dev/null 2>&1; then
        local exit_code=$?
        if [ "$exit_code" -eq "$expected_exit_code" ]; then
            log_success "✓ $test_name passed"
            ((TESTS_PASSED++))
        else
            log_error "✗ $test_name failed (expected exit $expected_exit_code, got $exit_code)"
            ((TESTS_FAILED++))
        fi
    else
        local exit_code=$?
        if [ "$exit_code" -eq "$expected_exit_code" ]; then
            log_success "✓ $test_name passed"
            ((TESTS_PASSED++))
        else
            log_error "✗ $test_name failed (expected exit $expected_exit_code, got $exit_code)"
            ((TESTS_FAILED++))
        fi
    fi
}

# Validation function
validate_json_output() {
    local test_name="$1"
    local command="$2"
    local expected_field="$3"
    local expected_value="$4"
    
    log_info "Validating JSON output: $test_name"
    
    local output
    output=$(eval "$command" 2>/dev/null)
    
    if echo "$output" | jq -e ".$expected_field" >/dev/null 2>&1; then
        local actual_value
        actual_value=$(echo "$output" | jq -r ".$expected_field")
        
        if [ "$actual_value" = "$expected_value" ]; then
            log_success "✓ $test_name passed ($expected_field = $actual_value)"
            ((TESTS_PASSED++))
        else
            log_error "✗ $test_name failed ($expected_field expected $expected_value, got $actual_value)"
            ((TESTS_FAILED++))
        fi
    else
        log_error "✗ $test_name failed (field $expected_field not found in JSON)"
        ((TESTS_FAILED++))
    fi
}

# Performance validation
validate_performance() {
    local test_name="$1"
    local command="$2"
    local max_duration_ms="$3"
    
    log_info "Validating performance: $test_name"
    
    local start_time
    start_time=$(date +%s%3N)
    
    eval "$command" >/dev/null 2>&1
    
    local end_time
    end_time=$(date +%s%3N)
    
    local duration_ms
    duration_ms=$((end_time - start_time))
    
    if [ "$duration_ms" -lt "$max_duration_ms" ]; then
        log_success "✓ $test_name passed (${duration_ms}ms < ${max_duration_ms}ms)"
        ((TESTS_PASSED++))
    else
        log_error "✗ $test_name failed (${duration_ms}ms >= ${max_duration_ms}ms)"
        ((TESTS_FAILED++))
    fi
}

# Main validation
main() {
    log_info "Starting comprehensive validation of envsense CLI"
    log_info "Environment: $(uname -s) $(uname -m)"
    log_info "Working directory: $(pwd)"
    
    # Check if we're in the right directory
    if [ ! -f "Cargo.toml" ]; then
        log_error "Must be run from the envsense project root directory"
        exit 1
    fi
    
    # Check if cargo is available
    if ! command -v cargo >/dev/null 2>&1; then
        log_error "cargo is not available"
        exit 1
    fi
    
    # Check if jq is available
    if ! command -v jq >/dev/null 2>&1; then
        log_error "jq is not available (required for JSON validation)"
        exit 1
    fi
    
    log_info "Building envsense..."
    if ! cargo build --quiet; then
        log_error "Failed to build envsense"
        exit 1
    fi
    
    log_info "Running comprehensive validation tests..."
    
    # 1. Basic CLI functionality
    run_test "Main help command" "cargo run -- --help" 0
    run_test "Info help command" "cargo run -- info --help" 0
    run_test "Check help command" "cargo run -- check --help" 0
    
    # 2. Schema version validation
    validate_json_output "Schema version 0.3.0" "cargo run -- info --json" "version" "0.3.0"
    
    # 3. JSON output structure validation
    validate_json_output "JSON has contexts field" "cargo run -- info --json" "contexts" "[]"
    validate_json_output "JSON has traits field" "cargo run -- info --json" "traits" "{}"
    validate_json_output "JSON has evidence field" "cargo run -- info --json" "evidence" "[]"
    
    # 4. Context detection validation
    run_test "Agent context detection" "cargo run -- check agent" 0
    run_test "IDE context detection" "cargo run -- check ide" 0
    run_test "Terminal context detection" "cargo run -- check terminal" 0
    run_test "CI context detection" "cargo run -- check ci" 0
    
    # 5. Field access validation
    run_test "Agent ID field access" "cargo run -- check agent.id" 0
    run_test "IDE ID field access" "cargo run -- check ide.id" 0
    run_test "Terminal interactive field access" "cargo run -- check terminal.interactive" 0
    run_test "Terminal stdin TTY field access" "cargo run -- check terminal.stdin.tty" 0
    run_test "Terminal stdout TTY field access" "cargo run -- check terminal.stdout.tty" 0
    run_test "Terminal stderr TTY field access" "cargo run -- check terminal.stderr.tty" 0
    run_test "CI ID field access" "cargo run -- check ci.id" 0
    
    # 6. Field comparison validation
    run_test "Agent ID comparison" "cargo run -- check 'agent.id=cursor'" 0
    run_test "IDE ID comparison" "cargo run -- check 'ide.id=vscode'" 0
    
    # 7. Negation validation
    run_test "Negated agent check" "cargo run -- check '!agent'" 0
    run_test "Negated CI check" "cargo run -- check '!ci'" 0
    
    # 8. Multiple predicates validation
    run_test "Multiple predicates" "cargo run -- check agent ide" 0
    
    # 9. CLI options validation
    run_test "Explain mode" "cargo run -- check --explain agent" 0
    run_test "Quiet mode" "cargo run -- check --quiet agent" 0
    run_test "JSON output mode" "cargo run -- check --json agent" 0
    run_test "Field listing" "cargo run -- check --list" 0
    
    # 10. Field filtering validation
    run_test "Field filtering" "cargo run -- info --json --fields contexts,traits" 0
    
    # 11. Performance validation
    validate_performance "Info command performance" "cargo run -- info --json" 1000
    validate_performance "Check command performance" "cargo run -- check agent" 500
    
    # 12. Edge case handling
    run_test "Empty input handling" "cargo run -- check ''" 1
    run_test "Long input handling" "cargo run -- check '$(printf 'a%.0s' {1..1000})'" 1
    
    # 13. README examples validation
    run_test "README agent example" "cargo run -- check agent" 0
    run_test "README agent.id example" "cargo run -- check agent.id" 0
    run_test "README agent.id=cursor example" "cargo run -- check 'agent.id=cursor'" 0
    run_test "README terminal.interactive example" "cargo run -- check terminal.interactive" 0
    run_test "README !ci example" "cargo run -- check '!ci'" 0
    
    # 14. Nested structure validation
    validate_json_output "Terminal stdin nested structure" "cargo run -- info --json" "traits.terminal.stdin.tty" "true"
    validate_json_output "Terminal stdout nested structure" "cargo run -- info --json" "traits.terminal.stdout.tty" "true"
    validate_json_output "Terminal stderr nested structure" "cargo run -- info --json" "traits.terminal.stderr.tty" "true"
    
    # Summary
    echo
    log_info "Validation complete!"
    log_info "Tests passed: $TESTS_PASSED"
    log_info "Tests failed: $TESTS_FAILED"
    
    if [ "$TESTS_FAILED" -eq 0 ]; then
        log_success "All tests passed! 🎉"
        exit 0
    else
        log_error "Some tests failed. Please review the output above."
        exit 1
    fi
}

# Run main function
main "$@"
