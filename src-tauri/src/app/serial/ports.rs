use serde::Serialize;
use serialport::{SerialPortType, UsbPortInfo};

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct BoardPort {
    pub path: String,
    pub vid: Option<u16>,
    pub pid: Option<u16>,
    pub product: Option<String>,
    pub serial_number: Option<String>,
    pub chip: &'static str,
}

pub trait PortScanner: Send + Sync {
    fn scan(&self) -> Vec<BoardPort>;
}

pub struct SystemPorts;

impl PortScanner for SystemPorts {
    fn scan(&self) -> Vec<BoardPort> {
        let mut ports: Vec<BoardPort> = serialport::available_ports()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|port| match port.port_type {
                SerialPortType::UsbPort(usb) => Some(BoardPort::from_usb(visible_path(&port.port_name), usb)),
                _ => None,
            })
            .collect();
        ports.sort_by(|a, b| a.path.cmp(&b.path));
        ports.dedup_by(|a, b| a.path == b.path);
        ports
    }
}

impl BoardPort {
    pub fn from_usb(path: String, usb: UsbPortInfo) -> Self {
        Self {
            chip: chip_for(usb.vid, usb.pid),
            path,
            vid: Some(usb.vid),
            pid: Some(usb.pid),
            product: usb.product,
            serial_number: usb.serial_number,
        }
    }
}

pub fn chip_for(vid: u16, pid: u16) -> &'static str {
    match (vid, pid) {
        (0x10C4, 0xEA60) => "cp210x",
        (0x1A86, 0x7523) => "ch340",
        (0x1A86, 0x55D4) => "ch9102",
        (0x0403, 0x6001) | (0x0403, 0x6015) => "ftdi",
        (0x303A, _) => "espressif",
        _ => "other",
    }
}

fn visible_path(name: &str) -> String {
    if cfg!(target_os = "macos") {
        name.replacen("/dev/tty.", "/dev/cu.", 1)
    } else {
        name.to_string()
    }
}

