pub mod bridge;
pub mod commands;
pub mod dispatcher;
pub mod window;

use crate::config::AppConfig;
use crate::ipc::server::IpcServer;
use bridge::WebviewBridge;
use dispatcher::Dispatcher;
use std::sync::Arc;
use tauri::Manager;
use window::WindowFactory;

pub fn run(config: AppConfig) {
    tauri::Builder::default()
        .setup(move |app| {
            let bridge = Arc::new(WebviewBridge::new(app.handle().clone(), config.page_timeout));
            app.manage(Arc::clone(&bridge));
            let dispatcher = Arc::new(Dispatcher::new(config.clone(), bridge));
            IpcServer::new(dispatcher).start()?;
            WindowFactory::new(config.clone()).open(app.handle())?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![commands::cli_ready, commands::cli_respond])
        .run(tauri::generate_context!())
        .expect("failed to run FloMotion");
}
