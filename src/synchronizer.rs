use std::sync::{Mutex, RwLock};
use esp_idf_svc::hal::gpio::{AnyOutputPin, Output, PinDriver};
use esp_idf_svc::sys::EspError;
use access_point::access_point::AccessPoint;
use clock::clock::Clock;
use disk::disk::Disk;
use display::display::Display;

////////////////////////////////////////////////////////////////////////////////////////////////////
/* Boxed Mutex */
pub type BoxedMutex<T> = Box<Mutex<T>>;

pub trait IntoBoxedMutex
where Self: Sized {
    fn into_boxed_mutex(self) -> BoxedMutex<Self> {
        Box::from(Mutex::new(self))
    }
}

impl<AlarmId> IntoBoxedMutex for Clock<AlarmId> {}
impl IntoBoxedMutex for AccessPoint<'_> {}
impl IntoBoxedMutex for Disk<'_> {}
impl IntoBoxedMutex for Display<'_> {}


////////////////////////////////////////////////////////////////////////////////////////////////////
/* Boxed RwLock */
pub type BoxedRwLock<T> = Box<RwLock<T>>;

pub trait IntoBoxedRwLock
where Self: Sized {
    fn into_boxed_rwlock(self) -> BoxedRwLock<Self> {
        Box::from(RwLock::new(self))
    }
}

impl<AlarmId> IntoBoxedRwLock for Clock<AlarmId> {}
impl IntoBoxedRwLock for AccessPoint<'_> {}
impl IntoBoxedRwLock for Disk<'_> {}
impl IntoBoxedRwLock for Display<'_> {}


////////////////////////////////////////////////////////////////////////////////////////////////////
/* Mutex Output PinDriver */
pub type MutexOutputPin<'a> = Mutex<PinDriver<'a, AnyOutputPin, Output>>;

pub trait IntoMutexOutputPin<'a>
where Self: Sized {
    fn try_into_mutex_output_pin(self) -> Result<MutexOutputPin<'a>, EspError>;
}

impl<'a> IntoMutexOutputPin<'a> for AnyOutputPin {
    fn try_into_mutex_output_pin(self) -> Result<MutexOutputPin<'a>, EspError> {
        Ok(
            Mutex::new(PinDriver::output(self)?)
        )
    }
}
