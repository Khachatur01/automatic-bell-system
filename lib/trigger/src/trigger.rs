use std::sync::{Arc, Mutex};
use chrono::{DateTime, Utc};
use interface::clock::ReadClock;
use interface::disk::ReadDisk;
use crate::alarm::Alarm;


pub struct Trigger {
    alarms: Vec<Alarm>,
    next_trigger_datetime: DateTime<Utc>
}

impl Trigger {
    pub fn new(clock: Arc<Mutex<impl ReadClock>>, disk: Arc<Mutex<impl ReadDisk>>) {

    }
}
