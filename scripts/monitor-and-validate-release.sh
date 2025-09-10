#!/usr/bin/env bash
# Monitor release workflow and automatically validate signing when complete

set -euo pipefail

# Configuration
REPO="${1:-technicalpickles/envsense}"
TIMEOUT_MINUTES="${2:-30}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
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

log_step() {
    echo -e "${CYAN}[STEP]${NC} $1"
}

# Check if we're in the right directory
check_environment() {
    if [ ! -f "Cargo.toml" ]; then
        log_error "This script must be run from the envsense repository root"
        exit 1
    fi
    
    if [ ! -f "scripts/monitor-release-workflow.sh" ]; then
        log_error "Missing monitor-release-workflow.sh script"
        exit 1
    fi
    
    if [ ! -f "scripts/validate-signing.sh" ]; then
        log_error "Missing validate-signing.sh script"
        exit 1
    fi
}

# Get the version that will be released
get_expected_version() {
    grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/'
}

# Wait for and validate a new release
main() {
    echo -e "${CYAN}"
    echo "🚀 Release Monitor & Validator"
    echo "=============================="
    echo -e "${NC}"
    
    check_environment
    
    local expected_version=$(get_expected_version)
    log_info "Expected release version: $expected_version"
    log_info "Repository: $REPO"
    log_info "Timeout: $TIMEOUT_MINUTES minutes"
    echo
    
    # Step 1: Monitor workflow
    log_step "Step 1: Monitoring release workflow..."
    if ./scripts/monitor-release-workflow.sh "$REPO" "Release" "main" "$TIMEOUT_MINUTES"; then
        log_success "Release workflow completed successfully!"
    else
        log_error "Release workflow failed or timed out"
        exit 1
    fi
    
    echo
    log_step "Step 2: Waiting for release to be published..."
    
    # Give GitHub a moment to publish the release
    sleep 10
    
    # Step 2: Validate the release
    log_step "Step 3: Validating release signatures..."
    
    if ./scripts/validate-signing.sh "$expected_version"; then
        log_success "🎉 Release validation completed successfully!"
        echo
        log_success "✅ Release $expected_version is ready!"
        log_success "✅ Signatures are valid and verifiable"
        log_success "✅ Ready for aqua registry submission"
        
        echo
        echo -e "${CYAN}Next steps:${NC}"
        echo "1. Test local aqua configuration: ./scripts/test-aqua-local.sh"
        echo "2. Submit to aqua registry: https://github.com/aquaproj/aqua-registry"
        echo "3. Update README with installation instructions"
        
        return 0
    else
        log_error "💥 Release validation failed!"
        echo
        log_warning "Possible issues:"
        log_warning "- Signature verification problems"
        log_warning "- Missing bundle or signature files"
        log_warning "- Certificate identity mismatch"
        
        echo
        echo -e "${CYAN}Debugging steps:${NC}"
        echo "1. Check release assets: gh release view $expected_version"
        echo "2. Run validation with debugging: ./scripts/validate-signing.sh $expected_version"
        echo "3. Check GitHub Actions logs for signing errors"
        
        return 1
    fi
}

# Handle help
if [ "${1:-}" = "--help" ] || [ "${1:-}" = "-h" ]; then
    echo "Usage: $0 [REPO] [TIMEOUT_MINUTES]"
    echo
    echo "Monitor release workflow and automatically validate signing when complete"
    echo
    echo "Arguments:"
    echo "  REPO              Repository (default: technicalpickles/envsense)"
    echo "  TIMEOUT_MINUTES   Timeout in minutes (default: 30)"
    echo
    echo "Examples:"
    echo "  $0                        # Use defaults"
    echo "  $0 myorg/myrepo          # Monitor different repo"
    echo "  $0 myorg/myrepo 60       # Custom timeout"
    echo
    echo "This script will:"
    echo "1. Monitor the release workflow until completion"
    echo "2. Wait for the release to be published"
    echo "3. Automatically validate signatures using validate-signing.sh"
    echo "4. Provide next steps for aqua registry submission"
    echo
    exit 0
fi

# Run main function
main "$@"
