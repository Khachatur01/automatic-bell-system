use crate::schedule_system::alarm_id::AlarmId;
use serde::{Deserialize, Serialize};
use uuid::{Error};

#[derive(Serialize, Deserialize, Debug)]
pub struct AlarmIdDTO {
    output_pin: u8,
    uuid: String
}

impl TryFrom<AlarmIdDTO> for AlarmId {
    type Error = Error;

    fn try_from(alarm_id_dto: AlarmIdDTO) -> Result<Self, Self::Error> {
        Ok(Self {
            output_pin: alarm_id_dto.output_pin,
            uuid: alarm_id_dto.uuid.as_str().try_into()?
        })
    }
}

impl From<AlarmId> for AlarmIdDTO {
    fn from(alarm_id: AlarmId) -> Self {
        Self {
            output_pin: alarm_id.output_pin,
            uuid: alarm_id.uuid.to_string()
        }
    }
}
