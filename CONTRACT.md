# Contract

This document lists invariants for `envsense`'s consumer‑facing schema.
All JSON field names are in `snake_case`.  Any consumer‑visible key or enum
string must remain stable; renames require `serde` aliases or a version bump.

## Schema Stability Guarantees

The JSON schema version is tracked in the `schema_version` field. Within a major 
schema version (e.g., `0.1.x`), all field names and enum values listed in this
contract MUST remain stable for backward compatibility.

### Breaking Changes Policy
- **Field renames**: Require `serde` aliases to maintain old names
- **Field removal**: Requires schema version bump  
- **Enum value changes**: Requires schema version bump or aliases
- **New fields**: Can be added freely (non-breaking)
- **New enum values**: Can be added freely (non-breaking)

### Consumer Examples
```bash
# These consumer patterns must continue to work:
jq '.contexts[]' <<< "$output"           # context array access
jq '.facets.ide_id' <<< "$output"        # facet field access  
jq '.traits.is_interactive' <<< "$output" # trait field access
jq -e '.facets.ci.is_ci == true' <<< "$output" # CI detection
```

## Field names

### EnvSense
```
contexts
facets
traits
evidence
version
rules_version
```

### contexts
```
agent
ide
ci
container
remote
```

### facets
```
agent_id
ide_id
ci_id
container_id
ci
```

### facets.ci
```
is_ci
vendor
name
pr
branch
```

### traits
```
is_interactive
is_tty_stdin
is_tty_stdout
is_tty_stderr
is_piped_stdin
is_piped_stdout
color_level
supports_hyperlinks
```

### evidence
```
signal
key
value
supports
confidence
```

## Enum strings
- Signal: `env`, `tty`, `proc`, `fs`
- ColorLevel: `none`, `ansi16`, `ansi256`, `truecolor`

## Detection Precedence Order

Detection follows this precedence hierarchy (highest to lowest):

1. **User override** - Explicit user configuration or flags
2. **Explicit signals** - Well-documented environment variables
   - `TERM_PROGRAM` (VS Code/IDE detection)
   - `GITHUB_ACTIONS`, `GITLAB_CI` (CI system detection)
   - `CURSOR_TRACE_ID` (Cursor agent detection)
3. **Channel** - Execution environment context  
   - SSH connection variables
   - Container runtime detection
   - Devcontainer markers
4. **Ancestry** - Process parent/child relationships (optional, behind flag)
5. **Heuristics** - File system and working directory analysis (last resort)

## Evidence and Confidence

Each detection result includes `Evidence` with:
- **Signal type**: `env`, `tty`, `proc`, `fs` 
- **Key/value**: The specific indicator (e.g., `TERM_PROGRAM=vscode`)
- **Supports**: Which contexts/facets this evidence supports
- **Confidence**: 0.0-1.0 score for reliability

### Evidence Cues
Evidence supports these detection targets:
- **Contexts**: `ide`, `agent`, `ci`, `container`, `remote`
- **Facets**: `ide_id`, `agent_id`, `ci_id`, `container_id`
- **Traits**: `is_interactive`, `supports_hyperlinks`, `color_level`

### CI-Specific Fields
The `ci` facet includes additional metadata:
- `vendor`: CI system identifier (`github_actions`, `gitlab_ci`)
- `name`: Human-readable CI system name  
- `pr`: Boolean indicating if running in pull request context
- `branch`: Current branch name (when available)

