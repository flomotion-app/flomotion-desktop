use crate::config::AppConfig;
use std::error::Error;
use tauri::{AppHandle, Url, WebviewUrl, WebviewWindowBuilder};

const WINDOW_LABEL: &str = "main";
#[cfg(windows)]
const WEBVIEW_ARGS_ENV: &str = "FLOMOTION_WEBVIEW_ARGS";
#[cfg(windows)]
const DEFAULT_BROWSER_ARGS: &str = "--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection";

pub struct WindowFactory {
    config: AppConfig,
}

impl WindowFactory {
    pub fn new(config: AppConfig) -> Self {
        Self { config }
    }

    pub fn open(&self, app: &AppHandle) -> Result<(), Box<dyn Error>> {
        let builder = WebviewWindowBuilder::new(app, WINDOW_LABEL, self.url()?)
            .title("FloMotion")
            .inner_size(1400.0, 900.0)
            .initialization_script(&self.marker_script());
        Self::with_browser_args(builder).build()?;
        Ok(())
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
