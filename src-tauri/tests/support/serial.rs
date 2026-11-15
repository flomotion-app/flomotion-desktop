#![allow(dead_code)]

use flomotion_desktop_lib::app::serial::events::SerialEvents;
use flomotion_desktop_lib::app::serial::links::PortOpener;
use flomotion_desktop_lib::app::serial::ports::{BoardPort, PortScanner};
use std::io::{ErrorKind, Read};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Default)]
pub struct RecordingEvents {
    pub data: Mutex<Vec<(String, Vec<u8>)>>,
    pub closed: Mutex<Vec<(String, String)>>,
    pub progress: Mutex<Vec<String>>,
}

impl RecordingEvents {
    pub fn wait_for_close(&self) {
        let start = Instant::now();
        while self.closed.lock().unwrap().is_empty() && start.elapsed() < Duration::from_secs(3) {
            std::thread::sleep(Duration::from_millis(10));
        }
    }
}

impl SerialEvents for RecordingEvents {
    fn ports(&self, _ports: &[BoardPort]) {}

    fn data(&self, path: &str, bytes: &[u8]) {
        self.data.lock().unwrap().push((path.into(), bytes.to_vec()));
    }

    fn closed(&self, path: &str, reason: &str) {
        self.closed.lock().unwrap().push((path.into(), reason.into()));
    }

    fn flash_progress(&self, _path: &str, stage: &str, _written: usize, _total: usize) {
        self.progress.lock().unwrap().push(stage.into());
    }
}

pub struct SwitchablePort(pub Arc<AtomicBool>);

impl PortScanner for SwitchablePort {
    fn scan(&self) -> Vec<BoardPort> {
        if !self.0.load(Ordering::SeqCst) {
            return vec![];
        }
        vec![BoardPort { path: "COM5".into(), vid: Some(0x10C4), pid: Some(0xEA60), product: None, serial_number: None, chip: "cp210x" }]
    }
}

struct ScriptedStream {
    chunks: Vec<std::io::Result<Vec<u8>>>,
}

impl Read for ScriptedStream {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        if self.chunks.is_empty() {
            std::thread::sleep(Duration::from_millis(5));
            return Err(ErrorKind::TimedOut.into());
        }
        let chunk = self.chunks.remove(0)?;
        buffer[..chunk.len()].copy_from_slice(&chunk);
        Ok(chunk.len())
    }
}

pub struct ScriptedOpener {
    pub fail: bool,
    pub error_after_data: bool,
}

impl PortOpener for ScriptedOpener {
    fn open(&self, _path: &str, _baud: u32) -> Result<Box<dyn Read + Send>, String> {
        if self.fail {
            return Err("denied".into());
        }
        let mut chunks = vec![Ok(b"roll=1.5\n".to_vec())];
        if self.error_after_data {
            chunks.push(Err(ErrorKind::BrokenPipe.into()));
        }
        Ok(Box::new(ScriptedStream { chunks }))
    }
}
