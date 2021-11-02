#![allow(unreachable_code)]
use embedded_graphics::{pixelcolor::Rgb565, prelude::*};
use rppal::{
    gpio::{self, Gpio, Level, Trigger},
    hal::Delay,
};
use st7789::{Orientation, ST7789};
use std::{error::Error, thread, time::Duration};
mod utils;
use utils::{chart, iptext, Pins};
static mut KEY1_PRESSED: bool = false;
fn main() -> Result<(), Box<dyn Error>> {
    let mut key1 = Gpio::new()?.get(Pins::KEY1 as u8)?.into_input_pullup();
    // key1.set_reset_on_drop(false);
    key1.set_async_interrupt(Trigger::RisingEdge, move |level: Level| {
        thread::sleep(Duration::from_millis(8));
        if level == gpio::Level::High {
            unsafe {
                KEY1_PRESSED = !KEY1_PRESSED;
                println!("{}", KEY1_PRESSED);
            }
        }
        ()
    })?;
    let (size_x, size_y) = (240i32, 240i32);
    let spi = utils::create_spi()?;
    let [rst, bl, dc] = utils::create_outputpin([
        Pins::RST as u8, // Reset
        Pins::BL as u8,  // Backlight
        Pins::DC as u8,  // DC
    ])?;
    let mut display = ST7789::new(
        spi,
        Some(rst),
        Some(bl),
        Some(dc),
        size_x as u16,
        size_y as u16,
    );

    // initialize
    let mut delay = Delay::new();
    display.init(&mut delay).unwrap();
    // set default orientation
    display.set_orientation(Orientation::Landscape).unwrap();
    display.clear(Rgb565::BLACK).unwrap();

    let (mut x, mut y) = (0i32, 0i32);
    loop {
        let points = [Point::new(x, y), Point::new(x + 1, y + 1)];
        x += 1;
        y += 1;
        chart::line_chart_demo(&mut display, &points).unwrap();
        unsafe{
        if KEY1_PRESSED {
            iptext::draw_ip(&mut display).unwrap();
        }

        }
        thread::sleep(Duration::from_millis(5));
        if x == 239 {
            display.clear(Rgb565::BLACK).unwrap();
            x = 0;
            y = 0;
        }
        // display.clear(Rgb565::BLACK).unwrap();
    }

    Ok(())
}
