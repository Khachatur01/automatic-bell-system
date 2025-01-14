mod schedule_system;
mod rest_interface;
mod web_interface;
mod synchronizer;
mod model;
mod constant;
mod security;

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
    log::info!("Logger initialized.");
    log::info!("Starting up...");
    log::info!("Patches linked.");

    let peripherals: Peripherals = Peripherals::take().unwrap();
    log::info!("Peripherals are ready.");

    let schedule_system: ScheduleSystem = ScheduleSystem::new(peripherals).unwrap();
    log::info!("Schedule system is ready.");

    let schedule_system: Arc<ScheduleSystem> = Arc::new(schedule_system);

    schedule_system.enable_access_point().unwrap();
    log::info!("Access point enabled.");

    let mut http_server: HttpServer = HttpServer::new().unwrap();

    rest_interface::serve(&mut http_server, Arc::clone(&schedule_system)).unwrap();
    log::info!("Rest interface is ready.");

    web_interface::serve(&mut http_server, Arc::clone(&schedule_system)).unwrap();
    log::info!("WEB interface is ready.");

    loop {
        thread::sleep(Duration::from_secs(10));
    }
}
