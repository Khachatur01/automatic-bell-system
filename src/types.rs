use std::sync::{Mutex, RwLock};
use esp_idf_svc::hal::gpio::{Gpio2, Gpio4};

pub type BoxedMutex<T> = Box<Mutex<T>>;
pub type BoxedRwLock<T> = Box<RwLock<T>>;
