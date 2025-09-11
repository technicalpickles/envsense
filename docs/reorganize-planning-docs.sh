#!/usr/bin/env bash

# reorganize-planning-docs.sh
#
# Purpose: Reorganize docs/planning to group related work and move completed items to archive
# Usage: ./docs/reorganize-planning-docs.sh (run from repository root)

set -euo pipefail

# Colors for output
RED='\033[0;31m'
YELLOW='\033[1;33m'
GREEN='\033[0;32m'
NC='\033[0m' # No Color

# Safe move function with warnings
move() {
    local src="$1"
    local dest="$2"
    
    # Check if source exists
    if [[ ! -f "$src" ]]; then
        echo -e "    ${RED}⚠️  WARNING: Source file does not exist: $src${NC}"
        return 1
    fi
    
    # Check if destination already exists
    if [[ -f "$dest" ]]; then
        echo -e "    ${YELLOW}⚠️  WARNING: Destination already exists, skipping: $dest${NC}"
        return 1
    fi
    
    # Create destination directory if it doesn't exist
    local dest_dir
    dest_dir=$(dirname "$dest")
    if [[ ! -d "$dest_dir" ]]; then
        mkdir -p "$dest_dir"
    fi
    
    # Perform the move
    mv "$src" "$dest"
    return 0
}

# Create directory structure
mkdir -p docs/archive/planning/lld-adoption
mkdir -p docs/archive/planning/declarative-consolidation
mkdir -p docs/planning/cli-streamlining
mkdir -p docs/planning/aqua-distribution
mkdir -p docs/planning/release-workflow

# Move completed work to archive
move docs/planning/adopt-lld.md docs/archive/planning/lld-adoption/plan.md
move docs/planning/lld.md docs/archive/planning/lld-adoption/strategy.md
move docs/planning/declarative-detector-consolidation.md docs/archive/planning/declarative-consolidation/plan.md
move docs/planning/testing-strategy-consolidation.md docs/archive/planning/declarative-consolidation/testing-strategy.md
move docs/planning/contextual-value-extraction.md docs/archive/planning/contextual-value-extraction.md
move docs/planning/override-system-design.md docs/archive/planning/override-system-design.md
move docs/planning/additional-cli-improvements.md docs/archive/planning/additional-cli-improvements.md
move docs/planning/additional-cli-improvements-implementation.md docs/archive/planning/additional-cli-improvements-implementation.md

# Group CLI streamlining work
move docs/planning/streamlining-cli.md docs/planning/cli-streamlining/plan.md
move docs/planning/cli-streamline-implementation.md docs/planning/cli-streamlining/implementation-overview.md
move docs/planning/cli-streamline-implementation-phase-1.md docs/planning/cli-streamlining/phase-1-foundation.md
move docs/planning/cli-streamline-implementation-phase-2.md docs/planning/cli-streamlining/phase-2-parser.md
move docs/planning/cli-streamline-implementation-phase-3.md docs/planning/cli-streamlining/phase-3-detection.md
move docs/planning/cli-streamline-implementation-phase-4.md docs/planning/cli-streamlining/phase-4-cli-integration.md
move docs/planning/cli-streamline-implementation-phase-5.md docs/planning/cli-streamlining/phase-5-migration.md

# Group aqua distribution work
move docs/planning/aqua-mise-distribution-plan.md docs/planning/aqua-distribution/plan.md
move docs/planning/signing-validation-strategy.md docs/planning/aqua-distribution/signing-validation.md

# Group release workflow work
move docs/planning/release-workflow-implementation-plan.md docs/planning/release-workflow/plan.md

echo "Reorganization complete. Check docs/planning/ for the new structure."
