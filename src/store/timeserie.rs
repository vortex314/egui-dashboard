use chrono::{DateTime, Duration, Local, Utc};
#[derive(Debug, Clone)]
struct TimeEntry {
    time: DateTime<Utc>,
    value: f64,
}

#[derive(Debug)]
pub struct TimeSerie {
    name: String,
    timespan: Duration,
    entries: Vec<TimeEntry>,
}

impl TimeSerie {
    fn new(name: String, timespan: Duration) -> TimeSerie {
        TimeSerie {
            name,
            timespan,
            entries: Vec::new(),
        }
    }
    fn add(&mut self, time: DateTime<Utc>, value: f64) {
        self.entries.push(TimeEntry { time, value });
        self.entries.retain(|entry| time.signed_duration_since(entry.time) < self.timespan);
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
    fn get_series(&self, time: DateTime<Utc>) -> Vec<TimeEntry> {
        let mut result = Vec::new();
        for entry in self.entries.iter() {
            if time - entry.time < self.timespan {
                result.push(entry.clone());
            }
        }
        result
    }
}