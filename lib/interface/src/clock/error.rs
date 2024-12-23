use std::fmt::{Debug, Display};

#[derive(Debug)]
pub enum ClockError {
    EspError,
    SynchronizationError,
    CanNotSubscribeToAlarmInterruption,
    MutexLockError,
    InvalidTimestamp(u64)
}
