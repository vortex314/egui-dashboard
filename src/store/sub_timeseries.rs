use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};


pub struct TimeserieEntry {
    timestamp: SystemTime,
    value: f64,
}

pub struct SubTimeseries {
    name: String,
    time_span: u64,
    entries: Vec<TimeserieEntry>,
}

impl SubTimeseries {
    pub fn new(name: String, time_span: u64) -> Self {
        Self {
            name,
            time_span,
            entries: Vec::new(),
        }
    }

    pub fn min_max_x_y(&self) -> (SystemTime, SystemTime, f64, f64) {
        let mut min_x = SystemTime::now()+Duration::from_secs(10000);
        let mut max_x = SystemTime::now()-Duration::from_secs(10000);
        let mut min_y = f64::MAX;
        let mut max_y = f64::MIN;
        for entry in self.entries.iter() {
            if entry.value < min_y {
                min_y = entry.value;
            }
            if entry.value > max_y {
                max_y = entry.value;
            }
            if entry.timestamp < min_x {
                min_x = entry.timestamp;
            }
            if entry.timestamp > max_x {
                max_x = entry.timestamp;
            }
        }
        (min_x, max_x, min_y, max_y)
    }


    pub fn min_max_x(&self) -> (f64, f64) {
        let mut min = 0.0;
        let mut max = 0.0;
        for entry in self.entries.iter() {
            if entry.value < min {
                min = entry.value;
            }
            if entry.value > max {
                max = entry.value;
            }
        }
        (min, max)
    }

    fn min_max_y(&self) -> (f64, f64) {
        let mut min = 0.0;
        let mut max = 0.0;
        for entry in self.entries.iter() {
            if entry.value < min {
                min = entry.value;
            }
            if entry.value > max {
                max = entry.value;
            }
        }
        (min, max)
    }

    pub fn add(&mut self, value: f64) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        self.entries.push(TimeserieEntry {
            timestamp: SystemTime::now(),
            value,
        });
        let oldest = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
            - self.time_span as u64;
        while SystemTime::now()
            .duration_since(self.entries[0].timestamp)
            .unwrap()
            > Duration::from_millis(self.time_span)
        {
            self.entries.remove(0);
        }
    }

    /*pub fn get_plot_points(&self) -> egui::plot::PlotPoints {
        let mut points = Vec::new();
        for entry in self.entries.iter() {
            points.push(egui::plot::PlotPoint::new(
                entry
                    .timestamp
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as f64,
                entry.value,
            ));
        }
        PlotPoints::Owned(points)
    }*/
}
