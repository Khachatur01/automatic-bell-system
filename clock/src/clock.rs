use std::num::NonZeroU32;
use crate::error::ClockError;
use crate::synchronize_by::SynchronizeBy;
use crate::system_time::SystemTime;
use chrono::{DateTime, Utc};
use ds323x::interface::I2cInterface;
use ds323x::{ic, Alarm1Matching, DateTimeAccess, DayAlarm1, Ds323x};
use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::gpio::{IOPin, Input, PinDriver};
use esp_idf_svc::hal::i2c::{I2cDriver, I2cError};
use esp_idf_svc::systime::EspSystemTime;
use shared_bus::I2cProxy;
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use esp_idf_svc::hal::{delay, task};
use esp_idf_svc::hal::task::notification;
use esp_idf_svc::hal::task::notification::{Notification, Notifier};

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
    pub fn new<INT: IOPin>(i2c_shared_proxy: I2cSharedProxy<'static>, synchronize_by: SynchronizeBy<INT>) -> Result<Self, ClockError> {
        let mut driver: Driver = Ds323x::new_ds3231(i2c_shared_proxy);

        let api: Arc<Mutex<Api>> = Arc::new(Mutex::new(
            Api {
                rtc_driver: driver,
                system_time: EspSystemTime
            }
        ));

        let _ = Clock::synchronize_time(Arc::clone(&api));

        match synchronize_by {
            SynchronizeBy::Delay { seconds } => {
                let api_clone: Arc<Mutex<Api>> = Arc::clone(&api);
                Clock::synchronize_by_delay(api_clone, seconds)?;
            }
            SynchronizeBy::Interruption { alarm, pin } => {
                let api_clone: Arc<Mutex<Api>> = Arc::clone(&api);

                /* set alarm if alarm is present */
                if let Some((alarm, alarm_matching)) = alarm {
                    let alarm: DayAlarm1 = DayAlarm1::from(alarm);
                    let alarm_matching: Alarm1Matching = Alarm1Matching::from(alarm_matching);

                    api_clone
                        .lock()
                        .map_err(|_| ClockError::ApiMutexLockError)?
                        .rtc_driver
                        .set_alarm1_day(alarm, alarm_matching)
                        .map_err(|_| ClockError::EspError)?;
                }

                Clock::synchronize_by_interruption(api_clone, pin)?;
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

    fn synchronize_by_delay(api: Arc<Mutex<Api>>, seconds: u32) -> Result<JoinHandle<()>, ClockError> {
        let join_handle: JoinHandle<()> = thread::spawn(move || {
            let milliseconds: u32 = seconds * 1000;

            loop {
                FreeRtos::delay_ms(milliseconds);
                let _ = Clock::synchronize_time(Arc::clone(&api));
            }
        });

        Ok(join_handle)
    }

    fn synchronize_by_interruption<INT: IOPin>(api: Arc<Mutex<Api>>,
                                               mut interrupt_pin: PinDriver<'static, INT, Input>) -> Result<JoinHandle<()>, ClockError> {

        let join_handle: JoinHandle<()> = thread::spawn(move || {
            let notification: Notification = Notification::new();
            let notifier: Arc<Notifier> = notification.notifier();

            unsafe {
                let _ = interrupt_pin
                    .subscribe(move || {
                        if let Some(non_zero_u32) = NonZeroU32::new(1) {
                            let _ = notifier.notify(non_zero_u32);
                        }
                    });
            };

            loop {
                let _ = interrupt_pin.enable_interrupt();

                if notification.wait(delay::BLOCK).is_some() {
                    let _ = Clock::synchronize_time(Arc::clone(&api));
                };
            }
        });

        Ok(join_handle)
    }

    fn synchronize_time(api: Arc<Mutex<Api>>) -> Result<(), ClockError> {
        let mut api: MutexGuard<Api> = api.lock().map_err(|_| ClockError::ApiMutexLockError)?;

        if let Ok(datetime) = api.rtc_driver.datetime() {
            let timestamp: u64 = datetime.and_utc().timestamp() as u64;

            api.system_time.set_time(
                Duration::new(timestamp, 0)
            );
            println!("Clock synchronized.");

            Ok(())
        } else {
            println!("Clock doesn't synchronize.");
            Err(ClockError::SynchronizationError)
        }
    }
}
