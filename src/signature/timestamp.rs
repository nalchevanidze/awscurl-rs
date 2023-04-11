use chrono::{DateTime, Utc};

pub struct Timestamp {
    timestamp: DateTime<Utc>,
}

impl Timestamp {
    pub fn new() -> Timestamp {
        Timestamp {
            timestamp: Utc::now(),
        }
    }
    pub fn date_stamp(&self) -> String {
        self.timestamp.format("%Y%m%d").to_string()
    }

    pub fn x_amz_date(&self) -> String {
        self.timestamp.format("%Y%m%dT%H%M%SZ").to_string()
    }
}