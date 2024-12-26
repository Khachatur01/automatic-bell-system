pub mod alarm_id;
mod error;

use std::collections::HashMap;
use crate::schedule_system::alarm_id::AlarmId;
use crate::schedule_system::error::ScheduleSystemError;
use access_point::access_point::AccessPoint;
use chrono::{DateTime, Utc};
use clock::clock::Clock;
use disk::disk::Disk;
use display::display::Display;
use esp_idf_svc::hal::gpio::{Gpio2, Gpio4, OutputPin};
use esp_idf_svc::hal::i2c::{I2cConfig, I2cDriver};
use esp_idf_svc::hal::peripheral::Peripheral;
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::spi::config::DriverConfig;
use esp_idf_svc::hal::spi::SpiDriver;
use interface::clock::{ReadClock, WriteClock};
use interface::disk::{ReadDisk, WriteDisk};
use interface::Path;
use shared_bus::BusManagerStd;
use std::sync::{Arc, Mutex};
use clock::alarm::Alarm;

type ScheduleSystemResult<Ok> = Result<Ok, ScheduleSystemError>;
type AlarmOutput = (Gpio2, Gpio4);

pub struct ScheduleSystem {
    access_point: Arc<Mutex<AccessPoint<'static>>>,
    clock: Arc<Mutex<Clock<AlarmId>>>,
    disk: Arc<Mutex<Disk<'static>>>,
    display: Arc<Mutex<Display<'static>>>,
    alarm_output: Arc<Mutex<AlarmOutput>>
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


        let disk: Disk = Disk::new(spi_driver, cs)
            .map_err(ScheduleSystemError::EspError)?;
        let disk = Arc::new(Mutex::new(disk));

        let access_point: AccessPoint = AccessPoint::new(peripherals.modem)
            .map_err(ScheduleSystemError::EspError)?;
        let access_point = Arc::new(Mutex::new(access_point));

        let display: Display = Display::new(i2c_bus_manager.acquire_i2c())
            .map_err(ScheduleSystemError::DisplayError)?;
        let display = Arc::new(Mutex::new(display));

        let clock: Clock<AlarmId> = Clock::new(
            i2c_bus_manager.acquire_i2c(),
            |result| {
                println!("Synchronizing...")
            })
            .map_err(ScheduleSystemError::ClockError)?;
        let clock = Arc::new(Mutex::new(clock));

        let alarm_output: AlarmOutput = (
            peripherals.pins.gpio2,
            peripherals.pins.gpio4,
        );
        let alarm_output = Arc::new(Mutex::new(alarm_output));

        Ok(Self {
            access_point,
            clock,
            disk,
            display,
            alarm_output
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
            .lock()
            .map_err(|_| ScheduleSystemError::MutexLockError)?
            .get_datetime()
            .map_err(ScheduleSystemError::ClockError)
    }

    pub fn set_time(&self, datetime: DateTime<Utc>) -> ScheduleSystemResult<()> {
        self.clock
            .lock()
            .map_err(|_| ScheduleSystemError::MutexLockError)?
            .set_datetime(datetime)
            .map_err(ScheduleSystemError::ClockError)
    }


    pub fn get_alarm(&self, alarm_id: &AlarmId) -> ScheduleSystemResult<Alarm> {
        self.clock
            .lock()
            .map_err(|_| ScheduleSystemError::MutexLockError)?
            .get_alarm(alarm_id)
            .map_err(ScheduleSystemError::ClockError)
    }

    pub fn get_alarms(&self) -> ScheduleSystemResult<HashMap<AlarmId, Alarm>> {
        self.clock
            .lock()
            .map_err(|_| ScheduleSystemError::MutexLockError)?
            .get_alarms()
            .map_err(ScheduleSystemError::ClockError)
    }

    pub fn get_alarms_by_output_index(&self, output_index: u8) -> ScheduleSystemResult<HashMap<AlarmId, Alarm>> {
        let alarms = self.clock
            .lock()
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
}
