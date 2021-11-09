use embedded_graphics::primitives::Rectangle;
use linux_embedded_hal::{
    spidev::{SpiModeFlags, SpidevOptions},
    Spidev,
};
use rppal::gpio::{self, Gpio};
use std::{boxed::Box, error::Error, io};
pub mod chart;
mod enums;
pub mod iptext;
pub use enums::Pins;
pub struct BoundingBoxes {
    pub ip: Option<Rectangle>,
    pub chart: Option<Rectangle>,
}
pub fn create_spi() -> io::Result<Spidev> {
    let mut spi = Spidev::open("/dev/spidev0.0")?;
    let options = SpidevOptions::new()
        .bits_per_word(8)
        .max_speed_hz(80_000_000)
        .lsb_first(false)
        .mode(SpiModeFlags::SPI_MODE_0)
        .build();
    spi.configure(&options)?;
    Ok(spi)
}
pub fn create_outputpin(pins: [u8; 3]) -> Result<[gpio::OutputPin; 3], Box<dyn Error + 'static>> {
    let [rst, bl, dc] = pins;
    Ok([
        Gpio::new()?.get(rst)?.into_output(),
        Gpio::new()?.get(bl)?.into_output(),
        Gpio::new()?.get(dc)?.into_output(),
    ])
}
