use esp_idf_svc::sys::EspError;
use nobcd::BcdError;

pub enum SetTimeError {
    EspError(EspError),
    BcdError(BcdError),
}