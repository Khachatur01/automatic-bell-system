mod auth;
mod clock;
mod alarm;

use crate::schedule_system::ScheduleSystem;
use esp_idf_svc::sys::EspError;
use http_server::http_request::HttpRequest;
use http_server::http_server::HttpServer;
use std::sync::Arc;

pub fn serve(http_server: &mut HttpServer, schedule_system: Arc<ScheduleSystem>) -> Result<(), EspError> {
    auth::controller::serve(http_server, Arc::clone(&schedule_system))?;
    clock::controller::serve(http_server, Arc::clone(&schedule_system))?;
    alarm::controller::serve(http_server, Arc::clone(&schedule_system))?;

    Ok(())
}
