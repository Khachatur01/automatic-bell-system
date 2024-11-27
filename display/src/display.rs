use chrono::{DateTime, NaiveDateTime, Utc};
use display_interface::DisplayError;
use embedded_graphics::geometry::Point;
use embedded_graphics::mono_font::ascii::FONT_5X7;
use embedded_graphics::mono_font::MonoTextStyleBuilder;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::text::{Baseline, Text};
use embedded_graphics::Drawable;
use esp_idf_svc::hal::i2c::I2cDriver;
use shared_bus::I2cProxy;
use ssd1306::mode::{BufferedGraphicsMode, DisplayConfig};
use ssd1306::prelude::I2CInterface;
use ssd1306::rotation::DisplayRotation;
use ssd1306::size::DisplaySize128x64;
use ssd1306::{I2CDisplayInterface, Ssd1306};
use std::sync::Mutex;


type I2cSharedProxy<'a> = I2cProxy<'a, Mutex<I2cDriver<'a>>>;
type Driver<'a> = Ssd1306<I2CInterface<I2cSharedProxy<'a>>, DisplaySize128x64, BufferedGraphicsMode<DisplaySize128x64>>;

pub struct Display<'a> {
    driver: Driver<'a>
}

impl<'a> Display<'a> {
    pub fn new(i2c_shared_proxy: I2cSharedProxy<'a>) -> Result<Self, DisplayError> {
        let mut display_driver: Driver = Ssd1306::new(
            I2CDisplayInterface::new(i2c_shared_proxy),
            DisplaySize128x64,
            DisplayRotation::Rotate0,
        ).into_buffered_graphics_mode();

        display_driver.init()?;
        display_driver.clear_buffer();

        Ok(Self { driver: display_driver })
    }

    pub fn clear(&mut self) -> () {
        self.driver.clear_buffer()
    }

    pub fn display_information(&mut self, local_datetime: DateTime<Utc>, next_bell_datetime: DateTime<Utc>) -> Result<(), DisplayError> {
        let local_time: String = local_datetime.format("%d/%m/%Y %I:%M:%S").to_string();
        let next_bell_time: String = next_bell_datetime.format("%d/%m/%Y %I:%M").to_string();

        let text_style = MonoTextStyleBuilder::new()
            .font(&FONT_5X7)
            .text_color(BinaryColor::On)
            .build();

        self.driver.clear_buffer();

        Text::with_baseline(
            &format!("Next bell\n{next_bell_time}\nLocal time\n{local_time}"),
            Point::zero(),
            text_style,
            Baseline::Top
        ).draw(&mut self.driver)?;

        self.driver.flush()
    }
}
