use std::collections::{HashMap, HashSet};
use crate::rest_interface::alarm::model::alarm::{AlarmDTO, AlarmMarcherDTO};
use crate::rest_interface::alarm::model::alarm_id::AlarmIdDTO;
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
use crate::rest_interface::alarm::model::add_alarm::AddAlarmDTO;
use crate::rest_interface::alarm::model::output_index::OutputIndexDTO;
use crate::schedule_system::model::output_index::OutputIndex;

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

    Ok(())
}

fn get_alarm(request: Request<&mut EspHttpConnection>, schedule_system: &Arc<ScheduleSystem>) -> RequestResult<(), EspIOError> {
    let alarm_id: AlarmId =
        request.parameters::<AlarmIdDTO>()?
            .try_into()
            .map_err(|error| RequestError::General(error))?;
    
    let alarm: Alarm =
        schedule_system
            .get_alarm(&alarm_id)
            .map_err(|error| RequestError::General(error.to_string()))?;
    
    let alarm_dto: AlarmDTO = alarm.into();
    
    request.ok(&alarm_dto)
}

fn get_alarms_by_output_index(request: Request<&mut EspHttpConnection>, schedule_system: &Arc<ScheduleSystem>) -> RequestResult<(), EspIOError> {
    let output_index: OutputIndex =
        request.parameters::<OutputIndexDTO>()?
            .try_into()
            .map_err(|error| RequestError::General(error))?;
    
    let alarms: HashMap<AlarmId, Alarm> =
        schedule_system
            .get_alarms_by_output_index(output_index)
            .map_err(|error| RequestError::General(error.to_string()))?;

    let alarms_dto: HashMap<String, AlarmDTO> = alarms
        .into_iter()
        .fold(HashMap::new(), |mut accumulator, (alarm_id, alarm)| {
            let alarm_id_dto: AlarmIdDTO = alarm_id.into();
            let alarm_id_json: String = serde_json::to_string(&alarm_id_dto).unwrap_or_default();

            accumulator.insert(alarm_id_json, alarm.into());
            accumulator
        });

    request.ok(&alarms_dto)
}

fn add_alarm(mut request: Request<&mut EspHttpConnection>, schedule_system: &Arc<ScheduleSystem>) -> RequestResult<(), EspIOError> {
    let output_index: OutputIndex =
        request.parameters::<OutputIndexDTO>()?
            .try_into()
            .map_err(|error| RequestError::General(error))?;

    let alarm: Alarm = request.read_all::<AlarmDTO>()?.into();
    
    schedule_system
        .add_alarm(output_index, alarm)
        .map_err(|error| RequestError::General(error.to_string()))?;

    request.ok(&"Alarm added")
}

fn delete_alarm(request: Request<&mut EspHttpConnection>, schedule_system: &Arc<ScheduleSystem>) -> RequestResult<(), EspIOError> {
    request.internal_server_error(&String::from(""))
}


fn alarms_to_dto(alarms: HashMap<AlarmId, Alarm>) -> HashMap<AlarmIdDTO, AlarmDTO> {
    alarms
        .into_iter()
        .fold(HashMap::new(), |mut accumulator, (alarm_id, alarm)| {
            accumulator.insert(alarm_id.into(), alarm.into());
            accumulator
        })
}
