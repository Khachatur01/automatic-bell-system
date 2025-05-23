pub mod alarm_id;
pub mod to_alarms_with_id;
mod error;

use crate::constant::{ACCESS_POINT_SSID, ALARMS_DIR, ALARM_MATCH_CHECK_INTERVAL_MS, RESET_BUTTON_PRESS_TIME_SECONDS, SYSTEM_DIR, WEB_UI_DIR};
use crate::model::alarm::alarm_with_id::AlarmWithIdDTO;
use crate::schedule_system::alarm_id::AlarmId;
use crate::schedule_system::error::ScheduleSystemError;
use crate::security::SecurityContext;
use crate::synchronizer::{BoxedMutex, BoxedRwLock, IntoBoxedMutex, IntoBoxedRwLock, IntoMutexOutputPin, MutexOutputPin};
use access_point::access_point::AccessPoint;
use chrono::{DateTime, Utc};
use clock::alarm::Alarm;
use clock::clock::Clock;
use disk::disk::Disk;
use display::display::Display;
use esp_idf_svc::hal::gpio::{AnyOutputPin, PinDriver, Pull};
use esp_idf_svc::hal::i2c::{I2cConfig, I2cDriver};
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::spi::config::DriverConfig;
use esp_idf_svc::hal::spi::SpiDriver;
use interface::clock::{ReadClock, WriteClock};
use interface::disk::path::directory_path::DirectoryPath;
use interface::disk::path::file_path::FilePath;
use interface::disk::{ReadDisk, WriteDisk};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use shared_bus::BusManagerStd;
use std::collections::HashMap;
use std::process::exit;
use std::thread;
use std::time::Duration;
use esp_idf_svc::systime::EspSystemTime;

type ScheduleSystemResult<Ok> = Result<Ok, ScheduleSystemError>;
type AlarmOutputs<'a> = Vec<MutexOutputPin<'a>>;


/* Wrap fields into box to prevent stack overflowing.*/
pub struct ScheduleSystem {
    access_point: BoxedMutex<AccessPoint<'static>>,
    /* Clock is RwLock, because it requires immutable reference for reading time. */
    clock: BoxedRwLock<Clock<AlarmId>>,
    disk: BoxedMutex<Disk<'static>>,
    alarm_output_indices: Vec<usize>,
}

impl ScheduleSystem {
    pub fn new(peripherals: Peripherals) -> Result<Self, ScheduleSystemError> {
        /* Init I2c bus */
        let i2c = peripherals.i2c0;
        let sda = peripherals.pins.gpio23;
        let scl = peripherals.pins.gpio22;
        let i2c_config = I2cConfig::default();
        let i2c_driver: I2cDriver = I2cDriver::new(i2c, sda, scl, &i2c_config).map_err(ScheduleSystemError::EspError)?;

        let i2c_bus_manager: &'static BusManagerStd<I2cDriver> = shared_bus::new_std!(I2cDriver = i2c_driver)
            .ok_or(ScheduleSystemError::I2cSharedBusError)?;
        log::info!("I2C driver initialized.");
        /* Init I2c bus */

        /* Init SPI driver */
        let spi = peripherals.spi2;
        let scl = peripherals.pins.gpio18;
        let sdo = peripherals.pins.gpio19;
        let sdi = peripherals.pins.gpio5;
        let cs = peripherals.pins.gpio21;

        let driver_config: DriverConfig = DriverConfig::default();
        let spi_driver: SpiDriver = SpiDriver::new(spi, scl, sdo, Some(sdi), &driver_config)
            .map_err(ScheduleSystemError::EspError)?;
        log::info!("SPI driver initialized.");
        /* Init SPI driver */

        /* Init reset button */
        let mut reset_button = PinDriver::input(peripherals.pins.gpio13)
            .map_err(ScheduleSystemError::EspError)?;
        reset_button.set_pull(Pull::Up)
            .map_err(ScheduleSystemError::EspError)?;

        thread::spawn(move || {
            return;
            let mut press_time: Option<u64> = None;

            loop {
                thread::sleep(Duration::from_secs(1));

                /* fix time when button pressed */
                if press_time.is_none() && reset_button.is_low() {
                    press_time = Some(EspSystemTime.now().as_secs())
                }

                /* drop press time when button released */
                if reset_button.is_high() {
                    press_time = None;
                }

                let Some(press_time_secs) = press_time else {
                    continue;
                };

                let seconds_passed: u64 = EspSystemTime.now().as_secs() - press_time_secs;
                if seconds_passed != RESET_BUTTON_PRESS_TIME_SECONDS {
                    continue;
                }

                press_time = None;

                log::info!("Resetting...");
                let Ok(security_context) = SecurityContext::get() else {
                    return;
                };
                log::info!("Got security context.");

                let _ = security_context.reset_access_point_password();
                let _ = security_context.reset_api_password();

                log::info!("Reset.");
                log::info!("Rebooting...");
                exit(0);
            }
        });
        /* Init reset button */

        /* display */
        let display: Display = Display::new(i2c_bus_manager.acquire_i2c())
            .map_err(ScheduleSystemError::DisplayError)?;
        let mut display: Box<Display> = Box::new(display);

        let _ = display.write_text("Booting...");
        log::info!("Display initialized.");

        let alarm_output_pins: Vec<MutexOutputPin> = vec![
            Into::<AnyOutputPin>::into(peripherals.pins.gpio14)
                .try_into_mutex_output_pin()
                .map_err(ScheduleSystemError::EspError)?,
            Into::<AnyOutputPin>::into(peripherals.pins.gpio4)
                .try_into_mutex_output_pin()
                .map_err(ScheduleSystemError::EspError)?,
        ];
        let output_pins_count: usize = alarm_output_pins.len();
        let alarm_output_indices: Vec<usize> = (0..output_pins_count).collect();

        log::info!("Alarm outputs initialized. Total count is {output_pins_count}.");

        /* clock */
        let clock: BoxedRwLock<Clock<AlarmId>> = Clock::new(
            i2c_bus_manager.acquire_i2c(),
            |_| log::info!("Synchronizing..."),
            move |alarm_id: &AlarmId, alarm: &Alarm, date_time| ScheduleSystem::on_alarm(alarm_id, alarm, date_time, &alarm_output_pins),
            ALARM_MATCH_CHECK_INTERVAL_MS
        )
        .map_err(ScheduleSystemError::ClockError)?
        .into_boxed_rwlock();
        log::info!("Clock initialized.");

        /* access point */
        let security_context: &SecurityContext = SecurityContext::get()
            .map_err(ScheduleSystemError::EspError)?;
        let access_point_password: String = security_context
            .get_access_point_password()
            .map_err(ScheduleSystemError::EspError)?;
        log::info!("Access point password - '{access_point_password}'.");

        let access_point: BoxedMutex<AccessPoint> = AccessPoint::new(peripherals.modem, ACCESS_POINT_SSID, access_point_password.as_str())
            .map_err(ScheduleSystemError::EspError)?
            .into_boxed_mutex();
        log::info!("Access point initialized.");

        /* disk */
        let disk: BoxedMutex<Disk> = Disk::new(spi_driver, cs)
            .map_err(ScheduleSystemError::EspError)?
            .into_boxed_mutex();
        log::info!("Disk initialized.");

        let this: Self = Self {
            access_point,
            clock,
            disk,
            alarm_output_indices
        };

        this.init_filesystem(output_pins_count)?;
        log::info!("File system initialized.");
        
        this.synchronize_alarms_from_disk()?;
        log::info!("Alarms are synchronized from disk.");

        thread::spawn(move || loop {
            let seconds: u64 = EspSystemTime.now().as_secs();
            let Some(datetime) = DateTime::from_timestamp(seconds as i64, 0) else {
                continue;
            };

            let datetime: String = datetime
                .naive_utc()
                .format("%d/%m/%Y\n%H:%M:%S")
                .to_string();

            let _ = display.write_text(datetime.as_str());

            /* update time every second */
            thread::sleep(Duration::from_secs(1));
        });

        Ok(this)
    }


    fn on_alarm(alarm_id: &AlarmId, alarm: &Alarm, date_time: &DateTime<Utc>, alarm_output_pins: &AlarmOutputs) {
        let output_index: usize = alarm_id.output_index as usize;

        let Some(output_pin) = alarm_output_pins.get(output_index) else {
            log::warn!("Alarm output index {output_index} out of bounds. Skipping alarm...");
            return;
        };

        let Ok(mut output_pin_driver) = output_pin.try_lock() else {
            log::warn!("Can't lock output GPIO pin {output_index}. Skipping alarm...");
            return;
        };

        // log::info!(
        //     "Alarming: Output - {}, Id - {}, time - {}, impulse length - {}ms.",
        //     alarm_id.output_index, alarm_id.identifier, date_time, alarm.impulse_length_millis
        // );

        let _ = output_pin_driver.set_high();
        thread::sleep(Duration::from_millis(alarm.impulse_length_millis));

        // log::info!(
        //     "Stoping alarm: Output - {}, Id - {}, time - {}.",
        //     alarm_id.output_index, alarm_id.identifier, date_time
        // );
        let _ = output_pin_driver.set_low();
    }
}

impl ScheduleSystem {
    pub fn alarm_output_indices(&self) -> &Vec<usize> {
        &self.alarm_output_indices
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
        println!("Locking disk for read...");
        self.disk
            .lock()
            .map_err(|_| ScheduleSystemError::MutexLockError)?
            .read_from_file(path)
            .map_err(ScheduleSystemError::DiskError)
    }

    pub fn read_from_file_bytes<OnRead: FnMut(&[u8], usize) -> Result<(), ()>>(&self, path: &FilePath, bytes: usize, on_read: OnRead) -> ScheduleSystemResult<()> {
        println!("Locking disk for read...");
        self.disk
            .lock()
            .map_err(|_| ScheduleSystemError::MutexLockError)?
            .read_from_file_bytes(path, bytes, on_read)
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

        clock
            .add_alarm(alarm_id.clone(), alarm.clone())
            .map_err(ScheduleSystemError::ClockError)?;

        self.write_alarm_to_disk(alarm_id, alarm)?;

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
        log::info!("Disk lock acquired!");

        let path: DirectoryPath = [SYSTEM_DIR, WEB_UI_DIR].as_slice().into();

        disk.make_dir(&path)
            .map_err(ScheduleSystemError::DiskError)?;
        log::info!("Created dir '{path}'.");

        let path: DirectoryPath = [SYSTEM_DIR, ALARMS_DIR].as_slice().into();

        disk.make_dir(&path)
            .map_err(ScheduleSystemError::DiskError)?;
        log::info!("Created dir '{path}'.");

        for output_index in 0..outputs_count {
            let path: DirectoryPath = [
                SYSTEM_DIR,
                ALARMS_DIR,
                output_index.to_string().as_str()
            ].as_slice().into();

            disk.make_dir(&path)
                .map_err(ScheduleSystemError::DiskError)?;
        }

        Ok(())
    }

    /**
     * Read all alarms from disk and add to clock.
     */
    fn synchronize_alarms_from_disk(&self) -> ScheduleSystemResult<()> {
        fn get_output_dir_names(disk: &mut Disk) -> ScheduleSystemResult<Vec<String>> {
            let path: DirectoryPath =
                [
                    SYSTEM_DIR,
                    ALARMS_DIR,
                ].as_slice().into();
            disk
                .list_dir(&path)
                .map_err(ScheduleSystemError::DiskError)
        }

        fn get_alarm_file_names(disk: &mut Disk, output_dir_name: &str) -> ScheduleSystemResult<Vec<String>> {
            let path: DirectoryPath =
                [
                    SYSTEM_DIR,
                    ALARMS_DIR,
                    output_dir_name
                ].as_slice().into();
            disk
                .list_files(&path)
                .map_err(ScheduleSystemError::DiskError)
        }

        fn read_alarm_file(disk: &mut Disk, output_dir_name: &str, alarm_file_name: &str) -> ScheduleSystemResult<String> {
            let file_path: FilePath = (
                [
                    SYSTEM_DIR,
                    ALARMS_DIR,
                    &output_dir_name
                ].as_slice(),
                alarm_file_name
            ).into();

            let content: Vec<u8> = disk.read_from_file(&file_path)
                .map_err(ScheduleSystemError::DiskError)?;

            let alarm: String = String::from_utf8_lossy(&content).to_string();

            Ok(alarm)
        }


        let mut disk = self
            .disk
            .lock()
            .map_err(|_| ScheduleSystemError::MutexLockError)?;
        let mut clock = self
            .clock
            .write()
            .map_err(|_| ScheduleSystemError::MutexLockError)?;

        let output_dir_names: Vec<String> = get_output_dir_names(&mut disk)?;

        for output_dir_name in output_dir_names {
            let alarm_file_names: Vec<String> = get_alarm_file_names(&mut disk, &output_dir_name)?;

            for alarm_file_name in alarm_file_names {
                let alarm_str: String = read_alarm_file(&mut disk, &output_dir_name, &alarm_file_name)?;

                let alarm_with_id: AlarmWithIdDTO =
                    match serde_json::from_str(&alarm_str) {
                        Ok(alarm_with_id) => alarm_with_id,
                        Err(_) => continue
                    };

                clock
                    .add_alarm(alarm_with_id.id.into(), alarm_with_id.alarm.into())
                    .map_err(ScheduleSystemError::ClockError)?;
            }
        }

        Ok(())
    }

    fn write_alarm_to_disk(&self, alarm_id: AlarmId, alarm: Alarm) -> ScheduleSystemResult<()> {
        let mut disk = self
            .disk
            .lock()
            .map_err(|_| ScheduleSystemError::MutexLockError)?;

        let alarm_with_id: AlarmWithIdDTO = (alarm_id, alarm).into();

        let output_index: u8 = alarm_with_id.id.output_index;
        let identifier: &str = alarm_with_id.id.identifier.as_str().into();

        let file_path: FilePath = (
            [
                SYSTEM_DIR,
                ALARMS_DIR,
                output_index.to_string().as_str()
            ].as_slice(),
            identifier
        ).into();

        let alarm_str: String = serde_json::to_string(&alarm_with_id).unwrap_or_default();

        disk.write_to_file(&file_path, alarm_str.as_bytes())
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
                    output_index.to_string().as_str()
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
                output_index.to_string().as_str()
            ].as_slice().into();

        disk.clear_dir(&path)
            .map_err(ScheduleSystemError::DiskError)?;

        Ok(())
    }
}
