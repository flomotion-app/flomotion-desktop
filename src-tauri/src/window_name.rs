use crate::error::{DesktopError, Result};

pub const DEFAULT_WINDOW: &str = "flomotion";
pub const WINDOW_ENV: &str = "FLOMOTION_WINDOW";
const MAX_LENGTH: usize = 32;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowName(String);

impl WindowName {
    pub fn parse(value: &str) -> Result<Self> {
        if value.is_empty() || value.len() > MAX_LENGTH {
            return Err(DesktopError::Remote(format!(
                "window name must be 1 to {MAX_LENGTH} characters, got {}",
                value.len()
            )));
        }
        if !value.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
            return Err(DesktopError::Remote(format!(
                "window name '{value}' may only contain letters, digits, underscore and dash"
            )));
        }
        Ok(Self(value.to_string()))
    }

    // Flag wins over the environment, which wins over the default.
    pub fn resolve(flag: Option<&str>, env: Option<String>) -> Result<Self> {
        match flag.or(env.as_deref()).filter(|v| !v.is_empty()) {
            Some(value) => Self::parse(value),
            None => Ok(Self::default()),
        }
    }

    pub fn from_env(flag: Option<&str>) -> Result<Self> {
        Self::resolve(flag, std::env::var(WINDOW_ENV).ok())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn is_default(&self) -> bool {
        self.0 == DEFAULT_WINDOW
    }

    pub fn socket_file(&self) -> String {
        if self.is_default() {
            return format!("{DEFAULT_WINDOW}.sock");
        }
        format!("{DEFAULT_WINDOW}-{}.sock", self.0)
    }
}

impl Default for WindowName {
    fn default() -> Self {
        Self(DEFAULT_WINDOW.to_string())
    }
}
