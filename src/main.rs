mod schedule_system;
mod rest_interface;
mod web_interface;
mod boxed_mutex;
mod types;

use crate::schedule_system::ScheduleSystem;
use esp_idf_svc::hal::prelude::Peripherals;
use http_server::http_server::HttpServer;
use interface::clock::ReadClock;
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

    let schedule_system: ScheduleSystem = ScheduleSystem::new(peripherals).unwrap();
    let schedule_system: Arc<ScheduleSystem> = Arc::new(schedule_system);

    schedule_system.enable_access_point().unwrap();

    let mut http_server: HttpServer = HttpServer::new().unwrap();

    rest_interface::serve(&mut http_server, Arc::clone(&schedule_system)).unwrap();
    web_interface::serve(&mut http_server, Arc::clone(&schedule_system)).unwrap();

    loop {
        thread::sleep(Duration::from_secs(10));
    }
}
