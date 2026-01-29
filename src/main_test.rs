#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::delay::Delay;

esp_bootloader_esp_idf::esp_app_desc!();

#[esp_hal::main]
fn main() -> ! {
    let _peripherals = esp_hal::init(esp_hal::Config::default());

    esp_println::println!("Hello from ESP32-S3!");

    let delay = Delay::new();

    loop {
        esp_println::println!("Alive!");
        delay.delay_millis(1000);
    }
}
