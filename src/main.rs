use clap::{Args, ColorChoice, CommandFactory, FromArgMatches, Parser, Subcommand, ValueEnum};
use colored::Colorize;
use envsense::check::{self, CONTEXTS, FACETS, ParsedCheck, TRAITS};
use envsense::envsense_ci::{CiFacet, ci_traits};
use envsense::schema::{EnvSense, Evidence};
use serde_json::{Map, Value, json};
use std::collections::BTreeMap;
use std::io::{IsTerminal, stdout};
use std::sync::OnceLock;

fn check_predicate_long_help() -> &'static str {
    static HELP: OnceLock<String> = OnceLock::new();
    HELP.get_or_init(|| {
        let mut s = String::from("Predicates to evaluate\n\n");
        s.push_str("Contexts:\n");
        for c in CONTEXTS {
            s.push_str("    ");
            s.push_str(c);
            s.push('\n');
        }
        s.push_str("Facets:\n");
        for f in FACETS {
            s.push_str("    facet:");
            s.push_str(f);
            s.push_str("=<VALUE>\n");
        }
        s.push_str("Traits:\n");
        for t in TRAITS {
            s.push_str("    trait:");
            s.push_str(t);
            s.push('\n');
        }
        s
    })
    .as_str()
}

#[derive(Parser)]
#[command(
    name = "envsense",
    about = "Environment awareness utilities",
    arg_required_else_help = true
)]
struct Cli {
    /// Disable color
    #[arg(long = "no-color", global = true)]
    no_color: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Show what envsense knows
    Info(InfoArgs),
    /// Evaluate predicates against the environment
    Check(CheckCmd),
}

#[derive(Args, Clone)]
struct InfoArgs {
    /// Output JSON (stable schema)
    #[arg(long)]
    json: bool,

    /// Plain text without colors/headers
    #[arg(long)]
    raw: bool,

    /// Comma-separated keys to include: contexts,traits,facets,meta
    #[arg(long, value_name = "list")]
    fields: Option<String>,
}

#[derive(Args, Clone)]
struct CheckCmd {
    #[arg(
        value_name = "PREDICATE",
        num_args = 1..,
        help = "Predicates to evaluate",
        long_help = check_predicate_long_help(),
        required_unless_present = "list_checks"
    )]
    predicates: Vec<String>,

    /// Succeed if any predicate matches (default: all must match)
    #[arg(long, action = clap::ArgAction::SetTrue, conflicts_with = "all")]
    any: bool,

    /// Require all predicates to match (default)
    #[arg(long, action = clap::ArgAction::SetTrue, conflicts_with = "any")]
    all: bool,

    /// Suppress output
    #[arg(short, long, alias = "silent", action = clap::ArgAction::SetTrue)]
    quiet: bool,

    /// Output format
    #[arg(long, value_enum, default_value_t = Format::Plain)]
    format: Format,

    /// Explain reasoning
    #[arg(long)]
    explain: bool,

    /// List known predicates
    #[arg(long = "list-checks", action = clap::ArgAction::SetTrue)]
    list_checks: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
enum Format {
    Plain,
    Pretty,
    Json,
}

#[derive(serde::Serialize)]
struct JsonCheck {
    predicate: String,
    result: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    signals: Option<BTreeMap<String, String>>,
}

#[derive(Debug)]
struct Snapshot {
    contexts: Vec<String>,
    traits: Value,
    facets: Value,
    meta: Value,
}

fn collect_snapshot() -> Snapshot {
    let env = EnvSense::default();
    let mut contexts = Vec::new();
    if env.contexts.agent {
        contexts.push("agent".to_string());
    }
    if env.contexts.ide {
        contexts.push("ide".to_string());
    }
    if env.contexts.ci {
        contexts.push("ci".to_string());
    }
    if env.contexts.container {
        contexts.push("container".to_string());
    }
    if env.contexts.remote {
        contexts.push("remote".to_string());
    }
    let mut traits_val = serde_json::to_value(env.traits).unwrap();
    if let Value::Object(map) = &mut traits_val {
        for (k, v) in ci_traits(&env.facets.ci) {
            map.insert(k, v);
        }
    }
    Snapshot {
        contexts,
        traits: traits_val,
        facets: serde_json::to_value(env.facets).unwrap(),
        meta: json!({
            "schema_version": env.version,
            "rules_version": env.rules_version,
        }),
    }
}

fn filter_json_fields(value: Value, fields: &str) -> Result<Value, String> {
    let requested: Vec<&str> = fields
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();
    let obj = value
        .as_object()
        .ok_or_else(|| "expected object".to_string())?;
    let mut map = Map::new();
    for k in requested {
        if let Some(v) = obj.get(k) {
            map.insert(k.to_string(), v.clone());
        } else {
            return Err(format!("unknown field: {}", k));
        }
    }
    Ok(Value::Object(map))
}

fn value_to_string(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        _ => v.to_string(),
    }
}

fn colorize_value(v: &str, color: bool) -> String {
    if !color {
        return v.to_string();
    }
    match v {
        "true" => v.green().to_string(),
        "false" | "none" => v.red().to_string(),
        _ => v.to_string(),
    }
}

fn render_human(
    snapshot: &Snapshot,
    fields: Option<&str>,
    color: bool,
    raw: bool,
) -> Result<String, String> {
    let default_fields = ["contexts", "traits", "facets"];
    let selected: Vec<&str> = match fields {
        Some(f) => f
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect(),
        None => default_fields.to_vec(),
    };
    for s in &selected {
        if !["contexts", "traits", "facets", "meta"].contains(s) {
            return Err(format!("unknown field: {}", s));
        }
    }
    let mut out = String::new();
    for (i, field) in selected.iter().enumerate() {
        match *field {
            "contexts" => {
                let mut ctx = snapshot.contexts.clone();
                ctx.sort();
                if raw {
                    for (j, c) in ctx.iter().enumerate() {
                        if j > 0 {
                            out.push('\n');
                        }
                        out.push_str(c);
                    }
                } else {
                    let heading = if color {
                        "Contexts:".bold().cyan().to_string()
                    } else {
                        "Contexts:".to_string()
                    };
                    out.push_str(&heading);
                    for c in ctx {
                        out.push('\n');
                        out.push_str("  ");
                        out.push_str(&c);
                    }
                }
            }
            "traits" => {
                let mut items: Vec<(String, String)> = if let Value::Object(map) = &snapshot.traits
                {
                    map.iter()
                        .map(|(k, v)| (k.clone(), value_to_string(v)))
                        .collect()
                } else {
                    Vec::new()
                };
                items.sort_by(|a, b| a.0.cmp(&b.0));
                if raw {
                    for (j, (k, v)) in items.into_iter().enumerate() {
                        if j > 0 {
                            out.push('\n');
                        }
                        out.push_str(&format!("{} = {}", k, v));
                    }
                } else {
                    let heading = if color {
                        "Traits:".bold().cyan().to_string()
                    } else {
                        "Traits:".to_string()
                    };
                    out.push_str(&heading);
                    for (k, v) in items {
                        out.push('\n');
                        out.push_str("  ");
                        out.push_str(&k);
                        out.push_str(" = ");
                        out.push_str(&colorize_value(&v, color));
                    }
                }
            }
            "facets" => {
                let mut ci: Option<CiFacet> = None;
                let mut items: Vec<(String, String)> = if let Value::Object(map) = &snapshot.facets
                {
                    ci = map
                        .get("ci")
                        .and_then(|v| serde_json::from_value::<CiFacet>(v.clone()).ok());
                    map.iter()
                        .filter(|(k, _)| k.as_str() != "ci")
                        .map(|(k, v)| (k.clone(), value_to_string(v)))
                        .collect()
                } else {
                    Vec::new()
                };
                items.sort_by(|a, b| a.0.cmp(&b.0));
                if !raw {
                    if let Some(ci) = ci {
                        let heading = if color {
                            "CI:".bold().cyan().to_string()
                        } else {
                            "CI:".to_string()
                        };
                        out.push_str(&heading);
                        out.push_str("\n  CI: ");
                        let yes = if color {
                            "Yes".green().to_string()
                        } else {
                            "Yes".to_string()
                        };
                        let no = if color {
                            "No".red().to_string()
                        } else {
                            "No".to_string()
                        };
                        out.push_str(if ci.is_ci { &yes } else { &no });
                        if ci.is_ci {
                            if let (Some(name), Some(vendor)) =
                                (ci.name.as_ref(), ci.vendor.as_ref())
                            {
                                out.push_str("\n  Vendor: ");
                                out.push_str(name);
                                out.push_str(" (");
                                out.push_str(vendor);
                                out.push(')');
                            }
                            if let Some(pr) = ci.pr {
                                out.push_str("\n  Pull Request: ");
                                out.push_str(if pr { &yes } else { &no });
                            }
                            if let Some(branch) = ci.branch {
                                out.push_str("\n  Branch: ");
                                out.push_str(&branch);
                            }
                        }
                        if !items.is_empty() {
                            out.push('\n');
                        }
                    }
                }
                if raw {
                    for (j, (k, v)) in items.into_iter().enumerate() {
                        if j > 0 {
                            out.push('\n');
                        }
                        out.push_str(&format!("{} = {}", k, v));
                    }
                } else if !items.is_empty() {
                    let heading = if color {
                        "Facets:".bold().cyan().to_string()
                    } else {
                        "Facets:".to_string()
                    };
                    out.push_str(&heading);
                    for (k, v) in items {
                        out.push('\n');
                        out.push_str("  ");
                        out.push_str(&k);
                        out.push_str(" = ");
                        out.push_str(&colorize_value(&v, color));
                    }
                }
            }
            "meta" => {
                let mut items: Vec<(String, String)> = if let Value::Object(map) = &snapshot.meta {
                    map.iter()
                        .map(|(k, v)| (k.clone(), value_to_string(v)))
                        .collect()
                } else {
                    Vec::new()
                };
                items.sort_by(|a, b| a.0.cmp(&b.0));
                if raw {
                    for (j, (k, v)) in items.into_iter().enumerate() {
                        if j > 0 {
                            out.push('\n');
                        }
                        out.push_str(&format!("{} = {}", k, v));
                    }
                } else {
                    let heading = if color {
                        "Meta:".bold().cyan().to_string()
                    } else {
                        "Meta:".to_string()
                    };
                    out.push_str(&heading);
                    for (k, v) in items {
                        out.push('\n');
                        out.push_str("  ");
                        out.push_str(&k);
                        out.push_str(" = ");
                        out.push_str(&colorize_value(&v, color));
                    }
                }
            }
            _ => {}
        }
        if i + 1 < selected.len() {
            out.push('\n');
        }
    }
    Ok(out)
}

fn evaluate(
    env: &EnvSense,
    parsed: ParsedCheck,
) -> (bool, Option<String>, Option<BTreeMap<String, String>>) {
    let (mut result, reason, signals) = match parsed.check {
        check::Check::Context(ctx) => {
            let value = match ctx.as_str() {
                "agent" => env.contexts.agent,
                "ide" => env.contexts.ide,
                "ci" => env.contexts.ci,
                "container" => env.contexts.container,
                "remote" => env.contexts.remote,
                _ => false,
            };
            let evidence = find_evidence(env, &ctx);
            (
                value,
                evidence_to_reason(&evidence),
                evidence_to_signals(evidence),
            )
        }
        check::Check::Facet { key, value } => {
            let ok = match key.as_str() {
                "agent_id" => env.facets.agent_id.as_deref() == Some(value.as_str()),
                "ide_id" => env.facets.ide_id.as_deref() == Some(value.as_str()),
                "ci_id" => env.facets.ci_id.as_deref() == Some(value.as_str()),
                "container_id" => env.facets.container_id.as_deref() == Some(value.as_str()),
                _ => false,
            };
            let evidence = find_evidence(env, &key);
            (
                ok,
                evidence_to_reason(&evidence),
                evidence_to_signals(evidence),
            )
        }
        check::Check::Trait { key } => {
            let ok = match key.as_str() {
                "is_interactive" => env.traits.is_interactive,
                "is_tty_stdin" => env.traits.is_tty_stdin,
                "is_tty_stdout" => env.traits.is_tty_stdout,
                "is_tty_stderr" => env.traits.is_tty_stderr,
                "is_piped_stdin" => env.traits.is_piped_stdin,
                "is_piped_stdout" => env.traits.is_piped_stdout,
                "supports_hyperlinks" => env.traits.supports_hyperlinks,
                "is_ci" => env.facets.ci.is_ci,
                "ci_pr" => env.facets.ci.pr.unwrap_or(false),
                _ => false,
            };
            let evidence = find_evidence(env, &key);
            (
                ok,
                evidence_to_reason(&evidence),
                evidence_to_signals(evidence),
            )
        }
    };
    if parsed.negated {
        result = !result;
    }
    (result, reason, signals)
}

fn find_evidence<'a>(env: &'a EnvSense, key: &str) -> Option<&'a Evidence> {
    env.evidence
        .iter()
        .find(|e| e.supports.iter().any(|s| s == key))
}

fn evidence_to_reason(e: &Option<&Evidence>) -> Option<String> {
    e.map(|ev| {
        if let Some(val) = &ev.value {
            format!("{}={}", ev.key, val)
        } else {
            ev.key.clone()
        }
    })
}

fn evidence_to_signals(e: Option<&Evidence>) -> Option<BTreeMap<String, String>> {
    e.map(|ev| {
        let mut map = BTreeMap::new();
        map.insert(ev.key.clone(), ev.value.clone().unwrap_or_default());
        map
    })
}

fn run_check(args: &CheckCmd) -> i32 {
    if args.list_checks {
        list_checks();
        return 0;
    }
    let env = EnvSense::default();
    if args.predicates.len() == 1 && args.predicates[0] == "ci" && !args.any && !args.all {
        let ci = env.facets.ci.clone();
        if ci.is_ci {
            if !args.quiet {
                let name = ci.name.unwrap_or_else(|| "Generic CI".into());
                let vendor = ci.vendor.unwrap_or_else(|| "generic".into());
                println!("CI detected: {} ({})", name, vendor);
            }
            return 0;
        } else {
            if !args.quiet {
                println!("No CI detected");
            }
            return 1;
        }
    }
    let mode_any = args.any;
    let mut results = Vec::new();
    for pred in &args.predicates {
        let parsed = match check::parse_predicate(pred) {
            Ok(p) => p,
            Err(_) => {
                eprintln!("invalid check expression");
                return 2;
            }
        };
        let (res, reason, signals) = evaluate(&env, parsed);
        if args.quiet {
            if mode_any && res {
                return 0;
            }
            if !mode_any && !res {
                return 1;
            }
        } else {
            results.push(JsonCheck {
                predicate: pred.clone(),
                result: res,
                reason,
                signals,
            });
        }
    }

    let overall = if mode_any {
        results.iter().any(|r| r.result)
    } else {
        results.iter().all(|r| r.result)
    };

    if !args.quiet {
        output_results(&results, overall, mode_any, args.format, args.explain);
    }

    if overall { 0 } else { 1 }
}

fn output_results(
    results: &[JsonCheck],
    overall: bool,
    mode_any: bool,
    format: Format,
    explain: bool,
) {
    match format {
        Format::Plain => {
            if results.len() == 1 {
                let r = &results[0];
                if let Some(reason) = r.reason.as_ref().filter(|_| explain) {
                    println!("{}  # reason: {}", r.result, reason);
                } else {
                    println!("{}", r.result);
                }
            } else {
                println!("overall={}", overall);
                for r in results {
                    if let Some(reason) = r.reason.as_ref().filter(|_| explain) {
                        println!("{}={}  # reason: {}", r.predicate, r.result, reason);
                    } else {
                        println!("{}={}", r.predicate, r.result);
                    }
                }
            }
        }
        Format::Pretty => {
            if results.len() == 1 {
                let r = &results[0];
                let mark = if r.result { '✓' } else { '✗' };
                if let Some(reason) = r.reason.as_ref().filter(|_| explain) {
                    println!("{} {} ({})", mark, r.predicate, reason);
                } else {
                    println!("{} {}", mark, r.predicate);
                }
            } else {
                for r in results {
                    let mark = if r.result { '✓' } else { '✗' };
                    if let Some(reason) = r.reason.as_ref().filter(|_| explain) {
                        println!("{} {} ({})", mark, r.predicate, reason);
                    } else {
                        println!("{} {}", mark, r.predicate);
                    }
                }
                let mark = if overall { '✓' } else { '✗' };
                let mode = if mode_any { "any" } else { "all" };
                println!("overall: {} ({})", mark, mode);
            }
        }
        Format::Json => {
            #[derive(serde::Serialize)]
            struct JsonOutput<'a> {
                overall: bool,
                mode: &'a str,
                checks: &'a [JsonCheck],
            }
            let out = JsonOutput {
                overall,
                mode: if mode_any { "any" } else { "all" },
                checks: results,
            };
            if explain {
                println!("{}", serde_json::to_string_pretty(&out).unwrap());
            } else {
                println!("{}", serde_json::to_string(&out).unwrap());
            }
        }
    }
}

fn list_checks() {
    println!("contexts:");
    for c in CONTEXTS {
        println!("  {}", c);
    }
    println!("facets:");
    for f in FACETS {
        println!("  {}", f);
    }
    println!("traits:");
    for t in TRAITS {
        println!("  {}", t);
    }
}

fn detect_color_choice() -> ColorChoice {
    // Scan args before clap so help/errors honor `--no-color`.
    // Mirror clap's parsing by stopping at `--` which terminates flags.
    let mut args = std::env::args_os();
    // Skip binary name
    args.next();
    let mut flag = false;
    for arg in args {
        if arg == "--" {
            break;
        }
        if arg == "--no-color" {
            flag = true;
            break;
        }
    }
    if flag || std::env::var_os("NO_COLOR").is_some_and(|v| !v.is_empty()) {
        ColorChoice::Never
    } else {
        ColorChoice::Auto
    }
}

fn run_info(args: InfoArgs, color: ColorChoice) -> Result<(), i32> {
    let snapshot = collect_snapshot();
    if args.json {
        let mut v = json!({
            "contexts": snapshot.contexts,
            "traits": snapshot.traits,
            "facets": snapshot.facets,
            "meta": snapshot.meta,
        });
        if let Some(f) = args.fields.as_deref() {
            v = match filter_json_fields(v, f) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("{}", e);
                    return Err(2);
                }
            };
        }
        match serde_json::to_string_pretty(&v) {
            Ok(s) => println!("{}", s),
            Err(_) => return Err(3),
        }
    } else {
        let want_color = stdout().is_terminal() && !matches!(color, ColorChoice::Never);
        let rendered = match render_human(&snapshot, args.fields.as_deref(), want_color, args.raw) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("{}", e);
                return Err(2);
            }
        };
        println!("{}", rendered);
    }
    Ok(())
}

fn main() {
    let color = detect_color_choice();
    let matches = Cli::command().color(color).get_matches();
    let cli = Cli::from_arg_matches(&matches).unwrap_or_else(|e| e.exit());
    match cli.command {
        Some(Commands::Info(args)) => {
            if let Err(code) = run_info(args, color) {
                std::process::exit(code);
            }
        }
        Some(Commands::Check(args)) => {
            let code = run_check(&args);
            std::process::exit(code);
        }
        None => {}
    }
}
