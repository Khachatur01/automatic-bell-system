use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ClockDTO {
    pub(crate) timestamp_millis: i64
}
