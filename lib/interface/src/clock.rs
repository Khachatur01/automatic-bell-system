use chrono::{DateTime, Utc};
use crate::clock::error::ClockError;

pub mod error;

pub trait ReadClock {
    fn read(&self) -> Result<DateTime<Utc>, ClockError>;
}

pub trait WriteClock {
    fn write(&self) -> Result<(), ClockError>;
}

pub trait ReadWriteClock: ReadClock + WriteClock {}
