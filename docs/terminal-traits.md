# Terminal Traits

`envsense` automatically detects several properties of the current terminal.
These traits are exposed through `envsense info` and the `--json` output.
Detection is powered by well established crates and honors common environment
variables such as `NO_COLOR` and `FORCE_COLOR`.

## Fields

- `color_level` – `none`, `ansi16`, `ansi256`, or `truecolor` as reported by
  [`supports-color`](https://crates.io/crates/supports-color).
- `is_interactive` – true when both stdin and stdout are TTYs.
- `is_tty_stdin`, `is_tty_stdout`, `is_tty_stderr` – whether each stream is
  attached to a TTY as detected by
  [`is-terminal`](https://crates.io/crates/is-terminal).
- `is_piped_stdin`, `is_piped_stdout` – derived inverses of the `is_tty_*`
  checks for convenience.
- `supports_hyperlinks` – terminal supports OSC 8 hyperlinks as detected by
  [`supports-hyperlinks`](https://crates.io/crates/supports-hyperlinks).

`is_interactive` is derived from the TTY checks and does not attempt to inspect
shell state.

## Usage

The traits are available in both human and JSON outputs:

```bash
envsense info --fields=traits
envsense info --json --fields=traits
```

The JSON form uses the same field names and values shown above.
