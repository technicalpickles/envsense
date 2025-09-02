use clap::{Args, ColorChoice, CommandFactory, FromArgMatches, Parser, Subcommand};
use colored::Colorize;
use envsense::check::{self, CONTEXTS, FACETS, FieldRegistry, TRAITS};
// Legacy CI detection removed - using declarative system
use envsense::schema::EnvSense;
use serde_json::{Map, Value, json};
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

    /// Output JSON (stable schema)
    #[arg(long)]
    json: bool,

    /// Explain reasoning
    #[arg(long)]
    explain: bool,

    /// List available predicates
    #[arg(long = "list", action = clap::ArgAction::SetTrue)]
    list_checks: bool,
}

// JsonCheck struct removed - using new EvaluationResult system

#[derive(Debug)]
struct Snapshot {
    contexts: Vec<String>,
    traits: Value,
    facets: Value,
    meta: Value,
    evidence: Value,
}

fn collect_snapshot() -> Snapshot {
    let env = EnvSense::detect();

    // For backward compatibility, convert to legacy format for CLI output
    let legacy_env = env.to_legacy();

    let contexts = env.contexts.clone(); // New schema already has Vec<String>
    let traits_val = serde_json::to_value(legacy_env.traits).unwrap();

    Snapshot {
        contexts,
        traits: traits_val,
        facets: serde_json::to_value(legacy_env.facets).unwrap(),
        meta: json!({
            "schema_version": env.version,
        }),
        evidence: serde_json::to_value(env.evidence).unwrap(),
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
                let mut items: Vec<(String, String)> = if let Value::Object(map) = &snapshot.facets
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

// Legacy evaluate function replaced by new evaluation system in check.rs
// This function is kept for backward compatibility but will be removed in future versions

// Legacy evidence helper functions removed - using new evaluation system

fn run_check(args: &CheckCmd) -> i32 {
    if args.list_checks {
        list_checks();
        return 0;
    }
    let env = EnvSense::detect();
    let registry = FieldRegistry::new();

    // Special case for single "ci" predicate for backward compatibility
    if args.predicates.len() == 1 && args.predicates[0] == "ci" && !args.any && !args.all {
        if env.contexts.contains(&"ci".to_string()) {
            if !args.quiet {
                let name = env.traits.ci.name.as_deref().unwrap_or("Generic CI");
                let vendor = env.traits.ci.vendor.as_deref().unwrap_or("generic");
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

        let eval_result = check::evaluate(&env, parsed, &registry);

        if args.quiet {
            let success = eval_result.result.as_bool();
            if mode_any && success {
                return 0;
            }
            if !mode_any && !success {
                return 1;
            }
        } else {
            results.push(eval_result);
        }
    }

    let overall = if mode_any {
        results.iter().any(|r| r.result.as_bool())
    } else {
        results.iter().all(|r| r.result.as_bool())
    };

    if !args.quiet {
        check::output_check_results(
            &results,
            &args.predicates,
            overall,
            mode_any,
            args.json,
            args.explain,
        );
    }

    if overall { 0 } else { 1 }
}

// Legacy output_results function removed - using new output system in check.rs

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
            "evidence": snapshot.evidence,
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
