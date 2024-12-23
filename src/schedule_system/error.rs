use esp_idf_svc::sys::EspError;
use interface::ClockError;
use std::fmt::{Debug, Display, Formatter};
use display_interface::DisplayError;
use embedded_sdmmc::Error as DiskError;
use embedded_sdmmc::sdcard::Error as SDCardError;

#[derive(Debug)]
pub enum ScheduleSystemError {
    EspError(EspError),
    ClockError(ClockError),
    DisplayError(DisplayError),
    DiskError(DiskError<SDCardError>),
    MutexLockError,
}

impl Display for ScheduleSystemError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}
