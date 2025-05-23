use crate::model::clock::clock::ClockDTO;
use crate::schedule_system::ScheduleSystem;
use chrono::{DateTime, Utc};
use esp_idf_svc::http::server::{EspHttpConnection, Request};
use esp_idf_svc::http::Method;
use esp_idf_svc::io::EspIOError;
use esp_idf_svc::sys::EspError;
use http_request::RequestResult;
use http_server::http_request;
use http_server::http_request::{IntoResponse, ReadData};
use http_server::http_server::HttpServer;
use std::sync::Arc;
use crate::rest_interface::security::authenticate_request;

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

fn get_clock(request: Request<&mut EspHttpConnection>, schedule_system: &Arc<ScheduleSystem>) -> RequestResult<(), EspIOError> {
    let timestamp_millis: i64 =
        match schedule_system.get_time() {
            Ok(datetime) => datetime.timestamp_millis(),
            Err(error) => return request.bad_request(&error.to_string())
        };

    let clock_dto = ClockDTO { timestamp_millis };

    request.ok(&clock_dto)
}

fn set_clock(mut request: Request<&mut EspHttpConnection>, schedule_system: &Arc<ScheduleSystem>) -> RequestResult<(), EspIOError> {
    authenticate_request(&request)?;

    let clock: ClockDTO = request.body()?;

    let datetime: DateTime<Utc> =
        match DateTime::<Utc>::from_timestamp_millis(clock.timestamp_millis) {
            Some(datetime) => datetime,
            None => return request.bad_request(&format!("Can't convert timestamp {} to datetime.", clock.timestamp_millis))
        };

    match schedule_system.set_time(datetime) {
        Ok(_) => request.ok(&"Time synchronized"),
        Err(error) => request.bad_request(&error.to_string())
    }
}
