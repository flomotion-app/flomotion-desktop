use super::events::SerialEvents;
use super::ports::PortScanner;
use std::collections::HashMap;
use std::io::{ErrorKind, Read};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

const READ_TIMEOUT: Duration = Duration::from_millis(50);
const PRESENCE_CHECK: Duration = Duration::from_secs(1);

pub trait PortOpener: Send + Sync {
    fn open(&self, path: &str, baud: u32) -> Result<Box<dyn Read + Send>, String>;
}

pub struct NativeOpener;

impl PortOpener for NativeOpener {
    fn open(&self, path: &str, baud: u32) -> Result<Box<dyn Read + Send>, String> {
        let mut port = serialport::new(path, baud).timeout(READ_TIMEOUT).open().map_err(|e| e.to_string())?;
        let _ = port.write_data_terminal_ready(false);
        let _ = port.write_request_to_send(false);
        Ok(Box::new(port))
    }
}

struct Link {
    stop: Arc<Mutex<Option<&'static str>>>,
    thread: JoinHandle<()>,
}

pub struct SerialLinks {
    scanner: Arc<dyn PortScanner>,
    events: Arc<dyn SerialEvents>,
    opener: Box<dyn PortOpener>,
    links: Mutex<HashMap<String, Link>>,
}

impl SerialLinks {
    pub fn new(scanner: Arc<dyn PortScanner>, events: Arc<dyn SerialEvents>, opener: Box<dyn PortOpener>) -> Self {
        Self { scanner, events, opener, links: Mutex::new(HashMap::new()) }
    }

    pub fn open(&self, path: &str, baud: u32) -> Result<(), String> {
        let mut links = self.links.lock().expect("links lock");
        if links.get(path).is_some_and(|link| !link.thread.is_finished()) {
            return Ok(());
        }
        if let Some(stale) = links.remove(path) {
            let _ = stale.thread.join();
        }
        let stream = self.opener.open(path, baud).map_err(|_| self.open_failure(path))?;
        let stop = Arc::new(Mutex::new(None));
        let reader = Reader { path: path.to_string(), stream, stop: Arc::clone(&stop), scanner: Arc::clone(&self.scanner), events: Arc::clone(&self.events) };
        let thread = std::thread::spawn(move || reader.run());
        links.insert(path.to_string(), Link { stop, thread });
        Ok(())
    }

    pub fn close(&self, path: &str, reason: &'static str) -> bool {
        let link = self.links.lock().expect("links lock").remove(path);
        match link {
            Some(link) => {
                let running = !link.thread.is_finished();
                *link.stop.lock().expect("stop lock") = Some(reason);
                let _ = link.thread.join();
                running
            }
            None => false,
        }
    }

    fn open_failure(&self, path: &str) -> String {
        if self.scanner.scan().iter().any(|port| port.path == path) {
            "busy".into()
        } else {
            "missing".into()
        }
    }
}

struct Reader {
    path: String,
    stream: Box<dyn Read + Send>,
    stop: Arc<Mutex<Option<&'static str>>>,
    scanner: Arc<dyn PortScanner>,
    events: Arc<dyn SerialEvents>,
}

impl Reader {
    fn run(mut self) {
        let reason = self.pump();
        drop(self.stream);
        self.events.closed(&self.path, reason);
    }

    fn pump(&mut self) -> &'static str {
        let mut buffer = [0u8; 4096];
        let mut checked = Instant::now();
        loop {
            if let Some(reason) = *self.stop.lock().expect("stop lock") {
                return reason;
            }
            match self.stream.read(&mut buffer) {
                Ok(0) => {}
                Ok(count) => self.events.data(&self.path, &buffer[..count]),
                Err(error) if matches!(error.kind(), ErrorKind::TimedOut | ErrorKind::Interrupted | ErrorKind::WouldBlock) => {}
                Err(_) => return self.lost_reason(),
            }
            if checked.elapsed() >= PRESENCE_CHECK {
                checked = Instant::now();
                if !self.present() {
                    return "unplugged";
                }
            }
        }
    }

    fn lost_reason(&self) -> &'static str {
        if let Some(reason) = *self.stop.lock().expect("stop lock") {
            return reason;
        }
        if self.present() {
            "error"
        } else {
            "unplugged"
        }
    }

    fn present(&self) -> bool {
        self.scanner.scan().iter().any(|port| port.path == self.path)
    }
}

