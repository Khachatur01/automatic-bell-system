mod auth;
mod clock;
mod alarm;

use crate::schedule_system::ScheduleSystem;
use http_server::http_request::HttpRequest;
use std::sync::{Arc, Mutex};
use esp_idf_svc::sys::EspError;
use http_server::http_server::HttpServer;

pub fn serve(http_server: &mut HttpServer, schedule_system: Arc<Mutex<ScheduleSystem>>) -> Result<(), EspError> {
    auth::serve(http_server, Arc::clone(&schedule_system))?;
    clock::serve(http_server, Arc::clone(&schedule_system))?;
    alarm::serve(http_server, Arc::clone(&schedule_system))?;

    Ok(())
}
