use std::sync::{Arc, Mutex, RwLock};
use esp_idf_svc::sys::EspError;
use http_server::http_server::HttpServer;
use crate::schedule_system::ScheduleSystem;

pub fn serve(http_server: &mut HttpServer, schedule_system: Arc<RwLock<ScheduleSystem>>) -> Result<(), EspError> {
    Ok(())
}