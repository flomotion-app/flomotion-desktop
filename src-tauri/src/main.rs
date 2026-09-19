// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use clap::Parser;
use flomotion_desktop_lib::cli::Cli;
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
        None => flomotion_desktop_lib::run_app(window),
        Some(command) => std::process::exit(flomotion_desktop_lib::run_cli(command, window)),
    }
}
