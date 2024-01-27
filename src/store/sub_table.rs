use chrono::{DateTime, Local};

pub enum OrderSort {
    Topic,
    Value,
    Time,
    Count,
}

pub struct LastValue {
    pub topic: String,
    pub value: String,
    pub time: DateTime<Local>,
    pub count: i32,
}

impl LastValue {
    fn new(topic: String, value: String, time: DateTime<Local>) -> LastValue {
        LastValue {
            topic,
            value,
            time,
            count: 1,
        }
    }
    fn update(&mut self, entry: &LastValue) {
        self.value = entry.value.clone();
        self.time = entry.time;
        self.count += 1;
    }
}

pub struct SubTable {
     pub entries: Vec<LastValue>,
}

impl SubTable {
    pub fn new() -> SubTable {
        SubTable {
            entries: Vec::new(),
        }
    }
    pub fn add(&mut self, topic: String, message: String) {
        let mut found = false;
        for entry in self.entries.iter_mut() {
            if entry.topic == topic {
                entry.update(&LastValue {
                    topic: topic.clone(),
                    value: message.clone(),
                    time: Local::now(),
                    count: 1,
                });
                found = true;
                break;
            }
        }
        if !found {
            self.entries.push(LastValue { 
                topic: topic.clone(),
                value: message.clone(),
                time: Local::now(),
                count: 1,
            });
        }
    }
    pub fn get(&self, topic: &str) -> Option<&LastValue> {
        for entry in self.entries.iter() {
            if entry.topic == topic {
                return Some(entry);
            }
        }
        None
    }
}

fn order_list(entry_list: &mut SubTable, ordering: OrderSort) {
    match ordering {
        OrderSort::Topic => {
            entry_list.entries.sort_by(|a, b| a.topic.cmp(&b.topic));
        }
        OrderSort::Value => {
            entry_list.entries.sort_by(|a, b| a.value.cmp(&b.value));
        }
        OrderSort::Time => {
            entry_list.entries.sort_by(|a, b| a.time.cmp(&b.time));
        }
        OrderSort::Count => {
            entry_list.entries.sort_by(|a, b| a.count.cmp(&b.count));
        }
    }
}
