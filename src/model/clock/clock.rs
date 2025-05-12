use serde::{Deserialize, Serialize};
use http_server::to_response_data::ToResponseData;

#[derive(Serialize, Deserialize, Debug)]
pub struct ClockDTO {
    pub(crate) timestamp_millis: i64
}

impl ToResponseData for ClockDTO {}
