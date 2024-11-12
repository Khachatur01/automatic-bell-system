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
type Directory<'a> = embedded_sdmmc::Directory<'a, BlockDevice<'a>, SdMmcClock, MAX_DIRS, MAX_FILES, MAX_VOLUMES>;

pub struct SDCard<'a> {
    volume_manager: VolumeManager<BlockDevice<'a>, SdMmcClock>
}

impl<'a> SDCard<'a> {
    pub fn new<SPI: SpiAnyPins>(spi: impl Peripheral<P = SPI> + 'a,
                                scl: impl Peripheral<P = impl OutputPin> + 'a,
                                sdo: impl Peripheral<P = impl OutputPin> + 'a,
                                sdi: impl Peripheral<P = impl InputPin> + 'a,
                                cs: impl Peripheral<P = impl OutputPin> + 'a) -> Result<Self, EspError> {

        let driver_config: DriverConfig = DriverConfig::default();
        let spi_driver: SpiDriver = SpiDriver::new(spi, scl, sdo, Some(sdi), &driver_config)?;

        let mut spi_config: Config = SpiConfig::new();
        spi_config.duplex = Duplex::Full;
        let spi_device_driver: SpiDeviceDriver<SpiDriver> = SpiDeviceDriver::new(spi_driver, Some(cs), &spi_config)?;

        let sdcard: SdCard<SpiDeviceDriver<SpiDriver>, FreeRtos> = SdCard::new(spi_device_driver, FreeRtos);

        Ok(Self { volume_manager: VolumeManager::new(sdcard, SdMmcClock) })
    }

    pub fn read_file(&mut self, path: Path) -> Result<String, Error<sdcard::Error>> {
        let mut directory = self.open_directory(path.absolute_directories);

        let mut file = directory.open_file_in_dir(path.filename.as_str(), Mode::ReadOnly)?;
        
        let mut buffer: Vec<u8> = vec![0; file.length() as usize];
        file.read(buffer.as_mut_slice())?;

        Ok(String::from_utf8_lossy(buffer.as_slice()).to_string())
    }

    fn open_directory(&mut self, absolute_directories: Vec<String>) -> Directory<'a> {
        let mut volume = self.volume_manager.open_volume(VolumeIdx(0))?;
        let mut root_directory = volume.open_root_dir().unwrap();

        root_directory
    }
}
