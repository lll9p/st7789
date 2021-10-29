use embedded_graphics::mono_font::{ascii::FONT_10X20, MonoTextStyle};
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
// use embedded_graphics::primitives::*;
use embedded_graphics::text::Text;
use rppal::gpio::Gpio;
use rppal::hal::Delay;
use rppal::spi::{Bus, Mode, SlaveSelect, Spi};
use st7789::{Orientation, ST7789};

use local_ip_address::local_ip;
use std::error::Error;
use std::thread;
use std::time::Duration;
/* 功能引脚  树莓派接口(BCM)  描述
KEY1  P21  按键1GPIO
KEY2  P20  按键2GPIO
KEY3  P16  按键3GPIO
摇杆UP  P6  摇杆上
摇杆Down  P19  摇杆下
摇杆Left  P5  摇杆左
摇杆Right  P26  摇杆右
摇杆Press  P13  摇杆按下
SCLK  P11/SCLK  SPI时钟线
MOSI  P10/MOSI  SPI数据线
CS  P8/CE0  片选
DC  P25  数据/命令选择
RST  P27  复位
BL  P24  背光  */
const GPIO_DC: u8 = 25; // 数据
const GPIO_CS: u8 = 8; // 数据
const GPIO_BL: u8 = 24; // 背光
const GPIO_RST: u8 = 27; // 背光

fn main() -> Result<(), Box<dyn Error>> {
    // max speed in hz ->40M;
    //40_000_000
    let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 80_000_000, Mode::Mode0)?;
    let dc = Gpio::new()?.get(GPIO_DC)?.into_output();
    let bl = Gpio::new()?.get(GPIO_BL)?.into_output();
    let rst = Gpio::new()?.get(GPIO_RST)?.into_output();
    let cs = Gpio::new()?.get(GPIO_CS)?.into_output();
    // let di = SPIInterfaceNoCS::new(spi, dc);
    let mut display = ST7789::new(spi, Some(rst), Some(bl), Some(dc), Some(cs), 240 as u16, 240 as u16);

    // initialize
    let mut delay = Delay::new();
    display.init(&mut delay).unwrap();
    // set default orientation
    display.set_orientation(Orientation::Landscape).unwrap();
    display.clear(Rgb565::BLACK).unwrap();

    // Create a new character style
    let style = MonoTextStyle::new(&FONT_10X20, Rgb565::RED);
    let mut i = 0u8;
    loop {
        // let my_local_ip: String = local_ip().unwrap().to_string();
        let ip = local_ip();
        let pi_ip = match ip {
            Ok(ip) => ip.to_string(),
            _ => "IP NOT FOUND.".to_string(),
        };
        // println!("Got IP!");
        // Create a text at position (20, 30) and draw it using the previously defined style
        Text::new(&pi_ip[..], Point::new(0, 20), style).draw(&mut display);
        thread::sleep(Duration::from_millis(50));
        let color = match i {
            0 => Rgb565::BLACK,
            1 => Rgb565::RED,
            2 => Rgb565::GREEN,
            3 => Rgb565::BLUE,
            4 => Rgb565::YELLOW,
            5 => Rgb565::MAGENTA,
            6 => Rgb565::CYAN,
            7 => Rgb565::WHITE,
            _ => {
                i = 0;
                Rgb565::BLACK
            }
        };
        i+=1;
        display.clear(Rgb565::BLACK);
    }

    // display.clear(Rgb565::BLACK).unwrap();
    Ok(())
}
