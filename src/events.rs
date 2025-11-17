use std::sync::mpsc::Sender;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub enum WorkbenchEvent {
    Started {
        bench_id: String,
        timestamp: Instant,
    },
    Progress {
        bench_id: String,
        addresses_generated: u64,
    },
    PotentialMatch {
        bench_id: String,
        path: [u32; 6],
        prefix_id: u8,
    },
    Stopped {
        bench_id: String,
        total_generated: u64,
        elapsed: Duration,
    },
}

#[derive(Clone)]
pub struct EventSender {
    inner: Sender<WorkbenchEvent>,
    bench_id: String,
}

impl EventSender {
    pub fn new(sender: Sender<WorkbenchEvent>, bench_id: String) -> Self {
        Self {
            inner: sender,
            bench_id,
        }
    }

    pub fn started(&self, timestamp: Instant) {
        self.inner
            .send(WorkbenchEvent::Started {
                bench_id: self.bench_id.clone(),
                timestamp,
            })
            .ok();
    }

    pub fn progress(&self, addresses_generated: u64) {
        self.inner
            .send(WorkbenchEvent::Progress {
                bench_id: self.bench_id.clone(),
                addresses_generated,
            })
            .ok();
    }

    pub fn potential_match(&self, path: [u32; 6], prefix_id: u8) {
        self.inner
            .send(WorkbenchEvent::PotentialMatch {
                bench_id: self.bench_id.clone(),
                path,
                prefix_id,
            })
            .ok();
    }

    pub fn stopped(&self, total_generated: u64, elapsed: Duration) {
        self.inner
            .send(WorkbenchEvent::Stopped {
                bench_id: self.bench_id.clone(),
                total_generated,
                elapsed,
            })
            .ok();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    use std::time::Instant;

    #[test]
    fn test_event_sender_started() {
        let (tx, rx) = mpsc::channel();
        let sender = EventSender::new(tx, "test-bench".to_string());
        let now = Instant::now();

        sender.started(now);

        let event = rx.recv().unwrap();
        match event {
            WorkbenchEvent::Started {
                bench_id,
                timestamp,
            } => {
                assert_eq!(bench_id, "test-bench");
                assert_eq!(timestamp, now);
            }
            _ => panic!("Expected Started event"),
        }
    }

    #[test]
    fn test_event_sender_progress() {
        let (tx, rx) = mpsc::channel();
        let sender = EventSender::new(tx, "test-bench".to_string());

        sender.progress(1000);

        let event = rx.recv().unwrap();
        match event {
            WorkbenchEvent::Progress {
                bench_id,
                addresses_generated,
            } => {
                assert_eq!(bench_id, "test-bench");
                assert_eq!(addresses_generated, 1000);
            }
            _ => panic!("Expected Progress event"),
        }
    }

    #[test]
    fn test_event_sender_potential_match() {
        let (tx, rx) = mpsc::channel();
        let sender = EventSender::new(tx, "test-bench".to_string());
        let path = [1, 2, 3, 4, 5, 6];

        sender.potential_match(path, 0);

        let event = rx.recv().unwrap();
        match event {
            WorkbenchEvent::PotentialMatch {
                bench_id,
                path: received_path,
                prefix_id,
            } => {
                assert_eq!(bench_id, "test-bench");
                assert_eq!(received_path, path);
                assert_eq!(prefix_id, 0);
            }
            _ => panic!("Expected PotentialMatch event"),
        }
    }

    #[test]
    fn test_event_sender_stopped() {
        let (tx, rx) = mpsc::channel();
        let sender = EventSender::new(tx, "test-bench".to_string());
        let duration = Duration::from_secs(10);

        sender.stopped(5000, duration);

        let event = rx.recv().unwrap();
        match event {
            WorkbenchEvent::Stopped {
                bench_id,
                total_generated,
                elapsed,
            } => {
                assert_eq!(bench_id, "test-bench");
                assert_eq!(total_generated, 5000);
                assert_eq!(elapsed, duration);
            }
            _ => panic!("Expected Stopped event"),
        }
    }

    #[test]
    fn test_event_sender_multiple_events_sequence() {
        let (tx, rx) = mpsc::channel();
        let sender = EventSender::new(tx, "bench1".to_string());
        let now = Instant::now();

        sender.started(now);
        sender.progress(100);
        sender.progress(200);
        sender.potential_match([1, 2, 3, 4, 5, 6], 0);
        sender.stopped(300, Duration::from_secs(5));

        let events: Vec<_> = rx.iter().take(5).collect();
        assert_eq!(events.len(), 5);

        match &events[0] {
            WorkbenchEvent::Started { bench_id, .. } => assert_eq!(bench_id, "bench1"),
            _ => panic!("Expected Started"),
        }
        match &events[1] {
            WorkbenchEvent::Progress {
                addresses_generated,
                ..
            } => assert_eq!(*addresses_generated, 100),
            _ => panic!("Expected Progress"),
        }
        match &events[2] {
            WorkbenchEvent::Progress {
                addresses_generated,
                ..
            } => assert_eq!(*addresses_generated, 200),
            _ => panic!("Expected Progress"),
        }
        match &events[3] {
            WorkbenchEvent::PotentialMatch { .. } => {}
            _ => panic!("Expected PotentialMatch"),
        }
        match &events[4] {
            WorkbenchEvent::Stopped {
                total_generated, ..
            } => assert_eq!(*total_generated, 300),
            _ => panic!("Expected Stopped"),
        }
    }

    #[test]
    fn test_event_sender_bench_id_is_included() {
        let (tx, rx) = mpsc::channel();
        let sender = EventSender::new(tx, "test123".to_string());

        sender.started(Instant::now());
        sender.progress(50);
        sender.potential_match([0; 6], 0);
        sender.stopped(100, Duration::from_secs(1));

        let events: Vec<_> = rx.iter().take(4).collect();

        for event in events {
            let bench_id = match event {
                WorkbenchEvent::Started { bench_id, .. } => bench_id,
                WorkbenchEvent::Progress { bench_id, .. } => bench_id,
                WorkbenchEvent::PotentialMatch { bench_id, .. } => bench_id,
                WorkbenchEvent::Stopped { bench_id, .. } => bench_id,
            };
            assert_eq!(bench_id, "test123");
        }
    }

    #[test]
    fn test_event_sender_ignores_disconnected_receiver() {
        let (tx, rx) = mpsc::channel();
        let sender = EventSender::new(tx, "test".to_string());

        drop(rx);

        sender.started(Instant::now());
        sender.progress(100);
        sender.potential_match([0; 6], 0);
        sender.stopped(100, Duration::from_secs(1));
    }
}
