use clap::{Args, Parser, Subcommand, ValueEnum};
use envsense::check::{self, CONTEXTS, FACETS, ParsedCheck, TRAITS};
use envsense::schema::{EnvSense, Evidence};
use std::collections::BTreeMap;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Evaluate check expressions (compatibility flag)
    #[arg(long, value_name = "PREDICATE", num_args = 1.., hide = true)]
    check: Vec<String>,

    /// Output JSON (compact)
    #[arg(long)]
    json: bool,

    /// Pretty-print JSON output
    #[arg(long)]
    pretty: bool,

    /// Output only a specific section: contexts, facets, traits, evidence
    #[arg(long)]
    only: Option<String>,

    /// When used with --check, also explain results
    #[arg(long)]
    explain: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Evaluate predicates against the environment
    Check(CheckCmd),
}

#[derive(Args, Clone)]
struct CheckCmd {
    #[arg(value_name = "PREDICATE", num_args = 1..)]
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

fn output_env(env: &EnvSense, pretty: bool) {
    if pretty {
        println!("{}", serde_json::to_string_pretty(env).unwrap());
    } else {
        println!("{}", serde_json::to_string(env).unwrap());
    }
}

fn output_json<T: serde::Serialize>(value: &T, pretty: bool) {
    if pretty {
        println!("{}", serde_json::to_string_pretty(value).unwrap());
    } else {
        println!("{}", serde_json::to_string(value).unwrap());
    }
}

fn main() {
    let cli = Cli::parse();
    if let Some(cmd) = &cli.command {
        let code = match cmd {
            Commands::Check(args) => run_check(args),
        };
        std::process::exit(code);
    }

    if !cli.check.is_empty() {
        let args = CheckCmd {
            predicates: cli.check.clone(),
            any: false,
            all: false,
            quiet: false,
            format: Format::Plain,
            explain: cli.explain,
            list_checks: false,
        };
        let code = run_check(&args);
        std::process::exit(code);
    }

    let env = EnvSense::default();
    if cli.json || cli.pretty || cli.only.is_some() || cli.explain {
        if let Some(section) = cli.only.as_deref() {
            match section {
                "contexts" => output_json(&env.contexts, cli.pretty || cli.explain),
                "facets" => output_json(&env.facets, cli.pretty || cli.explain),
                "traits" => output_json(&env.traits, cli.pretty || cli.explain),
                "evidence" => output_json(&env.evidence, cli.pretty || cli.explain),
                _ => output_env(&env, cli.pretty || cli.explain),
            }
        } else {
            output_env(&env, cli.pretty || cli.explain);
        }
        return;
    }

    // Default behavior: print compact JSON
    output_env(&env, false);
}
