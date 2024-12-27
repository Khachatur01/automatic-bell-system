use uuid::Uuid;
use crate::schedule_system::model::output_index::OutputIndex;

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub struct AlarmId {
    pub output_index: OutputIndex,
    pub uuid: Uuid
}
