use clap::{ColorChoice, CommandFactory, Parser};

#[derive(Parser)]
struct Cli {
    /// Disable color
    #[arg(long = "no-color")]
    _no_color: bool,
}

fn main() {
    let flag = std::env::args_os().any(|a| a == "--no-color");
    let env_no_color = std::env::var_os("NO_COLOR").map_or(false, |v| !v.is_empty());
    let choice = if flag {
        ColorChoice::Never
    } else if env_no_color {
        ColorChoice::Never
    } else {
        ColorChoice::Auto
    };
    let _ = Cli::command().color(choice).get_matches();
}
