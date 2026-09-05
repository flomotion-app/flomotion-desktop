pub mod app;
pub mod cli;
pub mod config;
pub mod error;
pub mod ipc;

use cli::files::{FileMaterializer, TempFileStore};
use cli::launcher::ProcessLauncher;
use cli::runner::CliRunner;
use cli::Command;
use config::AppConfig;
use ipc::client::SocketTransport;

pub fn run_app() {
    app::run(AppConfig::from_env());
}

pub fn run_cli(command: Command) -> i32 {
    let config = AppConfig::from_env();
    let runner = CliRunner::new(
        Box::new(SocketTransport),
        Box::new(ProcessLauncher),
        FileMaterializer::new(Box::new(TempFileStore::new())),
        config.app_start_timeout,
    );
    let mut stdout = std::io::stdout().lock();
    match runner.run(command, &mut stdout) {
        Ok(()) => 0,
        Err(error) => {
            eprintln!("error: {error}");
            2
        }
    }
}
