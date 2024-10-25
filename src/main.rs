use esp_idf_svc::{
    hal::prelude::Peripherals
};
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::delay::{FreeRtos, BLOCK};
use esp_idf_svc::hal::gpio::{Gpio21, Gpio22};
use esp_idf_svc::hal::i2c::{I2cConfig, I2cDriver, I2C1};
use esp_idf_svc::hal::units::FromValueType;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::wifi::{AccessPointConfiguration, Configuration, EspWifi};
use nobcd::BcdNumber;
use ds3231::date_time::DateTime;
use ds3231::ds3231::DS3231;

fn main() {
    /* It is necessary to call this function once. Otherwise, some patches to the runtime */
    /* implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71 */
    esp_idf_svc::sys::link_patches();

    /* Bind the log crate to the ESP Logging facilities */
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("Hello, world!");

    let peripherals: Peripherals = Peripherals::take().unwrap();
    let sys_loop = EspSystemEventLoop::take().unwrap();
    let nvs = EspDefaultNvsPartition::take().unwrap();
    let mut wifi: EspWifi = EspWifi::new(peripherals.modem, sys_loop, Some(nvs)).unwrap();
    wifi.set_configuration(&Configuration::AccessPoint(AccessPointConfiguration::default())).unwrap();
    wifi.start().unwrap();


    let i2c: I2C1 = peripherals.i2c1;
    let sda: Gpio21 = peripherals.pins.gpio21;
    let scl: Gpio22 = peripherals.pins.gpio22;

    let config = I2cConfig::new().baudrate(100.kHz().into());
    let ds3231_driver: I2cDriver = I2cDriver::new(i2c, sda, scl, &I2cConfig::default()).unwrap();

    let mut ds3231: DS3231 = DS3231::new(ds3231_driver);

    loop {
        if let Ok(datetime) = ds3231.get_date_time() {
            let DateTime {
                seconds,
                minutes,
                hours,
                year,
                month,
                day,
            } = datetime;

            println!("{hours}:{minutes}:{seconds} {day}/{month}/{year}");
        }

        FreeRtos::delay_ms(1000u32);
    }
}
