#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use clap::Parser;
use flomotion_desktop_lib::cli::{Cli, Command};
use flomotion_desktop_lib::config::AppConfig;
use flomotion_desktop_lib::di;
use flomotion_desktop_lib::window_name::WindowName;

fn main() {
    let cli = Cli::parse();
    let window = match WindowName::from_env(cli.window.as_deref()) {
        Ok(window) => window,
        Err(error) => {
            eprintln!("error: {error}");
            std::process::exit(2);
        }
    };
    match cli.command {
        None => di::desktop_app(AppConfig::from_env(), window).run(),
        Some(command) => std::process::exit(run_cli(command, window)),
    }
}

fn run_cli(command: Command, window: WindowName) -> i32 {
    let runner = di::cli_runner(AppConfig::from_env(), window);
    let mut stdout = std::io::stdout().lock();
    match runner.run(command, &mut stdout) {
        Ok(()) => 0,
        Err(error) => {
            eprintln!("error: {error}");
            2
        }
    }
}
