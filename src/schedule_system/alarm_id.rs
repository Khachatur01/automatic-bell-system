#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub struct AlarmId {
    pub output_index: u8,
    pub identifier: String
}
