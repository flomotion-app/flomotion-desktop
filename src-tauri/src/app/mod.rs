pub mod bridge;
pub mod commands;
pub mod files;
pub mod dispatcher;
pub mod serial;
pub mod window;

use crate::config::AppConfig;
use crate::ipc::client::SocketTransport;
use crate::ipc::protocol::{Request, Transport};
use crate::ipc::server::IpcServer;
use bridge::WebviewBridge;
use dispatcher::Dispatcher;
use std::sync::Arc;
use tauri::webview::PageLoadEvent;
use tauri::Manager;
use files::{PageFiles, TauriAssetScope};
use window::{TauriWindowControl, WindowFactory};
use crate::window_name::WindowName;
use serial::events::{SerialEvents, TauriSerialEvents};
use serial::flasher::EspFlasher;
use serial::links::{NativeOpener, SerialLinks};
use serial::ports::{PortScanner, SystemPorts};
use serial::watcher::PortWatcher;
use serial::SerialHub;

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
            app.manage(serial_hub(app.handle()));
            let window = Arc::new(TauriWindowControl::new(app.handle().clone()));
            let dispatcher = Arc::new(Dispatcher::new(config.clone(), bridge, window, files, window_name.clone()));
            IpcServer::new(dispatcher, window_name.clone()).start()?;
            WindowFactory::new(config.clone(), window_name.clone()).open(app.handle())?;
            Ok(())
        })
        .on_page_load(|webview, payload| {
            if matches!(payload.event(), PageLoadEvent::Started) {
                if let Some(bridge) = webview.try_state::<Arc<WebviewBridge>>() {
                    bridge.page_unloaded();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![commands::cli_ready, commands::cli_respond, commands::cli_file_url,
            serial::commands::serial_ports,
            serial::commands::serial_open,
            serial::commands::serial_close,
            serial::commands::serial_flash
        ])
        .run(tauri::generate_context!())
        .expect("failed to run FloMotion");
}

fn serial_hub(app: &tauri::AppHandle) -> Arc<SerialHub> {
    let scanner: Arc<dyn PortScanner> = Arc::new(SystemPorts);
    let events: Arc<dyn SerialEvents> = Arc::new(TauriSerialEvents::new(app.clone()));
    PortWatcher::new(Arc::clone(&scanner), Arc::clone(&events)).start();
    let links = SerialLinks::new(Arc::clone(&scanner), Arc::clone(&events), Box::new(NativeOpener));
    Arc::new(SerialHub::new(scanner, links, Box::new(EspFlasher::new(events))))
}

fn focus_running_instance(window: &WindowName) -> bool {
    let transport = SocketTransport::new(window.clone());
    if transport.send(Request::status()).is_err() {
        return false;
    }
    let _ = transport.send(Request::focus());
    true
}
