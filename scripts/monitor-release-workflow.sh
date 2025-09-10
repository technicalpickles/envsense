#!/usr/bin/env bash
# Monitor GitHub Actions release workflow and wait for completion

set -euo pipefail

# Configuration
REPO="${1:-technicalpickles/envsense}"
WORKFLOW_NAME="${2:-Release}"
BRANCH="${3:-main}"
TIMEOUT_MINUTES="${4:-30}"
POLL_INTERVAL_SECONDS="${5:-30}"

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

# Check dependencies
check_dependencies() {
    if ! command -v gh &> /dev/null; then
        log_error "GitHub CLI (gh) is not installed. Please install it first:"
        log_error "   brew install gh"
        exit 1
    fi

    if ! command -v jq &> /dev/null; then
        log_error "jq is not installed. Please install it first:"
        log_error "   brew install jq"
        exit 1
    fi

    # Check if authenticated
    if ! gh auth status &> /dev/null; then
        log_error "GitHub CLI is not authenticated. Please run:"
        log_error "   gh auth login"
        exit 1
    fi
}

# Get the latest workflow run ID for the release workflow
get_latest_release_run() {
    gh run list \
        --repo "$REPO" \
        --workflow "$WORKFLOW_NAME" \
        --branch "$BRANCH" \
        --limit 1 \
        --json databaseId,status,conclusion,createdAt,updatedAt \
        --jq '.[0]'
}

# Wait for a new release workflow to start
wait_for_new_release() {
    local baseline_run_id="$1"
    local start_time=$(date +%s)
    local timeout_seconds=$((TIMEOUT_MINUTES * 60))
    
    log_info "Waiting for new release workflow to start..."
    log_info "Baseline run ID: ${baseline_run_id:-"none"}"
    log_info "Timeout: $TIMEOUT_MINUTES minutes"
    
    while true; do
        local current_time=$(date +%s)
        local elapsed=$((current_time - start_time))
        
        if [ $elapsed -gt $timeout_seconds ]; then
            log_error "Timeout waiting for new release workflow to start"
            exit 1
        fi
        
        local latest_run=$(get_latest_release_run)
        local latest_run_id=$(echo "$latest_run" | jq -r '.databaseId // empty')
        
        if [ -n "$latest_run_id" ] && [ "$latest_run_id" != "$baseline_run_id" ]; then
            log_success "New release workflow detected: Run ID $latest_run_id"
            echo "$latest_run"
            return 0
        fi
        
        local remaining=$((timeout_seconds - elapsed))
        log_info "Still waiting... (${remaining}s remaining)"
        sleep $POLL_INTERVAL_SECONDS
    done
}

# Monitor a specific workflow run until completion
monitor_workflow_run() {
    local run_id="$1"
    local start_time=$(date +%s)
    local timeout_seconds=$((TIMEOUT_MINUTES * 60))
    
    log_info "Monitoring workflow run: $run_id"
    log_info "Repository: $REPO"
    log_info "Workflow: $WORKFLOW_NAME"
    
    while true; do
        local current_time=$(date +%s)
        local elapsed=$((current_time - start_time))
        
        if [ $elapsed -gt $timeout_seconds ]; then
            log_error "Timeout monitoring workflow run"
            exit 1
        fi
        
        # Get current run status
        local run_info=$(gh run view "$run_id" \
            --repo "$REPO" \
            --json status,conclusion,createdAt,updatedAt,url)
        
        local status=$(echo "$run_info" | jq -r '.status')
        local conclusion=$(echo "$run_info" | jq -r '.conclusion // empty')
        local url=$(echo "$run_info" | jq -r '.url')
        
        log_info "Status: $status | Conclusion: ${conclusion:-"in progress"}"
        
        # Check if completed
        if [ "$status" = "completed" ]; then
            echo
            if [ "$conclusion" = "success" ]; then
                log_success "🎉 Workflow completed successfully!"
                log_success "URL: $url"
                return 0
            else
                log_error "💥 Workflow failed with conclusion: $conclusion"
                log_error "URL: $url"
                return 1
            fi
        fi
        
        # Show progress for running workflows
        if [ "$status" = "in_progress" ]; then
            # Get job details for more granular status
            local jobs=$(gh run view "$run_id" \
                --repo "$REPO" \
                --json jobs \
                --jq '.jobs[] | {name: .name, status: .status, conclusion: .conclusion}')
            
            echo "  Jobs status:"
            echo "$jobs" | jq -r '  "    \(.name): \(.status) \(if .conclusion then "(\(.conclusion))" else "" end)"'
        fi
        
        sleep $POLL_INTERVAL_SECONDS
    done
}

# Main function
main() {
    log_info "🔍 Release Workflow Monitor"
    log_info "Repository: $REPO"
    log_info "Workflow: $WORKFLOW_NAME"
    log_info "Branch: $BRANCH"
    echo
    
    check_dependencies
    
    # Get current latest run as baseline
    local baseline_run=$(get_latest_release_run)
    local baseline_run_id=$(echo "$baseline_run" | jq -r '.databaseId // empty')
    
    if [ -n "$baseline_run_id" ]; then
        local baseline_status=$(echo "$baseline_run" | jq -r '.status')
        local baseline_conclusion=$(echo "$baseline_run" | jq -r '.conclusion // empty')
        
        log_info "Current latest run: $baseline_run_id ($baseline_status)"
        
        # If the latest run is already in progress, monitor it instead of waiting
        if [ "$baseline_status" = "in_progress" ] || [ "$baseline_status" = "queued" ]; then
            log_info "Latest run is already in progress, monitoring it..."
            monitor_workflow_run "$baseline_run_id"
            return $?
        fi
    else
        log_info "No previous release workflows found"
    fi
    
    # Wait for new workflow to start
    local new_run=$(wait_for_new_release "$baseline_run_id")
    local new_run_id=$(echo "$new_run" | jq -r '.databaseId')
    
    # Monitor the new workflow
    monitor_workflow_run "$new_run_id"
}

# Handle script arguments and help
if [ "${1:-}" = "--help" ] || [ "${1:-}" = "-h" ]; then
    echo "Usage: $0 [REPO] [WORKFLOW_NAME] [BRANCH] [TIMEOUT_MINUTES] [POLL_INTERVAL_SECONDS]"
    echo
    echo "Monitor GitHub Actions release workflow and wait for completion"
    echo
    echo "Arguments:"
    echo "  REPO                    Repository (default: technicalpickles/envsense)"
    echo "  WORKFLOW_NAME           Workflow name (default: Release)"
    echo "  BRANCH                  Branch to monitor (default: main)"
    echo "  TIMEOUT_MINUTES         Timeout in minutes (default: 30)"
    echo "  POLL_INTERVAL_SECONDS   Polling interval (default: 30)"
    echo
    echo "Examples:"
    echo "  $0                                    # Use all defaults"
    echo "  $0 myorg/myrepo                      # Monitor different repo"
    echo "  $0 myorg/myrepo CI main 60 10        # Custom timeout and interval"
    echo
    echo "Environment Variables:"
    echo "  GITHUB_TOKEN    GitHub token (or use 'gh auth login')"
    echo
    exit 0
fi

# Run main function
main "$@"
