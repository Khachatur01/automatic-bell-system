use std::ops::Deref;
use serde::{Deserialize, Serialize};
use crate::schedule_system::model::output_index::OutputIndex;

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

impl TryFrom<OutputIndexDTO> for OutputIndex {
    type Error = String;

    fn try_from(output_index_dto: OutputIndexDTO) -> Result<Self, Self::Error> {
        OutputIndex::try_from(*output_index_dto)
    }
}

impl From<OutputIndex> for OutputIndexDTO {
    fn from(output_index: OutputIndex) -> Self {
        Self {
            output_index: *output_index
        }
    }
}
