use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

pub enum ClockError {
    EspError,
    SynchronizationError,
    CanNotSubscribeToAlarmInterruption,
    ApiMutexLockError,
    InvalidTimestamp(u64)
}

impl Debug for ClockError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Display for ClockError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for ClockError {}
