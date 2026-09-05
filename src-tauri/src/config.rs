use serde::Deserialize;
use std::path::PathBuf;
use std::time::Duration;

pub const LOCAL_PAGE: &str = "local";
pub const URL_ENV: &str = "FLOMOTION_URL";
const DEFAULT_WEB_URL: &str = "https://flomotion.app";
const START_PATH: &str = "/projects";
const CONFIG_DIR: &str = ".flomotion";
const CONFIG_FILE: &str = "config.json";

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub web_url: String,
    pub version: String,
    pub page_timeout: Duration,
    pub app_start_timeout: Duration,
}

#[derive(Debug, Default, Deserialize)]
pub struct ConfigFile {
    pub web_url: Option<String>,
}

impl AppConfig {
    pub fn from_env() -> Self {
        let file = Self::config_path().and_then(|p| std::fs::read_to_string(p).ok());
        Self::resolve(std::env::var(URL_ENV).ok(), file.as_deref())
    }

    pub fn resolve(env_url: Option<String>, file_text: Option<&str>) -> Self {
        let file: ConfigFile = file_text.and_then(|t| serde_json::from_str(t).ok()).unwrap_or_default();
        let web_url = env_url
            .filter(|u| !u.is_empty())
            .or(file.web_url)
            .unwrap_or_else(|| DEFAULT_WEB_URL.to_string());
        Self {
            web_url,
            version: env!("CARGO_PKG_VERSION").to_string(),
            page_timeout: Duration::from_secs(600),
            app_start_timeout: Duration::from_secs(60),
        }
    }

    pub fn config_path() -> Option<PathBuf> {
        directories::BaseDirs::new().map(|d| d.home_dir().join(CONFIG_DIR).join(CONFIG_FILE))
    }

    pub fn uses_local_page(&self) -> bool {
        self.web_url == LOCAL_PAGE
    }

    pub fn start_url(&self) -> String {
        format!("{}{}", self.web_url.trim_end_matches('/'), START_PATH)
    }
}
