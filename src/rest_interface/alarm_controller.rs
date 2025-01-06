use crate::schedule_system::alarm_id::AlarmId;
use crate::schedule_system::ScheduleSystem;
use clock::alarm::Alarm;
use esp_idf_svc::http::server::{EspHttpConnection, Request};
use esp_idf_svc::http::Method;
use esp_idf_svc::io::EspIOError;
use esp_idf_svc::sys::EspError;
use http_request::RequestResult;
use http_server::http_request;
use http_server::http_request::{HttpRequest, RequestError};
use http_server::http_server::HttpServer;
use std::sync::Arc;
use crate::model::alarm::alarm::AlarmDTO;
use crate::model::alarm::alarm_id::AlarmIdDTO;
use crate::model::alarm::alarm_with_id::AlarmWithIdDTO;
use crate::model::alarm::output_index::OutputIndexDTO;
use crate::schedule_system::to_alarms_with_id::ToAlarmsWithId;

pub fn serve(http_server: &mut HttpServer, schedule_system: Arc<ScheduleSystem>) -> Result<(), EspError> {
    let schedule_system_clone: Arc<ScheduleSystem> = Arc::clone(&schedule_system);
    http_server.add_handler(
        "/api/v1/alarm", Method::Get,
        move |request| get_alarm(request, &schedule_system_clone)
    )?;

    let schedule_system_clone: Arc<ScheduleSystem> = Arc::clone(&schedule_system);
    http_server.add_handler(
        "/api/v1/alarms", Method::Get,
        move |request| get_alarms_by_output_index(request, &schedule_system_clone)
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

    let schedule_system_clone: Arc<ScheduleSystem> = Arc::clone(&schedule_system);
    http_server.add_handler(
        "/api/v1/alarms", Method::Delete,
        move |request| delete_alarms_by_output_index(request, &schedule_system_clone)
    )?;

    Ok(())
}

fn get_alarm(request: Request<&mut EspHttpConnection>, schedule_system: &Arc<ScheduleSystem>) -> RequestResult<(), EspIOError> {
    let alarm_id: AlarmId = request.parameters::<AlarmIdDTO>()?.into();

    let alarm: Alarm =
        schedule_system
            .get_alarm(&alarm_id)
            .map_err(|error| RequestError::General(error.to_string()))?;

    let alarm_dto: AlarmDTO = alarm.into();

    request.ok(&alarm_dto)
}

fn get_alarms_by_output_index(request: Request<&mut EspHttpConnection>, schedule_system: &Arc<ScheduleSystem>) -> RequestResult<(), EspIOError> {
    let output_index: u8 = *request.parameters::<OutputIndexDTO>()?;

    let alarms_with_id_dto: Vec<AlarmWithIdDTO> =
        schedule_system
            .get_alarms_by_output_index(output_index)
            .map_err(|error| RequestError::General(error.to_string()))?
            .to_alarms_with_id();

    request.ok(&alarms_with_id_dto)
}

fn add_alarm(mut request: Request<&mut EspHttpConnection>, schedule_system: &Arc<ScheduleSystem>) -> RequestResult<(), EspIOError> {
    let output_index: u8 = *request.parameters::<OutputIndexDTO>()?;

    let alarm: Alarm = request.read_all::<AlarmDTO>()?.into();

    schedule_system
        .add_alarm(output_index, alarm)
        .map_err(|error| RequestError::General(error.to_string()))?;

    request.ok(&"Alarm added")
}

fn delete_alarm(request: Request<&mut EspHttpConnection>, schedule_system: &Arc<ScheduleSystem>) -> RequestResult<(), EspIOError> {
    let alarm_id: AlarmId = request.parameters::<AlarmIdDTO>()?.into();

    schedule_system
        .remove_alarm(&alarm_id)
        .map_err(|error| RequestError::General(error.to_string()))?;

    request.ok(&"Alarm removed")
}

fn delete_alarms_by_output_index(request: Request<&mut EspHttpConnection>, schedule_system: &Arc<ScheduleSystem>) -> RequestResult<(), EspIOError> {
    let output_index: u8 = *request.parameters::<OutputIndexDTO>()?;

    schedule_system
        .remove_alarms_by_output_index(output_index)
        .map_err(|error| RequestError::General(error.to_string()))?;

    request.ok(&"Alarms removed")
}
