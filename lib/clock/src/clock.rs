use crate::alarm::Alarm;
use crate::system_time::SystemTime;
use chrono::{DateTime, NaiveDateTime, Timelike, Utc};
use ds323x::interface::I2cInterface;
use ds323x::{ic, DateTimeAccess, Ds323x};
use esp_idf_svc::hal::i2c::{I2cDriver, I2cError};
use esp_idf_svc::systime::EspSystemTime;
use interface::clock::{ReadClock, WriteClock};
use interface::ClockError;
use shared_bus::I2cProxy;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;


pub type AlarmId = String;

type Error = ds323x::Error<I2cError, ()>;
type I2cSharedProxy<'a> = I2cProxy<'a, Mutex<I2cDriver<'a>>>;
type Driver<'a> = Ds323x<I2cInterface<I2cSharedProxy<'a>>, ic::DS3231>;

type Callback = dyn Fn(&AlarmId, &DateTime<Utc>) + Send + 'static;
type Alarms = HashMap<AlarmId, (Alarm, Box<Callback>)>;


struct Api {
    rtc_driver: Driver<'static>,
    system_time: EspSystemTime,
}

pub struct Clock {
    api: Arc<Mutex<Api>>,
    alarms: Arc<Mutex<Alarms>>,
    shutdown: Arc<Mutex<AtomicBool>>,
}

impl Clock {
    pub fn new<OnSynchronize>(i2c_shared_proxy: I2cSharedProxy<'static>,
                              on_synchronize: OnSynchronize) -> Result<Self, ClockError>
    where OnSynchronize: Fn(Result<(), ClockError>) + Send + 'static {

        let mut driver: Driver = Ds323x::new_ds3231(i2c_shared_proxy);

        let mut api = Api {
            rtc_driver: driver,
            system_time: EspSystemTime,
        };

        let _ = Clock::synchronize_datetime(&mut api);

        let mut api: Arc<Mutex<Api>> = Arc::new(Mutex::new(api));
        let mut this: Self = Self {
            api,
            alarms: Arc::new(Mutex::new(HashMap::new())),
            shutdown: Arc::new(Mutex::new(AtomicBool::new(false))),
        };

        this.start_alarm_matching(on_synchronize);

        Ok(this)
    }

    pub fn add_alarm(&mut self, id: AlarmId, alarm: Alarm, callback: fn(&AlarmId, &DateTime<Utc>)) -> Result<(), ClockError> {
        let _ = self
            .alarms
            .lock()
            .map_err(|_| ClockError::MutexLockError)?
            .insert(id, (alarm, Box::new(callback)));

        Ok(())
    }

    pub fn remove_alarm(&mut self, id: AlarmId) -> Result<(), ClockError> {
        let _ = self
            .alarms
            .lock()
            .map_err(|_| ClockError::MutexLockError)?
            .remove(&id);

        Ok(())
    }

    pub fn clear_all_alarms(&mut self) -> Result<(), ClockError> {
        self
            .alarms
            .lock()
            .map_err(|_| ClockError::MutexLockError)?
            .clear();

        Ok(())
    }

    fn start_alarm_matching<OnSynchronize>(&mut self,
                                           on_synchronize: OnSynchronize) -> JoinHandle<()>
    where OnSynchronize: Fn(Result<(), ClockError>) + Send + 'static {

        let api: Arc<Mutex<Api>> = Arc::clone(&self.api);
        let alarms: Arc<Mutex<Alarms>> = Arc::clone(&self.alarms);
        let shutdown: Arc<Mutex<AtomicBool>> = Arc::clone(&self.shutdown);

        thread::spawn(move || loop {
            if let (Ok(mut api), Ok(alarms)) = (api.lock(), alarms.lock()) {
                let seconds: u64 = api.system_time.get_time().as_secs();
                let datetime: Option<DateTime<Utc>> = DateTime::from_timestamp(seconds as i64, 0);

                if let Some(datetime) = datetime {
                    alarms.iter().for_each(|(id, (alarm, callback))| {
                        if alarm.matches(&datetime) {
                            callback(id, &datetime)
                        }
                    });

                    /* synchronize datetime every hour */
                    if datetime.minute() == 0 && datetime.second() == 0 {
                        let result: Result<(), ClockError> = Clock::synchronize_datetime(&mut *api);
                        on_synchronize(result);
                    }
                }
            }

            if let Ok(shutdown) = shutdown.lock() {
                if shutdown.load(Ordering::SeqCst) {
                    break;
                }
            }

            thread::sleep(Duration::from_secs(1));
        })
    }

    /**
    * Synchronize ESP32 system time by getting external RTC time.
    */
    fn synchronize_datetime(api: &mut Api) -> Result<(), ClockError> {
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

impl Drop for Clock {
    fn drop(&mut self) {
        if let Ok(shutdown) = self.shutdown.lock() {
            shutdown.store(true, Ordering::SeqCst);
        }
    }
}


impl ReadClock for Clock {
    fn get_datetime(&self) -> Result<DateTime<Utc>, ClockError> {
        let seconds: u64 = self
            .api
            .lock()
            .map_err(|_| ClockError::MutexLockError)?
            .system_time
            .get_time()
            .as_secs();

        DateTime::from_timestamp(seconds as i64, 0)
            .ok_or(ClockError::InvalidTimestamp(seconds))
    }
}

impl WriteClock for Clock {
    fn set_datetime(&mut self, datetime: DateTime<Utc>) -> Result<(), ClockError> {
        let mut api: MutexGuard<Api> = self.api.lock().map_err(|_| ClockError::MutexLockError)?;
        let naive_date_time: NaiveDateTime = datetime.naive_utc();
        let timestamp: u64 = naive_date_time.and_utc().timestamp() as u64;

        api.rtc_driver
            .set_datetime(&naive_date_time)
            .map_err(|_| ClockError::MutexLockError)?;


        api.system_time.set_time(
            Duration::new(timestamp, 0)
        );

        Ok(())
    }
}
