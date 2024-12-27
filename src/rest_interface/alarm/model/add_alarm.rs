use crate::rest_interface::alarm::model::alarm::AlarmDTO;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct AddAlarmDTO {
    pub output_index: u8,
    pub alarm: AlarmDTO,
}
