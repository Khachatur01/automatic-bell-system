use std::sync::{Arc, Mutex, RwLock};
use esp_idf_svc::http::Method;
use esp_idf_svc::http::server::{EspHttpConnection, Request};
use esp_idf_svc::io::EspIOError;
use esp_idf_svc::sys::EspError;
use http_server::http_request::HttpRequest;
use http_server::http_server::HttpServer;
use crate::schedule_system::ScheduleSystem;

pub fn serve(http_server: &mut HttpServer, schedule_system: Arc<ScheduleSystem>) -> Result<(), EspError> {
    let schedule_system_clone: Arc<ScheduleSystem> = Arc::clone(&schedule_system);

    http_server
        .add_handler("/api/v1/clock", Method::Get,
            move |request| get_clock(request, &schedule_system_clone)
        )?;

    let schedule_system_clone: Arc<ScheduleSystem> = Arc::clone(&schedule_system);
    http_server
        .add_handler("/api/v1/clock", Method::Post,
            move |request| set_clock(request, &schedule_system_clone)
        )?;

    Ok(())
}

fn get_clock(request: Request<&mut EspHttpConnection>, schedule_system: &Arc<ScheduleSystem>) -> Result<(), EspIOError> {
    let timestamp_millis: i64 = schedule_system.get_time().unwrap().timestamp_millis();

    request.ok(timestamp_millis.to_string().as_bytes())
}

fn set_clock(mut request: Request<&mut EspHttpConnection>, schedule_system: &Arc<ScheduleSystem>) -> Result<(), EspIOError> {
    let data = request.read_all()?;
    println!("{}", request.uri());
    println!("{:?}", data);

    let timestamp_millis: i64 = schedule_system.get_time().unwrap().timestamp_millis();

    request.ok(timestamp_millis.to_string().as_bytes())
}
