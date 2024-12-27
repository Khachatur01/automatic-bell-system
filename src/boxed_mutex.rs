use std::sync::Mutex;
use access_point::access_point::AccessPoint;
use clock::clock::Clock;
use disk::disk::Disk;
use display::display::Display;
use crate::types::AlarmOutput;

pub trait IntoBoxedMutex
where Self: Sized {
    fn into_boxed_mutex(self) -> Box<Mutex<Self>> {
        Box::from(Mutex::new(self))
    }
}

impl<AlarmId> IntoBoxedMutex for Clock<AlarmId> {}
impl IntoBoxedMutex for AccessPoint<'_> {}
impl IntoBoxedMutex for Disk<'_> {}
impl IntoBoxedMutex for Display<'_> {}
impl IntoBoxedMutex for AlarmOutput {}
