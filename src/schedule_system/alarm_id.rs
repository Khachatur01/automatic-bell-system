use uuid::Uuid;

#[derive(Eq, PartialEq, Hash)]
pub struct AlarmId {
    pub output_pin: u8,
    pub uuid: Uuid
}
