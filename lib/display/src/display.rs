use std::net::Ipv4Addr;
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
use ssd1306::prelude::{DisplaySize128x32, I2CInterface};
use ssd1306::rotation::DisplayRotation;
use ssd1306::{I2CDisplayInterface, Ssd1306};
use std::sync::Mutex;

type I2cSharedProxy<'a> = I2cProxy<'a, Mutex<I2cDriver<'a>>>;
type Driver<'a> = Ssd1306<I2CInterface<I2cSharedProxy<'a>>, DisplaySize128x32, BufferedGraphicsMode<DisplaySize128x32>>;

pub struct Display<'a> {
    driver: Driver<'a>
}

impl<'a> Display<'a> {
    pub fn new(i2c_shared_proxy: I2cSharedProxy<'a>) -> Result<Self, DisplayError> {
        let mut display_driver: Driver = Ssd1306::new(
            I2CDisplayInterface::new(i2c_shared_proxy),
            DisplaySize128x32,
            DisplayRotation::Rotate0,
        ).into_buffered_graphics_mode();

        display_driver.init()?;
        display_driver.clear_buffer();
        display_driver.flush()?;

        Ok(Self { driver: display_driver })
    }

    pub fn clear(&mut self) -> Result<(), DisplayError> {
        self.driver.clear_buffer();
        self.driver.flush()
    }

    pub fn update(&mut self, datetime: String, ip_v4: String) -> Result<(), DisplayError> {
        self.driver.clear_buffer();

        let text_style = MonoTextStyleBuilder::new()
            .font(&FONT_5X7)
            .text_color(BinaryColor::On)
            .build();

        Text::with_baseline(
            &format!("IP: {ip_v4}"),
            Point::zero(),
            text_style,
            Baseline::Top
        ).draw(&mut self.driver)?;

        Text::with_baseline(
            &format!("{datetime}"),
            Point::new(0, text_style.font.character_size.height as i32),
            text_style,
            Baseline::Top
        ).draw(&mut self.driver)?;

        self.driver.flush()
    }
}
