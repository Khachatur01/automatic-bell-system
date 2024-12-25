use std::collections::HashSet;
use crate::schedule_system::ScheduleSystem;
use esp_idf_svc::http::server::{EspHttpConnection, Request};
use esp_idf_svc::http::Method;
use esp_idf_svc::io::EspIOError;
use esp_idf_svc::sys::EspError;
use http_server::http_request::HttpRequest;
use http_server::http_server::HttpServer;
use std::sync::Arc;
use chrono::{Month, Weekday};
use clock::alarm::{Alarm, AlarmMarcher};
use http_server::http_request;
use http_request::RequestResult;
use crate::rest_interface::alarm::model::alarm::AlarmDTO;
use crate::rest_interface::alarm::model::alarm_id::AlarmIdDTO;
use crate::schedule_system::alarm_id::AlarmId;

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
    let alarm_id: AlarmIdDTO = request.parameters()?;
    println!("alarm_id: {:?}", alarm_id);

    let alarm = Alarm {
        year: AlarmMarcher::Ignore,
        month: AlarmMarcher::DoNotMatch(HashSet::from([Month::August, Month::May])),
        month_day: AlarmMarcher::Ignore,
        week_day: AlarmMarcher::Match(HashSet::from([Weekday::Mon, Weekday::Fri])),
        hour: AlarmMarcher::Ignore,
        minute: AlarmMarcher::Match(HashSet::from([4, 7, 8, 9])),
        second: AlarmMarcher::DoNotMatch(HashSet::from([2, 1, 4])),
    };

    let alarm_dto: AlarmDTO = alarm.into();

    request.ok(&alarm_dto)
}

fn get_alarms(request: Request<&mut EspHttpConnection>, schedule_system: &Arc<ScheduleSystem>) -> RequestResult<(), EspIOError> {
    request.internal_server_error(&String::from(""))
}

fn add_alarm(mut request: Request<&mut EspHttpConnection>, schedule_system: &Arc<ScheduleSystem>) -> RequestResult<(), EspIOError> {
    let alarm_dto: AlarmDTO = request.read_all()?;
    let alarm: Alarm = alarm_dto.into();

    request.ok(&String::from(""))
}

fn delete_alarm(request: Request<&mut EspHttpConnection>, schedule_system: &Arc<ScheduleSystem>) -> RequestResult<(), EspIOError> {
    request.internal_server_error(&String::from(""))
}
