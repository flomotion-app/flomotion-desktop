use crate::config::AppConfig;
use crate::window_name::WindowName;
use std::error::Error;
use tauri::utils::config::BackgroundThrottlingPolicy;
use tauri::{AppHandle, Manager, Url, WebviewUrl, WebviewWindowBuilder};

const WINDOW_LABEL: &str = "main";
#[cfg(windows)]
const WEBVIEW_ARGS_ENV: &str = "FLOMOTION_WEBVIEW_ARGS";
#[cfg(windows)]
const DEFAULT_BROWSER_ARGS: &str = "--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection";

pub trait WindowControl: Send + Sync {
    fn focus(&self);
}

pub struct TauriWindowControl {
    app: AppHandle,
}

impl TauriWindowControl {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }
}

impl WindowControl for TauriWindowControl {
    fn focus(&self) {
        if let Some(window) = self.app.get_webview_window(WINDOW_LABEL) {
            let _ = window.unminimize();
            let _ = window.show();
            let _ = window.set_focus();
        }
    }
}

pub struct WindowFactory {
    config: AppConfig,
    window_name: WindowName,
}

impl WindowFactory {
    pub fn new(config: AppConfig, window_name: WindowName) -> Self {
        Self { config, window_name }
    }

    pub fn open(&self, app: &AppHandle) -> Result<(), Box<dyn Error>> {
        let mut builder = WebviewWindowBuilder::new(app, WINDOW_LABEL, self.url()?)
            .title(&self.title())
            .inner_size(1400.0, 900.0)
            .background_throttling(BackgroundThrottlingPolicy::Disabled)
            .initialization_script(&self.marker_script());
        // Two windows cannot share one webview profile, so a named window gets its own.
        if let Some(directory) = self.data_directory(app) {
            builder = builder.data_directory(directory);
        }
        Self::with_browser_args(builder).build()?;
        Ok(())
    }

    fn title(&self) -> String {
        if self.window_name.is_default() {
            return "FloMotion".to_string();
        }
        format!("FloMotion ({})", self.window_name.as_str())
    }

    // The default window keeps the original profile so existing sessions survive.
    fn data_directory(&self, app: &AppHandle) -> Option<std::path::PathBuf> {
        if self.window_name.is_default() {
            return None;
        }
        let base = app.path().app_local_data_dir().ok()?;
        Some(base.join("windows").join(self.window_name.as_str()))
    }

    #[cfg(windows)]
    fn with_browser_args<'a, R: tauri::Runtime, M: tauri::Manager<R>>(builder: WebviewWindowBuilder<'a, R, M>) -> WebviewWindowBuilder<'a, R, M> {
        let extra = std::env::var(WEBVIEW_ARGS_ENV).unwrap_or_default();
        builder.additional_browser_args(&format!("{DEFAULT_BROWSER_ARGS} {extra}"))
    }

    #[cfg(not(windows))]
    fn with_browser_args<'a, R: tauri::Runtime, M: tauri::Manager<R>>(builder: WebviewWindowBuilder<'a, R, M>) -> WebviewWindowBuilder<'a, R, M> {
        builder
    }

    fn url(&self) -> Result<WebviewUrl, Box<dyn Error>> {
        if self.config.uses_local_page() {
            return Ok(WebviewUrl::App("index.html".into()));
        }
        let parsed: Url = self.config.start_url().parse()?;
        Ok(WebviewUrl::External(parsed))
    }

    fn marker_script(&self) -> String {
        format!("window.__FLOMOTION_DESKTOP__ = {{ version: \"{}\" }};", self.config.version)
    }
}
