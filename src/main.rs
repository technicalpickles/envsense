use clap::{Args, ColorChoice, CommandFactory, FromArgMatches, Parser, Subcommand};
use colored::Colorize;
use envsense::check::{self, FieldRegistry};
use envsense::config::CliConfig;
// Legacy CI detection removed - using declarative system
use envsense::schema::EnvSense;
use serde_json::{Map, Value, json};
use std::io::{IsTerminal, stdout};

fn check_predicate_long_help() -> &'static str {
    check::check_predicate_long_help()
}

#[derive(Parser)]
#[command(
    name = "envsense",
    about = "Environment awareness utilities",
    version = env!("CARGO_PKG_VERSION"),
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

    /// Use tree structure for nested display (hierarchical is default)
    #[arg(long)]
    tree: bool,

    /// Compact output without extra formatting
    #[arg(long)]
    compact: bool,
}

#[derive(Args, Clone)]
pub struct CheckCmd {
    /// Predicates to evaluate
    #[arg(
        value_name = "PREDICATE",
        help = "Predicates to evaluate",
        long_help = check_predicate_long_help()
    )]
    pub predicates: Vec<String>,

    /// Show explanations for results
    #[arg(short, long)]
    pub explain: bool,

    /// Output results as JSON
    #[arg(long)]
    pub json: bool,

    /// Suppress output (useful in scripts)
    #[arg(short, long)]
    pub quiet: bool,

    /// Use ANY mode (default is ALL)
    #[arg(long)]
    pub any: bool,

    /// Require all predicates to match (default behavior)
    #[arg(long)]
    pub all: bool,

    /// List available predicates
    #[arg(long)]
    pub list: bool,

    /// Use lenient mode (don't error on invalid fields)
    #[arg(long)]
    pub lenient: bool,

    /// Show context descriptions in list mode
    #[arg(long, requires = "list")]
    pub descriptions: bool,
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

    Snapshot {
        contexts: env.contexts, // Now Vec<String> instead of Contexts struct
        traits: serde_json::to_value(env.traits).unwrap(), // Nested structure
        facets: json!({}),      // Empty for new schema (backward compatibility)
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

fn colorize_value_with_rainbow(v: &str, color: bool) -> String {
    if !color {
        return v.to_string();
    }

    // Apply rainbow effect to "truecolor" when colors are enabled
    if v == "truecolor" {
        return format_color_level_with_rainbow(v, true);
    }

    match v {
        "true" => v.green().to_string(),
        "false" | "none" => v.red().to_string(),
        _ => v.to_string(),
    }
}

fn format_color_level_with_rainbow(value: &str, enable_rainbow: bool) -> String {
    if !enable_rainbow || value != "truecolor" {
        return value.to_string();
    }

    // Create rainbow effect for "truecolor" using colored crate
    value
        .chars()
        .enumerate()
        .map(|(i, c)| {
            let char_str = c.to_string();
            match i % 7 {
                0 => char_str.red().to_string(),
                1 => char_str.bright_red().to_string(), // Orange approximation
                2 => char_str.yellow().to_string(),
                3 => char_str.green().to_string(),
                4 => char_str.blue().to_string(),
                5 => char_str.magenta().to_string(),
                6 => char_str.cyan().to_string(),
                _ => char_str,
            }
        })
        .collect()
}

fn render_nested_value_with_rainbow(
    value: &serde_json::Value,
    indent: usize,
    color: bool,
) -> String {
    let indent_str = "  ".repeat(indent);

    match value {
        serde_json::Value::Object(map) => {
            let mut result = String::new();
            for (key, val) in map {
                match val {
                    serde_json::Value::Object(obj_map) => {
                        if obj_map.is_empty() {
                            // Empty object shows as "= none" with red color
                            let none_value = if color {
                                "none".red().to_string()
                            } else {
                                "none".to_string()
                            };
                            result.push_str(&format!("{}{}: {}\n", indent_str, key, none_value));
                        } else {
                            // For nested objects, show the key with colon and expand recursively
                            result.push_str(&format!("{}{}:\n", indent_str, key));
                            result.push_str(&render_nested_value_with_rainbow(
                                val,
                                indent + 1,
                                color,
                            ));
                        }
                    }
                    _ => {
                        // For simple values, show key = value
                        let formatted_value = format_simple_value(val);
                        let colored_value = colorize_value_with_rainbow(&formatted_value, color);
                        result.push_str(&format!("{}{}: {}\n", indent_str, key, colored_value));
                    }
                }
            }
            result
        }
        _ => {
            let formatted_value = format_simple_value(value);
            let colored_value = colorize_value_with_rainbow(&formatted_value, color);
            format!("{}{}\n", indent_str, colored_value)
        }
    }
}

fn format_simple_value(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Null => "null".to_string(),
        serde_json::Value::Object(_) => {
            // This should be handled by the caller, but fallback to JSON if needed
            value.to_string()
        }
        serde_json::Value::Array(arr) => {
            // Simple array formatting
            format!(
                "[{}]",
                arr.iter()
                    .map(format_simple_value)
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
    }
}

fn render_nested_traits(traits: &Value, color: bool, raw: bool, out: &mut String) {
    if let Value::Object(map) = traits {
        if raw {
            // For raw output, flatten the nested structure
            let mut all_items: Vec<(String, String)> = Vec::new();
            for (context, context_traits) in map {
                if let Value::Object(fields) = context_traits {
                    for (field, value) in fields {
                        all_items.push((format!("{}.{}", context, field), value_to_string(value)));
                    }
                }
            }
            all_items.sort_by(|a, b| a.0.cmp(&b.0));
            for (j, (k, v)) in all_items.into_iter().enumerate() {
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

            // Sort contexts for consistent output
            let mut contexts: Vec<_> = map.keys().collect();
            contexts.sort();

            for context in contexts {
                if let Some(Value::Object(fields)) = map.get(context) {
                    // Only show contexts that have at least one non-null field
                    let has_values = fields.iter().any(|(_, value)| {
                        !(value.is_null()
                            || (value.is_string() && value.as_str() == Some(""))
                            || (value.is_object()
                                && value.as_object().is_some_and(|obj| obj.is_empty())))
                    });

                    if has_values {
                        out.push('\n');
                        out.push_str("  ");
                        out.push_str(context);
                        out.push(':');

                        // Sort fields within each context
                        let mut field_items: Vec<_> = fields.iter().collect();
                        field_items.sort_by(|a, b| a.0.cmp(b.0));

                        for (field, value) in field_items {
                            // Skip null/empty values
                            if value.is_null()
                                || (value.is_string() && value.as_str() == Some(""))
                                || (value.is_object()
                                    && value.as_object().is_some_and(|obj| obj.is_empty()))
                            {
                                continue;
                            }

                            out.push('\n');
                            out.push_str("    ");
                            out.push_str(field);
                            out.push_str(" = ");
                            out.push_str(&colorize_value_with_rainbow(
                                &value_to_string(value),
                                color,
                            ));
                        }
                    }
                }
            }
        }
    }
}

fn render_human(
    snapshot: &Snapshot,
    fields: Option<&str>,
    color: bool,
    raw: bool,
) -> Result<String, String> {
    let default_fields = ["contexts", "traits"];
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
                    out.push('\n');
                    for context in &ctx {
                        out.push_str(&format!("  - {}\n", context));
                    }
                }
            }
            "traits" => {
                if raw {
                    render_nested_traits(&snapshot.traits, color, raw, &mut out);
                } else {
                    let heading = if color {
                        "Traits:".bold().cyan().to_string()
                    } else {
                        "Traits:".to_string()
                    };
                    out.push_str(&heading);
                    out.push('\n');
                    out.push_str(&render_nested_value_with_rainbow(
                        &snapshot.traits,
                        1, // Start with 1 level of indentation for traits
                        color,
                    ));
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
                        out.push_str(&colorize_value_with_rainbow(&v, color));
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
                        out.push_str(&colorize_value_with_rainbow(&v, color));
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

fn run_check(args: CheckCmd, _config: &CliConfig) -> Result<(), i32> {
    // Validate flag combinations first
    if let Err(validation_error) = validate_check_flags(&args) {
        eprintln!("{}", validation_error);
        return Err(1);
    }

    if args.list {
        list_checks();
        return Ok(());
    }

    if args.predicates.is_empty() {
        display_check_usage_error();
        return Err(1);
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
            return Ok(());
        } else {
            if !args.quiet {
                println!("No CI detected");
            }
            return Err(1);
        }
    }

    let mut results = Vec::new();

    for predicate in &args.predicates {
        let parsed = match check::parse_predicate(predicate) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Error parsing '{}': {}", predicate, e);
                return Err(2);
            }
        };

        // Perform strict field validation for nested fields
        if let check::Check::NestedField { ref path, .. } = parsed.check
            && let Err(validation_error) = check::validate_field_path(path, &registry)
        {
            eprintln!("Error: {}", validation_error);
            return Err(2);
        }

        let eval_result = check::evaluate(&env, parsed, &registry);
        results.push(eval_result);
    }

    let overall = if args.any {
        results.iter().any(|r| r.result.as_bool())
    } else {
        // Default is ALL mode, --all flag is explicit but same behavior
        results.iter().all(|r| r.result.as_bool())
    };

    if !args.quiet {
        check::output_check_results(
            &results,
            &args.predicates,
            overall,
            args.any,
            args.json,
            args.explain,
        );
    }

    if overall { Ok(()) } else { Err(1) }
}

// Legacy output_results function removed - using new output system in check.rs

#[derive(Debug)]
enum FlagValidationError {
    ListWithEvaluationFlags,
    ListWithPredicates,
    ListWithQuiet,
    AnyWithAll,
}

impl std::fmt::Display for FlagValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FlagValidationError::ListWithEvaluationFlags => {
                writeln!(
                    f,
                    "Error: invalid flag combination: --list cannot be used with --any or --all"
                )?;
                writeln!(f)?;
                writeln!(
                    f,
                    "The --list flag shows available predicates, while --any/--all control evaluation logic."
                )?;
                writeln!(
                    f,
                    "These flags serve different purposes and cannot be combined."
                )?;
                writeln!(f)?;
                writeln!(f, "Usage examples:")?;
                writeln!(
                    f,
                    "  envsense check --list                    # List available predicates"
                )?;
                writeln!(
                    f,
                    "  envsense check --any agent ide          # Check if ANY predicate is true"
                )?;
                write!(
                    f,
                    "  envsense check --all agent ide          # Check if ALL predicates are true"
                )
            }
            FlagValidationError::ListWithPredicates => {
                writeln!(
                    f,
                    "Error: invalid flag combination: --list cannot be used with predicates"
                )?;
                writeln!(f)?;
                writeln!(
                    f,
                    "The --list flag shows all available predicates, so providing specific predicates is redundant."
                )?;
                writeln!(f)?;
                writeln!(f, "Usage examples:")?;
                writeln!(
                    f,
                    "  envsense check --list                    # List all available predicates"
                )?;
                writeln!(
                    f,
                    "  envsense check agent                    # Check specific predicate"
                )?;
                write!(
                    f,
                    "  envsense check agent ide                # Check multiple predicates"
                )
            }
            FlagValidationError::ListWithQuiet => {
                writeln!(
                    f,
                    "Error: invalid flag combination: --list cannot be used with --quiet"
                )?;
                writeln!(f)?;
                writeln!(
                    f,
                    "The --list flag is designed to show information, while --quiet suppresses output."
                )?;
                writeln!(
                    f,
                    "These flags have contradictory purposes and cannot be combined."
                )?;
                writeln!(f)?;
                writeln!(f, "Usage examples:")?;
                writeln!(
                    f,
                    "  envsense check --list                    # Show available predicates"
                )?;
                write!(
                    f,
                    "  envsense check agent --quiet            # Check predicate quietly"
                )
            }
            FlagValidationError::AnyWithAll => {
                writeln!(
                    f,
                    "Error: invalid flag combination: --any and --all cannot be used together"
                )?;
                writeln!(f)?;
                writeln!(
                    f,
                    "These flags control different evaluation modes and are mutually exclusive."
                )?;
                writeln!(f, "--any: succeeds if ANY predicate matches")?;
                writeln!(
                    f,
                    "--all: succeeds if ALL predicates match (default behavior)"
                )?;
                writeln!(f)?;
                writeln!(f, "Usage examples:")?;
                writeln!(
                    f,
                    "  envsense check agent ide                # Default: ALL predicates must match"
                )?;
                writeln!(
                    f,
                    "  envsense check --any agent ide         # ANY predicate can match"
                )?;
                write!(
                    f,
                    "  envsense check --all agent ide         # Explicit: ALL predicates must match"
                )
            }
        }
    }
}

fn validate_check_flags(args: &CheckCmd) -> Result<(), FlagValidationError> {
    // Check for --any and --all conflict first
    if args.any && args.all {
        return Err(FlagValidationError::AnyWithAll);
    }

    if args.list {
        if args.any || args.all {
            return Err(FlagValidationError::ListWithEvaluationFlags);
        }
        if !args.predicates.is_empty() {
            return Err(FlagValidationError::ListWithPredicates);
        }
        if args.quiet {
            return Err(FlagValidationError::ListWithQuiet);
        }
    }
    Ok(())
}

fn display_check_usage_error() {
    eprintln!("Error: no predicates specified");
    eprintln!();
    eprintln!("Usage: envsense check <predicate> [<predicate>...]");
    eprintln!();
    eprintln!("Examples:");
    eprintln!("  envsense check agent                    # Check if running in an agent");
    eprintln!("  envsense check ide.cursor              # Check if Cursor IDE is active");
    eprintln!("  envsense check ci.github               # Check if in GitHub CI");
    eprintln!("  envsense check agent.id=cursor         # Check specific agent ID");
    eprintln!("  envsense check --list                  # List all available predicates");
    eprintln!();
    eprintln!("For more information, see: envsense check --help");
}

fn list_checks() {
    let registry = FieldRegistry::new();

    println!("Available contexts:");
    for context in registry.get_contexts() {
        println!(
            "- {}: {}",
            context,
            registry.get_context_description(context)
        );
    }

    println!("\nAvailable fields:");
    for context in registry.get_contexts() {
        let context_fields = registry.get_context_fields(context);
        if !context_fields.is_empty() {
            println!("\n  {} fields:", context);
            let mut sorted_fields = context_fields;
            sorted_fields.sort_by(|a, b| a.0.cmp(b.0));

            for (field_path, field_info) in sorted_fields {
                println!("    {:<25} # {}", field_path, field_info.description);
            }
        }
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

fn run_info(args: InfoArgs, color: ColorChoice, _config: &CliConfig) -> Result<(), i32> {
    let snapshot = collect_snapshot();
    if args.json {
        let mut v = json!({
            "version": snapshot.meta["schema_version"],
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
    let config = CliConfig::load();
    let color = detect_color_choice();
    let matches = Cli::command().color(color).get_matches();
    let cli = Cli::from_arg_matches(&matches).unwrap_or_else(|e| e.exit());
    match cli.command {
        Some(Commands::Info(args)) => {
            if let Err(code) = run_info(args, color, &config) {
                std::process::exit(code);
            }
        }
        Some(Commands::Check(args)) => {
            if let Err(code) = run_check(args, &config) {
                std::process::exit(code);
            }
        }
        None => {}
    }
}
