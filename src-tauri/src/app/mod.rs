pub mod bridge;
pub mod commands;
pub mod dispatcher;
pub mod files;
pub mod serial;
pub mod services;
pub mod setup;
pub mod window;

use crate::ipc::protocol::{Request, Transport};
use setup::AppSetup;
use std::sync::Arc;

pub struct DesktopApp {
    setup: Arc<AppSetup>,
    running_instance: Box<dyn Transport>,
}

impl DesktopApp {
    pub fn new(setup: AppSetup, running_instance: Box<dyn Transport>) -> Self {
        Self { setup: Arc::new(setup), running_instance }
    }

    pub fn run(self) {
        if self.focus_running_instance() {
            return;
        }
        let setup = Arc::clone(&self.setup);
        let pages = Arc::clone(&self.setup);
        tauri::Builder::default()
            .setup(move |app| setup.install(app))
            .on_page_load(move |webview, payload| pages.page_loaded(webview, payload))
            .invoke_handler(tauri::generate_handler![
                commands::cli_ready,
                commands::cli_respond,
                commands::cli_file_url,
                serial::commands::serial_ports,
                serial::commands::serial_open,
                serial::commands::serial_close,
                serial::commands::serial_flash
            ])
            .run(tauri::generate_context!())
            .expect("failed to run FloMotion");
    }

    fn focus_running_instance(&self) -> bool {
        if self.running_instance.send(Request::status()).is_err() {
            return false;
        }
        let _ = self.running_instance.send(Request::focus());
        true
    }
}
