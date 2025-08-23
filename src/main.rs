use clap::{Args, Parser, Subcommand};
use colored::Colorize;
use envsense::engine;
use envsense::schema::Report;

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
}

#[derive(Args, Clone)]
struct InfoArgs {
    #[arg(long)]
    json: bool,
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Info(args) => run_info(args),
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

fn print_report(report: &Report) {
    fn bool_str(v: bool) -> colored::ColoredString {
        if v { "true".green() } else { "false".red() }
    }

    fn enum_to_str<T: serde::Serialize>(e: &T) -> String {
        serde_json::to_string(e)
            .unwrap()
            .trim_matches('"')
            .to_string()
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
