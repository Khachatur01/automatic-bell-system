use embedded_sdmmc::{DirEntry, Error, Mode, SdCard, ShortFileName, TimeSource, Timestamp, VolumeIdx};
use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::gpio::OutputPin;
use esp_idf_svc::hal::peripheral::Peripheral;
use esp_idf_svc::hal::spi::config::Duplex;
use esp_idf_svc::hal::spi::{SpiConfig, SpiDeviceDriver, SpiDriver};
use esp_idf_svc::sys::EspError;
use interface::disk::path::directory_path::DirectoryPath;
use interface::disk::path::file_path::FilePath;
use interface::disk::{DiskResult, ReadDisk, WriteDisk};

const MAX_DIRS: usize = 16;
const MAX_FILES: usize = 16;
const MAX_VOLUMES: usize = 1;


type BlockDevice<'spi> = SdCard<SpiDeviceDriver<'spi, SpiDriver<'spi>>, FreeRtos>;
type VolumeManager<'spi> = embedded_sdmmc::VolumeManager<BlockDevice<'spi>, SdMmcClock, MAX_DIRS, MAX_FILES, MAX_VOLUMES>;
type Volume<'spi, 'vol> = embedded_sdmmc::Volume<'vol, BlockDevice<'spi>, SdMmcClock, MAX_DIRS, MAX_FILES, MAX_VOLUMES>;
type Directory<'spi, 'vol> = embedded_sdmmc::Directory<'vol, BlockDevice<'spi>, SdMmcClock, MAX_DIRS, MAX_FILES, MAX_VOLUMES>;
type File<'spi, 'vol> = embedded_sdmmc::File<'vol, BlockDevice<'spi>, SdMmcClock, MAX_DIRS, MAX_FILES, MAX_VOLUMES>;


pub struct SdMmcClock;

impl TimeSource for SdMmcClock {
    fn get_timestamp(&self) -> Timestamp {
        Timestamp {
            year_since_1970: 0,
            zero_indexed_month: 0,
            zero_indexed_day: 0,
            hours: 0,
            minutes: 0,
            seconds: 0,
        }
    }
}

pub struct Disk<'spi> {
    volume_manager: VolumeManager<'spi>,
    // volume: Volume<'spi, 'vol>
}

impl<'spi> Disk<'spi> {
    pub fn new<CS: Peripheral<P = impl OutputPin> + 'spi>(spi_driver: SpiDriver<'spi>, cs: CS) -> Result<Self, EspError> {
        let mut spi_config = SpiConfig::new();
        spi_config.duplex = Duplex::Full;

        let spi_device_driver: SpiDeviceDriver<'spi, SpiDriver<'spi>> = SpiDeviceDriver::new(spi_driver, Some(cs), &spi_config)?;
        let sdcard: BlockDevice<'spi> = SdCard::new(spi_device_driver, FreeRtos);

        let volume_manager = VolumeManager::new_with_limits(sdcard, SdMmcClock, 5000);
        // let volume: Volume<'spi, 'vol> = volume_manager.open_volume(VolumeIdx(0)).unwrap();

        Ok(Self { volume_manager })
    }

    pub fn list_dir(&mut self, path: &DirectoryPath) -> DiskResult<Vec<String>> {
        let mut volume: Volume = self.volume_manager.open_volume(VolumeIdx(0))?;
        let mut directory: Directory = volume.open_root_dir()?;
        // let mut directory: Directory = self.volume.open_root_dir()?;

        for dir in &path.directories_path {
            directory.change_dir(dir.as_str())?;
        }

        let mut dirs: Vec<String> = vec![];

        directory.iterate_dir(|dir_entry: &DirEntry| {
            let dir_name: String = dir_entry.name.to_string();

            /* skip special names for current and parent directories */
            if dir_name == "." || dir_name == ".." {
                return;
            }

            if dir_entry.attributes.is_directory() {
                dirs.push(dir_name);
            }
        })?;

        Ok(dirs)
    }

    pub fn list_files(&mut self, path: &DirectoryPath) -> DiskResult<Vec<String>> {
        let mut volume: Volume = self.volume_manager.open_volume(VolumeIdx(0))?;
        let mut directory: Directory = volume.open_root_dir()?;
        // let mut directory: Directory = self.volume.open_root_dir()?;

        for dir in &path.directories_path {
            directory.change_dir(dir.as_str())?;
        }

        let mut dirs: Vec<String> = vec![];

        directory.iterate_dir(|dir_entry: &DirEntry| {
            if dir_entry.attributes.is_directory() {
                return;
            }

            dirs.push(dir_entry.name.to_string());
        })?;

        Ok(dirs)
    }

    pub fn make_dir(&mut self, path: &DirectoryPath) -> DiskResult<()> {
        let mut volume: Volume = self.volume_manager.open_volume(VolumeIdx(0))?;
        let mut directory: Directory = volume.open_root_dir()?;
        // let mut directory: Directory = self.volume.open_root_dir()?;

        for dir in &path.directories_path {
            if let Err(error) = directory.make_dir_in_dir(dir.as_str()) {
                /* Raise an error if it's not DirAlreadyExists error. */
                if !matches!(error, Error::DirAlreadyExists) {
                    return Err(error);
                }
            }

            directory.change_dir(dir.as_str())?;
        }

        Ok(())
    }

    pub fn remove_dir(&mut self, path: &DirectoryPath) -> DiskResult<()> {
        let mut volume: Volume = self.volume_manager.open_volume(VolumeIdx(0))?;
        let mut directory: Directory = volume.open_root_dir()?;
        // let mut directory: Directory = self.volume.open_root_dir()?;

        let length: usize = path.directories_path.len();

        if let (directories_path, [removable_dir]) = path.directories_path.as_slice().split_at(length) {
            for dir in directories_path {
                directory.change_dir(dir.as_str())?;
            }

            directory.delete_file_in_dir(removable_dir.as_str())?;
        }

        Ok(())
    }

    pub fn clear_dir(&mut self, path: &DirectoryPath) -> DiskResult<()> {
        let mut volume: Volume = self.volume_manager.open_volume(VolumeIdx(0))?;
        let mut directory: Directory = volume.open_root_dir()?;
        // let mut directory: Directory = self.volume.open_root_dir()?;

        for dir in &path.directories_path {
            directory.change_dir(dir.as_str())?;
        }

        let mut filenames: Vec<ShortFileName> = vec![];

        directory.iterate_dir(|dir_entry: &DirEntry| {
            if !dir_entry.attributes.is_directory() {
                filenames.push(dir_entry.name.clone());
            }
        })?;

        for filename in filenames {
            directory.delete_file_in_dir(&filename)?;
        }

        Ok(())
    }

    pub fn delete_file(&mut self, path: &FilePath) -> DiskResult<()> {
        let mut volume: Volume = self.volume_manager.open_volume(VolumeIdx(0))?;
        let mut directory: Directory = volume.open_root_dir()?;
        // let mut directory: Directory = self.volume.open_root_dir()?;

        for dir in &path.directories_path {
            directory.change_dir(dir.as_str())?;
        }

        directory.delete_file_in_dir(path.filename.as_str())?;

        Ok(())
    }
}

impl<'spi> ReadDisk for Disk<'spi> {
    fn read_from_file(&mut self, path: &FilePath) -> DiskResult<Vec<u8>> {
        println!("Opening volume 0...");
        let mut volume: Volume = self.volume_manager.open_volume(VolumeIdx(0))?;
        let mut directory: Directory = volume.open_root_dir()?;
        // let mut directory: Directory = self.volume.open_root_dir()?;
        println!("Volume 0 opened!");

        for dir in &path.directories_path {
            println!("Directory: {}", dir);
            directory.change_dir(dir.as_str())?;
        }

        let mut file: File = directory.open_file_in_dir(path.filename.as_str(), Mode::ReadOnly)?;

        println!("File opened");

        let mut buffer: Vec<u8> = vec![0; file.length() as usize];
        file.read(buffer.as_mut_slice())?;

        println!("File read");

        Ok(buffer)
    }
    fn read_from_file_bytes<OnRead: FnMut(&[u8])>(&mut self, path: &FilePath, bytes: usize, mut on_read: OnRead) -> DiskResult<()> {
        println!("Opening volume 0...");
        let mut volume: Volume = self.volume_manager.open_volume(VolumeIdx(0))?;
        let mut directory: Directory = volume.open_root_dir()?;
        // let mut directory: Directory = self.volume.open_root_dir()?;
        println!("Volume 0 opened!");

        for dir in &path.directories_path {
            println!("Directory: {}", dir);
            directory.change_dir(dir.as_str())?;
        }

        let mut file: File = directory.open_file_in_dir(path.filename.as_str(), Mode::ReadOnly)?;

        println!("File opened");

        let mut buffer: Vec<u8> = vec![0; bytes];
        while !file.is_eof() {
            file.read(buffer.as_mut_slice())?;
            on_read(buffer.as_slice());
        }

        println!("File read");

        Ok(())
    }
}

impl<'spi> WriteDisk for Disk<'spi> {
    fn write_to_file(&mut self, path: &FilePath, data_buffer: &[u8]) -> DiskResult<()> {
        let mut volume: Volume = self.volume_manager.open_volume(VolumeIdx(0))?;
        let mut directory: Directory = volume.open_root_dir()?;
        // let mut directory: Directory = self.volume.open_root_dir()?;

        for dir in &path.directories_path {
            directory.change_dir(dir.as_str())?;
        }

        let mut file: File = directory.open_file_in_dir(path.filename.as_str(), Mode::ReadWriteCreate)?;

        file.write(data_buffer)?;

        Ok(())
    }
}
