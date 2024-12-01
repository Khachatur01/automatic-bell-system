mod web_interface;

use crate::web_interface::run_web_interface;
use access_point::access_point::AccessPoint;
use clock::clock::Clock;
use disk::disk::Disk;
use display::display::Display;
use esp_idf_svc::hal::i2c::{I2cConfig, I2cDriver};
use esp_idf_svc::hal::prelude::Peripherals;
use esp_idf_svc::hal::spi::config::DriverConfig;
use esp_idf_svc::hal::spi::SpiDriver;
use http_server::http_server::HttpServer;
use interface::clock::ReadClock;
use interface::Path;
use shared_bus::BusManagerStd;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

fn main() {
    /* It is necessary to call this function once. Otherwise, some patches to the runtime */
    /* implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71 */
    esp_idf_svc::sys::link_patches();

    /* Bind the log crate to the ESP Logging facilities */
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals: Peripherals = Peripherals::take().unwrap();

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

    let mut access_point: AccessPoint = AccessPoint::new(peripherals.modem).unwrap();
    access_point.start().unwrap();

    /* Clock init */
    let mut clock: Clock = Clock::new(
        i2c_bus_manager.acquire_i2c(),
        |result| {
            println!("Synchronizing...")
        }).unwrap();
    /* Clock init */

    /* Display init */
    let mut display: Display = Display::new(i2c_bus_manager.acquire_i2c()).unwrap();
    /* Display init */

    /* HTTP server init */
    let mut http_server: HttpServer = HttpServer::new().unwrap();
    /* HTTP server init */

    /* SD Card init */
    let disk: Disk = Disk::new(spi_driver, cs).unwrap();

    // let path = Path::try_from(String::from("/test.txt")).unwrap();

    // match disk.read_from_file(&path) {
    //     Ok(buffer) => {
    //         let content = String::from_utf8(buffer).unwrap();
    //
    //         println!("content: |{content}|");
    //     }
    //     Err(_) => {}
    // }
    /* SD Card init */

    // println!("1");
    // let sd = SdHost::new_with_spi(&Default::default(), SpiDevice::new(spi_driver, cs, None::<Gpio25>, None::<Gpio25>, None::<Gpio25>, None::<bool>));
    // println!("2");
    // Fat::mount(Default::default(), sd, "/sdcard").unwrap();
    // println!("3");
    // let paths = fs::read_dir("/sdcard").unwrap();
    // println!("4");
    //
    // for path in paths {
    //     println!("Name: {}", path.unwrap().path().display())
    // }
    // println!("5");

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

    run_web_interface(&mut http_server, Arc::new(Mutex::new(disk)));

    loop {
        // let datetime = clock.get_datetime().unwrap();
        //
        // println!("{}", datetime);

        // display.display_information(datetime, datetime).unwrap();
        thread::sleep(Duration::from_secs(1));
    }
}
