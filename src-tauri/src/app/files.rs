use crate::error::{DesktopError, Result};
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};

pub trait AssetScope: Send + Sync {
    fn allow(&self, path: &Path) -> Result<()>;
}

pub struct TauriAssetScope {
    app: AppHandle,
}

impl TauriAssetScope {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }
}

impl AssetScope for TauriAssetScope {
    fn allow(&self, path: &Path) -> Result<()> {
        self.app
            .asset_protocol_scope()
            .allow_file(path)
            .map_err(|e| DesktopError::Remote(format!("cannot grant access to {}: {e}", path.display())))
    }
}

pub struct PageFiles {
    scope: Box<dyn AssetScope>,
}

impl PageFiles {
    pub fn new(scope: Box<dyn AssetScope>) -> Self {
        Self { scope }
    }

    pub fn url(&self, path: &str) -> Result<String> {
        let resolved = Self::resolve(path)?;
        self.scope.allow(&resolved)?;
        Ok(Self::asset_url(&resolved))
    }

    fn resolve(path: &str) -> Result<PathBuf> {
        let resolved = std::fs::canonicalize(path).map_err(|e| DesktopError::Remote(format!("cannot read {path}: {e}")))?;
        if !resolved.is_file() {
            return Err(DesktopError::Remote(format!("{path} is not a file")));
        }
        Ok(resolved)
    }

    fn asset_url(path: &Path) -> String {
        let text = path.to_string_lossy();
        let plain = text.strip_prefix(r"\\?\").unwrap_or(&text);
        let encoded = urlencoding(plain);
        if cfg!(windows) {
            format!("http://asset.localhost/{encoded}")
        } else {
            format!("asset://localhost/{encoded}")
        }
    }
}

fn urlencoding(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b'/' | b':' => out.push(byte as char),
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}
