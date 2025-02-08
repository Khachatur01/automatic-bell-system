use std::fmt::{Display, Formatter};
use esp_idf_svc::sys::EspError;

#[derive(Debug)]
pub enum SecurityError {
    EspError(EspError),
    ReadLockError,
    WriteLockError,
    WrongCredentials,
}

impl Display for SecurityError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SecurityError::EspError(esp_error) => esp_error.fmt(f),
            SecurityError::ReadLockError => f.write_str("Could not read lock."),
            SecurityError::WriteLockError => f.write_str("Could not write lock."),
            SecurityError::WrongCredentials => f.write_str("Wrong credentials."),
        }
    }
}
