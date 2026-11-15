use super::events::SerialEvents;
use espflash::connection::{Connection, ResetAfterOperation, ResetBeforeOperation};
use espflash::flasher::Flasher;
use espflash::target::{Chip, ProgressCallbacks};
use serialport::{FlowControl, SerialPortType, UsbPortInfo};
use std::sync::Arc;

const BAUD: u32 = 115_200;
const FAST_BAUD: u32 = 460_800;

pub trait BoardFlasher: Send + Sync {
    fn flash(&self, path: &str, chip: &str, image: &[u8], offset: u32) -> Result<(), String>;
}

pub struct EspFlasher {
    events: Arc<dyn SerialEvents>,
}

impl EspFlasher {
    pub fn new(events: Arc<dyn SerialEvents>) -> Self {
        Self { events }
    }

    fn connect(&self, path: &str, chip: Chip) -> Result<Flasher, String> {
        let serial = serialport::new(path, BAUD).flow_control(FlowControl::None).open_native().map_err(|e| format!("cannot open {path}: {e}"))?;
        let connection = Connection::new(serial, usb_info(path), ResetAfterOperation::HardReset, ResetBeforeOperation::DefaultReset, BAUD);
        Flasher::connect(connection, true, true, false, Some(chip), Some(FAST_BAUD)).map_err(|e| format!("cannot connect to the board: {e}"))
    }
}

impl BoardFlasher for EspFlasher {
    fn flash(&self, path: &str, chip: &str, image: &[u8], offset: u32) -> Result<(), String> {
        let chip = chip_named(chip)?;
        self.events.flash_progress(path, "connecting", 0, image.len());
        let mut flasher = self.connect(path, chip)?;
        self.events.flash_progress(path, "erasing", 0, image.len());
        let mut progress = Progress { events: Arc::clone(&self.events), path: path.to_string(), total: image.len(), chunks: 1 };
        flasher.write_bin_to_flash(offset, image, &mut progress).map_err(|e| format!("flashing failed: {e}"))?;
        flasher.connection().reset_after(true, chip).map_err(|e| format!("reset failed: {e}"))?;
        self.events.flash_progress(path, "done", image.len(), image.len());
        Ok(())
    }
}

fn chip_named(name: &str) -> Result<Chip, String> {
    match name {
        "esp32" => Ok(Chip::Esp32),
        "esp32s3" => Ok(Chip::Esp32s3),
        "esp32c3" => Ok(Chip::Esp32c3),
        other => Err(format!("unsupported chip {other}")),
    }
}

fn usb_info(path: &str) -> UsbPortInfo {
    serialport::available_ports()
        .unwrap_or_default()
        .into_iter()
        .find(|port| port.port_name == path || port.port_name.replacen("/dev/tty.", "/dev/cu.", 1) == path)
        .and_then(|port| match port.port_type {
            SerialPortType::UsbPort(usb) => Some(usb),
            _ => None,
        })
        .unwrap_or(UsbPortInfo { vid: 0, pid: 0, serial_number: None, manufacturer: None, product: None })
}

struct Progress {
    events: Arc<dyn SerialEvents>,
    path: String,
    total: usize,
    chunks: usize,
}

impl ProgressCallbacks for Progress {
    fn init(&mut self, _addr: u32, total: usize) {
        self.chunks = total.max(1);
        self.events.flash_progress(&self.path, "writing", 0, self.total);
    }

    fn update(&mut self, current: usize) {
        let written = self.total * current.min(self.chunks) / self.chunks;
        self.events.flash_progress(&self.path, "writing", written, self.total);
    }

    fn verifying(&mut self) {
        self.events.flash_progress(&self.path, "verifying", self.total, self.total);
    }

    fn finish(&mut self, _skipped: bool) {}
}

