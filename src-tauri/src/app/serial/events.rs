use super::ports::BoardPort;
use base64::Engine;
use serde::Serialize;
use tauri::{AppHandle, Emitter};

pub trait SerialEvents: Send + Sync {
    fn ports(&self, ports: &[BoardPort]);
    fn data(&self, path: &str, bytes: &[u8]);
    fn closed(&self, path: &str, reason: &str);
    fn flash_progress(&self, path: &str, stage: &str, written: usize, total: usize);
}

#[derive(Serialize, Clone)]
struct DataEvent<'a> {
    path: &'a str,
    data: String,
}

#[derive(Serialize, Clone)]
struct ClosedEvent<'a> {
    path: &'a str,
    reason: &'a str,
}

#[derive(Serialize, Clone)]
struct ProgressEvent<'a> {
    path: &'a str,
    stage: &'a str,
    written: usize,
    total: usize,
}

pub struct TauriSerialEvents {
    app: AppHandle,
}

impl TauriSerialEvents {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }
}

impl SerialEvents for TauriSerialEvents {
    fn ports(&self, ports: &[BoardPort]) {
        let _ = self.app.emit("serial-ports", ports);
    }

    fn data(&self, path: &str, bytes: &[u8]) {
        let data = base64::engine::general_purpose::STANDARD.encode(bytes);
        let _ = self.app.emit("serial-data", DataEvent { path, data });
    }

    fn closed(&self, path: &str, reason: &str) {
        let _ = self.app.emit("serial-closed", ClosedEvent { path, reason });
    }

    fn flash_progress(&self, path: &str, stage: &str, written: usize, total: usize) {
        let _ = self.app.emit("serial-flash-progress", ProgressEvent { path, stage, written, total });
    }
}
