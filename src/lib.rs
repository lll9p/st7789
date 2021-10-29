#![no_std]
// associated re-typing not supported in rust yet
#![allow(clippy::type_complexity)]

//! This crate provides a ST7789 driver to connect to TFT displays.

pub mod enums;

pub use crate::enums::{
    BacklightState, DataFormat, DisplayError, Error, Instruction, Orientation, TearingEffect,
};
use core::iter::once;

use embedded_hal::delay::blocking::DelayUs;
use embedded_hal::digital::blocking::OutputPin;
use embedded_hal::spi::blocking as spi;
#[cfg(feature = "graphics")]
mod graphics;

#[cfg(feature = "batch")]
mod batch;

///
/// ST7789 driver to connect to TFT displays.
///
pub struct ST7789<SPI, OUT>
where
    SPI: spi::Write<u8>,
    OUT: OutputPin,
{
    // Display interface
    spi: SPI,
    // Reset pin.
    rst: Option<OUT>,
    // Backlight pin,
    bl: Option<OUT>,
    // Data pin,
    dc: Option<OUT>,
    cs: Option<OUT>,
    // Visible size (x, y)
    size_x: u16,
    size_y: u16,
    // Current orientation
    orientation: Orientation,
}

type Result_ = core::result::Result<(), DisplayError>;

fn send_u8<SPI: spi::Write<u8>>(spi: &mut SPI, words: DataFormat<'_>) -> Result_ {
    match words {
        DataFormat::U8(slice) => spi.write(slice).map_err(|_| DisplayError::BusWriteError),
        DataFormat::U16(slice) => {
            use byte_slice_cast::*;
            spi.write(slice.as_byte_slice())
                .map_err(|_| DisplayError::BusWriteError)
        }
        DataFormat::U16LE(slice) => {
            use byte_slice_cast::*;
            for v in slice.as_mut() {
                *v = v.to_le();
            }
            spi.write(slice.as_byte_slice())
                .map_err(|_| DisplayError::BusWriteError)
        }
        DataFormat::U16BE(slice) => {
            use byte_slice_cast::*;
            for v in slice.as_mut() {
                *v = v.to_be();
            }
            spi.write(slice.as_byte_slice())
                .map_err(|_| DisplayError::BusWriteError)
        }
        DataFormat::U8Iter(iter) => {
            let mut buf = [0; 32];
            let mut i = 0;

            for v in iter.into_iter() {
                buf[i] = v;
                i += 1;

                if i == buf.len() {
                    spi.write(&buf).map_err(|_| DisplayError::BusWriteError)?;
                    i = 0;
                }
            }

            if i > 0 {
                spi.write(&buf[..i])
                    .map_err(|_| DisplayError::BusWriteError)?;
            }

            Ok(())
        }
        DataFormat::U16LEIter(iter) => {
            use byte_slice_cast::*;
            let mut buf = [0; 32];
            let mut i = 0;

            for v in iter.map(u16::to_le) {
                buf[i] = v;
                i += 1;

                if i == buf.len() {
                    spi.write(&buf.as_byte_slice())
                        .map_err(|_| DisplayError::BusWriteError)?;
                    i = 0;
                }
            }

            if i > 0 {
                spi.write(&buf[..i].as_byte_slice())
                    .map_err(|_| DisplayError::BusWriteError)?;
            }

            Ok(())
        }
        DataFormat::U16BEIter(iter) => {
            use byte_slice_cast::*;
            let mut buf = [0; 64];
            let mut i = 0;
            let len = buf.len();

            for v in iter.map(u16::to_be) {
                buf[i] = v;
                i += 1;

                if i == len {
                    spi.write(&buf.as_byte_slice())
                        .map_err(|_| DisplayError::BusWriteError)?;
                    i = 0;
                }
            }

            if i > 0 {
                spi.write(&buf[..i].as_byte_slice())
                    .map_err(|_| DisplayError::BusWriteError)?;
            }

            Ok(())
        }
        _ => Err(DisplayError::DataFormatNotImplemented),
    }
}
impl<SPI, OUT, PinE> ST7789<SPI, OUT>
where
    SPI: spi::Write<u8>,
    OUT: OutputPin<Error = PinE>,
{
    ///
    /// Creates a new ST7789 driver instance
    ///
    /// # Arguments
    ///
    /// * `di` - a display interface for talking with the display
    /// * `rst` - display hard reset pin
    /// * `bl` - backlight pin
    /// * `size_x` - x axis resolution of the display in pixels
    /// * `size_y` - y axis resolution of the display in pixels
    ///
    pub fn new(
        spi: SPI,
        rst: Option<OUT>,
        bl: Option<OUT>,
        dc: Option<OUT>,
        cs: Option<OUT>,
        size_x: u16,
        size_y: u16,
    ) -> Self {
        Self {
            spi,
            rst,
            bl,
            dc,
            cs,
            size_x,
            size_y,
            orientation: Orientation::default(),
        }
    }

    ///
    /// Runs commands to initialize the display
    ///
    /// # Arguments
    ///
    /// * `delay_source` - mutable reference to a delay provider
    ///
    pub fn init(&mut self, delay_source: &mut impl DelayUs<u32>) -> Result<(), Error<PinE>> {
        self.hard_reset(delay_source)?;
        if let Some(bl) = self.bl.as_mut() {
            bl.set_low().map_err(Error::Pin)?;
            delay_source.delay_us(10_000).unwrap();
            bl.set_high().map_err(Error::Pin)?;
        }

        self.write_command(Instruction::SWRESET)?; // reset display
        delay_source.delay_us(150_000).unwrap();
        self.write_command(Instruction::RAMWR)?; // Init ram
        delay_source.delay_us(10_000).unwrap();
        self.write_command(Instruction::SLPOUT)?; // turn off sleep
        delay_source.delay_us(10_000).unwrap();
        self.write_command(Instruction::INVOFF)?; // turn off invert
        self.write_command(Instruction::VSCRDER)?; // vertical scroll definition
        self.write_data(&[0u8, 0u8, 0x14u8, 0u8, 0u8, 0u8])?; // 0 TSA, 320 VSA, 0 BSA
        self.write_command(Instruction::MADCTL)?; // left -> right, bottom -> top RGB
        self.write_data(&[0b0000_0000])?;
        self.write_command(Instruction::COLMOD)?; // 16bit 65k colors
        self.write_data(&[0b0101_0101])?;
        self.write_command(Instruction::INVON)?; // hack?
        delay_source.delay_us(10_000).unwrap();
        self.write_command(Instruction::NORON)?; // turn on display
        delay_source.delay_us(10_000).unwrap();
        self.write_command(Instruction::DISPON)?; // turn on display
        delay_source.delay_us(10_000).unwrap();
        self.write_command(Instruction::PORCTRL)?; // porch control
        self.write_data(&[0x0Cu8, 0x0Cu8, 0x00u8, 0x33u8, 0x33u8])?;
        delay_source.delay_us(10_000).unwrap();
        self.write_command(Instruction::GCTRL)?; // gate control
        self.write_data(&[0x35u8])?;
        delay_source.delay_us(10_000).unwrap();
        self.write_command(Instruction::VCOMS)?; // VCOM Setting
        self.write_data(&[0x37u8])?;
        delay_source.delay_us(10_000).unwrap();
        self.write_command(Instruction::LCMCTRL)?; // LCM Control
        self.write_data(&[0x2Cu8])?;
        delay_source.delay_us(10_000).unwrap();
        self.write_command(Instruction::VDVVRHEN)?; // VDV and VRH Command Enable
        self.write_data(&[0x01u8])?;
        delay_source.delay_us(10_000).unwrap();
        self.write_command(Instruction::VRHS)?; // VRH set
        self.write_data(&[0x12u8])?;
        delay_source.delay_us(10_000).unwrap();
        self.write_command(Instruction::VDVS)?; // VDV SET
        self.write_data(&[0x20u8])?;
        delay_source.delay_us(10_000).unwrap();
        self.write_command(Instruction::FRCTRL2)?; // Frame Rate Control in Normal Mode
        self.write_data(&[0x0Fu8])?;
        delay_source.delay_us(10_000).unwrap();
        self.write_command(Instruction::PWCTRL1)?; // Power Control 1
        self.write_data(&[0xA4u8, 0xA1u8])?;
        delay_source.delay_us(10_000).unwrap();
        self.write_command(Instruction::PVGAMCTRL)?; // Positive Voltage Gamma Control
        self.write_data(&[
            0xD0u8, 0x04u8, 0x0Du8, 0x11u8, 0x13u8, 0x2Bu8, 0x3Fu8, 0x54u8, 0x4Cu8, 0x18u8, 0x0Du8,
            0x0Bu8, 0x1Fu8, 0x23u8,
        ])?;
        delay_source.delay_us(10_000).unwrap();
        self.write_command(Instruction::NVGAMCTRL)?; // Negative Voltage Gamma Control
        self.write_data(&[
            0xD0u8, 0x04u8, 0x0Cu8, 0x11u8, 0x13u8, 0x2Cu8, 0x3Fu8, 0x44u8, 0x51u8, 0x2Fu8, 0x1Fu8,
            0x1Fu8, 0x20u8, 0x23u8,
        ])?;
        delay_source.delay_us(10_000).unwrap();
        self.write_command(Instruction::INVON)?; // hack?
        delay_source.delay_us(10_000).unwrap();
        self.write_command(Instruction::DISPON)?; // Turn on display
        delay_source.delay_us(10_000).unwrap();
        Ok(())
    }

    ///
    /// Performs a hard reset using the RST pin sequence
    ///
    /// # Arguments
    ///
    /// * `delay_source` - mutable reference to a delay provider
    ///
    pub fn hard_reset(&mut self, delay_source: &mut impl DelayUs<u32>) -> Result<(), Error<PinE>> {
        if let Some(rst) = self.rst.as_mut() {
            rst.set_high().map_err(Error::Pin)?;
            delay_source.delay_us(10).unwrap(); // ensure the pin change will get registered
            rst.set_low().map_err(Error::Pin)?;
            delay_source.delay_us(10).unwrap(); // ensure the pin change will get registered
            rst.set_high().map_err(Error::Pin)?;
            delay_source.delay_us(10).unwrap(); // ensure the pin change will get registered
        }

        Ok(())
    }

    pub fn set_backlight(
        &mut self,
        state: BacklightState,
        delay_source: &mut impl DelayUs<u32>,
    ) -> Result<(), Error<PinE>> {
        if let Some(bl) = self.bl.as_mut() {
            match state {
                BacklightState::On => bl.set_high().map_err(Error::Pin)?,
                BacklightState::Off => bl.set_low().map_err(Error::Pin)?,
            }
            delay_source.delay_us(10).unwrap(); // ensure the pin change will get registered
        }
        Ok(())
    }

    ///
    /// Returns currently set orientation
    ///
    pub fn orientation(&self) -> Orientation {
        self.orientation
    }

    ///
    /// Sets display orientation
    ///
    pub fn set_orientation(&mut self, orientation: Orientation) -> Result<(), Error<PinE>> {
        self.write_command(Instruction::MADCTL)?;
        self.write_data(&[orientation as u8])?;
        self.orientation = orientation;
        Ok(())
    }

    ///
    /// Sets a pixel color at the given coords.
    ///
    /// # Arguments
    ///
    /// * `x` - x coordinate
    /// * `y` - y coordinate
    /// * `color` - the Rgb565 color value
    ///
    pub fn set_pixel(&mut self, x: u16, y: u16, color: u16) -> Result<(), Error<PinE>> {
        self.set_address_window(x, y, x, y)?;
        self.write_command(Instruction::RAMWR)?;
        self.send_data(DataFormat::U16BEIter(&mut once(color)))
            .map_err(|_| Error::DisplayError)?;

        Ok(())
    }

    ///
    /// Sets pixel colors in given rectangle bounds.
    ///
    /// # Arguments
    ///
    /// * `sx` - x coordinate start
    /// * `sy` - y coordinate start
    /// * `ex` - x coordinate end
    /// * `ey` - y coordinate end
    /// * `colors` - anything that can provide `IntoIterator<Item = u16>` to iterate over pixel data
    ///
    pub fn set_pixels<T>(
        &mut self,
        sx: u16,
        sy: u16,
        ex: u16,
        ey: u16,
        colors: T,
    ) -> Result<(), Error<PinE>>
    where
        T: IntoIterator<Item = u16>,
    {
        self.set_address_window(sx, sy, ex, ey)?;
        self.write_command(Instruction::RAMWR)?;
        self.send_data(DataFormat::U16BEIter(&mut colors.into_iter()))
            .map_err(|_| Error::DisplayError)
    }

    ///
    /// Sets scroll offset "shifting" the displayed picture
    /// # Arguments
    ///
    /// * `offset` - scroll offset in pixels
    ///
    pub fn set_scroll_offset(&mut self, offset: u16) -> Result<(), Error<PinE>> {
        self.write_command(Instruction::VSCAD)?;
        self.write_data(&offset.to_be_bytes())
    }

    ///
    /// Release resources allocated to this driver back.
    /// This returns the display interface and the RST pin deconstructing the driver.
    ///
    pub fn release(self) -> (SPI, Option<OUT>, Option<OUT>) {
        (self.spi, self.rst, self.bl)
    }

    fn write_command(&mut self, command: Instruction) -> Result<(), Error<PinE>> {
        self.send_commands(DataFormat::U8Iter(&mut once(command as u8)))
            .map_err(|_| Error::DisplayError)?;
        Ok(())
    }

    fn write_data(&mut self, data: &[u8]) -> Result<(), Error<PinE>> {
        self.send_data(DataFormat::U8Iter(&mut data.iter().cloned()))
            .map_err(|_| Error::DisplayError)
    }
    fn send_commands(&mut self, cmds: DataFormat<'_>) -> Result_ {
        // Assert chip select pin
        if let Some(cs) = self.cs.as_mut() {
            cs.set_low().map_err(|_| DisplayError::CSError)?;
        }
        // 1 = data, 0 = command
        if let Some(dc) = self.dc.as_mut() {
            dc.set_low().map_err(|_| DisplayError::DCError)?;
        }

        // Send words over SPI
        let result = send_u8(&mut self.spi, cmds);
        // Deassert chip select pin
        if let Some(cs) = self.cs.as_mut() {
            cs.set_high().ok();
        }
        result
    }

    fn send_data(&mut self, buf: DataFormat<'_>) -> Result_ {
        // Assert chip select pin
        if let Some(cs) = self.cs.as_mut() {
            cs.set_low().map_err(|_| DisplayError::CSError)?;
        }
        // 1 = data, 0 = command
        if let Some(dc) = self.dc.as_mut() {
            dc.set_high().map_err(|_| DisplayError::DCError)?;
        }

        // Send words over SPI
        let result = send_u8(&mut self.spi, buf);
        // Deassert chip select pin
        if let Some(cs) = self.cs.as_mut() {
            cs.set_high().ok();
        }
        result
    }
    // Sets the address window for the display.
    fn set_address_window(
        &mut self,
        sx: u16,
        sy: u16,
        ex: u16,
        ey: u16,
    ) -> Result<(), Error<PinE>> {
        self.write_command(Instruction::CASET)?;
        self.write_data(&sx.to_be_bytes())?;
        self.write_data(&ex.to_be_bytes())?;
        self.write_command(Instruction::RASET)?;
        self.write_data(&sy.to_be_bytes())?;
        self.write_data(&ey.to_be_bytes())
    }

    ///
    /// Configures the tearing effect output.
    ///
    pub fn set_tearing_effect(&mut self, tearing_effect: TearingEffect) -> Result<(), Error<PinE>> {
        match tearing_effect {
            TearingEffect::Off => self.write_command(Instruction::TEOFF),
            TearingEffect::Vertical => {
                self.write_command(Instruction::TEON)?;
                self.write_data(&[0])
            }
            TearingEffect::HorizontalAndVertical => {
                self.write_command(Instruction::TEON)?;
                self.write_data(&[1])
            }
        }
    }
}
