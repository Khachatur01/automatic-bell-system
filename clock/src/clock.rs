use crate::system_time::SystemTime;
use chrono::{DateTime, Utc};
use ds323x::interface::I2cInterface;
use ds323x::Hours::H24;
use ds323x::{ic, Alarm1Matching, DateTimeAccess, DayAlarm1, Ds323x};
use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::gpio::{IOPin, Input, PinDriver};
use esp_idf_svc::hal::i2c::{I2cDriver, I2cError};
use esp_idf_svc::sys::EspError;
use esp_idf_svc::systime::EspSystemTime;
use shared_bus::I2cProxy;
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use crate::error::ClockError;
use crate::synchronize_by::SynchronizeBy;


type Error = ds323x::Error<I2cError, ()>;
type I2cSharedProxy<'a> = I2cProxy<'a, Mutex<I2cDriver<'a>>>;
type Driver<'a> = Ds323x<I2cInterface<I2cSharedProxy<'a>>, ic::DS3231>;


struct Api {
    rtc_driver: Driver<'static>,
    system_time: EspSystemTime,
}

pub struct Clock {
    api: Arc<Mutex<Api>>
}

impl Clock {
    pub fn new<INT: IOPin>(i2c_shared_proxy: I2cSharedProxy<'static>, synchronize_by: SynchronizeBy<INT>) -> Result<Self, EspError> {
        let mut driver: Driver = Ds323x::new_ds3231(i2c_shared_proxy);

        let api: Arc<Mutex<Api>> = Arc::new(Mutex::new(
            Api {
                rtc_driver: driver,
                system_time: EspSystemTime
            }
        ));

        match synchronize_by {
            SynchronizeBy::Delay { seconds } => {
                let api_clone: Arc<Mutex<Api>> = Arc::clone(&api);
                Clock::synchronize_by_delay(api_clone, seconds);
            }
            SynchronizeBy::Interrupt { pin } => {
                let api_clone: Arc<Mutex<Api>> = Arc::clone(&api);
                Clock::synchronize_by_interruption(api_clone, pin);
            }
        }

        Ok(Self { api })
    }

    pub fn datetime(&self) -> Result<DateTime<Utc>, ClockError> {
        let seconds: u64 = self
            .api
            .lock()
            .map_err(|_| ClockError::ApiMutexLockError)?
            .system_time
            .get_time()
            .as_secs();

        DateTime::from_timestamp(seconds as i64, 0)
            .ok_or(ClockError::InvalidTimestamp(seconds))
    }

    fn synchronize_by_delay(api: Arc<Mutex<Api>>, seconds: u32) -> JoinHandle<()> {
        thread::spawn(move || {
            let milliseconds: u32 = seconds * 100;

            loop {
                if let Ok(()) = Clock::synchronize_time(Arc::clone(&api)) {
                    println!("Synchronizing clock...");
                } else {
                    todo!(log warning)
                }

                FreeRtos::delay_ms(milliseconds);
            }
        })
    }

    fn synchronize_by_interruption<INT: IOPin>(api: Arc<Mutex<Api>>,
                                               interrupt_pin: PinDriver<'static, INT, Input>) -> JoinHandle<()> {

        // api.lock().unwrap().rtc_driver.set_alarm1_day(
        //     DayAlarm1 {
        //         day: 0,
        //         hour: H24(0),
        //         minute: 0,
        //         second: 0
        //     },
        //     Alarm1Matching::MinutesAndSecondsMatch
        // ).unwrap();

        thread::spawn(move || {
            loop {
                todo!();
            }
        })
    }

    fn synchronize_time(api: Arc<Mutex<Api>>) -> Result<(), ClockError> {
        let mut api: MutexGuard<Api> = api.lock().map_err(|_| ClockError::ApiMutexLockError)?;

        if let Ok(datetime) = api.rtc_driver.datetime() {
            let timestamp: u64 = datetime.and_utc().timestamp() as u64;

            api.system_time.set_time(
                Duration::new(timestamp, 0)
            );

            Ok(())
        } else {
            Err(ClockError::SynchronizationError)
        }
    }
}
