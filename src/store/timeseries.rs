use chrono::{DateTime, Duration, Local, Utc};
#[derive(Debug, Clone)]
struct TimeEntry {
    time: DateTime<Utc>,
    value: f64,
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
    pub fn add(&mut self, time: DateTime<Utc>, value: f64) {
        self.entries.push(TimeEntry { time, value });
        self.entries.retain(|entry| time.signed_duration_since(entry.time) < self.timespan );
        loop {
            if self.entries.len() < self.max_entries { break ;}
            self.entries.pop();
        }
    }
    fn get(&self, time: DateTime<Utc>) -> f64 {
        let mut result = 0.0;
        for entry in self.entries.iter() {
            if time - entry.time < self.timespan {
                result = entry.value;
            }
        }
        result
    }
    pub fn get_series(&self, time: DateTime<Utc>) -> Vec<TimeEntry> {
        let mut result = Vec::new();
        for entry in self.entries.iter() {
            if time - entry.time < self.timespan {
                result.push(entry.clone());
            }
        }
        result
    }
}