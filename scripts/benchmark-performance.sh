#!/usr/bin/env bash

# benchmark-performance.sh
#
# Purpose: Performance benchmarking and regression testing for envsense
# Created: 2024-01-15
# Used for: Performance monitoring and regression detection
#
# This script measures performance metrics for key operations and can be used
# to detect performance regressions over time.

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
WARMUP_RUNS=3
BENCHMARK_RUNS=10
TIMEOUT_SECONDS=30

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

# Timing function
time_command() {
    local command="$1"
    local timeout="$2"
    
    # Use timeout if available
    if command -v timeout >/dev/null 2>&1; then
        timeout "$timeout" bash -c "$command" 2>/dev/null
    else
        # Fallback for systems without timeout
        bash -c "$command" 2>/dev/null
    fi
}

# Benchmark a single command
benchmark_command() {
    local test_name="$1"
    local command="$2"
    local description="$3"
    
    log_info "Benchmarking: $test_name - $description"
    
    # Warmup runs
    for i in $(seq 1 $WARMUP_RUNS); do
        log_info "  Warmup run $i/$WARMUP_RUNS"
        time_command "$command" $TIMEOUT_SECONDS >/dev/null 2>&1 || true
    done
    
    # Actual benchmark runs
    local times=()
    for i in $(seq 1 $BENCHMARK_RUNS); do
        log_info "  Benchmark run $i/$BENCHMARK_RUNS"
        
        local start_time
        start_time=$(date +%s%3N)
        
        if time_command "$command" $TIMEOUT_SECONDS >/dev/null 2>&1; then
            local end_time
            end_time=$(date +%s%3N)
            local duration_ms
            duration_ms=$((end_time - start_time))
            times+=("$duration_ms")
        else
            log_warning "  Run $i failed or timed out"
        fi
    done
    
    # Calculate statistics
    if [ ${#times[@]} -eq 0 ]; then
        log_error "  All benchmark runs failed"
        return 1
    fi
    
    # Sort times for percentile calculation
    IFS=$'\n' sorted_times=($(sort -n <<<"${times[*]}"))
    unset IFS
    
    local count=${#sorted_times[@]}
    local total=0
    for time in "${sorted_times[@]}"; do
        total=$((total + time))
    done
    
    local mean=$((total / count))
    local median=${sorted_times[$((count / 2))]}
    local p95=${sorted_times[$((count * 95 / 100))]}
    local min=${sorted_times[0]}
    local max=${sorted_times[-1]}
    
    # Output results
    echo "  Results for $test_name:"
    echo "    Runs: $count"
    echo "    Mean: ${mean}ms"
    echo "    Median: ${median}ms"
    echo "    95th percentile: ${p95}ms"
    echo "    Min: ${min}ms"
    echo "    Max: ${max}ms"
    
    # Store results for summary
    echo "$test_name|$mean|$median|$p95|$min|$max" >> "/tmp/envsense_benchmark_$(date +%Y%m%d_%H%M%S).txt"
    
    log_success "  Benchmark completed for $test_name"
}

# Main benchmarking function
main() {
    log_info "Starting envsense performance benchmarking"
    log_info "Environment: $(uname -s) $(uname -m)"
    log_info "Working directory: $(pwd)"
    log_info "Warmup runs: $WARMUP_RUNS"
    log_info "Benchmark runs: $BENCHMARK_RUNS"
    log_info "Timeout: ${TIMEOUT_SECONDS}s"
    
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
    
    # Build the project
    log_info "Building envsense..."
    if ! cargo build --release --quiet; then
        log_error "Failed to build envsense"
        exit 1
    fi
    
    # Create results file
    local results_file="/tmp/envsense_benchmark_$(date +%Y%m%d_%H%M%S).txt"
    echo "Test|Mean(ms)|Median(ms)|P95(ms)|Min(ms)|Max(ms)" > "$results_file"
    
    log_info "Starting benchmarks..."
    
    # 1. Basic info command
    benchmark_command \
        "info_basic" \
        "cargo run --release -- info" \
        "Basic info command without JSON"
    
    # 2. Info command with JSON
    benchmark_command \
        "info_json" \
        "cargo run --release -- info --json" \
        "Info command with JSON output"
    
    # 3. Info command with field filtering
    benchmark_command \
        "info_filtered" \
        "cargo run --release -- info --json --fields contexts,traits" \
        "Info command with field filtering"
    
    # 4. Context detection
    benchmark_command \
        "check_agent" \
        "cargo run --release -- check agent" \
        "Agent context detection"
    
    benchmark_command \
        "check_ide" \
        "cargo run --release -- check ide" \
        "IDE context detection"
    
    benchmark_command \
        "check_terminal" \
        "cargo run --release -- check terminal" \
        "Terminal context detection"
    
    benchmark_command \
        "check_ci" \
        "cargo run --release -- check ci" \
        "CI context detection"
    
    # 5. Field access
    benchmark_command \
        "check_agent_id" \
        "cargo run --release -- check agent.id" \
        "Agent ID field access"
    
    benchmark_command \
        "check_terminal_interactive" \
        "cargo run --release -- check terminal.interactive" \
        "Terminal interactive field access"
    
    benchmark_command \
        "check_terminal_stdin_tty" \
        "cargo run --release -- check terminal.stdin.tty" \
        "Terminal stdin TTY field access"
    
    # 6. Field comparison
    benchmark_command \
        "check_agent_id_cursor" \
        "cargo run --release -- check 'agent.id=cursor'" \
        "Agent ID comparison"
    
    benchmark_command \
        "check_terminal_interactive_true" \
        "cargo run --release -- check 'terminal.interactive=true'" \
        "Terminal interactive comparison"
    
    # 7. Multiple predicates
    benchmark_command \
        "check_multiple" \
        "cargo run --release -- check agent ide terminal" \
        "Multiple predicate evaluation"
    
    # 8. Negation
    benchmark_command \
        "check_negation" \
        "cargo run --release -- check '!ci'" \
        "Negated predicate evaluation"
    
    # 9. Explain mode
    benchmark_command \
        "check_explain" \
        "cargo run --release -- check --explain agent" \
        "Explain mode output"
    
    # 10. JSON output mode
    benchmark_command \
        "check_json" \
        "cargo run --release -- check --json agent" \
        "Check command with JSON output"
    
    # 11. Field listing
    benchmark_command \
        "check_list" \
        "cargo run --release -- check --list" \
        "Field listing generation"
    
    # 12. Complex nested field access
    benchmark_command \
        "check_nested_deep" \
        "cargo run --release -- check terminal.stdin.tty" \
        "Deep nested field access"
    
    # Summary
    echo
    log_info "Benchmarking complete!"
    log_info "Results saved to: $results_file"
    
    # Display summary table
    echo
    log_info "Performance Summary:"
    echo "======================"
    column -t -s '|' "$results_file"
    
    # Performance thresholds and warnings
    echo
    log_info "Performance Analysis:"
    
    local warning_count=0
    while IFS='|' read -r test_name mean median p95 min max; do
        # Skip header
        if [ "$test_name" = "Test" ]; then
            continue
        fi
        
        # Check for performance issues
        if [ "$mean" -gt 1000 ]; then
            log_warning "$test_name: Mean time ${mean}ms exceeds 1 second threshold"
            ((warning_count++))
        fi
        
        if [ "$p95" -gt 2000 ]; then
            log_warning "$test_name: 95th percentile ${p95}ms exceeds 2 second threshold"
            ((warning_count++))
        fi
        
        # Check for high variance
        local variance=$((max - min))
        if [ "$variance" -gt "$mean" ]; then
            log_warning "$test_name: High variance detected (${variance}ms range)"
            ((warning_count++))
        fi
    done < "$results_file"
    
    if [ "$warning_count" -eq 0 ]; then
        log_success "All performance metrics within acceptable ranges! 🎉"
    else
        log_warning "$warning_count performance warnings detected"
    fi
    
    log_info "Results file: $results_file"
    log_info "You can compare this with previous runs to detect regressions"
}

# Run main function
main "$@"
