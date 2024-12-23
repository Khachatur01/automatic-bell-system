pub mod alarm_id;

use std::sync::RwLock;
use access_point::access_point::AccessPoint;
use clock::clock::Clock;
use disk::disk::Disk;
use display::display::Display;
use interface::clock::ReadClock;
use interface::disk::{DiskResult, ReadDisk, WriteDisk};
use interface::{ClockError, Path};
use crate::schedule_system::alarm_id::AlarmId;

pub struct ScheduleSystem {
    access_point: AccessPoint<'static>,
    clock: Clock<AlarmId>,
    disk: Disk<'static>,
    display: Display<'static>,
}

impl ScheduleSystem {
    pub fn new(access_point: AccessPoint<'static>,
               clock: Clock<AlarmId>,
               disk: Disk<'static>,
               display: Display<'static>,) -> Self {

        Self {
            access_point,
            clock,
            disk,
            display,
        }
    }

    pub fn read_from_file(&mut self, path: &Path) -> DiskResult<Vec<u8>> {
        self.disk.read_from_file(path)
    }

    pub fn write_to_file(&mut self, path: &Path, data_buffer: &mut [u8]) -> DiskResult<()> {
        self.disk.write_to_file(path, data_buffer)
    }

    // pub fn get_time(&mut self) -> Result<DateTime<Utc>, ClockError> {
    //     self.clock.get_datetime()
    // }

    pub fn set_time(&mut self) {
    }
}
