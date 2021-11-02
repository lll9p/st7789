#[allow(dead_code)]
#[repr(u8)]
pub enum Pins {
    KEY1 = 21u8,  // Key1
    KEY2 = 20u8,  // Key2
    KEY3 = 16u8,  // Key3
    UP = 6u8,     // Up
    DOWN = 19u8,  // Down
    LEFT = 5u8,   // Left
    RIGHT = 26u8, // Right
    PRESS = 13u8, // Press
    SCLK = 11u8,  // Spi SCLK
    MOSI = 10u8,  // Spi MOSI
    CS = 8u8,     // Slave select/ CE0
    DC = 25u8,    // Data
    RST = 27u8,   // Reset
    BL = 24u8,    // Backlight
}
