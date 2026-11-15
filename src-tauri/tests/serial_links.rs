#[path = "support/serial.rs"]
mod serial;

use flomotion_desktop_lib::app::serial::links::SerialLinks;
use serial::{RecordingEvents, ScriptedOpener, SwitchablePort};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

fn links(fail: bool, error_after_data: bool, plugged: Arc<AtomicBool>) -> (SerialLinks, Arc<RecordingEvents>) {
    let events = Arc::new(RecordingEvents::default());
    let links = SerialLinks::new(Arc::new(SwitchablePort(plugged)), events.clone(), Box::new(ScriptedOpener { fail, error_after_data }));
    (links, events)
}

#[test]
fn open_reads_data_and_close_reports_closed() {
    let (links, events) = links(false, false, Arc::new(AtomicBool::new(true)));
    links.open("COM5", 115_200).unwrap();
    links.open("COM5", 115_200).unwrap();
    std::thread::sleep(Duration::from_millis(50));
    assert!(links.close("COM5", "closed"));
    assert!(!links.close("COM5", "closed"));
    assert_eq!(events.data.lock().unwrap().as_slice(), &[("COM5".to_string(), b"roll=1.5\n".to_vec())]);
    assert_eq!(events.closed.lock().unwrap().as_slice(), &[("COM5".to_string(), "closed".to_string())]);
}

#[test]
fn close_for_flashing_reports_flashing() {
    let (links, events) = links(false, false, Arc::new(AtomicBool::new(true)));
    links.open("COM5", 115_200).unwrap();
    links.close("COM5", "flashing");
    assert_eq!(events.closed.lock().unwrap()[0].1, "flashing");
}

#[test]
fn failed_open_is_busy_when_listed_and_missing_when_not() {
    let plugged = Arc::new(AtomicBool::new(true));
    let (links, _) = links(true, false, plugged.clone());
    assert_eq!(links.open("COM5", 115_200), Err("busy".into()));
    plugged.store(false, Ordering::SeqCst);
    assert_eq!(links.open("COM5", 115_200), Err("missing".into()));
}

#[test]
fn read_error_after_unplug_reports_unplugged() {
    let (links, events) = links(false, true, Arc::new(AtomicBool::new(false)));
    links.open("COM5", 115_200).unwrap();
    events.wait_for_close();
    assert_eq!(events.closed.lock().unwrap()[0].1, "unplugged");
}

#[test]
fn read_error_while_plugged_reports_error() {
    let (links, events) = links(false, true, Arc::new(AtomicBool::new(true)));
    links.open("COM5", 115_200).unwrap();
    events.wait_for_close();
    assert_eq!(events.closed.lock().unwrap()[0].1, "error");
}

#[test]
fn a_port_reopens_after_it_was_lost() {
    let (links, events) = links(false, true, Arc::new(AtomicBool::new(true)));
    links.open("COM5", 115_200).unwrap();
    events.wait_for_close();
    links.open("COM5", 115_200).unwrap();
    std::thread::sleep(Duration::from_millis(50));
    assert_eq!(events.data.lock().unwrap().len(), 2);
}
