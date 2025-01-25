use esp_idf_svc::sys::EspError;

#[derive(Debug)]
pub enum SecurityError {
    EspError(EspError),
    ReadLockError,
    WriteLockError,
    WrongCredentials,
}