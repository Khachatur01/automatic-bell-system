use crate::rest_interface::clock::model::ClockDTO;
use crate::schedule_system::ScheduleSystem;
use chrono::{DateTime, Utc};
use esp_idf_svc::http::server::{EspHttpConnection, Request};
use esp_idf_svc::http::Method;
use esp_idf_svc::io::EspIOError;
use esp_idf_svc::sys::EspError;
use http_server::http_request::HttpRequest;
use http_server::http_server::HttpServer;
use std::sync::Arc;

pub fn serve(http_server: &mut HttpServer, schedule_system: Arc<ScheduleSystem>) -> Result<(), EspError> {
    let schedule_system_clone: Arc<ScheduleSystem> = Arc::clone(&schedule_system);
    http_server.add_handler(
        "/api/v1/clock", Method::Get, 
        move |request| get_clock(request, &schedule_system_clone)
    )?;

    let schedule_system_clone: Arc<ScheduleSystem> = Arc::clone(&schedule_system);
    http_server.add_handler(
        "/api/v1/clock", Method::Put, 
        move |request| set_clock(request, &schedule_system_clone)
    )?;

    Ok(())
}

fn get_clock(request: Request<&mut EspHttpConnection>, schedule_system: &Arc<ScheduleSystem>) -> Result<(), EspIOError> {
    let timestamp_millis: i64 = schedule_system.get_time().unwrap().timestamp_millis();
    let clock_dto = ClockDTO {
        timestamp_millis,
    };

    match serde_json::to_string(&clock_dto) {
        Ok(clock_json) => request.ok(clock_json.as_bytes()),
        Err(error) => request.internal_server_error(error.to_string().as_bytes())
    }
}

fn set_clock(mut request: Request<&mut EspHttpConnection>, schedule_system: &Arc<ScheduleSystem>) -> Result<(), EspIOError> {
    let data: Vec<u8> = request.read_all()?;
    let clock_json: String = String::from_utf8_lossy(&data).to_string();

    let clock: ClockDTO = match serde_json::from_str::<ClockDTO>(&clock_json) {
        Ok(clock) => clock,
        Err(error) => return request.bad_request(error.to_string().as_bytes())
    };

    let datetime: DateTime<Utc> = match DateTime::<Utc>::from_timestamp_millis(clock.timestamp_millis) {
        Some(datetime) => datetime,
        None => return request.bad_request(format!("Can't convert timestamp {} to datetime.", clock.timestamp_millis).as_bytes())
    };

    match schedule_system.set_time(datetime) {
        Ok(_) => request.ok("Time synchronized".as_bytes()),
        Err(error) => request.bad_request(error.to_string().as_bytes())
    }
}
