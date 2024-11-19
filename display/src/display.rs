use std::fmt;
use std::fmt::Write;
use std::net::Ipv4Addr;
use chrono::NaiveDateTime;
use esp_idf_svc::hal::i2c::I2cDriver;
use ssd1306::mode::{DisplayConfig, TerminalMode, TerminalModeError};
use ssd1306::prelude::{DisplaySize128x32, I2CInterface};
use ssd1306::rotation::DisplayRotation;
use ssd1306::{I2CDisplayInterface, Ssd1306};


type Driver<'a> = Ssd1306<I2CInterface<I2cDriver<'a>>, DisplaySize128x32, TerminalMode>;

pub struct Display<'a> {
    driver: Driver<'a>
}

impl<'a> Display<'a> {
    pub fn new(i2c_driver: I2cDriver<'a>) -> Result<Self, TerminalModeError> {
        let interface: I2CInterface<I2cDriver> = I2CDisplayInterface::new(i2c_driver);

        let mut display_driver: Driver = Ssd1306::new(
            interface,
            DisplaySize128x32,
            DisplayRotation::Rotate0,
        ).into_terminal_mode();

        display_driver.init()?;
        display_driver.clear()?;

        Ok(Self { driver: display_driver })
    }

    pub fn clear(&mut self) -> Result<(), TerminalModeError> {
        self.driver.clear()
    }

    pub fn display_information(&mut self, current_datetime: NaiveDateTime, ipv4: &Ipv4Addr) -> Result<(), fmt::Error> {
        let ip_address: String = ipv4.to_string();
        let current_time: String = current_datetime.time().to_string();

        self.driver.write_str(&format!("Ip: {ip_address}\n{current_time}"))
    }
}
