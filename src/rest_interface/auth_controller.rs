use crate::schedule_system::ScheduleSystem;
use esp_idf_svc::http::server::{EspHttpConnection, Request};
use esp_idf_svc::http::Method;
use esp_idf_svc::io::EspIOError;
use esp_idf_svc::sys::EspError;
use http_server::http_request::{IntoResponse, RequestResult};
use http_server::http_server::HttpServer;
use std::sync::Arc;

pub fn serve(http_server: &mut HttpServer, schedule_system: Arc<ScheduleSystem>) -> Result<(), EspError> {
    let schedule_system_clone: Arc<ScheduleSystem> = Arc::clone(&schedule_system);
    http_server.add_handler(
        "/api/v1/login", Method::Get,
        move |request| login(request, &schedule_system_clone)
    )?;

    Ok(())
}

fn login(request: Request<&mut EspHttpConnection>, schedule_system: &Arc<ScheduleSystem>) -> RequestResult<(), EspIOError> {
    request.ok(&"")
}