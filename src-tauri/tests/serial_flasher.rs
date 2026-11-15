#[path = "support/serial.rs"]
mod serial;

use flomotion_desktop_lib::app::serial::flasher::{BoardFlasher, EspFlasher};
use serial::RecordingEvents;
use std::sync::Arc;

#[test]
fn unsupported_chip_fails_before_touching_the_port() {
    let events = Arc::new(RecordingEvents::default());
    let flasher = EspFlasher::new(events.clone());
    let result = flasher.flash("COM99", "esp8266", &[0u8; 16], 0);
    assert_eq!(result, Err("unsupported chip esp8266".into()));
    assert!(events.progress.lock().unwrap().is_empty());
}

#[test]
fn a_missing_port_fails_after_connecting_stage() {
    let events = Arc::new(RecordingEvents::default());
    let flasher = EspFlasher::new(events.clone());
    let result = flasher.flash("COM_FLOMOTION_NONE", "esp32", &[0u8; 16], 0);
    assert!(result.unwrap_err().starts_with("cannot open COM_FLOMOTION_NONE"));
    assert_eq!(events.progress.lock().unwrap().as_slice(), &["connecting".to_string()]);
}
