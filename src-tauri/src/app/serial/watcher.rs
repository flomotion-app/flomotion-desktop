use super::events::SerialEvents;
use super::ports::{BoardPort, PortScanner};
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

    pub fn start(self) {
        std::thread::spawn(move || {
            let mut last = self.scanner.scan();
            loop {
                std::thread::sleep(POLL);
                last = self.tick(last);
            }
        });
    }

    fn tick(&self, last: Vec<BoardPort>) -> Vec<BoardPort> {
        let now = self.scanner.scan();
        if now != last {
            self.events.ports(&now);
        }
        now
    }
}
