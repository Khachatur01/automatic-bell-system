use access_point::access_point::AccessPoint;
use clock::clock::Clock;
use disk::disk::Disk;
use disk::path::Path;
use display::display::Display;
use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::gpio::{Gpio25, Input, PinDriver};
use esp_idf_svc::hal::i2c::{I2cConfig, I2cDriver};
use esp_idf_svc::hal::prelude::Peripherals;
use esp_idf_svc::hal::spi::config::DriverConfig;
use esp_idf_svc::hal::spi::SpiDriver;
use http_server::http_server::HttpServer;
use shared_bus::BusManagerStd;
use clock::synchronize_by::SynchronizeBy;

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
    let mut clock: Clock = Clock::new(i2c_bus_manager.acquire_i2c(), SynchronizeBy::<Gpio25>::Delay { seconds: 1 * 60 * 60 }).unwrap();

    // clock.subscribe_alarm_interruption(|| {
    //
    // }).unwrap();
    /* Clock init */

    /* Display init */

    let mut display: Display = Display::new(i2c_bus_manager.acquire_i2c()).unwrap();
    /* Display init */

    /* HTTP server init */
    let http_server: HttpServer = HttpServer::new().unwrap();
    /* HTTP server init */

    /* SD Card init */
    let mut sd_card: Disk = Disk::new(spi_driver, cs).unwrap();

    let path = Path::try_from(String::from("/test.txt")).unwrap();

    match sd_card.read_from_file(&path) {
        Ok(buffer) => {
            let content = String::from_utf8(buffer).unwrap();

            println!("content: |{content}|");
        }
        Err(_) => {}
    }
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

    loop {
        let datetime = clock.datetime().unwrap();

        println!("{}", datetime);

        display.display_information(datetime, datetime).unwrap();
        // clock.enable_interrupt().unwrap();
        FreeRtos::delay_ms(1000u32);
    }
}
