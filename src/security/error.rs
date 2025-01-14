use esp_idf_svc::sys::EspError;

pub enum SecurityError {
    EspError(EspError),
    WrongCredentials,
}