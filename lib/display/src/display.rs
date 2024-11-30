use display_interface::DisplayError;
use embedded_graphics::pixelcolor::BinaryColor;
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
        display_driver.flush();

        Ok(Self { driver: display_driver })
    }

    pub fn clear(&mut self) -> Result<(), DisplayError> {
        self.driver.clear_buffer();
        self.driver.flush()
    }

    pub fn draw(&mut self, drawable: impl Drawable<Color = BinaryColor>) -> Result<(), DisplayError> {
        self.driver.clear_buffer();

        drawable.draw(&mut self.driver)?;

        self.driver.flush()
    }

    // pub fn display_information(&mut self) -> Result<(), DisplayError> {
    //
    //     let text_style = MonoTextStyleBuilder::new()
    //         .font(&FONT_5X7)
    //         .text_color(BinaryColor::On)
    //         .build();
    //
    //     self.driver.clear_buffer();
    //
    //     Text::with_baseline(
    //         &format!("Next bell\nLocal time\n"),
    //         Point::zero(),
    //         text_style,
    //         Baseline::Top
    //     ).draw(&mut self.driver)?;
    //
    //     self.driver.flush()
    // }
}
