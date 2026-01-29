#![no_std]
#![no_main]

extern crate alloc;

mod board;
mod drivers;
mod protocol;
mod ui;

use esp_alloc as _;
use esp_backtrace as _;

esp_bootloader_esp_idf::esp_app_desc!();
use esp_hal::{
    clock::CpuClock,
    delay::Delay,
    dma::DmaDescriptor,
    gpio::{Input, InputConfig, Level, Output, OutputConfig, Pull},
    i2c::master::I2c,
    i2s::master::{Channels, Config as I2sConfig, DataFormat, I2s},
    spi::master::Spi,
    time::Rate,
    usb_serial_jtag::UsbSerialJtag,
};
use log::info;

use display_interface_spi::SPIInterface;

use core::cell::RefCell;
use embedded_hal_bus::spi::{NoDelay, RefCellDevice};
use mipidsi::{options::ColorInversion, Builder};

use crate::board::pins;
use crate::drivers::{Audio, Pn532};
use crate::protocol::st25tb::ChipData;
use crate::protocol::St25tb;
use crate::ui::{ChipEditor, Display};

static mut TX_DESCRIPTORS: [DmaDescriptor; 8] = [DmaDescriptor::EMPTY; 8];

fn hex_char_to_nibble(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'A'..=b'F' => Some(c - b'A' + 10),
        b'a'..=b'f' => Some(c - b'a' + 10),
        _ => None,
    }
}

fn parse_hex_byte(high: u8, low: u8) -> Option<u8> {
    let h = hex_char_to_nibble(high)?;
    let l = hex_char_to_nibble(low)?;
    Some((h << 4) | l)
}

#[derive(Clone, Copy, PartialEq)]
enum AppState {
    Menu,
    Reading,
    Viewing,
    Writing,
    Error,
}

const MENU_ITEMS: [&str; 6] = [
    "Read Chip",
    "Write Chip",
    "Dump Serial",
    "Load Serial",
    "View Data",
    "Exit",
];

#[esp_hal::main]
fn main() -> ! {
    esp_alloc::heap_allocator!(size: 32 * 1024);

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_println::logger::init_logger_from_env();

    let mut delay = Delay::new();

    info!("Init power enable (GPIO15)...");
    let _pwr_en = Output::new(peripherals.GPIO15, Level::High, OutputConfig::default());

    info!("Init backlight...");
    let _bl = Output::new(peripherals.GPIO21, Level::High, OutputConfig::default());
    delay.delay_millis(100);

    info!("Init SPI...");

    info!("Setting all SPI CS pins HIGH to prevent bus contention...");
    let display_cs = Output::new(peripherals.GPIO41, Level::High, OutputConfig::default());
    let spi = Spi::new(
        peripherals.SPI2,
        esp_hal::spi::master::Config::default()
            .with_frequency(Rate::from_mhz(40))
            .with_mode(esp_hal::spi::Mode::_0),
    )
    .unwrap()
    .with_sck(peripherals.GPIO11)
    .with_mosi(peripherals.GPIO9)
    .with_miso(peripherals.GPIO10);

    let spi_bus: &'static RefCell<Spi<'static, esp_hal::Blocking>> = {
        static mut SPI_BUS: Option<RefCell<Spi<'static, esp_hal::Blocking>>> = None;
        unsafe {
            SPI_BUS = Some(RefCell::new(spi));
            SPI_BUS.as_ref().unwrap()
        }
    };

    let dc = Output::new(peripherals.GPIO16, Level::Low, OutputConfig::default());

    info!("Init display...");
    let display_spi = RefCellDevice::new(spi_bus, display_cs, NoDelay).unwrap();
    let spi_iface = SPIInterface::new(display_spi, dc);

    let display_driver = match Builder::new(mipidsi::models::ST7789, spi_iface)
        .display_size(pins::DISPLAY_WIDTH, pins::DISPLAY_HEIGHT)
        .display_offset(35, 0)
        .invert_colors(ColorInversion::Inverted)
        .init(&mut delay)
    {
        Ok(d) => {
            info!("Display OK");
            d
        }
        Err(e) => {
            info!("Display init failed: {:?}", e);
            loop {
                core::hint::spin_loop();
            }
        }
    };

    info!("Init I2C at 100kHz...");
    let i2c_config = esp_hal::i2c::master::Config::default().with_frequency(Rate::from_khz(100));
    let mut i2c = I2c::new(peripherals.I2C0, i2c_config)
        .unwrap()
        .with_sda(peripherals.GPIO8)
        .with_scl(peripherals.GPIO18);

    let pn532_irq = Input::new(
        peripherals.GPIO17,
        InputConfig::default().with_pull(Pull::Up),
    );
    let mut pn532_rst = Output::new(peripherals.GPIO45, Level::High, OutputConfig::default());

    pn532_rst.set_low();
    delay.delay_millis(100);
    pn532_rst.set_high();
    delay.delay_millis(500);

    info!("Scanning I2C bus...");
    for addr in 1..128 {
        let mut buf = [0u8; 1];
        if i2c.read(addr, &mut buf).is_ok() {
            info!("Found I2C device at 0x{:02X}", addr);
        }
    }
    info!("I2C scan complete");

    info!("Init PN532 at addr 0x{:02X}...", pins::PN532_I2C_ADDR);
    let mut pn532 = Pn532::new(i2c, pn532_irq, pn532_rst, pins::PN532_I2C_ADDR);

    if pn532.probe() {
        info!("PN532 found on I2C bus");
    } else {
        info!("PN532 NOT found on I2C bus!");
    }

    match pn532.init() {
        Ok(_) => info!("PN532 initialized"),
        Err(e) => {
            info!("PN532 init failed: {:?}", e);
        }
    }

    if let Ok((ic, ver, rev)) = pn532.get_firmware_version() {
        info!("PN532 IC:{:02X} Ver:{}.{}", ic, ver, rev);
    }

    let mut display = Display::new(
        display_driver,
        pins::DISPLAY_WIDTH as u32,
        pins::DISPLAY_HEIGHT as u32,
    );

    info!("Init I2S audio...");
    let i2s = I2s::new(
        peripherals.I2S0,
        peripherals.DMA_CH0,
        I2sConfig::new_tdm_philips()
            .with_sample_rate(Rate::from_hz(16000))
            .with_data_format(DataFormat::Data16Channel16)
            .with_channels(Channels::STEREO),
    )
    .unwrap();

    let i2s_tx = i2s
        .i2s_tx
        .with_bclk(peripherals.GPIO46)
        .with_ws(peripherals.GPIO40)
        .with_dout(peripherals.GPIO7)
        .build(unsafe { &mut *core::ptr::addr_of_mut!(TX_DESCRIPTORS) });

    let mut audio = Audio::new(i2s_tx);
    info!("I2S audio initialized");

    info!("Init USB Serial...");
    let mut usb_serial = UsbSerialJtag::new(peripherals.USB_DEVICE);

    let enc_a = Input::new(
        peripherals.GPIO4,
        InputConfig::default().with_pull(Pull::Up),
    );
    let enc_b = Input::new(
        peripherals.GPIO5,
        InputConfig::default().with_pull(Pull::Up),
    );
    let enc_btn = Input::new(
        peripherals.GPIO0,
        InputConfig::default().with_pull(Pull::Up),
    );
    let back_btn = Input::new(
        peripherals.GPIO6,
        InputConfig::default().with_pull(Pull::Up),
    );

    let mut state = AppState::Menu;
    let mut menu_selected: usize = 0;
    let mut editor: Option<ChipEditor> = None;
    let mut last_enc_a = enc_a.is_high();
    let mut btn_pressed = false;
    let mut back_pressed = false;

    display.show_menu(&MENU_ITEMS, menu_selected);

    loop {
        let enc_a_state = enc_a.is_high();
        if enc_a_state != last_enc_a && !enc_a_state {
            let direction = enc_b.is_high();

            match state {
                AppState::Menu => {
                    if direction {
                        if menu_selected > 0 {
                            menu_selected -= 1;
                        }
                    } else {
                        if menu_selected < MENU_ITEMS.len() - 1 {
                            menu_selected += 1;
                        }
                    }
                    display.show_menu(&MENU_ITEMS, menu_selected);
                }
                AppState::Viewing => {
                    if let Some(ref mut ed) = editor {
                        if direction {
                            ed.move_up();
                        } else {
                            ed.move_down();
                        }
                        display.show_chip_data(
                            &ed.data,
                            ed.selected_block,
                            ed.selected_byte,
                            ed.edit_mode,
                            false,
                        );
                    }
                }
                _ => {}
            }
        }
        last_enc_a = enc_a_state;

        let btn_state = enc_btn.is_low();
        if btn_state && !btn_pressed {
            btn_pressed = true;

            match state {
                AppState::Menu => match menu_selected {
                    0 => {
                        state = AppState::Reading;

                        for attempt in 1..=10 {
                            let mut msg: heapless::String<32> = heapless::String::new();
                            let _ =
                                core::fmt::write(&mut msg, format_args!("Read #{}...", attempt));
                            display.show_status(&msg);

                            let mut st25tb = St25tb::new(&mut pn532);
                            match st25tb.read_full_chip() {
                                Ok(data) => {
                                    info!("Chip read OK, UID: {:02X?}", data.uid);
                                    audio.beep();
                                    editor = Some(ChipEditor::new(data));
                                    state = AppState::Viewing;
                                    if let Some(ref ed) = editor {
                                        display.show_chip_data(
                                            &ed.data,
                                            ed.selected_block,
                                            ed.selected_byte,
                                            ed.edit_mode,
                                            true,
                                        );
                                    }
                                    break;
                                }
                                Err(e) => {
                                    info!("Read attempt {} error: {:?}", attempt, e);
                                    if attempt >= 10 {
                                        display.show_status("Read failed!");
                                        state = AppState::Error;
                                    }
                                    delay.delay_millis(200);
                                }
                            }
                        }
                    }
                    1 => {
                        if let Some(ref ed) = editor {
                            state = AppState::Writing;
                            display.show_status("Writing chip...");

                            let mut st25tb = St25tb::new(&mut pn532);
                            match st25tb.write_full_chip(&ed.data) {
                                Ok(_) => {
                                    info!("Chip written OK");
                                    display.show_status("Write OK!");
                                }
                                Err(e) => {
                                    info!("Write error: {:?}", e);
                                    display.show_status("Write failed!");
                                }
                            }
                            delay.delay_millis(1000);
                            state = AppState::Menu;
                            display.show_menu(&MENU_ITEMS, menu_selected);
                        } else {
                            display.show_status("No data to write!");
                            delay.delay_millis(1000);
                            display.show_menu(&MENU_ITEMS, menu_selected);
                        }
                    }
                    2 => {
                        if let Some(ref ed) = editor {
                            display.show_status("Dumping to Serial...");

                            info!("=== RFID DUMP START ===");
                            info!("UID: {:02X?}", ed.data.uid);
                            info!("Blocks: {}", ed.data.block_count);
                            info!("--- HEX DATA ---");
                            for i in 0..ed.data.block_count {
                                info!(
                                    "B{:03}: {:02X} {:02X} {:02X} {:02X}",
                                    i,
                                    ed.data.blocks[i][0],
                                    ed.data.blocks[i][1],
                                    ed.data.blocks[i][2],
                                    ed.data.blocks[i][3]
                                );
                            }
                            info!("=== RFID DUMP END ===");

                            display.show_status("Dump sent to Serial!");
                            audio.beep();
                            delay.delay_millis(1500);
                            display.show_menu(&MENU_ITEMS, menu_selected);
                        } else {
                            display.show_status("No data to dump!");
                            delay.delay_millis(1000);
                            display.show_menu(&MENU_ITEMS, menu_selected);
                        }
                    }
                    3 => {
                        display.show_status("Paste dump, END to finish");
                        info!("=== PASTE DUMP NOW ===");
                        info!("Format: B000: 0F FF FF FF");
                        info!("Type END when done");

                        let mut data = ChipData::default();
                        let mut line_buf: heapless::Vec<u8, 64> = heapless::Vec::new();
                        let mut loading = true;
                        let mut blocks_loaded = 0usize;

                        while loading {
                            if let Ok(c) = usb_serial.read_byte() {
                                if c == b'\n' || c == b'\r' {
                                    if line_buf.len() >= 3 {
                                        if &line_buf[..3] == b"END" || &line_buf[..3] == b"end" {
                                            loading = false;
                                            continue;
                                        }
                                        if line_buf.len() >= 4 && line_buf[0] == b'B' {
                                            if let (Some(h1), Some(h2), Some(h3)) = (
                                                hex_char_to_nibble(line_buf[1]),
                                                hex_char_to_nibble(line_buf[2]),
                                                hex_char_to_nibble(line_buf[3]),
                                            ) {
                                                let block_idx = ((h1 as usize) * 100)
                                                    + ((h2 as usize) * 10)
                                                    + (h3 as usize);
                                                if block_idx < 256 {
                                                    let hex_start = line_buf
                                                        .iter()
                                                        .position(|&x| x == b':')
                                                        .map(|p| p + 1);
                                                    if let Some(start) = hex_start {
                                                        let hex_chars: heapless::Vec<u8, 16> =
                                                            line_buf[start..]
                                                                .iter()
                                                                .filter(|&&ch| ch != b' ')
                                                                .cloned()
                                                                .collect();
                                                        if hex_chars.len() >= 8 {
                                                            if let (
                                                                Some(b0),
                                                                Some(b1),
                                                                Some(b2),
                                                                Some(b3),
                                                            ) = (
                                                                parse_hex_byte(
                                                                    hex_chars[0],
                                                                    hex_chars[1],
                                                                ),
                                                                parse_hex_byte(
                                                                    hex_chars[2],
                                                                    hex_chars[3],
                                                                ),
                                                                parse_hex_byte(
                                                                    hex_chars[4],
                                                                    hex_chars[5],
                                                                ),
                                                                parse_hex_byte(
                                                                    hex_chars[6],
                                                                    hex_chars[7],
                                                                ),
                                                            ) {
                                                                data.blocks[block_idx] =
                                                                    [b0, b1, b2, b3];
                                                                if block_idx >= data.block_count {
                                                                    data.block_count =
                                                                        block_idx + 1;
                                                                }
                                                                blocks_loaded += 1;
                                                                info!("Loaded B{:03}: {:02X} {:02X} {:02X} {:02X}", block_idx, b0, b1, b2, b3);
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        if line_buf.len() >= 4 && &line_buf[..4] == b"UID:" {
                                            info!("UID line detected (skipped)");
                                        }
                                    }
                                    line_buf.clear();
                                } else if line_buf.len() < 64 {
                                    let _ = line_buf.push(c);
                                }
                            }

                            if back_btn.is_low() {
                                loading = false;
                                info!("Load cancelled by user");
                            }

                            delay.delay_millis(1);
                        }

                        if blocks_loaded > 0 {
                            info!("=== LOAD COMPLETE: {} blocks ===", blocks_loaded);
                            editor = Some(ChipEditor::new(data));
                            audio.beep();
                            let mut msg: heapless::String<32> = heapless::String::new();
                            let _ = core::fmt::write(
                                &mut msg,
                                format_args!("Loaded {} blocks!", blocks_loaded),
                            );
                            display.show_status(&msg);
                        } else {
                            display.show_status("No data loaded");
                        }
                        delay.delay_millis(1500);
                        display.show_menu(&MENU_ITEMS, menu_selected);
                    }
                    4 => {
                        if let Some(ref ed) = editor {
                            state = AppState::Viewing;
                            display.show_chip_data(
                                &ed.data,
                                ed.selected_block,
                                ed.selected_byte,
                                ed.edit_mode,
                                true,
                            );
                        } else {
                            display.show_status("No data loaded!");
                            delay.delay_millis(1000);
                            display.show_menu(&MENU_ITEMS, menu_selected);
                        }
                    }
                    5 => {
                        display.show_status("Goodbye!");
                        loop {
                            core::hint::spin_loop();
                        }
                    }
                    _ => {}
                },
                AppState::Viewing => {
                    if let Some(ref mut ed) = editor {
                        ed.toggle_edit_mode();
                        display.show_chip_data(
                            &ed.data,
                            ed.selected_block,
                            ed.selected_byte,
                            ed.edit_mode,
                            false,
                        );
                    }
                }
                AppState::Error => {
                    state = AppState::Menu;
                    display.show_menu(&MENU_ITEMS, menu_selected);
                }
                _ => {}
            }
        }
        if !btn_state {
            btn_pressed = false;
        }

        let back_state = back_btn.is_low();
        if back_state && !back_pressed {
            back_pressed = true;
            match state {
                AppState::Viewing => {
                    if let Some(ref mut ed) = editor {
                        if ed.edit_mode {
                            ed.exit_edit_mode();
                            display.show_chip_data(
                                &ed.data,
                                ed.selected_block,
                                ed.selected_byte,
                                ed.edit_mode,
                                false,
                            );
                        } else {
                            state = AppState::Menu;
                            display.show_menu(&MENU_ITEMS, menu_selected);
                        }
                    }
                }
                AppState::Error | AppState::Reading | AppState::Writing => {
                    state = AppState::Menu;
                    display.show_menu(&MENU_ITEMS, menu_selected);
                }
                _ => {}
            }
        }
        if !back_state {
            back_pressed = false;
        }

        delay.delay_millis(10);
    }
}
