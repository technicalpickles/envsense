use clap::{Args, Parser, Subcommand};
use envsense::engine;

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
        println!("{:?}", report);
    }
}
