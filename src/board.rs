//! T-Embed CC1101 board pin definitions

pub mod pins {
    // I2C (PN532, Battery ICs)
    pub const I2C_SDA: u8 = 8;
    pub const I2C_SCL: u8 = 18;

    // PN532 NFC
    pub const PN532_IRQ: u8 = 17;
    pub const PN532_RF_REST: u8 = 45;
    pub const PN532_I2C_ADDR: u8 = 0x24;

    // Display ST7789 (320x170)
    pub const DISPLAY_WIDTH: u16 = 170;
    pub const DISPLAY_HEIGHT: u16 = 320;
    pub const DISPLAY_BL: u8 = 21;
    pub const DISPLAY_CS: u8 = 41;
    pub const DISPLAY_MOSI: u8 = 9;
    pub const DISPLAY_SCK: u8 = 11;
    pub const DISPLAY_DC: u8 = 16;
    pub const DISPLAY_RST: u8 = 40;

    // Rotary Encoder
    pub const ENCODER_A: u8 = 4;
    pub const ENCODER_B: u8 = 5;
    pub const ENCODER_BTN: u8 = 0;

    // Power
    pub const PWR_EN: u8 = 15;
    pub const USER_KEY: u8 = 6;
}
