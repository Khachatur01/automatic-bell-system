use nobcd::{BcdError, BcdNumber};
use crate::byte_offset::ByteOffset;
pub struct DateTime {
    pub seconds: u8,
    pub minutes: u8,
    pub hours: u8,

    pub year: u8,
    pub month: u8,
    pub day: u8,
}
impl DateTime {
    pub fn from_bcd_array(bcd_array: [u8; 7]) -> Result<Self, BcdError> {
        let seconds: u8 = BcdNumber::from_bcd_bytes([bcd_array[ByteOffset::Seconds as usize]])?
            .value();
        let minutes: u8 = BcdNumber::from_bcd_bytes([bcd_array[ByteOffset::Minutes as usize]])?
            .value();
        let hours: u8 = BcdNumber::from_bcd_bytes([bcd_array[ByteOffset::Hours as usize]])?
            .value();

        let day: u8 = BcdNumber::from_bcd_bytes([bcd_array[ByteOffset::Day as usize]])?
            .value();
        let month: u8 = BcdNumber::from_bcd_bytes([bcd_array[ByteOffset::Month as usize]])?
            .value();
        let year: u8 = BcdNumber::from_bcd_bytes([bcd_array[ByteOffset::Year as usize]])?
            .value();

        let this = Self {
            year, month, day,
            hours, minutes, seconds
        };

        Ok(this)
    }
}
