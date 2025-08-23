# Contract

This document lists invariants for `envsense`'s consumer‑facing schema.
All JSON field names are in `snake_case`.  Any consumer‑visible key or enum
string must remain stable; renames require `serde` aliases or a version bump.

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

## Precedence order
`user override` > `explicit signals` > `channel` > `ancestry` > `heuristics`

## Evidence cues
`ide`, `ide_id`, `agent`, `agent_id`

