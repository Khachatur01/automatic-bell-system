pub mod alarm_id;
pub mod model;
mod error;

use crate::synchronizer::{IntoBoxedMutex, IntoBoxedRwLock};
use crate::schedule_system::alarm_id::AlarmId;
use crate::schedule_system::error::ScheduleSystemError;
use crate::types::{BoxedMutex, BoxedRwLock};
use access_point::access_point::AccessPoint;
use chrono::{DateTime, Utc};
use clock::alarm::Alarm;
use clock::clock::Clock;
use disk::disk::Disk;
use display::display::Display;
use esp_idf_svc::hal::gpio::OutputPin;
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
use uuid::Uuid;
use crate::schedule_system::model::alarm_outputs::AlarmOutputs;
use crate::schedule_system::model::output_index::OutputIndex;

type ScheduleSystemResult<Ok> = Result<Ok, ScheduleSystemError>;

/* Wrap fields into box to prevent stack overflowing.*/
pub struct ScheduleSystem {
    access_point: BoxedMutex<AccessPoint<'static>>,
    /* Clock is RwLock, because it requires immutable reference for reading time. */
    clock: BoxedRwLock<Clock<AlarmId>>,
    disk: BoxedMutex<Disk<'static>>,
    display: BoxedMutex<Display<'static>>,
    alarm_output: BoxedMutex<AlarmOutputs>
}

impl ScheduleSystem {
    pub fn new(peripherals: Peripherals) -> Result<Self, ScheduleSystemError> {
        /* Init I2c bus */
        let i2c = peripherals.i2c0;
        let sda = peripherals.pins.gpio22;
        let scl = peripherals.pins.gpio23;
        let i2c_config = I2cConfig::default();
        let i2c_driver: I2cDriver = I2cDriver::new(i2c, sda, scl, &i2c_config).unwrap();

        let i2c_bus_manager: &'static BusManagerStd<I2cDriver> = shared_bus::new_std!(I2cDriver = i2c_driver).unwrap();
        /* Init I2c bus */

        /* Init SPI driver */
        let spi = peripherals.spi2;
        let scl = peripherals.pins.gpio18;
        let sdo = peripherals.pins.gpio19;
        let sdi = peripherals.pins.gpio21;
        let cs = peripherals.pins.gpio5;

        let driver_config: DriverConfig = DriverConfig::default();
        let spi_driver: SpiDriver = SpiDriver::new(spi, scl, sdo, Some(sdi), &driver_config).unwrap();
        /* Init SDA driver */


        let access_point: BoxedMutex<AccessPoint> = AccessPoint::new(peripherals.modem)
            .map_err(ScheduleSystemError::EspError)?
            .into_boxed_mutex();

        let clock: BoxedRwLock<Clock<AlarmId>> = Clock::new(
            i2c_bus_manager.acquire_i2c(),
            |result| println!("Synchronizing..."))
            .map_err(ScheduleSystemError::ClockError)?
            .into_boxed_rwlock();

        let disk: BoxedMutex<Disk> = Disk::new(spi_driver, cs)
            .map_err(ScheduleSystemError::EspError)?
            .into_boxed_mutex();

        let display: BoxedMutex<Display> = Display::new(i2c_bus_manager.acquire_i2c())
            .map_err(ScheduleSystemError::DisplayError)?
            .into_boxed_mutex();

        let alarm_output: BoxedMutex<AlarmOutputs> = (
            peripherals.pins.gpio2,
            peripherals.pins.gpio4,
        ).into_boxed_mutex();

        Ok(Self {
            access_point,
            clock,
            disk,
            display,
            alarm_output,
        })
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
            .add_alarm(alarm_id, alarm, |alarm_id, date_time| println!("Alarming {:?}, {date_time}", *alarm_id))
            .map_err(ScheduleSystemError::ClockError)?;

        Ok(alarms)
    }

    pub fn remove_alarm(&self, alarm_id: &AlarmId) -> ScheduleSystemResult<()> {
        self.clock
            .write()
            .map_err(|_| ScheduleSystemError::MutexLockError)?
            .remove_alarm(alarm_id)
            .map_err(ScheduleSystemError::ClockError)
    }

    pub fn remove_alarms_by_output_index(&self, output_index: &OutputIndex) -> ScheduleSystemResult<()> {
        self.clock
            .write()
            .map_err(|_| ScheduleSystemError::MutexLockError)?
            .remove_alarm_if(|alarm_id: &AlarmId| alarm_id.output_index == *output_index)
            .map_err(ScheduleSystemError::ClockError)
    }
}
