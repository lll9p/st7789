#![allow(unreachable_code)]
use embedded_graphics::{pixelcolor::Rgb565, prelude::*, primitives::PrimitiveStyle};
use rppal::{
    gpio::{self, Gpio, Level, Trigger},
    hal::Delay,
};
use st7789::{Orientation, TearingEffect, ST7789};
use std::{error::Error, thread, time::Duration};
mod utils;
use std::sync::{Arc, Mutex};
use utils::{chart, iptext, Pins};

fn main() -> Result<(), Box<dyn Error>> {
    // Keys innitials
    let key1_pressed = Arc::new(Mutex::new(false));
    // let (sender, receiver) = mpsc::channel();
    let mut key1 = Gpio::new()?.get(Pins::KEY1 as u8)?.into_input_pullup();
    {
        let pressed = Arc::clone(&key1_pressed);
        key1.set_async_interrupt(Trigger::RisingEdge, move |level: Level| {
            thread::sleep(Duration::from_millis(8));
            if level == gpio::Level::High {
                let mut flag = (*pressed).lock().unwrap();
                *flag = !(*flag);
            }
            ()
        })?;
    }
    let key2_pressed = Arc::new(Mutex::new(false));
    // let (sender, receiver) = mpsc::channel();
    let mut key2 = Gpio::new()?.get(Pins::KEY2 as u8)?.into_input_pullup();
    {
        let pressed = Arc::clone(&key2_pressed);
        key2.set_async_interrupt(Trigger::RisingEdge, move |level: Level| {
            thread::sleep(Duration::from_millis(8));
            if level == gpio::Level::High {
                let mut flag = (*pressed).lock().unwrap();
                *flag = !(*flag);
            }
            ()
        })?;
    }

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
    display
        .set_tearing_effect(TearingEffect::HorizontalAndVertical)
        .unwrap();
    display.clear(Rgb565::BLUE).unwrap();
    // Read test data
    let mut data_feeder = chart::DataFeeder::new("test.txt")?;
    // let y = data_feeder.read()?;
    // let y = 239 - (y.round() as i32) / 20;
    // let mut x = 0i32;
    let style = PrimitiveStyle::with_stroke(Rgb565::RED, 1);
    let clear_style_ip = PrimitiveStyle::with_fill(Rgb565::BLUE);
    let mut line_chart = chart::LineChart::new(style);
    let mut bounding_boxes = utils::BoundingBoxes {
        chart: None,
        ip: None,
    };
    loop {
        if let Ok(mut flag) = key1_pressed.clone().lock() {
            if *flag {
                iptext::draw_ip(&mut display, &mut bounding_boxes).unwrap();
                *flag = false;
            }
        }
        if let Ok(mut flag) = key2_pressed.clone().lock() {
            if *flag {
                if let Some(clear_bbox) = bounding_boxes.ip {
                    clear_bbox
                        .into_styled(clear_style_ip)
                        .draw(&mut display)
                        .unwrap();
                }
                *flag = false;
            }
        }
        let y = data_feeder.read()?;
        // let y = 239 - (y.round() as i32) / 20;
        line_chart.draw_data(y, &mut display).unwrap();
        // x += 1;
        // thread::sleep(Duration::from_millis(5));
    }

    Ok(())
}
