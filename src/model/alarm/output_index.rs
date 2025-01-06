use std::ops::Deref;
use serde::{Deserialize, Serialize};

#[derive(Eq, PartialEq, Hash, Clone, Debug, Serialize, Deserialize)]
pub struct OutputIndexDTO {
    pub output_index: u8
}

impl Deref for OutputIndexDTO {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        &self.output_index
    }
}
