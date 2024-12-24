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
use std::hash::Hash;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, RwLock, RwLockWriteGuard};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

type Error = ds323x::Error<I2cError, ()>;
type I2cSharedProxy<'a> = I2cProxy<'a, Mutex<I2cDriver<'a>>>;
type Driver<'a> = Ds323x<I2cInterface<I2cSharedProxy<'a>>, ic::DS3231>;

type Callback<AlarmId> = dyn Fn(&AlarmId, &DateTime<Utc>) + Send + Sync + 'static;
type Alarms<AlarmId> = HashMap<AlarmId, (Alarm, Box<Callback<AlarmId>>)>;


struct Api {
    rtc_driver: Driver<'static>,
    system_time: EspSystemTime,
}

pub struct Clock<AlarmId> {
    api: Arc<RwLock<Api>>,
    alarms: Arc<RwLock<Alarms<AlarmId>>>,
    shutdown: Arc<RwLock<AtomicBool>>,
}

impl<AlarmId> Clock<AlarmId>
where AlarmId: Eq + Hash + Send + Sync + 'static {
    pub fn new<OnSynchronize>(i2c_shared_proxy: I2cSharedProxy<'static>,
                              on_synchronize: OnSynchronize) -> Result<Self, ClockError>
    where OnSynchronize: Fn(Result<(), ClockError>) + Send + 'static {

        let mut driver: Driver = Ds323x::new_ds3231(i2c_shared_proxy);

        let mut api = Api {
            rtc_driver: driver,
            system_time: EspSystemTime,
        };

        let _ = Clock::<AlarmId>::synchronize_datetime(&mut api);

        let mut this: Self = Self {
            api: Arc::new(RwLock::new(api)),
            alarms: Arc::new(RwLock::new(HashMap::new())),
            shutdown: Arc::new(RwLock::new(AtomicBool::new(false))),
        };

        this.start_alarm_matching(on_synchronize);

        Ok(this)
    }

    pub fn add_alarm(&mut self, id: AlarmId, alarm: Alarm, callback: fn(&AlarmId, &DateTime<Utc>)) -> Result<(), ClockError> {
        let _ = self
            .alarms
            .write()
            .map_err(|_| ClockError::MutexLockError)?
            .insert(id, (alarm, Box::new(callback)));

        Ok(())
    }

    pub fn remove_alarm(&mut self, id: AlarmId) -> Result<(), ClockError> {
        let _ = self
            .alarms
            .write()
            .map_err(|_| ClockError::MutexLockError)?
            .remove(&id);

        Ok(())
    }

    pub fn clear_all_alarms(&mut self) -> Result<(), ClockError> {
        self
            .alarms
            .write()
            .map_err(|_| ClockError::MutexLockError)?
            .clear();

        Ok(())
    }

    fn start_alarm_matching<OnSynchronize>(&mut self,
                                           on_synchronize: OnSynchronize) -> JoinHandle<()>
    where OnSynchronize: Fn(Result<(), ClockError>) + Send + 'static {

        let api_lock: Arc<RwLock<Api>> = Arc::clone(&self.api);
        let alarms_lock: Arc<RwLock<Alarms<AlarmId>>> = Arc::clone(&self.alarms);
        let shutdown_lock: Arc<RwLock<AtomicBool>> = Arc::clone(&self.shutdown);

        thread::spawn(move || loop {
            if let (Ok(mut api), Ok(alarms)) = (api_lock.read(), alarms_lock.read()) {
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
                        if let Ok(mut api) = api_lock.write() {
                            let result: Result<(), ClockError> = Clock::<AlarmId>::synchronize_datetime(&mut *api);
                            on_synchronize(result);                            
                        }
                    }
                }
            }

            if let Ok(shutdown) = shutdown_lock.read() {
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

impl<AlarmId> Drop for Clock<AlarmId> {
    fn drop(&mut self) {
        if let Ok(shutdown) = self.shutdown.write() {
            shutdown.store(true, Ordering::SeqCst);
        }
    }
}


impl<AlarmId> ReadClock for Clock<AlarmId> {
    fn get_datetime(&self) -> Result<DateTime<Utc>, ClockError> {
        let seconds: u64 = self
            .api
            .read()
            .map_err(|_| ClockError::MutexLockError)?
            .system_time
            .get_time()
            .as_secs();

        DateTime::from_timestamp(seconds as i64, 0)
            .ok_or(ClockError::InvalidTimestamp(seconds))
    }
}

impl<AlarmId> WriteClock for Clock<AlarmId> {
    fn set_datetime(&mut self, datetime: DateTime<Utc>) -> Result<(), ClockError> {
        let mut api: RwLockWriteGuard<Api> = self.api.write().map_err(|_| ClockError::MutexLockError)?;
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
