use serde::{Deserialize, Serialize};
use crate::model::alarm::alarm::AlarmDTO;

#[derive(Serialize, Deserialize, Debug)]
pub struct AddAlarmDTO {
    pub output_index: u8,
    pub alarm: AlarmDTO,
}
