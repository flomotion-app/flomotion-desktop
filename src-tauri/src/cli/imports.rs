use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use crate::error::{DesktopError, Result};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

pub const INLINE_LIMIT_BYTES: u64 = 8 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ImportTransport {
    Path,
    Inline,
    Auto,
}

impl ImportTransport {
    pub fn parse(value: Option<&str>) -> Result<Self> {
        match value {
            None | Some("auto") => Ok(Self::Auto),
            Some("path") => Ok(Self::Path),
            Some("inline") => Ok(Self::Inline),
            Some(other) => Err(DesktopError::Remote(format!("unknown transport '{other}', expected path, inline or auto"))),
        }
    }
}

pub struct ImportPayload;

impl ImportPayload {
    pub fn build(path: &Path, transport: ImportTransport) -> Result<Value> {
        let absolute = Self::absolute(path)?;
        let size = std::fs::metadata(&absolute)?.len();
        let name = absolute
            .file_name()
            .map(|file_name| file_name.to_string_lossy().into_owned())
            .ok_or_else(|| DesktopError::Remote("import needs a file, not a directory".into()))?;
        if Self::inline(transport, size)? {
            let mut payload = Self::encode(&absolute)?;
            payload["size"] = json!(size);
            return Ok(payload);
        }
        Ok(json!({ "name": name, "path": Self::for_page(&absolute), "size": size }))
    }

    pub fn encode(path: &Path) -> Result<Value> {
        let bytes = std::fs::read(path)?;
        let name = path.file_name().map(|file_name| file_name.to_string_lossy().into_owned()).unwrap_or_else(|| "file.bin".into());
        Ok(json!({ "name": name, "data_base64": STANDARD.encode(bytes) }))
    }

    fn inline(transport: ImportTransport, size: u64) -> Result<bool> {
        match transport {
            ImportTransport::Path => Ok(false),
            ImportTransport::Inline if size > INLINE_LIMIT_BYTES => Err(DesktopError::Remote(format!(
                "file is {} MB, over the {} MB inline limit; use --transport path",
                size / (1024 * 1024),
                INLINE_LIMIT_BYTES / (1024 * 1024)
            ))),
            ImportTransport::Inline => Ok(true),
            ImportTransport::Auto => Ok(size <= INLINE_LIMIT_BYTES),
        }
    }

    fn absolute(path: &Path) -> Result<PathBuf> {
        let resolved = std::fs::canonicalize(path).map_err(|error| DesktopError::Remote(format!("cannot read {}: {error}", path.display())))?;
        if !resolved.is_file() {
            return Err(DesktopError::Remote(format!("{} is not a file", resolved.display())));
        }
        Ok(resolved)
    }

    fn for_page(path: &Path) -> String {
        let text = path.to_string_lossy();
        text.strip_prefix(r"\\?\").unwrap_or(&text).replace('\\', "/")
    }
}
