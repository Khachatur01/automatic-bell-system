use std::fmt::Display;
use crate::schedule_system::alarm_id::AlarmId;
use serde::{Deserialize, Serialize};
use http_server::to_response_data::ToResponseData;

impl ToResponseData for AlarmIdDTO {}

#[derive(Eq, PartialEq, Hash, Clone, Debug, Serialize, Deserialize)]
pub struct AlarmIdDTO {
    pub output_index: u8,
    pub identifier: String
}

impl From<AlarmIdDTO> for AlarmId {
    fn from(alarm_id_dto: AlarmIdDTO) -> Self {
        Self {
            output_index: alarm_id_dto.output_index,
            identifier: alarm_id_dto.identifier
        }
    }
}

impl From<AlarmId> for AlarmIdDTO {
    fn from(alarm_id: AlarmId) -> Self {
        Self {
            output_index: alarm_id.output_index,
            identifier: alarm_id.identifier,
        }
    }
}
