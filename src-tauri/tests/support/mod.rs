#![allow(dead_code)]

use flomotion_desktop_lib::app::bridge::PageChannel;
use flomotion_desktop_lib::cli::files::FileStore;
use flomotion_desktop_lib::cli::launcher::AppLauncher;
use flomotion_desktop_lib::config::AppConfig;
use flomotion_desktop_lib::error::{DesktopError, Result};
use flomotion_desktop_lib::ipc::protocol::{Request, Response, Transport};
use serde_json::json;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub fn config() -> AppConfig {
    AppConfig {
        web_url: "local".into(),
        version: "0.0.1".into(),
        page_timeout: Duration::from_secs(1),
        app_start_timeout: Duration::from_secs(2),
    }
}

pub fn ready() -> Result<Response> {
    Ok(Response::success(json!({"page_ready": true})))
}

pub struct FakePage {
    pub ready: bool,
    pub calls: Mutex<Vec<Request>>,
}

impl FakePage {
    pub fn new(ready: bool) -> Arc<Self> {
        Arc::new(Self { ready, calls: Mutex::new(vec![]) })
    }
}

impl PageChannel for FakePage {
    fn is_ready(&self) -> bool {
        self.ready
    }

    fn call(&self, request: Request) -> Result<Response> {
        if !self.ready {
            return Err(DesktopError::PageNotReady);
        }
        self.calls.lock().unwrap().push(request);
        Ok(Response::success(json!({"echo": true})))
    }
}

#[derive(Default)]
pub struct TransportLog {
    pub responses: Mutex<Vec<Result<Response>>>,
    pub sent: Mutex<Vec<Request>>,
}

pub struct FakeTransport {
    pub log: Arc<TransportLog>,
}

impl FakeTransport {
    pub fn scripted(responses: Vec<Result<Response>>) -> (Self, Arc<TransportLog>) {
        let log = Arc::new(TransportLog { responses: Mutex::new(responses), sent: Mutex::new(vec![]) });
        (Self { log: Arc::clone(&log) }, log)
    }
}

impl Transport for FakeTransport {
    fn send(&self, request: Request) -> Result<Response> {
        self.log.sent.lock().unwrap().push(request);
        let mut responses = self.log.responses.lock().unwrap();
        if responses.is_empty() {
            return ready();
        }
        responses.remove(0)
    }
}

pub struct FakeLauncher {
    pub launches: Arc<AtomicUsize>,
}

impl FakeLauncher {
    pub fn new() -> (Self, Arc<AtomicUsize>) {
        let launches = Arc::new(AtomicUsize::new(0));
        (Self { launches: Arc::clone(&launches) }, launches)
    }
}

impl AppLauncher for FakeLauncher {
    fn launch(&self) -> Result<()> {
        self.launches.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

pub struct FakeStore {
    pub written: Arc<Mutex<Vec<(String, Vec<u8>)>>>,
}

impl FakeStore {
    pub fn new() -> (Self, Arc<Mutex<Vec<(String, Vec<u8>)>>>) {
        let written = Arc::new(Mutex::new(vec![]));
        (Self { written: Arc::clone(&written) }, written)
    }
}

impl FileStore for FakeStore {
    fn store(&self, name: &str, bytes: &[u8], dir: Option<&std::path::Path>) -> Result<PathBuf> {
        self.written.lock().unwrap().push((name.to_string(), bytes.to_vec()));
        Ok(dir.map(|d| d.join(name)).unwrap_or_else(|| PathBuf::from(format!("/tmp/{name}"))))
    }
}
