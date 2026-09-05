use crate::error::{DesktopError, Result};
use crate::ipc::protocol::{Request, Response};
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{channel, RecvTimeoutError, Sender};
use std::sync::Mutex;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

pub const REQUEST_EVENT: &str = "cli-request";

pub trait PageChannel: Send + Sync {
    fn is_ready(&self) -> bool;
    fn call(&self, request: Request) -> Result<Response>;
}

#[derive(Serialize, Clone)]
struct RequestEvent {
    id: u64,
    command: String,
    payload: Value,
}

pub struct WebviewBridge {
    app: AppHandle,
    pending: Mutex<HashMap<u64, Sender<Response>>>,
    next_id: AtomicU64,
    ready: AtomicBool,
    timeout: Duration,
}

impl WebviewBridge {
    pub fn new(app: AppHandle, timeout: Duration) -> Self {
        Self {
            app,
            pending: Mutex::new(HashMap::new()),
            next_id: AtomicU64::new(1),
            ready: AtomicBool::new(false),
            timeout,
        }
    }

    pub fn set_ready(&self) {
        self.ready.store(true, Ordering::SeqCst);
    }

    pub fn complete(&self, id: u64, response: Response) {
        let sender = self.pending.lock().expect("pending lock").remove(&id);
        if let Some(sender) = sender {
            let _ = sender.send(response);
        }
    }

    fn register(&self) -> (u64, std::sync::mpsc::Receiver<Response>) {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let (sender, receiver) = channel();
        self.pending.lock().expect("pending lock").insert(id, sender);
        (id, receiver)
    }

    fn forget(&self, id: u64) {
        self.pending.lock().expect("pending lock").remove(&id);
    }
}

impl PageChannel for WebviewBridge {
    fn is_ready(&self) -> bool {
        self.ready.load(Ordering::SeqCst)
    }

    fn call(&self, request: Request) -> Result<Response> {
        if !self.is_ready() {
            return Err(DesktopError::PageNotReady);
        }
        let (id, receiver) = self.register();
        let event = RequestEvent { id, command: request.command, payload: request.payload };
        if let Err(error) = self.app.emit(REQUEST_EVENT, event) {
            self.forget(id);
            return Err(DesktopError::Remote(format!("could not reach page: {error}")));
        }
        match receiver.recv_timeout(self.timeout) {
            Ok(response) => Ok(response),
            Err(RecvTimeoutError::Timeout) => {
                self.forget(id);
                Err(DesktopError::PageTimeout(self.timeout.as_secs()))
            }
            Err(RecvTimeoutError::Disconnected) => Err(DesktopError::Remote("page dropped the request".into())),
        }
    }
}
