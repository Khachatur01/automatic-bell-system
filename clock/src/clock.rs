use ds323x::interface::I2cInterface;
use ds323x::{ic, DateTimeAccess, Ds323x, NaiveDateTime};
use esp_idf_svc::hal::gpio::{IOPin, Input, PinDriver};
use esp_idf_svc::hal::i2c::{I2cDriver, I2cError};
use esp_idf_svc::sys::EspError;
use shared_bus::I2cProxy;
use std::sync::Mutex;


type Error = ds323x::Error<I2cError, ()>;
type I2cSharedProxy<'a> = I2cProxy<'a, Mutex<I2cDriver<'a>>>;
type Driver<'a> = Ds323x<I2cInterface<I2cSharedProxy<'a>>, ic::DS3231>;

pub struct Clock<'a, INT: IOPin> {
    driver: Driver<'a>,
    interrupt_pin: Option<PinDriver<'a, INT, Input>>
}

impl<'a, INT: IOPin> Clock<'a, INT> {
    pub fn new(i2c_shared_proxy: I2cSharedProxy<'a>, interrupt_pin: Option<PinDriver<'a, INT, Input>>) -> Result<Self, EspError> {
        let driver: Driver = Ds323x::new_ds3231(i2c_shared_proxy);

        Ok(Self { driver, interrupt_pin })
    }

    pub fn datetime(&mut self) -> Result<NaiveDateTime, Error> {
        self.driver.datetime()
    }

    pub fn subscribe_alarm_interruption<F>(&mut self, callback: F) -> Result<(), EspError>
    where
        F: FnMut() + Send + 'static {

        if let Some(interrupt_pin) = self.interrupt_pin.as_mut() {
            unsafe {
                interrupt_pin.subscribe(callback)?;
            }
        }

        Ok(())
    }

    pub fn enable_interrupt(&mut self) -> Result<(), EspError> {
        if let Some(interrupt_pin) = self.interrupt_pin.as_mut() {
            unsafe {
                interrupt_pin.enable_interrupt()?;
            }
        }

        Ok(())
    }
}
