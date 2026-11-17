use crate::app::bridge::WebviewBridge;
use crate::app::dispatcher::Dispatcher;
use crate::app::files::{PageFiles, TauriAssetScope};
use crate::app::serial::events::{SerialEvents, TauriSerialEvents};
use crate::app::serial::flasher::EspFlasher;
use crate::app::serial::links::{NativeOpener, SerialLinks};
use crate::app::serial::ports::{PortScanner, SystemPorts};
use crate::app::serial::watcher::PortWatcher;
use crate::app::serial::SerialHub;
use crate::app::services::AppServices;
use crate::app::setup::AppSetup;
use crate::app::window::{TauriWindowControl, WindowFactory};
use crate::app::DesktopApp;
use crate::cli::files::{FileMaterializer, TempFileStore};
use crate::cli::launcher::ProcessLauncher;
use crate::cli::runner::CliRunner;
use crate::cli::uploads::HttpUploadTransport;
use crate::config::AppConfig;
use crate::ipc::client::SocketTransport;
use crate::ipc::server::IpcServer;
use crate::window_name::WindowName;
use std::sync::Arc;
use tauri::AppHandle;

pub fn desktop_app(config: AppConfig, window: WindowName) -> DesktopApp {
    DesktopApp::new(AppSetup::new(config, window.clone()), Box::new(SocketTransport::new(window)))
}

pub fn cli_runner(config: AppConfig, window: WindowName) -> CliRunner {
    CliRunner::new(
        Box::new(SocketTransport::new(window.clone())),
        Box::new(ProcessLauncher::new(window)),
        FileMaterializer::new(Box::new(TempFileStore::new())),
        Box::new(HttpUploadTransport),
        config.app_start_timeout,
    )
}

pub fn app_services(app: AppHandle, config: AppConfig, window: WindowName) -> AppServices {
    let bridge = Arc::new(WebviewBridge::new(app.clone(), config.page_timeout));
    let files = Arc::new(PageFiles::new(Box::new(TauriAssetScope::new(app.clone()))));
    let control = Arc::new(TauriWindowControl::new(app.clone()));
    let dispatcher = Arc::new(Dispatcher::new(config.clone(), bridge.clone(), control, files.clone(), window.clone()));
    AppServices {
        bridge,
        files,
        serial: serial_hub(app),
        ipc: IpcServer::new(dispatcher, window.clone()),
        window: WindowFactory::new(config, window),
    }
}

fn serial_hub(app: AppHandle) -> Arc<SerialHub> {
    let scanner: Arc<dyn PortScanner> = Arc::new(SystemPorts);
    let events: Arc<dyn SerialEvents> = Arc::new(TauriSerialEvents::new(app));
    let watcher = PortWatcher::new(scanner.clone(), events.clone());
    let links = SerialLinks::new(scanner.clone(), events.clone(), Box::new(NativeOpener));
    Arc::new(SerialHub::new(scanner, watcher, links, Box::new(EspFlasher::new(events))))
}
