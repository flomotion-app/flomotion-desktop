use clap::Parser;
use flomotion_desktop_lib::cli::Cli;

fn main() {
    let cli = Cli::parse();
    match cli.command {
        None => flomotion_desktop_lib::run_app(),
        Some(command) => std::process::exit(flomotion_desktop_lib::run_cli(command)),
    }
}
