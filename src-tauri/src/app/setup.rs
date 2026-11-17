use crate::config::AppConfig;
use crate::window_name::WindowName;
use std::error::Error;
use std::sync::Arc;
use crate::app::bridge::WebviewBridge;
use tauri::webview::{PageLoadEvent, PageLoadPayload};
use tauri::{App, Manager, Webview, Wry};

pub struct AppSetup {
    config: AppConfig,
    window_name: WindowName,
}

impl AppSetup {
    pub fn new(config: AppConfig, window_name: WindowName) -> Self {
        Self { config, window_name }
    }

    pub fn install(&self, app: &mut App) -> Result<(), Box<dyn Error>> {
        let services = crate::di::app_services(app.handle().clone(), self.config.clone(), self.window_name.clone());
        app.manage(Arc::clone(&services.bridge));
        app.manage(Arc::clone(&services.files));
        app.manage(Arc::clone(&services.serial));
        services.serial.watch();
        services.ipc.start()?;
        services.window.open(app.handle())?;
        Ok(())
    }

    pub fn page_loaded(&self, webview: &Webview<Wry>, payload: &PageLoadPayload<'_>) {
        if !matches!(payload.event(), PageLoadEvent::Started) {
            return;
        }
        if let Some(bridge) = webview.try_state::<Arc<WebviewBridge>>() {
            bridge.page_unloaded();
        }
    }
}
