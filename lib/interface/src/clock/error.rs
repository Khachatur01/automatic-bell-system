use std::fmt::{Debug};

#[derive(Debug)]
pub enum ClockError {
    EspError,
    SynchronizationError,
    CanNotSubscribeToAlarmInterruption,
    AlarmNotFound,
    MutexLockError,
    InvalidTimestamp(u64)
}
