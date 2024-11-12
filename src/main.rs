use access_point::access_point::AccessPoint;
use ds3231::ds3231::DS3231;
use http_server::http_server::HttpServer;
use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::prelude::Peripherals;
use std::sync::atomic::{AtomicBool, AtomicU8};
use esp_idf_svc::hal::gpio::Gpio21;
use ds3231::date_time::DateTime;
use sd_card::sd_card::SDCard;

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
        peripherals.pins.gpio5, /* sda */
        peripherals.pins.gpio18 /* scl */
    ).unwrap();

    let mut server: HttpServer = HttpServer::new().unwrap();

    let mut sd_card: SDCard = SDCard::new(
        peripherals.spi2,
        peripherals.pins.gpio19,
        peripherals.pins.gpio23,
        peripherals.pins.gpio22,
        peripherals.pins.gpio21,
    ).unwrap();
    
    let content = sd_card.read_file("test.txt").unwrap();
    println!("content: |{content}|");

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