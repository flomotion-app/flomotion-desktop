use crate::error::{DesktopError, Result};
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const FILES_KEY: &str = "files";

pub trait FileStore: Send + Sync {
    fn store(&self, name: &str, bytes: &[u8], dir: Option<&Path>) -> Result<PathBuf>;
}

pub struct TempFileStore {
    dir: PathBuf,
    counter: AtomicU64,
}

impl TempFileStore {
    pub fn new() -> Self {
        Self { dir: std::env::temp_dir().join("flomotion"), counter: AtomicU64::new(0) }
    }

    fn unique_name(&self, name: &str) -> String {
        let millis = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis()).unwrap_or(0);
        let index = self.counter.fetch_add(1, Ordering::SeqCst);
        format!("{millis}-{index}-{name}")
    }
}

impl FileStore for TempFileStore {
    fn store(&self, name: &str, bytes: &[u8], dir: Option<&Path>) -> Result<PathBuf> {
        let safe_name: String = name.chars().map(|c| if c.is_alphanumeric() || c == '.' || c == '-' || c == '_' { c } else { '_' }).collect();
        let path = match dir {
            Some(dir) => dir.join(safe_name),
            None => self.dir.join(self.unique_name(&safe_name)),
        };
        std::fs::create_dir_all(path.parent().unwrap_or(&self.dir))?;
        std::fs::write(&path, bytes)?;
        Ok(path)
    }
}

pub struct FileMaterializer {
    store: Box<dyn FileStore>,
}

impl FileMaterializer {
    pub fn new(store: Box<dyn FileStore>) -> Self {
        Self { store }
    }

    pub fn materialize(&self, value: Value) -> Result<Value> {
        self.materialize_into(value, None)
    }

    pub fn materialize_into(&self, mut value: Value, dir: Option<&Path>) -> Result<Value> {
        let Some(files) = value.get(FILES_KEY).and_then(Value::as_array).cloned() else {
            return Ok(value);
        };
        let mut paths = Vec::with_capacity(files.len());
        for file in &files {
            paths.push(Value::String(self.write(file, dir)?.to_string_lossy().into_owned()));
        }
        value[FILES_KEY] = Value::Array(paths);
        Ok(value)
    }

    pub fn encode(path: &Path) -> Result<Value> {
        let bytes = std::fs::read(path)?;
        let name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| "file.bin".into());
        Ok(serde_json::json!({ "name": name, "data_base64": STANDARD.encode(bytes) }))
    }

    fn write(&self, file: &Value, dir: Option<&Path>) -> Result<PathBuf> {
        let name = file.get("name").and_then(Value::as_str).unwrap_or("file.bin");
        let encoded = file
            .get("data_base64")
            .and_then(Value::as_str)
            .ok_or_else(|| DesktopError::Remote(format!("file {name} has no data_base64")))?;
        let bytes = STANDARD.decode(encoded).map_err(|e| DesktopError::Remote(format!("file {name}: {e}")))?;
        self.store.store(name, &bytes, dir)
    }
}
