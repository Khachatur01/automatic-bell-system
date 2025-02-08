use esp_idf_svc::hal::i2c::I2cDriver;
use shared_bus::I2cProxy;
use ssd1306::mode::{BufferedGraphicsMode, DisplayConfig};
use ssd1306::prelude::{DisplaySize128x32, I2CInterface};
use ssd1306::rotation::DisplayRotation;
use ssd1306::{I2CDisplayInterface, Ssd1306};
use std::sync::Mutex;
use display_interface::DisplayError;
use embedded_graphics::Drawable;
use embedded_graphics::geometry::Point;
use embedded_graphics::mono_font::ascii::FONT_9X18_BOLD;
use embedded_graphics::mono_font::MonoTextStyleBuilder;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::text::{Baseline, Text};

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

    pub fn write_text(&mut self, text: &str) -> Result<(), DisplayError> {
        self.driver.clear_buffer();

        let text_style = MonoTextStyleBuilder::new()
            .font(&FONT_9X18_BOLD)
            .text_color(BinaryColor::On)
            .build();

        Text::with_baseline(
            text,
            Point::zero(),
            text_style,
            Baseline::Top
        ).draw(&mut self.driver)?;

        self.driver.flush()?;

        Ok(())
    }
}
