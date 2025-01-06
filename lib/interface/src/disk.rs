use embedded_sdmmc::{sdcard, Error};
use crate::disk::path::file_path::FilePath;

pub mod path;

pub type DiskResult<Ok> = Result<Ok, Error<sdcard::Error>>;

pub trait ReadDisk {
    fn read_from_file(&mut self, path: &FilePath) -> DiskResult<Vec<u8>>;
}

pub trait WriteDisk {
    fn write_to_file(&mut self, path: &FilePath, data_buffer: &[u8]) -> DiskResult<()>;
}

pub trait ReadWriteDisk: ReadDisk + WriteDisk {}
