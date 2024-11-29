use std::collections::HashSet;
use chrono::{DateTime, Utc};

pub enum DateTimeMatch {
    Year,
    Month,
    MonthDay,
    WeekDay,

    Hour,
    Minute,
    Second,
}

pub enum AlarmMatch {
    All,
    DateTime(HashSet<DateTimeMatch>)
}

pub struct Alarm {
    datetime: DateTime<Utc>,
    alarm_match: AlarmMatch,
}

impl Alarm {
    pub fn new(datetime: DateTime<Utc>, alarm_match: AlarmMatch) -> Self {
        Self { datetime, alarm_match }
    }
}
