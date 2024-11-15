use ds323x::interface::I2cInterface;
use ds323x::{ic, Ds323x};
use esp_idf_svc::hal::gpio::{IOPin, Input, InterruptType, OutputPin, PinDriver, Pull};
use esp_idf_svc::hal::i2c::config::Config;
use esp_idf_svc::hal::i2c::{I2c, I2cConfig, I2cDriver};
use esp_idf_svc::hal::peripheral::Peripheral;
use esp_idf_svc::sys::EspError;

pub type Driver<'a> = Ds323x<I2cInterface<I2cDriver<'a>>, ic::DS3231>;

pub struct Clock<'a, INT: IOPin> {
    driver: Driver<'a>,
    interrupt_pin: PinDriver<'a, INT, Input>
}

impl<'a, INT: IOPin> Clock<'a, INT> {
    pub fn new(i2c_driver: I2cDriver<'a>, interrupt_pin: INT) -> Result<Self, EspError> {
        let driver: Driver = Ds323x::new_ds3231(i2c_driver);


        let mut interrupt_pin: PinDriver<INT, Input> = PinDriver::input(interrupt_pin)?;
        interrupt_pin.set_pull(Pull::Up)?;
        interrupt_pin.set_interrupt_type(InterruptType::PosEdge)?;

        Ok(Self { driver, interrupt_pin })
    }

    pub fn interruption_subscribe<F>(&mut self, callback: F) -> Result<(), EspError>
    where
        F: FnMut() + Send + 'static {

        unsafe {
            self.interrupt_pin.subscribe(callback)?;
        }

        Ok(())
    }

    pub fn enable_interrupt(&mut self) -> Result<(), EspError> {
        self.interrupt_pin.enable_interrupt()?;

        Ok(())
    }
}
