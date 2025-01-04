use esp_idf_svc::sys::EspError;
use interface::{ClockError, PathParseError};
use std::fmt::{Debug, Display, Formatter};
use display_interface::DisplayError;
use embedded_sdmmc::Error as DiskError;
use embedded_sdmmc::sdcard::Error as SDCardError;

#[derive(Debug)]
pub enum ScheduleSystemError {
    EspError(EspError),
    I2cSharedBusError,
    AlarmIdParseError(String),
    ClockError(ClockError),
    DisplayError(DisplayError),
    DiskError(DiskError<SDCardError>),
    PathParseError(PathParseError),
    SerdeError(serde_json::error::Error),
    MutexLockError,
}

impl Display for ScheduleSystemError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Schedule System Error: {:?}", self)
    }
}
