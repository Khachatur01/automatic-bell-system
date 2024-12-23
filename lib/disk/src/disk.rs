use embedded_sdmmc::{Mode, SdCard, TimeSource, Timestamp, VolumeIdx, VolumeManager};
use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::gpio::OutputPin;
use esp_idf_svc::hal::peripheral::Peripheral;
use esp_idf_svc::hal::spi::config::Duplex;
use esp_idf_svc::hal::spi::{SpiConfig, SpiDeviceDriver, SpiDriver};
use esp_idf_svc::sys::EspError;
use interface::disk::{DiskResult, ReadDisk, WriteDisk};
use interface::Path;

type BlockDevice<'a> = SdCard<SpiDeviceDriver<'a, SpiDriver<'a>>, FreeRtos>;
type Volume<'a, 'b> = embedded_sdmmc::Volume<'b, BlockDevice<'a>, SdMmcClock, 4, 4, 1>;
type Directory<'a, 'b> = embedded_sdmmc::Directory<'b, BlockDevice<'a>, SdMmcClock, 4, 4, 1>;
type File<'a, 'b> = embedded_sdmmc::File<'b, BlockDevice<'a>, SdMmcClock, 4, 4, 1>;


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

pub struct Disk<'a> {
    volume_manager: VolumeManager<BlockDevice<'a>, SdMmcClock>
}

impl<'a> Disk<'a> {
    pub fn new<CS: Peripheral<P = impl OutputPin> + 'a>(spi_driver: SpiDriver<'a>, cs: CS) -> Result<Self, EspError> {
        let mut spi_config = SpiConfig::new();
        spi_config.duplex = Duplex::Full;

        let spi_device_driver: SpiDeviceDriver<SpiDriver> = SpiDeviceDriver::new(spi_driver, Some(cs), &spi_config)?;
        let sdcard: SdCard<SpiDeviceDriver<SpiDriver>, FreeRtos> = SdCard::new(spi_device_driver, FreeRtos);

        Ok(Self { volume_manager: VolumeManager::new(sdcard, SdMmcClock) })
    }
}

impl<'a> ReadDisk for Disk<'a> {
    fn read_from_file(&mut self, path: &Path) -> DiskResult<Vec<u8>> {
        let mut volume: Volume = self.volume_manager.open_volume(VolumeIdx(0))?;
        let mut directory: Directory = volume.open_root_dir()?;

        for dir in &path.directories_path {
            directory.change_dir(dir.as_str())?;
        }

        let mut file: File = directory.open_file_in_dir(path.filename.as_str(), Mode::ReadOnly)?;

        let mut buffer: Vec<u8> = vec![0; file.length() as usize];
        file.read(buffer.as_mut_slice())?;

        Ok(buffer)
    }
}

impl<'a> WriteDisk for Disk<'a> {
    fn write_to_file(&mut self, path: &Path, data_buffer: &mut [u8]) -> DiskResult<()> {
        let mut volume: Volume = self.volume_manager.open_volume(VolumeIdx(0))?;
        let mut directory: Directory = volume.open_root_dir()?;

        for dir in &path.directories_path {
            directory.change_dir(dir.as_str())?;
        }

        let mut file: File = directory.open_file_in_dir(path.filename.as_str(), Mode::ReadWriteCreate)?;

        file.write(data_buffer)?;

        Ok(())
    }
}
