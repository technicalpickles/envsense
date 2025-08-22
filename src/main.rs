use clap::Parser;
use envsense::check::{self, Check};
use envsense::schema::EnvSense;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Evaluate a single check expression against the environment
    #[arg(long)]
    check: Option<String>,

    /// Output JSON (compact)
    #[arg(long)]
    json: bool,

    /// Pretty-print JSON output
    #[arg(long)]
    pretty: bool,

    /// Output only a specific section: contexts, facets, traits, evidence
    #[arg(long)]
    only: Option<String>,

    /// When used with --check, also print the environment
    #[arg(long)]
    explain: bool,
}

fn evaluate(env: &EnvSense, check: Check) -> bool {
    match check {
        Check::Context(ctx) => match ctx.as_str() {
            "agent" => env.contexts.agent,
            "ide" => env.contexts.ide,
            "ci" => env.contexts.ci,
            "container" => env.contexts.container,
            "remote" => env.contexts.remote,
            _ => false,
        },
        Check::Facet { key, value } => match key.as_str() {
            "agent_id" => env.facets.agent_id.as_deref() == Some(value.as_str()),
            "ide_id" => env.facets.ide_id.as_deref() == Some(value.as_str()),
            "ci_id" => env.facets.ci_id.as_deref() == Some(value.as_str()),
            "container_id" => env.facets.container_id.as_deref() == Some(value.as_str()),
            _ => false,
        },
        Check::Trait { key } => match key.as_str() {
            "is_interactive" => env.traits.is_interactive,
            "is_tty_stdin" => env.traits.is_tty_stdin,
            "is_tty_stdout" => env.traits.is_tty_stdout,
            "is_tty_stderr" => env.traits.is_tty_stderr,
            "is_piped_stdin" => env.traits.is_piped_stdin,
            "is_piped_stdout" => env.traits.is_piped_stdout,
            "supports_hyperlinks" => env.traits.supports_hyperlinks,
            _ => false,
        },
    }
}

fn main() {
    let cli = Cli::parse();
    let env = EnvSense::default();

    if let Some(expr) = cli.check.as_deref() {
        match check::parse(expr) {
            Ok(parsed) => {
                let ok = evaluate(&env, parsed);
                if cli.explain {
                    output_env(&env, cli.pretty);
                }
                std::process::exit(if ok { 0 } else { 1 });
            }
            Err(_) => {
                eprintln!("invalid check expression");
                std::process::exit(2);
            }
        }
    }

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
