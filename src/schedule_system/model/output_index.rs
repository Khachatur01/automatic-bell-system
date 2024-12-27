use std::ops::Deref;

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub struct OutputIndex(u8);

impl TryFrom<u8> for OutputIndex {
    type Error = String;

    fn try_from(index: u8) -> Result<Self, Self::Error> {
        if index > 1 { /* todo: dynamically get AlarmOutputs count */
            Err(String::from("Index should be between 0 and 1"))
        } else {
            Ok(OutputIndex(index))
        }
    }
}

impl Deref for OutputIndex {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
