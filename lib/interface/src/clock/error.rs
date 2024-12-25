use std::fmt::{Debug, Display};

#[derive(Debug)]
pub enum ClockError {
    EspError,
    SynchronizationError,
    CanNotSubscribeToAlarmInterruption,
    AlarmNotFound,
    MutexLockError,
    InvalidTimestamp(u64)
}
