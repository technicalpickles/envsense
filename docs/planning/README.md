# Planning Documentation

This directory contains active planning documents for ongoing and future
envsense development work.

## Active Projects

### 🚧 CLI Streamlining

**Status**: In Progress (Phase 3 of 5)  
**Directory**: `cli-streamlining/`

Major overhaul of the CLI interface to use dot notation syntax and nested JSON
schema.

- ✅ Phase 1: Foundation & Schema (completed)
- ✅ Phase 2: Parser & Evaluation (completed)
- 🔄 Phase 3: Detection System (in progress)
- ⏳ Phase 4: CLI Integration (pending)
- ⏳ Phase 5: Migration & Cleanup (pending)

### 📦 Aqua Distribution

**Status**: Planned  
**Directory**: `aqua-distribution/`

Implementation of `mise install aqua:envsense` support with release signing and
registry submission. Waiting for CLI stabilization before implementation.

### 🚀 Release Workflow

**Status**: Planned  
**Directory**: `release-workflow/`

Automated GitHub Actions workflow for building and publishing cross-platform
binaries. Lower priority item for when distribution is needed.

## Completed Projects

Completed planning work has been moved to `docs/archive/planning/`:

- **LLD Adoption** - Fast linker integration (100% complete)
- **Declarative Detector Consolidation** - Code consolidation and override
  system (100% complete)
- **Contextual Value Extraction** - CI-specific value mappings (100% complete)
- **Override System Design** - Comprehensive override system (100% complete)
- **Additional CLI Improvements** - Error handling and output formatting (100%
  complete)

## Directory Structure

```
docs/planning/
├── README.md                    # This file
├── cli-streamlining/           # Active: CLI interface overhaul
│   ├── status.md               # Current status and progress
│   ├── plan.md                 # Original plan
│   ├── implementation-overview.md
│   └── phase-*.md              # Individual phase details
├── aqua-distribution/          # Planned: Package distribution
│   ├── status.md
│   ├── plan.md
│   └── signing-validation.md
└── release-workflow/           # Planned: Automated releases
    ├── status.md
    └── plan.md
```

## File Naming Conventions

Within each project directory:

- `status.md` - Current progress and next steps
- `plan.md` - Original planning document
- `implementation-*.md` - Implementation details
- `phase-*.md` - Phase-specific documentation

This structure groups related work together and makes it easy to track progress
on multi-document projects.

## Reorganization

To reorganize the planning directory (move completed work to archive and group
related documents):

```bash
# Run from repository root
./docs/reorganize-planning-docs.sh
```

This script will:

- Move completed work to `docs/archive/planning/`
- Group related documents into project directories
- Apply standardized naming conventions (`plan.md`, `status.md`, etc.)
- Preserve all content while improving organization

The script includes safety checks and will warn about missing files or existing
destinations without overwriting.
