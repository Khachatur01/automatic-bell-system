use access_point::access_point::AccessPoint;
use ds3231::ds3231::DS3231;
use http_server::http_server::HttpServer;
use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::prelude::Peripherals;
use std::sync::atomic::{AtomicBool, AtomicU8};

static ACCESS_POINT_STATE: AtomicBool = AtomicBool::new(false);
static FLAG: AtomicU8 = AtomicU8::new(1);

fn main() {
    /* It is necessary to call this function once. Otherwise, some patches to the runtime */
    /* implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71 */
    esp_idf_svc::sys::link_patches();

    /* Bind the log crate to the ESP Logging facilities */
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals: Peripherals = Peripherals::take().unwrap();

    let mut access_point: AccessPoint = AccessPoint::new(peripherals.modem).unwrap();
    access_point.start().unwrap();

    let mut ds3231: DS3231 = DS3231::new(
        peripherals.i2c1,
        peripherals.pins.gpio21, /* sda */
        peripherals.pins.gpio22 /* scl */
    ).unwrap();

    let mut server: HttpServer = HttpServer::new().unwrap();

    loop {
        // if let Ok(datetime) = ds3231.get_date_time() {
        //     let DateTime {
        //         seconds,
        //         minutes,
        //         hours,
        //         year,
        //         month,
        //         day,
        //     } = datetime;
        // 
        //     println!("{hours}:{minutes}:{seconds} {day}/{month}/{year}");
        // }
        FreeRtos::delay_ms(1000u32);
    }
}