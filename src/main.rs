use access_point::access_point::AccessPoint;
use clock::clock::{Clock};
use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::i2c::{I2cConfig, I2cDriver};
use esp_idf_svc::hal::prelude::Peripherals;
use esp_idf_svc::hal::spi::config::{DriverConfig, Duplex};
use esp_idf_svc::hal::spi::{SpiConfig, SpiDeviceDriver, SpiDriver};
use http_server::http_server::HttpServer;
use sd_card::path::Path;
use sd_card::sd_card::SDCard;

fn main() {
    /* It is necessary to call this function once. Otherwise, some patches to the runtime */
    /* implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71 */
    esp_idf_svc::sys::link_patches();

    /* Bind the log crate to the ESP Logging facilities */
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals: Peripherals = Peripherals::take().unwrap();

    let mut access_point: AccessPoint = AccessPoint::new(peripherals.modem).unwrap();
    access_point.start().unwrap();

    /* Clock init */
    let i2c = peripherals.i2c1;
    let sda = peripherals.pins.gpio5;
    let scl = peripherals.pins.gpio18;

    let i2c_config = I2cConfig::default();
    let i2c_driver: I2cDriver = I2cDriver::new(i2c, sda, scl, &i2c_config).unwrap();

    let mut clock = Clock::new(i2c_driver, peripherals.pins.gpio13).unwrap();

    clock.interruption_subscribe(|| {
        
    }).unwrap();
    /* Clock init */

    /* HTTP server init */
    let http_server: HttpServer = HttpServer::new().unwrap();
    /* HTTP server init */

    /* SD Card init */
    let spi = peripherals.spi2;
    let scl = peripherals.pins.gpio19;
    let sdo = peripherals.pins.gpio23;
    let sdi = peripherals.pins.gpio22;
    let cs = peripherals.pins.gpio21;

    let driver_config: DriverConfig = DriverConfig::default();
    let spi_driver: SpiDriver = SpiDriver::new(spi, scl, sdo, Some(sdi), &driver_config).unwrap();

    let mut spi_config = SpiConfig::new();
    spi_config.duplex = Duplex::Full;

    let spi_device_driver: SpiDeviceDriver<SpiDriver> = SpiDeviceDriver::new(spi_driver, Some(cs), &spi_config).unwrap();
    let mut sd_card: SDCard = SDCard::new(spi_device_driver).unwrap();
    /* SD Card init */

    let path = Path::try_from(String::from("/test.txt")).unwrap();

    let buffer = sd_card.read_from_file(&path).unwrap();
    let content = String::from_utf8(buffer).unwrap();

    println!("content: |{content}|");

    loop {
        clock.enable_interrupt().unwrap();
        FreeRtos::delay_ms(1000u32);
    }
}