# envsense

**envsense** is a cross-language library and CLI for detecting the runtime environment and exposing it in a structured way. It helps tools and scripts adapt their behavior based on where they are running.

## Motivation

Most developers end up writing brittle ad-hoc checks in their shell configs or tools:

* *"If I’m in VS Code, set `EDITOR=code -w`"*
* *"If stdout is piped, disable color and paging"*
* *"If running in a coding agent, simplify my prompt"*

These heuristics get duplicated across dotfiles and codebases. **envsense** centralizes and standardizes this detection.

---

## Quick Start (Shell)

```bash
# Detect and show agent status
envsense check agent
envsense check --format json agent

# Simple check for any coding agent
envsense check agent && echo "Running inside a coding agent"

# Check if specifically running in Cursor
envsense check facet:agent_id=cursor && echo "Cursor detected"

# Check if running in VS Code or VS Code Insiders
envsense check facet:ide_id=vscode && echo "VS Code"
envsense check facet:ide_id=vscode-insiders && echo "VS Code Insiders"

# Combine checks in scripts
if envsense check -q facet:ide_id=cursor; then
  export PAGER=cat
fi
```

---

## Exploring the Environment

You can also inspect the full structured output to see what envsense detects:

```bash
# Human-friendly summary
envsense info

# Print detected environment as JSON
envsense info --json

# Restrict to specific keys
envsense info --fields contexts,traits

# Pipe-friendly text
envsense info --raw
```

Example JSON output:

```json
{
  "contexts": ["agent", "ide"],
  "traits": {
    "is_interactive": true,
    "color_level": "truecolor",
    "supports_hyperlinks": true,
    "is_piped_stdout": false
  },
  "facets": {
    "agent_id": "cursor",
    "ide_id": "vscode"
  },
  "meta": {
    "schema_version": "0.1.0",
    "rules_version": ""
  }
}
```

## Key Concepts

* **Contexts** — broad categories of environment (`agent`, `ide`, `ci`, `container`, `remote`).
* **Facets** — more specific identifiers (`agent_id=cursor`, `ide_id=vscode`).
* **Traits** — capabilities or properties (`is_interactive`, `supports_hyperlinks`, `color_level`).
* **Evidence** — why envsense believes something (env vars, TTY checks, etc.), with confidence scores.

### Ask: Which category is it?

* *“Running in VS Code”* → **Context/Facet** (`ide`, `ide_id=vscode`).
* *“Running in GitHub Actions”* → **Facet** (`ci_id=github`).
* *“Running in CI at all”* → **Context** (`ci=true`).
* *“Can print hyperlinks”* → **Trait** (`supports_hyperlinks=true`).
* *“stdout is not a TTY”* → **Trait** (`is_interactive=false`).

### Snippet Examples

```bash
# Context: any CI
envsense check ci && echo "In CI"

# Facet: specifically GitHub Actions
envsense check facet:ci_id=github && echo "In GitHub Actions"

# Trait: non-interactive
if ! envsense check trait:is_interactive; then
  echo "Running non-interactive"
fi
```

---

## Language Bindings

* **Node.js**:

  ```js
  import { detect } from "envsense";
  const ctx = await detect();
  if (ctx.contexts.agent) console.log("Agent detected");
  if (ctx.facets.ide_id === "vscode") console.log("VS Code detected");
  if (!ctx.traits.is_interactive) console.log("Non-interactive session");
  ```

---

## Detection Strategy

1. **Explicit signals** — documented env vars (e.g. `TERM_PROGRAM`, `INSIDE_EMACS`, `CI`).
2. **Execution channel** — SSH env vars, container cgroups, devcontainer markers.
3. **Process ancestry** — parent process names (optional, behind a flag).
4. **Heuristics** — last resort (file paths, working dir markers).

Precedence is: user override > explicit > channel > ancestry > heuristics.

---

## Terminal Features Detected

* **Interactivity**

  * Shell flags (interactive mode)
  * TTY checks for stdin/stdout/stderr
  * Pipe/redirect detection
* **Colors**

  * Honors `NO_COLOR`, `FORCE_COLOR`
  * Detects depth: none, basic, 256, truecolor
* **Hyperlinks (OSC 8)**

  * Known supporting terminals (iTerm2, kitty, WezTerm, VS Code, etc.)
  * Optional probe for fallback

---

## Project Status

* Prototype phase
* Rust core + CLI planned
* Node.js bindings first
* Rule definitions in YAML for extensibility

---

## Roadmap

* [ ] Implement baseline detectors (env vars, TTY)
* [ ] CLI output in JSON + pretty modes
* [ ] Rule engine for contexts/facets/traits
* [ ] Node binding via napi-rs
* [ ] Profiles for common cases (agent, CI, IDE)

---

## License

MIT

---

**envsense**: environment awareness for any tool, in any language.