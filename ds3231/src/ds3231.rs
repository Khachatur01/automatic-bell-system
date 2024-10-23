mod set_time_error;

use esp_idf_svc::hal::delay::BLOCK;
use esp_idf_svc::hal::i2c::I2cDriver;
use esp_idf_svc::sys::EspError;
use nobcd::BcdNumber;
use crate::byte_offset::ByteOffset;
use crate::date_time::DateTime;
use crate::ds3231::set_time_error::SetTimeError;

const DS3231_ADDRESS: u8 = 0x68;


pub struct DS3231<'a> {
    i2c_driver: I2cDriver<'a>
}

impl<'a> DS3231<'a> {
    pub fn new(i2c_driver: I2cDriver<'a>) -> Self {
        Self {
            i2c_driver
        }
    }
}

impl<'a> DS3231<'a> {
    pub fn get_date_time(&mut self) -> Result<DateTime, SetTimeError> {
        let mut data: [u8; 7] = [0_u8; 7];

        self.i2c_driver.write(DS3231_ADDRESS, &[0_u8], BLOCK)
            .map_err(|error| SetTimeError::EspError(error))?;
        self.i2c_driver.read(DS3231_ADDRESS, &mut data, BLOCK)
            .map_err(|error| SetTimeError::EspError(error))?;

        DateTime::from_bcd_array(data)
            .map_err(|error| SetTimeError::BcdError(error))
    }

    pub fn set_date_time(&mut self, date_time: DateTime) -> Result<(), SetTimeError> {
        let [seconds] = BcdNumber::new(date_time.seconds)
            .map_err(|error| SetTimeError::BcdError(error))?
            .bcd_bytes();
        let [minutes] = BcdNumber::new(date_time.minutes)
            .map_err(|error| SetTimeError::BcdError(error))?
            .bcd_bytes();
        let [hours] = BcdNumber::new(date_time.hours)
            .map_err(|error| SetTimeError::BcdError(error))?
            .bcd_bytes();
        let [day] = BcdNumber::new(date_time.day)
            .map_err(|error| SetTimeError::BcdError(error))?
            .bcd_bytes();
        let [month] = BcdNumber::new(date_time.month)
            .map_err(|error| SetTimeError::BcdError(error))?
            .bcd_bytes();
        let [year] = BcdNumber::new(date_time.year)
            .map_err(|error| SetTimeError::BcdError(error))?
            .bcd_bytes();

        self.i2c_driver
            .write(DS3231_ADDRESS, &[ByteOffset::Seconds as u8, seconds], BLOCK)
            .map_err(|error| SetTimeError::EspError(error))?;
        self.i2c_driver
            .write(DS3231_ADDRESS, &[ByteOffset::Minutes as u8, minutes], BLOCK)
            .map_err(|error| SetTimeError::EspError(error))?;
        self.i2c_driver
            .write(DS3231_ADDRESS, &[ByteOffset::Hours as u8, hours], BLOCK)
            .map_err(|error| SetTimeError::EspError(error))?;

        self.i2c_driver
            .write(DS3231_ADDRESS, &[ByteOffset::Day as u8, day], BLOCK)
            .map_err(|error| SetTimeError::EspError(error))?;
        self.i2c_driver
            .write(DS3231_ADDRESS, &[ByteOffset::Month as u8, month], BLOCK)
            .map_err(|error| SetTimeError::EspError(error))?;
        self.i2c_driver
            .write(DS3231_ADDRESS, &[ByteOffset::Year as u8, year], BLOCK)
            .map_err(|error| SetTimeError::EspError(error))?;

        Ok(())
    }
}
