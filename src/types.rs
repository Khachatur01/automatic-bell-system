use std::sync::Mutex;
use esp_idf_svc::hal::gpio::{Gpio2, Gpio4};

pub type AlarmOutput = (Gpio2, Gpio4);
pub type BoxedMutex<T> = Box<Mutex<T>>;
