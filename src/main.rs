use access_point::access_point::AccessPoint;
use ds3231::date_time::DateTime;
use ds3231::ds3231::DS3231;
use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::gpio::{Gpio21, Gpio22};
use esp_idf_svc::hal::i2c::I2C1;
use esp_idf_svc::hal::prelude::Peripherals;

fn main() {
    /* It is necessary to call this function once. Otherwise, some patches to the runtime */
    /* implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71 */
    esp_idf_svc::sys::link_patches();

    /* Bind the log crate to the ESP Logging facilities */
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals: Peripherals = Peripherals::take().unwrap();

    let mut access_point: AccessPoint = AccessPoint::new(peripherals.modem).unwrap();
    access_point.start().unwrap();

    /* RTC - Real Time Clock. DS3231 is a Real Time Clock */
    let rtc_i2c: I2C1 = peripherals.i2c1;
    let rtc_sda: Gpio21 = peripherals.pins.gpio21;
    let rtc_scl: Gpio22 = peripherals.pins.gpio22;
    let mut ds3231: DS3231 = DS3231::new(rtc_i2c, rtc_sda, rtc_scl).unwrap();

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
