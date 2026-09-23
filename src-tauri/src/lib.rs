pub mod app;
pub mod cli;
pub mod config;
pub mod error;
pub mod ipc;
pub mod window_name;

use cli::files::{FileMaterializer, TempFileStore};
use cli::uploads::HttpUploadTransport;
use cli::launcher::ProcessLauncher;
use cli::runner::CliRunner;
use cli::Command;
use config::AppConfig;
use ipc::client::SocketTransport;
use window_name::WindowName;

pub fn run_app(window: WindowName) {
    app::run(AppConfig::from_env(), window);
}

pub fn run_cli(command: Command, window: WindowName) -> i32 {
    let config = AppConfig::from_env();
    let runner = CliRunner::new(
        Box::new(SocketTransport::new(window.clone())),
        Box::new(ProcessLauncher::new(window)),
        FileMaterializer::new(Box::new(TempFileStore::new())),
        Box::new(HttpUploadTransport),
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
