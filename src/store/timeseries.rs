use std::time::{Duration, Instant};
#[derive(Debug, Clone)]
pub struct TimeEntry {
    pub time: std::time::Instant,
    pub value: f64,
}

#[derive(Debug)]
pub struct TimeSeries {
    name: String,
    timespan: Duration,
    max_entries: usize,
    entries: Vec<TimeEntry>,
}

impl TimeSeries {
    pub fn new(name: String, timespan: Duration,max_entries:usize) -> TimeSeries {
        TimeSeries {
            name,
            timespan,
            max_entries,
            entries: Vec::new(),
        }
    }
    pub fn add(&mut self, time: std::time::Instant, value: f64) {
        self.entries.push(TimeEntry { time, value });
        let now = Instant::now();
        self.entries.retain(|entry| now - entry.time < self.timespan );
        loop {
            if self.entries.len() < self.max_entries { break ;}
            self.entries.pop();
        }
    }
    fn get(&self, time: Instant) -> f64 {
        let mut result = 0.0;
        for entry in self.entries.iter() {
            if time - entry.time < self.timespan {
                result = entry.value;
            }
        }
        result
    }
    pub fn get_series(&self) -> Vec<TimeEntry> {
        let mut result = Vec::new();
        for entry in self.entries.iter() {
                result.push(entry.clone());
        }
        result
    }
}