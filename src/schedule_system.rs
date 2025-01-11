pub mod alarm_id;
pub mod to_alarms_with_id;
mod error;

use crate::model::alarm::alarm::AlarmDTO;
use crate::schedule_system::alarm_id::AlarmId;
use crate::schedule_system::error::ScheduleSystemError;
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
use http_server::to_response_data::ToResponseData;
use interface::clock::{ReadClock, WriteClock};
use interface::disk::path::directory_path::DirectoryPath;
use interface::disk::path::file_path::FilePath;
use interface::disk::{ReadDisk, WriteDisk};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use shared_bus::BusManagerStd;
use std::collections::HashMap;
use std::ops::Deref;
use std::thread;
use std::time::Duration;
use crate::constant::{ALARMS_DIR, OUTPUT_DIR, SYSTEM_DIR, WEB_UI_DIR};

type ScheduleSystemResult<Ok> = Result<Ok, ScheduleSystemError>;
type AlarmOutputs<'a> = Vec<MutexOutputPin<'a>>;


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

        let i2c_bus_manager: &'static BusManagerStd<I2cDriver> = shared_bus::new_std!(I2cDriver = i2c_driver)
            .ok_or(ScheduleSystemError::I2cSharedBusError)?;
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
        let output_pins_count: usize = alarm_output_pins.len();

        let clock: BoxedRwLock<Clock<AlarmId>> = Clock::new(
            i2c_bus_manager.acquire_i2c(),
            |result| println!("Synchronizing..."),
            move |alarm_id: &AlarmId, alarm: &Alarm, date_time| ScheduleSystem::on_alarm(alarm_id, alarm, date_time, &alarm_output_pins))
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

        this.init_filesystem(output_pins_count)?;
        this.synchronize_alarms_from_disk()?;

        Ok(this)
    }


    fn on_alarm(alarm_id: &AlarmId, alarm: &Alarm, date_time: &DateTime<Utc>, alarm_output_pins: &Vec<MutexOutputPin>) {
        /* fixme: remove print statement */
        println!("Alarming {} {} {}", alarm_id.output_index, alarm_id.identifier, date_time);

        let output_index: usize = alarm_id.output_index as usize;

        let Some(output_pin) = alarm_output_pins.get(output_index) else {
            return;
        };

        let Ok(mut output_pin_driver) = output_pin.try_lock() else {
            return;
        };

        let _ = output_pin_driver.set_high();
        thread::sleep(Duration::from_millis(alarm.impulse_length_millis));
        let _ = output_pin_driver.set_low();
    }
}

/* access point */
impl ScheduleSystem {
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
}

/* disk */
impl ScheduleSystem {
    pub fn read_from_file(&self, path: &FilePath) -> ScheduleSystemResult<Vec<u8>> {
        self.disk
            .lock()
            .map_err(|_| ScheduleSystemError::MutexLockError)?
            .read_from_file(path)
            .map_err(ScheduleSystemError::DiskError)
    }

    pub fn write_to_file(&self, path: &FilePath, data_buffer: &mut [u8]) -> ScheduleSystemResult<()> {
        self.disk
            .lock()
            .map_err(|_| ScheduleSystemError::MutexLockError)?
            .write_to_file(path, data_buffer)
            .map_err(ScheduleSystemError::DiskError)
    }
}

/* clock */
impl ScheduleSystem {
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

    pub fn get_alarms_by_output_index(&self, output_index: u8) -> ScheduleSystemResult<HashMap<AlarmId, Alarm>> {
        let alarms = self
            .clock
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

    pub fn add_alarm(&self, output_index: u8, alarm: Alarm) -> ScheduleSystemResult<()> {
        let mut clock = self
            .clock
            .write()
            .map_err(|_| ScheduleSystemError::MutexLockError)?;

        /* generate random identifier until unique one found */
        let alarm_id: AlarmId = loop {
            let identifier: String = thread_rng()
                .sample_iter(&Alphanumeric)
                .take(8)
                .map(char::from)
                .collect();

            let alarm_id: AlarmId = AlarmId {
                output_index,
                identifier,
            };

            let is_alarm_id_unique: bool = clock
                .is_alarm_id_unique(&alarm_id)
                .map_err(ScheduleSystemError::ClockError)?;

            if is_alarm_id_unique {
                break alarm_id;
            }
        };

        self.write_alarm_to_disk(alarm_id.clone(), alarm.clone())?;

        clock
            .add_alarm(alarm_id, alarm)
            .map_err(ScheduleSystemError::ClockError)?;

        Ok(())
    }

    pub fn remove_alarm(&self, alarm_id: &AlarmId) -> ScheduleSystemResult<()> {
        self.clock
            .write()
            .map_err(|_| ScheduleSystemError::MutexLockError)?
            .remove_alarm(alarm_id)
            .map_err(ScheduleSystemError::ClockError)?;

        self.remove_alarm_from_disk_by_id(&alarm_id)
    }

    pub fn remove_alarms_by_output_index(&self, output_index: u8) -> ScheduleSystemResult<()> {
        self.clock
            .write()
            .map_err(|_| ScheduleSystemError::MutexLockError)?
            .remove_alarm_if(|alarm_id: &AlarmId| alarm_id.output_index == output_index)
            .map_err(ScheduleSystemError::ClockError)?;

        self.remove_alarm_from_disk_by_output_index(output_index)
    }
}

/* disk synchronization */
impl ScheduleSystem {
    fn init_filesystem(&self, outputs_count: usize) -> ScheduleSystemResult<()> {
        let mut disk = self
            .disk
            .lock()
            .map_err(|_| ScheduleSystemError::MutexLockError)?;

        let path: DirectoryPath = [SYSTEM_DIR, WEB_UI_DIR].as_slice().into();

        disk.make_dir(&path)
            .map_err(ScheduleSystemError::DiskError)?;

        let path: DirectoryPath = [SYSTEM_DIR, ALARMS_DIR].as_slice().into();

        disk.make_dir(&path)
            .map_err(ScheduleSystemError::DiskError)?;

        for output_index in 0..outputs_count {
            let path: DirectoryPath = [
                SYSTEM_DIR,
                ALARMS_DIR,
                format!("{OUTPUT_DIR}_{output_index}").as_str()
            ].as_slice().into();

            disk.make_dir(&path)
                .map_err(ScheduleSystemError::DiskError)?;
        }

        Ok(())
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

    fn write_alarm_to_disk(&self, alarm_id: AlarmId, alarm: Alarm) -> ScheduleSystemResult<()> {
        let mut disk = self
            .disk
            .lock()
            .map_err(|_| ScheduleSystemError::MutexLockError)?;

        let output_index: u8 = alarm_id.output_index;
        let identifier: &str = alarm_id.identifier.as_str().into();

        let alarm_dto: AlarmDTO = alarm.into();

        let file_path: FilePath = (
            [
                SYSTEM_DIR,
                ALARMS_DIR,
                format!("{OUTPUT_DIR}_{output_index}").as_str()
            ].as_slice(),
            identifier
        ).into();

        disk.write_to_file(&file_path, alarm_dto.to_response_data().as_bytes())
            .map_err(ScheduleSystemError::DiskError)?;

        Ok(())
    }

    fn remove_alarm_from_disk_by_id(&self, AlarmId { output_index, identifier }: &AlarmId) -> ScheduleSystemResult<()> {
        let mut disk = self
            .disk
            .lock()
            .map_err(|_| ScheduleSystemError::MutexLockError)?;

        let path: FilePath =
            (
                [
                    SYSTEM_DIR,
                    ALARMS_DIR,
                    format!("{OUTPUT_DIR}_{output_index}").as_str(),
                ].as_slice(),
                identifier.as_str()
            ).into();

        disk.delete_file(&path)
            .map_err(ScheduleSystemError::DiskError)?;

        Ok(())
    }

    fn remove_alarm_from_disk_by_output_index(&self, output_index: u8) -> ScheduleSystemResult<()> {
        let mut disk = self
            .disk
            .lock()
            .map_err(|_| ScheduleSystemError::MutexLockError)?;

        let path: DirectoryPath =
            [
                SYSTEM_DIR,
                ALARMS_DIR,
                format!("{OUTPUT_DIR}_{output_index}").as_str()
            ].as_slice().into();

        disk.clear_dir(&path)
            .map_err(ScheduleSystemError::DiskError)?;

        Ok(())
    }
}
