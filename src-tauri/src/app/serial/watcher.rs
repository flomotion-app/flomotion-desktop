use super::events::SerialEvents;
use super::ports::PortScanner;
use std::sync::Arc;
use std::time::Duration;

const POLL: Duration = Duration::from_secs(1);

pub struct PortWatcher {
    scanner: Arc<dyn PortScanner>,
    events: Arc<dyn SerialEvents>,
}

impl PortWatcher {
    pub fn new(scanner: Arc<dyn PortScanner>, events: Arc<dyn SerialEvents>) -> Self {
        Self { scanner, events }
    }

    pub fn start(&self) {
        let scanner = Arc::clone(&self.scanner);
        let events = Arc::clone(&self.events);
        std::thread::spawn(move || {
            let mut last = scanner.scan();
            loop {
                std::thread::sleep(POLL);
                let now = scanner.scan();
                if now != last {
                    events.ports(&now);
                }
                last = now;
            }
        });
    }
}
