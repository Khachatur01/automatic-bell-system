use chrono::{DateTime, Utc};
use interface::clock::ReadClock;
use interface::disk::ReadDisk;
use interface::ClockError;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;


pub struct Trigger<Clock, Disk> {
    next_trigger_datetime: Option<DateTime<Utc>>,
    clock: Arc<Mutex<Clock>>,
    disk: Arc<Mutex<Disk>>,
}

impl<Clock: ReadClock + Send + 'static, Disk: ReadDisk + Send + 'static> Trigger<Clock, Disk> {

    pub fn run_loop(clock: Arc<Mutex<Clock>>,
                    disk: Arc<Mutex<Disk>>,
                    on_trigger: fn(DateTime<Utc>)) {

        thread::spawn(move || {
            let mut next_trigger_datetime_option: Option<DateTime<Utc>> = None;

            loop {
                if let Ok(clock) = clock.lock() {
                    if let Ok(current_datetime) = clock.get_datetime() {
                        if let Some(next_trigger_datetime) = next_trigger_datetime_option {
                            if current_datetime.cmp(&next_trigger_datetime).is_eq() {
                                on_trigger(current_datetime);
                                next_trigger_datetime_option = Self::calculate_next_trigger_datetime(Arc::clone(&disk)).ok();
                            }
                        }
                    }
                };

                thread::sleep(Duration::from_secs(1));
            }
        });
    }

    pub fn calculate_next_trigger_datetime(disk: Arc<Mutex<Disk>>) -> Result<DateTime<Utc>, ClockError> {
        Err(ClockError::MutexLockError)
    }
}
