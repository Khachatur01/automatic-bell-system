use crate::disk::path::Path;
use crate::PathParseError;

pub mod path;

pub trait ReadDisk {
    fn read_from_file(&self, path: &Path) -> Result<Vec<u8>, PathParseError>;
}

pub trait WriteDisk {
    fn write_to_file(&self, path: &Path, data_buffer: Vec<u8>) -> Result<(), PathParseError>;
}

pub trait ReadWriteDisk: ReadDisk + WriteDisk {}
