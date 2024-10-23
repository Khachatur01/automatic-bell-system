#[repr(u8)]
pub enum ByteOffset {
    Seconds,
    Minutes,
    Hours,
    DayOfWeek,
    Day,
    Month,
    Year,
}