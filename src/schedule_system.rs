pub mod alarm_id;
mod error;

use crate::schedule_system::alarm_id::AlarmId;
use crate::schedule_system::error::ScheduleSystemError;
use access_point::access_point::AccessPoint;
use chrono::{DateTime, Utc};
use clock::clock::Clock;
use disk::disk::Disk;
use display::display::Display;
use esp_idf_svc::hal::gpio::OutputPin;
use esp_idf_svc::hal::i2c::I2cDriver;
use esp_idf_svc::hal::modem;
use esp_idf_svc::hal::peripheral::Peripheral;
use esp_idf_svc::hal::spi::SpiDriver;
use interface::clock::{ReadClock, WriteClock};
use interface::disk::{ReadDisk, WriteDisk};
use interface::Path;
use shared_bus::BusManagerStd;
use std::sync::{Arc, Mutex};

type ScheduleSystemResult<Ok> = Result<Ok, ScheduleSystemError>;

pub struct ScheduleSystem {
    access_point: Arc<Mutex<AccessPoint<'static>>>,
    clock: Arc<Mutex<Clock<AlarmId>>>,
    disk: Arc<Mutex<Disk<'static>>>,
    display: Arc<Mutex<Display<'static>>>,
}

impl ScheduleSystem {
    pub fn new<CS: Peripheral<P = impl OutputPin> + 'static>(
        i2c_bus_manager: &'static BusManagerStd<I2cDriver<'static>>,
        spi_driver: SpiDriver<'static>, cs: CS,
        modem: modem::Modem
    ) -> Result<Self, ScheduleSystemError> {

        let disk: Disk = Disk::new(spi_driver, cs)
            .map_err(ScheduleSystemError::EspError)?;
        let disk = Arc::new(Mutex::new(disk));

        let access_point: AccessPoint = AccessPoint::new(modem)
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

        Ok(Self {
            access_point,
            clock,
            disk,
            display,
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
}
