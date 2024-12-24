use crate::schedule_system::ScheduleSystem;
use esp_idf_svc::http::server::{EspHttpConnection, Request};
use esp_idf_svc::http::Method;
use esp_idf_svc::io::EspIOError;
use esp_idf_svc::sys::EspError;
use http_server::http_request::HttpRequest;
use http_server::http_server::HttpServer;
use std::sync::Arc;
use http_server::http_request;
use http_request::RequestResult;

pub fn serve(http_server: &mut HttpServer, schedule_system: Arc<ScheduleSystem>) -> Result<(), EspError> {
    let schedule_system_clone: Arc<ScheduleSystem> = Arc::clone(&schedule_system);
    http_server.add_handler(
        "/api/v1/alarm", Method::Get,
        move |request| get_alarm(request, &schedule_system_clone)
    )?;

    let schedule_system_clone: Arc<ScheduleSystem> = Arc::clone(&schedule_system);
    http_server.add_handler(
        "/api/v1/alarms", Method::Get,
        move |request| get_alarms(request, &schedule_system_clone)
    )?;

    let schedule_system_clone: Arc<ScheduleSystem> = Arc::clone(&schedule_system);
    http_server.add_handler(
        "/api/v1/alarm", Method::Post,
        move |request| add_alarm(request, &schedule_system_clone)
    )?;

    let schedule_system_clone: Arc<ScheduleSystem> = Arc::clone(&schedule_system);
    http_server.add_handler(
        "/api/v1/alarm", Method::Delete,
        move |request| delete_alarm(request, &schedule_system_clone)
    )?;

    Ok(())
}

fn get_alarm(request: Request<&mut EspHttpConnection>, schedule_system: &Arc<ScheduleSystem>) -> RequestResult<(), EspIOError> {
    request.internal_server_error(&String::from(""))
}

fn get_alarms(request: Request<&mut EspHttpConnection>, schedule_system: &Arc<ScheduleSystem>) -> RequestResult<(), EspIOError> {
    request.internal_server_error(&String::from(""))
}

fn add_alarm(request: Request<&mut EspHttpConnection>, schedule_system: &Arc<ScheduleSystem>) -> RequestResult<(), EspIOError> {
    request.internal_server_error(&String::from(""))
}

fn delete_alarm(request: Request<&mut EspHttpConnection>, schedule_system: &Arc<ScheduleSystem>) -> RequestResult<(), EspIOError> {
    request.internal_server_error(&String::from(""))
}
