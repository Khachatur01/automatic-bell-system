mod schedule_system;
mod rest_interface;
mod web_interface;

use crate::schedule_system::ScheduleSystem;
use esp_idf_svc::hal::i2c::{I2cConfig, I2cDriver};
use esp_idf_svc::hal::prelude::Peripherals;
use esp_idf_svc::hal::spi::config::DriverConfig;
use esp_idf_svc::hal::spi::SpiDriver;
use http_server::http_server::HttpServer;
use interface::clock::ReadClock;
use shared_bus::BusManagerStd;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

fn main() {
    /* It is necessary to call this function once. Otherwise, some patches to the runtime */
    /* implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71 */
    esp_idf_svc::sys::link_patches();

    /* Bind the log crate to the ESP Logging facilities */
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals: Peripherals = Peripherals::take().unwrap();

    // /* Init I2c bus */
    // let i2c = peripherals.i2c0;
    // let sda = peripherals.pins.gpio22;
    // let scl = peripherals.pins.gpio23;
    // let i2c_config = I2cConfig::default();
    // let i2c_driver: I2cDriver = I2cDriver::new(i2c, sda, scl, &i2c_config).unwrap();
    // 
    // let i2c_bus_manager: &'static BusManagerStd<I2cDriver> = shared_bus::new_std!(I2cDriver = i2c_driver).unwrap();
    // /* Init I2c bus */
    // 
    // /* Init SPI driver */
    // let spi = peripherals.spi2;
    // let scl = peripherals.pins.gpio18;
    // let sdo = peripherals.pins.gpio19;
    // let sdi = peripherals.pins.gpio21;
    // let cs = peripherals.pins.gpio5;
    // 
    // let driver_config: DriverConfig = DriverConfig::default();
    // let spi_driver: SpiDriver = SpiDriver::new(spi, scl, sdo, Some(sdi), &driver_config).unwrap();
    // /* Init SDA driver */

    // clock.add_alarm("alarm1".parse().unwrap(), Alarm {
    //     year: AlarmMarcher::Ignore,
    //     month: AlarmMarcher::Ignore,
    //     month_day: AlarmMarcher::Ignore,
    //     week_day: AlarmMarcher::Ignore,
    //
    //     hour: AlarmMarcher::Ignore,
    //     minute: AlarmMarcher::Ignore,
    //     second: AlarmMarcher::Match((0..60).filter(|n| n % 2 != 0).collect()),
    // }, |datetime| {
    //     println!("Alarm 1 {datetime} ...")
    // }).expect("TODO: panic message");
    //
    // clock.add_alarm("alarm2".parse().unwrap(), Alarm {
    //     year: AlarmMarcher::Ignore,
    //     month: AlarmMarcher::Ignore,
    //     month_day: AlarmMarcher::Ignore,
    //     week_day: AlarmMarcher::Ignore,
    //
    //     hour: AlarmMarcher::Ignore,
    //     minute: AlarmMarcher::Ignore,
    //     second: AlarmMarcher::Match((0..60).collect()),
    // }, |datetime| {
    //     println!("Alarm 2 {datetime} ...")
    // }).expect("TODO: panic message");

    let schedule_system: ScheduleSystem = ScheduleSystem::new(peripherals).unwrap();
    // let schedule_system: ScheduleSystem = ScheduleSystem::new(
    //     i2c_bus_manager,
    //     spi_driver, cs, peripherals.modem
    // ).unwrap();

    let schedule_system: Arc<ScheduleSystem> = Arc::new(schedule_system);
    schedule_system.enable_access_point().unwrap();

    let mut http_server: HttpServer = HttpServer::new().unwrap();

    rest_interface::serve(&mut http_server, Arc::clone(&schedule_system)).unwrap();
    web_interface::serve(&mut http_server, Arc::clone(&schedule_system)).unwrap();

    loop {
        thread::sleep(Duration::from_secs(10));
    }
}
