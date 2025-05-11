use crate::model::alarm::alarm::AlarmDTO;
use crate::model::alarm::alarm_id::AlarmIdDTO;
use crate::schedule_system::alarm_id::AlarmId;
use clock::alarm::Alarm;
use serde::{Deserialize, Serialize};
use http_server::to_response_data::ToResponseData;

#[derive(Serialize, Deserialize, Debug)]
pub struct AlarmWithIdDTO {
    pub id: AlarmIdDTO,
    pub alarm: AlarmDTO,
}

impl ToResponseData for AlarmWithIdDTO {}


impl From<(AlarmId, Alarm)> for AlarmWithIdDTO {
    fn from((alarm_id_dto, alarm_dto): (AlarmId, Alarm)) -> Self {
        Self {
            id: alarm_id_dto.into(),
            alarm: alarm_dto.into(),
        }
    }
}
