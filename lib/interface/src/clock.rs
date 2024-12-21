use chrono::{DateTime, Utc};
use crate::clock::error::ClockError;

pub mod error;

pub trait ReadClock {
    fn get_datetime(&self) -> Result<DateTime<Utc>, ClockError>;
}

pub trait WriteClock {
    fn set_datetime(&mut self, datetime: DateTime<Utc>) -> Result<(), ClockError>;
}

pub trait ReadWriteClock: ReadClock + WriteClock {}
