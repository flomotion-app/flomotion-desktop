pub mod commands;
pub mod events;
pub mod flasher;
pub mod links;
pub mod ports;
pub mod watcher;

use base64::Engine;
use flasher::BoardFlasher;
use links::SerialLinks;
use ports::{BoardPort, PortScanner};
use watcher::PortWatcher;
use std::sync::{Arc, Mutex};

pub struct SerialHub {
    scanner: Arc<dyn PortScanner>,
    watcher: PortWatcher,
    links: SerialLinks,
    flasher: Box<dyn BoardFlasher>,
    flashing: Mutex<()>,
}

impl SerialHub {
    pub fn new(scanner: Arc<dyn PortScanner>, watcher: PortWatcher, links: SerialLinks, flasher: Box<dyn BoardFlasher>) -> Self {
        Self { scanner, watcher, links, flasher, flashing: Mutex::new(()) }
    }

    pub fn watch(&self) {
        self.watcher.start();
    }

    pub fn ports(&self) -> Vec<BoardPort> {
        self.scanner.scan()
    }

    pub fn open(&self, path: &str, baud: u32) -> Result<(), String> {
        self.links.open(path, baud)
    }

    pub fn close(&self, path: &str) {
        self.links.close(path, "closed");
    }

    pub fn flash(&self, path: &str, chip: &str, image: &str, offset: u32) -> Result<(), String> {
        let bytes = base64::engine::general_purpose::STANDARD.decode(image).map_err(|error| format!("image is not base64: {error}"))?;
        let _guard = self.flashing.try_lock().map_err(|_| "another flash is running".to_string())?;
        self.links.close(path, "flashing");
        self.flasher.flash(path, chip, &bytes, offset)
    }
}
