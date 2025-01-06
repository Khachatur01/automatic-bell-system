use http_server::to_response_data::ToResponseData;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::ops::Deref;

#[derive(Eq, PartialEq, Hash, Clone, Debug, Serialize, Deserialize)]
pub struct OutputIndexDTO {
    pub output_index: u8
}

impl ToResponseData for OutputIndexDTO {}

impl Deref for OutputIndexDTO {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        &self.output_index
    }
}
