use crate::model::alarm::alarm::AlarmDTO;
use http_server::to_response_data::ToResponseData;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Serialize, Deserialize, Debug)]
pub struct AddAlarmDTO {
    pub output_index: u8,
    pub alarm: AlarmDTO,
}

impl ToResponseData for AddAlarmDTO {}
