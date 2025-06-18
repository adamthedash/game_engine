use std::time::{Duration, Instant};

use rustc_hash::FxHashMap;

pub struct StopWatch {
    pub start: Instant,
    pub trackers: FxHashMap<String, Vec<Duration>>,
}

impl Default for StopWatch {
    fn default() -> Self {
        Self::new()
    }
}

impl StopWatch {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
            trackers: Default::default(),
        }
    }

    pub fn reset(&mut self) {
        self.start = Instant::now();
    }

    pub fn stamp_and_reset(&mut self, name: &str) {
        let duration = Instant::now().duration_since(self.start);
        self.trackers
            .entry(name.to_string())
            .or_default()
            .push(duration);
        self.reset();
    }

    pub fn get_debug_strings(&self) -> Vec<String> {
        self.trackers
            .iter()
            .map(|(name, durations)| {
                let last = durations.last().expect("Should have at least 1 duration");
                let last_5 = durations.iter().rev().take(5).collect::<Vec<_>>();
                let sum = last_5.iter().fold(Duration::ZERO, |acc, d| acc + **d);
                let avg = sum / last_5.len() as u32;

                format!("{name}: {avg:?}")
            })
            .collect()
    }
}
