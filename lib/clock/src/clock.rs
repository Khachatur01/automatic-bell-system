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
type Alarms<AlarmId> = HashMap<AlarmId, Alarm>;


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
where AlarmId: Eq + Hash + Send + Sync + Clone + 'static {
    pub fn new<OnSynchronize, OnAlarm>(i2c_shared_proxy: I2cSharedProxy<'static>,
                                       on_synchronize: OnSynchronize,
                                       on_alarm: OnAlarm,
                                       alarm_match_check_interval_ms: u64,) -> Result<Self, ClockError>
    where OnSynchronize: Fn(Result<(), ClockError>) + Send + 'static,
          OnAlarm: Fn(&AlarmId, &Alarm, &DateTime<Utc>) + Send + 'static, {

        let mut driver: Driver = Ds323x::new_ds3231(i2c_shared_proxy);

        let mut api = Api {
            rtc_driver: driver,
            system_time: EspSystemTime,
        };

        Clock::<AlarmId>::synchronize_datetime(&mut api)?;

        let mut this: Self = Self {
            api: Arc::new(RwLock::new(api)),
            alarms: Arc::new(RwLock::new(HashMap::new())),
            shutdown: Arc::new(RwLock::new(AtomicBool::new(false)))
        };

        this.start_alarm_matching(on_synchronize, on_alarm, alarm_match_check_interval_ms);

        Ok(this)
    }

    pub fn is_alarm_id_unique(&self, id: &AlarmId) -> Result<bool, ClockError> {
        let contains: bool = self
            .alarms
            .read()
            .map_err(|_| ClockError::MutexLockError)?
            .contains_key(id);

        Ok(!contains)
    }

    pub fn get_alarms(&self) -> Result<HashMap<AlarmId, Alarm>, ClockError> {
        let alarms: HashMap<AlarmId, Alarm> = self
            .alarms
            .read()
            .map_err(|_| ClockError::MutexLockError)?
            .iter()
            /* collect alarms into new hashmap to remove callbacks */
            .fold(HashMap::new(), |mut accumulator, (alarm_id, alarm)| {
                accumulator.insert(alarm_id.clone(), alarm.clone());
                accumulator
            });

        Ok(alarms)
    }

    pub fn get_alarm(&self, id: &AlarmId) -> Result<Alarm, ClockError> {
        self
            .alarms
            .read()
            .map_err(|_| ClockError::MutexLockError)?
            .get(id)
            .map(Clone::clone)
            .ok_or(ClockError::AlarmNotFound)
    }

    pub fn add_alarm(&mut self, id: AlarmId, alarm: Alarm) -> Result<(), ClockError> {
        let _ = self
            .alarms
            .write()
            .map_err(|_| ClockError::MutexLockError)?
            .insert(id, alarm);

        Ok(())
    }

    pub fn remove_alarm(&mut self, id: &AlarmId) -> Result<(), ClockError> {
        let _ = self
            .alarms
            .write()
            .map_err(|_| ClockError::MutexLockError)?
            .remove(id);

        Ok(())
    }

    pub fn remove_alarm_if<F>(&mut self, predicate: F) -> Result<(), ClockError>
    where F: Fn(&AlarmId) -> bool {

        let mut alarms: RwLockWriteGuard<Alarms<AlarmId>> = self
            .alarms
            .write()
            .map_err(|_| ClockError::MutexLockError)?;

        let removable_ids = alarms
            .keys()
            .filter(|alarm_id| predicate(alarm_id))
            .map(Clone::clone)
            .collect::<Vec<AlarmId>>();

        for alarm_id in removable_ids {
            alarms.remove(&alarm_id);
        }

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

    fn start_alarm_matching<OnSynchronize, OnAlarm>(&mut self,
                                                    on_synchronize: OnSynchronize,
                                                    on_alarm: OnAlarm,
                                                    alarm_match_check_interval_ms: u64) -> JoinHandle<()>
    where OnSynchronize: Fn(Result<(), ClockError>) + Send + 'static,
          OnAlarm: Fn(&AlarmId, &Alarm, &DateTime<Utc>) + Send + 'static, {

        let api_lock: Arc<RwLock<Api>> = Arc::clone(&self.api);
        let alarms_lock: Arc<RwLock<Alarms<AlarmId>>> = Arc::clone(&self.alarms);
        let shutdown_lock: Arc<RwLock<AtomicBool>> = Arc::clone(&self.shutdown);

        thread::spawn(move || loop {
            /* lock(read) api to read current time */
            let datetime: Option<DateTime<Utc>> = 
                api_lock
                    .read()
                    .map_or(None, |api| {
                        let seconds: u64 = api.system_time.get_time().as_secs();
                        DateTime::from_timestamp(seconds as i64, 0)                
                    });

            let Some(datetime) = datetime else {
                continue;
            };

            /* check matching alarms */
            if let Ok(alarms) = alarms_lock.read() {
                alarms
                    .iter()
                    .filter(|(id, alarm)| alarm.matches(&datetime))
                    .for_each(|(id, alarm)| on_alarm(id, alarm, &datetime));
            }

            /* lock(write) api to synchronize time every hour */
            if datetime.minute() == 0 && datetime.second() == 0 {
                if let Ok(mut api) = api_lock.write() {
                    let result: Result<(), ClockError> = Clock::<AlarmId>::synchronize_datetime(&mut *api);
                    on_synchronize(result);
                }
            };

            if let Ok(shutdown) = shutdown_lock.read() {
                if shutdown.load(Ordering::SeqCst) {
                    break;
                }
            }

            thread::sleep(Duration::from_millis(alarm_match_check_interval_ms));
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
