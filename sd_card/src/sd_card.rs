use embedded_sdmmc::{sdcard, Error, Mode, SdCard, TimeSource, Timestamp, VolumeIdx, VolumeManager};
use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::gpio::{InputPin, OutputPin};
use esp_idf_svc::hal::peripheral::Peripheral;
use esp_idf_svc::hal::spi::config::{Config, DriverConfig, Duplex};
use esp_idf_svc::hal::spi::{SpiAnyPins, SpiConfig, SpiDeviceDriver, SpiDriver};
use esp_idf_svc::sys::EspError;
use crate::path::Path;

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

type BlockDevice<'a> = SdCard<SpiDeviceDriver<'a, SpiDriver<'a>>, FreeRtos>;
type Volume<'a, 'b> = embedded_sdmmc::Volume<'b, BlockDevice<'a>, SdMmcClock, 4, 4, 1>;
type Directory<'a, 'b> = embedded_sdmmc::Directory<'b, BlockDevice<'a>, SdMmcClock, 4, 4, 1>;
type File<'a, 'b> = embedded_sdmmc::File<'b, BlockDevice<'a>, SdMmcClock, 4, 4, 1>;

type SdResult<Ok> = Result<Ok, Error<sdcard::Error>>;


pub struct SDCard<'a> {
    volume_manager: VolumeManager<BlockDevice<'a>, SdMmcClock>
}

impl<'a> SDCard<'a> {
    pub fn new(spi_device_driver: SpiDeviceDriver<'a, SpiDriver<'a>>) -> Result<Self, EspError> {
        let sdcard: SdCard<SpiDeviceDriver<SpiDriver>, FreeRtos> = SdCard::new(spi_device_driver, FreeRtos);

        Ok(Self { volume_manager: VolumeManager::new(sdcard, SdMmcClock) })
    }

    pub fn read_from_file(&mut self, path: &Path) -> SdResult<Vec<u8>> {
        let mut volume: Volume = self.volume_manager.open_volume(VolumeIdx(0))?;
        let mut directory: Directory = volume.open_root_dir()?;

        directory.change_dir(path.directories_path.as_str())?;

        let mut file: File = directory.open_file_in_dir(path.filename.as_str(), Mode::ReadOnly)?;

        let mut buffer: Vec<u8> = vec![0; file.length() as usize];
        file.read(buffer.as_mut_slice())?;

        Ok(buffer)
    }

    pub fn write_to_file(&mut self, path: &Path, data_buffer: Vec<u8>) -> SdResult<()> {
        let mut volume: Volume = self.volume_manager.open_volume(VolumeIdx(0))?;
        let mut directory: Directory = volume.open_root_dir()?;

        directory.change_dir(path.directories_path.as_str())?;

        let mut file: File = directory.open_file_in_dir(path.filename.as_str(), Mode::ReadWriteCreate)?;

        file.write(data_buffer.as_slice())?;

        Ok(())
    }
}
