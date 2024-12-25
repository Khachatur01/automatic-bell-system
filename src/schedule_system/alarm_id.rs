use uuid::Uuid;

#[derive(Eq, PartialEq, Hash, Debug)]
pub struct AlarmId {
    pub output_pin: u8,
    pub uuid: Uuid
}
