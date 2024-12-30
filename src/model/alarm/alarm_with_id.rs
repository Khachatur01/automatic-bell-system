use serde::{Deserialize, Serialize};
use clock::alarm::Alarm;
use crate::model::alarm::alarm::AlarmDTO;
use crate::model::alarm::alarm_id::AlarmIdDTO;
use crate::schedule_system::alarm_id::AlarmId;


#[derive(Debug, Serialize, Deserialize)]
pub struct AlarmWithIdDTO {
    pub id: AlarmIdDTO,
    pub alarm: AlarmDTO,
}

impl From<(AlarmId, Alarm)> for AlarmWithIdDTO {
    fn from((alarm_id_dto, alarm_dto): (AlarmId, Alarm)) -> Self {
        Self {
            id: alarm_id_dto.into(),
            alarm: alarm_dto.into(),
        }
    }
}
