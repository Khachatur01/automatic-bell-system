pub mod alarm_id;
pub mod model;
pub mod to_alarms_with_id;
mod error;

use crate::schedule_system::alarm_id::AlarmId;
use crate::schedule_system::error::ScheduleSystemError;
use crate::schedule_system::model::output_index::OutputIndex;
use crate::schedule_system::to_alarms_with_id::ToAlarmsWithId;
use crate::synchronizer::{BoxedMutex, BoxedRwLock, IntoBoxedMutex, IntoBoxedRwLock, IntoMutexOutputPin, MutexOutputPin};
use access_point::access_point::AccessPoint;
use chrono::{DateTime, Utc};
use clock::alarm::Alarm;
use clock::clock::Clock;
use disk::disk::Disk;
use display::display::Display;
use esp_idf_svc::hal::gpio::{AnyOutputPin, OutputPin, Pin};
use esp_idf_svc::hal::i2c::{I2cConfig, I2cDriver};
use esp_idf_svc::hal::peripheral::Peripheral;
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::spi::config::DriverConfig;
use esp_idf_svc::hal::spi::SpiDriver;
use interface::clock::{ReadClock, WriteClock};
use interface::disk::{ReadDisk, WriteDisk};
use interface::Path;
use shared_bus::BusManagerStd;
use std::collections::HashMap;
use std::ops::Deref;
use std::thread;
use std::time::Duration;
use uuid::Uuid;

type ScheduleSystemResult<Ok> = Result<Ok, ScheduleSystemError>;
type AlarmOutputs<'a> = Vec<MutexOutputPin<'a>>;

const ALARMS_LOCATION: &str = "/alarms";
const ALARMS_FILE_NAME: &str = "a.jso";


/* Wrap fields into box to prevent stack overflowing.*/
pub struct ScheduleSystem {
    access_point: BoxedMutex<AccessPoint<'static>>,
    /* Clock is RwLock, because it requires immutable reference for reading time. */
    clock: BoxedRwLock<Clock<AlarmId>>,
    disk: BoxedMutex<Disk<'static>>,
    display: BoxedMutex<Display<'static>>,
}

impl ScheduleSystem {
    pub fn new(peripherals: Peripherals) -> Result<Self, ScheduleSystemError> {
        /* Init I2c bus */
        let i2c = peripherals.i2c0;
        let sda = peripherals.pins.gpio22;
        let scl = peripherals.pins.gpio23;
        let i2c_config = I2cConfig::default();
        let i2c_driver: I2cDriver = I2cDriver::new(i2c, sda, scl, &i2c_config).map_err(ScheduleSystemError::EspError)?;

        let i2c_bus_manager: &'static BusManagerStd<I2cDriver> = shared_bus::new_std!(I2cDriver = i2c_driver).ok_or(ScheduleSystemError::I2cSharedBusError)?;
        /* Init I2c bus */

        /* Init SPI driver */
        let spi = peripherals.spi2;
        let scl = peripherals.pins.gpio18;
        let sdo = peripherals.pins.gpio19;
        let sdi = peripherals.pins.gpio21;
        let cs = peripherals.pins.gpio5;

        let driver_config: DriverConfig = DriverConfig::default();
        let spi_driver: SpiDriver = SpiDriver::new(spi, scl, sdo, Some(sdi), &driver_config).map_err(ScheduleSystemError::EspError)?;
        /* Init SDA driver */

        let alarm_output_pins: Vec<MutexOutputPin> = vec![
            Into::<AnyOutputPin>::into(peripherals.pins.gpio2)
                .try_into_mutex_output_pin()
                .map_err(ScheduleSystemError::EspError)?,
            Into::<AnyOutputPin>::into(peripherals.pins.gpio4)
                .try_into_mutex_output_pin()
                .map_err(ScheduleSystemError::EspError)?,
        ];

        let clock: BoxedRwLock<Clock<AlarmId>> = Clock::new(
            i2c_bus_manager.acquire_i2c(),
            |result| println!("Synchronizing..."),
            move |alarm_id: &AlarmId, date_time| ScheduleSystem::on_alarm(alarm_id, date_time, &alarm_output_pins))
            .map_err(ScheduleSystemError::ClockError)?
            .into_boxed_rwlock();

        let access_point: BoxedMutex<AccessPoint> = AccessPoint::new(peripherals.modem)
            .map_err(ScheduleSystemError::EspError)?
            .into_boxed_mutex();

        let disk: BoxedMutex<Disk> = Disk::new(spi_driver, cs)
            .map_err(ScheduleSystemError::EspError)?
            .into_boxed_mutex();

        let display: BoxedMutex<Display> = Display::new(i2c_bus_manager.acquire_i2c())
            .map_err(ScheduleSystemError::DisplayError)?
            .into_boxed_mutex();

        let this: Self = Self {
            access_point,
            clock,
            disk,
            display,
        };

        this.synchronize_alarms_from_disk()?;

        Ok(this)
    }


    pub fn enable_access_point(&self) -> ScheduleSystemResult<()> {
        self.access_point
            .lock()
            .map_err(|_| ScheduleSystemError::MutexLockError)?
            .start()
            .map_err(ScheduleSystemError::EspError)?;

        Ok(())
    }

    pub fn disable_access_point(&self) -> ScheduleSystemResult<()> {
        self.access_point
            .lock()
            .map_err(|_| ScheduleSystemError::MutexLockError)?
            .stop()
            .map_err(ScheduleSystemError::EspError)?;

        Ok(())
    }


    pub fn read_from_file(&self, path: &Path) -> ScheduleSystemResult<Vec<u8>> {
        self.disk
            .lock()
            .map_err(|_| ScheduleSystemError::MutexLockError)?
            .read_from_file(path)
            .map_err(ScheduleSystemError::DiskError)
    }

    pub fn write_to_file(&self, path: &Path, data_buffer: &mut [u8]) -> ScheduleSystemResult<()> {
        self.disk
            .lock()
            .map_err(|_| ScheduleSystemError::MutexLockError)?
            .write_to_file(path, data_buffer)
            .map_err(ScheduleSystemError::DiskError)
    }


    pub fn get_time(&self) -> ScheduleSystemResult<DateTime<Utc>> {
        self.clock
            .read()
            .map_err(|_| ScheduleSystemError::MutexLockError)?
            .get_datetime()
            .map_err(ScheduleSystemError::ClockError)
    }

    pub fn set_time(&self, datetime: DateTime<Utc>) -> ScheduleSystemResult<()> {
        self.clock
            .write()
            .map_err(|_| ScheduleSystemError::MutexLockError)?
            .set_datetime(datetime)
            .map_err(ScheduleSystemError::ClockError)
    }


    pub fn get_alarm(&self, alarm_id: &AlarmId) -> ScheduleSystemResult<Alarm> {
        self.clock
            .read()
            .map_err(|_| ScheduleSystemError::MutexLockError)?
            .get_alarm(alarm_id)
            .map_err(ScheduleSystemError::ClockError)
    }

    pub fn get_alarms(&self) -> ScheduleSystemResult<HashMap<AlarmId, Alarm>> {
        self.clock
            .read()
            .map_err(|_| ScheduleSystemError::MutexLockError)?
            .get_alarms()
            .map_err(ScheduleSystemError::ClockError)
    }

    pub fn get_alarms_by_output_index(&self, output_index: OutputIndex) -> ScheduleSystemResult<HashMap<AlarmId, Alarm>> {
        let alarms = self.clock
            .read()
            .map_err(|_| ScheduleSystemError::MutexLockError)?
            .get_alarms()
            .map_err(ScheduleSystemError::ClockError)?
            .into_iter()
            .fold(HashMap::new(), |mut accumulator, (alarm_id, alarm)| {
                if alarm_id.output_index == output_index {
                    accumulator.insert(alarm_id, alarm.clone());
                }

                accumulator
            });

        self.write_alarms_to_disk()?;

        Ok(alarms)
    }

    pub fn add_alarm(&self, output_index: OutputIndex, alarm: Alarm) -> ScheduleSystemResult<()> {
        let alarm_id: AlarmId = AlarmId {
            output_index,
            uuid: Uuid::new_v4(),
        };

        let alarms = self.clock
            .write()
            .map_err(|_| ScheduleSystemError::MutexLockError)?
            .add_alarm(alarm_id, alarm)
            .map_err(ScheduleSystemError::ClockError)?;

        self.write_alarms_to_disk()?;

        Ok(alarms)
    }

    pub fn remove_alarm(&self, alarm_id: &AlarmId) -> ScheduleSystemResult<()> {
        self.clock
            .write()
            .map_err(|_| ScheduleSystemError::MutexLockError)?
            .remove_alarm(alarm_id)
            .map_err(ScheduleSystemError::ClockError)?;

        self.write_alarms_to_disk()
    }

    pub fn remove_alarms_by_output_index(&self, output_index: &OutputIndex) -> ScheduleSystemResult<()> {
        self.clock
            .write()
            .map_err(|_| ScheduleSystemError::MutexLockError)?
            .remove_alarm_if(|alarm_id: &AlarmId| alarm_id.output_index == *output_index)
            .map_err(ScheduleSystemError::ClockError)?;

        self.write_alarms_to_disk()
    }

    fn synchronize_alarms_from_disk(&self) -> ScheduleSystemResult<()> {
        // let mut disk = self
        //     .disk
        //     .lock()
        //     .map_err(|_| ScheduleSystemError::MutexLockError)?;
        // let mut clock = self
        //     .clock
        //     .write()
        //     .map_err(|_| ScheduleSystemError::MutexLockError)?;
        // 
        // let path: Path = Path::try_from(format!("{ALARMS_LOCATION}/{ALARMS_FILE_NAME}"))
        //     .map_err(ScheduleSystemError::PathParseError)?;
        // 
        // let alarms_json_buffer: Vec<u8> = disk
        //     .read_from_file(&path)
        //     .map_err(ScheduleSystemError::DiskError)?;
        // let alarms_json: String = String::from_utf8_lossy(&alarms_json_buffer).to_string();
        // 
        // let alarms: Vec<AlarmWithIdDTO> = serde_json::from_str(&alarms_json)
        //     .map_err(ScheduleSystemError::SerdeError)?;
        // 
        // clock.clear_all_alarms()
        //     .map_err(ScheduleSystemError::ClockError)?;
        // 
        // for alarm in alarms {
        //     let alarm_id: AlarmId = alarm
        //         .id
        //         .try_into()
        //         .map_err(ScheduleSystemError::AlarmIdParseError)?;
        // 
        //     clock
        //         .add_alarm(alarm_id, alarm.alarm.into())
        //         .map_err(ScheduleSystemError::ClockError)?;
        // }

        Ok(())
    }

    fn write_alarms_to_disk(&self) -> ScheduleSystemResult<()> {
        // let mut disk = self
        //     .disk
        //     .lock()
        //     .map_err(|_| ScheduleSystemError::MutexLockError)?;
        // let clock = self
        //     .clock
        //     .read()
        //     .map_err(|_| ScheduleSystemError::MutexLockError)?;
        // 
        // let alarms: Vec<AlarmWithIdDTO> = clock.get_alarms()
        //     .map_err(ScheduleSystemError::ClockError)?
        //     .to_alarms_with_id();
        // 
        // let alarms_json: String = serde_json::to_string(&alarms)
        //     .map_err(ScheduleSystemError::SerdeError)?;
        // 
        // let path: Path = Path::try_from(format!("{ALARMS_LOCATION}/{ALARMS_FILE_NAME}"))
        //     .map_err(ScheduleSystemError::PathParseError)?;
        // disk.write_to_file(&path, &alarms_json.as_bytes())
        //     .map_err(ScheduleSystemError::DiskError)?;

        Ok(())
    }


    fn on_alarm(alarm_id: &AlarmId, date_time: &DateTime<Utc>, alarm_output_pins: &Vec<MutexOutputPin>) {
        println!("Alarming {} {} {}", *alarm_id.output_index, alarm_id.uuid, date_time);

        let output_index: usize = *alarm_id.output_index as usize;

        if let Some(output_pin) = alarm_output_pins.get(output_index) {
            /* Using try_lock(). Ignore alarm if output is already locked. */
            if let Ok(mut output_pin_driver) = output_pin.try_lock() {
                let _ = output_pin_driver.set_high();

                thread::sleep(Duration::from_secs(5));

                let _ = output_pin_driver.set_low();
            }
        };
    }
}
