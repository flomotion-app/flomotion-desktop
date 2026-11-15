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
use std::sync::{Arc, Mutex};

pub struct SerialHub {
    scanner: Arc<dyn PortScanner>,
    links: SerialLinks,
    flasher: Box<dyn BoardFlasher>,
    flashing: Mutex<()>,
}

impl SerialHub {
    pub fn new(scanner: Arc<dyn PortScanner>, links: SerialLinks, flasher: Box<dyn BoardFlasher>) -> Self {
        Self { scanner, links, flasher, flashing: Mutex::new(()) }
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
        let bytes = base64::engine::general_purpose::STANDARD.decode(image).map_err(|e| format!("image is not base64: {e}"))?;
        let _guard = self.flashing.try_lock().map_err(|_| "another flash is running".to_string())?;
        self.links.close(path, "flashing");
        self.flasher.flash(path, chip, &bytes, offset)
    }
}
