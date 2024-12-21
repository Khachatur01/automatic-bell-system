use crate::disk::path::Path;
use embedded_sdmmc::{sdcard, Error};

pub mod path;

pub type DiskResult<Ok> = Result<Ok, Error<sdcard::Error>>;

pub trait ReadDisk {
    fn read_from_file(&mut self, path: &Path) -> DiskResult<Vec<u8>>;
}

pub trait WriteDisk {
    fn write_to_file(&mut self, path: &Path, data_buffer: &mut [u8]) -> DiskResult<()>;
}

pub trait ReadWriteDisk: ReadDisk + WriteDisk {}
