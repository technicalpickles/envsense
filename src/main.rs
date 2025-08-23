use clap::{Args, Parser, Subcommand, ValueEnum};
use colored::Colorize;
use envsense::check::{self, CONTEXTS, FACETS, ParsedCheck, TRAITS};
use envsense::engine;
use envsense::schema::{ContextKind, Report};
use std::sync::OnceLock;

#[derive(Parser)]
#[command(
    name = "envsense",
    about = "Environment awareness utilities",
    arg_required_else_help = true
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Info(InfoArgs),
    Check(CheckArgs),
}

#[derive(Args, Clone)]
struct InfoArgs {
    #[arg(long)]
    json: bool,
}

#[derive(Args, Clone)]
struct CheckArgs {
    #[arg(
        value_name = "PREDICATE",
        num_args = 1..,
        help = "Predicates to evaluate",
        long_help = check_predicate_long_help(),
        required_unless_present = "list_checks"
    )]
    predicates: Vec<String>,

    #[arg(long, action = clap::ArgAction::SetTrue, conflicts_with = "all")]
    any: bool,

    #[arg(long, action = clap::ArgAction::SetTrue, conflicts_with = "any")]
    all: bool,

    #[arg(short, long, alias = "silent", action = clap::ArgAction::SetTrue)]
    quiet: bool,

    #[arg(long, value_enum, default_value_t = Format::Plain)]
    format: Format,

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
}

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

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Info(args) => run_info(args),
        Commands::Check(args) => {
            let code = run_check(&args);
            std::process::exit(code);
        }
    }
}

fn run_info(args: InfoArgs) {
    let report = engine::detect();
    if args.json {
        println!("{}", serde_json::to_string_pretty(&report).unwrap());
    } else {
        print_report(&report);
    }
}

fn run_check(args: &CheckArgs) -> i32 {
    if args.list_checks {
        list_checks();
        return 0;
    }
    let report = engine::detect();
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
        let res = evaluate(&report, parsed);
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
            });
        }
    }

    let overall = if mode_any {
        results.iter().any(|r| r.result)
    } else {
        results.iter().all(|r| r.result)
    };

    if !args.quiet {
        output_results(&results, overall, mode_any, args.format);
    }

    if overall { 0 } else { 1 }
}

fn evaluate(report: &Report, parsed: ParsedCheck) -> bool {
    let mut result = match parsed.check {
        check::Check::Context(ctx) => match ctx.as_str() {
            "agent" => report.contexts.contains(&ContextKind::Agent),
            "ide" => report.contexts.contains(&ContextKind::Ide),
            "ci" => report.contexts.contains(&ContextKind::Ci),
            "container" => report.contexts.contains(&ContextKind::Container),
            "remote" => report.contexts.contains(&ContextKind::Remote),
            _ => false,
        },
        check::Check::Facet { key, value } => match key.as_str() {
            "agent_id" => report.facets.agent_id.as_ref().map(enum_to_str) == Some(value),
            "ide_id" => report.facets.ide_id.as_ref().map(enum_to_str) == Some(value),
            "ci_id" => report.facets.ci_id.as_ref().map(enum_to_str) == Some(value),
            _ => false,
        },
        check::Check::Trait { key } => match key.as_str() {
            "is_interactive" => report.traits.is_interactive,
            "supports_hyperlinks" => report.traits.supports_hyperlinks,
            "is_piped_stdin" => report.traits.is_piped_stdin,
            "is_piped_stdout" => report.traits.is_piped_stdout,
            "is_tty_stdin" => report.traits.is_tty_stdin,
            "is_tty_stdout" => report.traits.is_tty_stdout,
            "is_tty_stderr" => report.traits.is_tty_stderr,
            _ => false,
        },
    };
    if parsed.negated {
        result = !result;
    }
    result
}

fn output_results(results: &[JsonCheck], overall: bool, mode_any: bool, format: Format) {
    match format {
        Format::Plain => {
            if results.len() == 1 {
                println!("{}", results[0].result);
            } else {
                println!("overall={}", overall);
                for r in results {
                    println!("{}={}", r.predicate, r.result);
                }
            }
        }
        Format::Pretty => {
            if results.len() == 1 {
                let r = &results[0];
                let mark = if r.result { '✓' } else { '✗' };
                println!("{} {}", mark, r.predicate);
            } else {
                for r in results {
                    let mark = if r.result { '✓' } else { '✗' };
                    println!("{} {}", mark, r.predicate);
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
            println!("{}", serde_json::to_string_pretty(&out).unwrap());
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

fn enum_to_str<T: serde::Serialize>(e: &T) -> String {
    serde_json::to_string(e)
        .unwrap()
        .trim_matches('"')
        .to_string()
}

fn print_report(report: &Report) {
    fn bool_str(v: bool) -> colored::ColoredString {
        if v { "true".green() } else { "false".red() }
    }

    println!("{}", "Report".bold());

    println!("{}", "contexts".bold());
    for c in &report.contexts {
        println!("  - {}", enum_to_str(c).green());
    }

    println!("{}", "traits".bold());
    let t = &report.traits;
    println!(
        "  {:<20} {}",
        "is_interactive".cyan(),
        bool_str(t.is_interactive)
    );
    println!(
        "  {:<20} {}",
        "color_level".cyan(),
        enum_to_str(&t.color_level)
    );
    println!(
        "  {:<20} {}",
        "supports_hyperlinks".cyan(),
        bool_str(t.supports_hyperlinks)
    );
    println!(
        "  {:<20} {}",
        "is_piped_stdin".cyan(),
        bool_str(t.is_piped_stdin)
    );
    println!(
        "  {:<20} {}",
        "is_piped_stdout".cyan(),
        bool_str(t.is_piped_stdout)
    );
    println!(
        "  {:<20} {}",
        "is_tty_stdin".cyan(),
        bool_str(t.is_tty_stdin)
    );
    println!(
        "  {:<20} {}",
        "is_tty_stdout".cyan(),
        bool_str(t.is_tty_stdout)
    );
    println!(
        "  {:<20} {}",
        "is_tty_stderr".cyan(),
        bool_str(t.is_tty_stderr)
    );

    println!("{}", "facets".bold());
    let f = &report.facets;
    println!(
        "  {:<20} {}",
        "agent_id".cyan(),
        f.agent_id
            .as_ref()
            .map(enum_to_str)
            .unwrap_or_else(|| "none".to_string())
    );
    println!(
        "  {:<20} {}",
        "ide_id".cyan(),
        f.ide_id
            .as_ref()
            .map(enum_to_str)
            .unwrap_or_else(|| "none".to_string())
    );
    println!(
        "  {:<20} {}",
        "ci_id".cyan(),
        f.ci_id
            .as_ref()
            .map(enum_to_str)
            .unwrap_or_else(|| "none".to_string())
    );

    println!("{}", "meta".bold());
    println!(
        "  {:<20} {}",
        "schema_version".cyan(),
        report.meta.schema_version.as_str()
    );
    println!(
        "  {:<20} {}",
        "rules_version".cyan(),
        report.meta.rules_version.as_str()
    );

    if !report.evidence.is_empty() {
        println!("{}", "evidence".bold());
        for ev in &report.evidence {
            println!(
                "  - {} {}={} ({})",
                enum_to_str(&ev.source).magenta(),
                ev.key.yellow(),
                ev.value.clone().unwrap_or_else(|| "".to_string()).bold(),
                ev.weight
            );
        }
    }
}
