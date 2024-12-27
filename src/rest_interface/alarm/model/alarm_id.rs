use crate::schedule_system::alarm_id::AlarmId;
use serde::{Deserialize, Serialize};

#[derive(Eq, PartialEq, Hash, Clone, Debug, Serialize, Deserialize)]
pub struct AlarmIdDTO {
    output_index: u8,
    uuid: String
}

impl TryFrom<AlarmIdDTO> for AlarmId {
    type Error = String;

    fn try_from(alarm_id_dto: AlarmIdDTO) -> Result<Self, Self::Error> {
        Ok(Self {
            output_index: alarm_id_dto
                .output_index
                .try_into()
                .map_err(String::from)?,
            uuid: alarm_id_dto
                .uuid
                .as_str()
                .try_into()
                .map_err(|error: uuid::Error| error.to_string())?
        })
    }
}

impl From<AlarmId> for AlarmIdDTO {
    fn from(alarm_id: AlarmId) -> Self {
        Self {
            output_index: *alarm_id.output_index,
            uuid: alarm_id.uuid.to_string()
        }
    }
}
