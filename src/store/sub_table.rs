use chrono::{DateTime, Local};

pub enum OrderSort {
    Topic,
    Value,
    Time,
    Count,
}

pub struct Entry {
    pub topic: String,
    pub value: String,
    pub time: DateTime<Local>,
    pub count: i32,
}

impl Entry {
    fn new(topic: String, value: String, time: DateTime<Local>) -> Entry {
        Entry {
            topic,
            value,
            time,
            count: 1,
        }
    }
    fn update(&mut self, entry: &Entry) {
        self.value = entry.value.clone();
        self.time = entry.time;
        self.count += 1;
    }
}

pub struct EntryList {
     pub entries: Vec<Entry>,
}

impl EntryList {
    pub fn new() -> EntryList {
        EntryList {
            entries: Vec::new(),
        }
    }
    pub fn add(&mut self, topic: String, message: String) {
        let mut found = false;
        for entry in self.entries.iter_mut() {
            if entry.topic == topic {
                entry.update(&Entry {
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
            self.entries.push(Entry {
                topic: topic.clone(),
                value: message.clone(),
                time: Local::now(),
                count: 1,
            });
        }
    }
    pub fn get(&self, topic: &str) -> Option<&Entry> {
        for entry in self.entries.iter() {
            if entry.topic == topic {
                return Some(entry);
            }
        }
        None
    }
}

fn order_list(entry_list: &mut EntryList, ordering: OrderSort) {
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
