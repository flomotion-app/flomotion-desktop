pub mod bridge;
pub mod commands;
pub mod files;
pub mod dispatcher;
pub mod window;

use crate::config::AppConfig;
use crate::ipc::client::SocketTransport;
use crate::ipc::protocol::{Request, Transport};
use crate::ipc::server::IpcServer;
use bridge::WebviewBridge;
use dispatcher::Dispatcher;
use std::sync::Arc;
use tauri::Manager;
use files::{PageFiles, TauriAssetScope};
use window::{TauriWindowControl, WindowFactory};
use crate::window_name::WindowName;

pub fn run(config: AppConfig, window_name: WindowName) {
    if focus_running_instance(&window_name) {
        return;
    }
    tauri::Builder::default()
        .setup(move |app| {
            let bridge = Arc::new(WebviewBridge::new(app.handle().clone(), config.page_timeout));
            app.manage(Arc::clone(&bridge));
            let files = Arc::new(PageFiles::new(Box::new(TauriAssetScope::new(app.handle().clone()))));
            app.manage(Arc::clone(&files));
            let window = Arc::new(TauriWindowControl::new(app.handle().clone()));
            let dispatcher = Arc::new(Dispatcher::new(config.clone(), bridge, window, files, window_name.clone()));
            IpcServer::new(dispatcher, window_name.clone()).start()?;
            WindowFactory::new(config.clone(), window_name.clone()).open(app.handle())?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![commands::cli_ready, commands::cli_respond, commands::cli_file_url])
        .run(tauri::generate_context!())
        .expect("failed to run FloMotion");
}

fn focus_running_instance(window: &WindowName) -> bool {
    let transport = SocketTransport::new(window.clone());
    if transport.send(Request::status()).is_err() {
        return false;
    }
    let _ = transport.send(Request::focus());
    true
}
