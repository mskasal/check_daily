use chrono::{DateTime, Duration, Utc};
use std::fmt;

pub enum TimestampCategory {
    Today,
    Yesterday,
    Other,
}

pub struct TimestampAnalyzer {
    pub timestamp: i64,
}

pub trait Analyzer {
    fn new(timestamp: i64) -> Self;

    fn categorize_timestamp(&self) -> TimestampCategory;
}

impl Analyzer for TimestampAnalyzer {
    fn new(timestamp: i64) -> Self {
        TimestampAnalyzer { timestamp }
    }

    fn categorize_timestamp(&self) -> TimestampCategory {
        let timestamp_time = DateTime::from_timestamp(self.timestamp, 0).unwrap();
        let current_time = Utc::now();

        if timestamp_time.date_naive() == current_time.date_naive() {
            TimestampCategory::Today
        } else if timestamp_time.date_naive() == current_time.date_naive() - Duration::days(1) {
            TimestampCategory::Yesterday
        } else {
            TimestampCategory::Other
        }
    }
}

impl fmt::Display for TimestampAnalyzer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let category = self.categorize_timestamp();

        match category {
            TimestampCategory::Today => write!(f, "Today"),
            TimestampCategory::Yesterday => write!(f, "Yesterday"),
            TimestampCategory::Other => {
                let datetime = DateTime::from_timestamp(self.timestamp, 0).unwrap();
                let formatted_time = datetime.format("%d/%m/%Y").to_string();

                write!(f, "{}", formatted_time)
            }
        }
    }
}
