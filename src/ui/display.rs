use crate::protocol::st25tb::ChipData;
use core::fmt::Write;
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
    text::Text,
};
use heapless::String;

pub struct Display<D> {
    driver: D,
    width: u32,
    height: u32,
}

impl<D> Display<D>
where
    D: DrawTarget<Color = Rgb565>,
{
    pub fn new(driver: D, width: u32, height: u32) -> Self {
        Self {
            driver,
            width,
            height,
        }
    }

    pub fn clear(&mut self) {
        let _ = self.driver.fill_solid(
            &Rectangle::new(Point::zero(), Size::new(self.width, self.height)),
            Rgb565::BLACK,
        );
    }

    fn clear_area(&mut self, x: i32, y: i32, w: u32, h: u32) {
        let _ = Rectangle::new(Point::new(x, y), Size::new(w, h))
            .into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK))
            .draw(&mut self.driver);
    }

    pub fn show_status(&mut self, msg: &str) {
        self.clear();
        let style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);
        let _ = Text::new(msg, Point::new(10, 20), style).draw(&mut self.driver);
    }

    pub fn show_chip_data(
        &mut self,
        data: &ChipData,
        selected_block: usize,
        selected_byte: usize,
        selected_nibble: usize,
        edit_mode: bool,
        force_clear: bool,
    ) {
        if force_clear {
            self.clear();
        }

        let header_style = MonoTextStyle::new(&FONT_6X10, Rgb565::CYAN);
        let normal_style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);
        let selected_style = MonoTextStyle::new(&FONT_6X10, Rgb565::YELLOW);
        let edit_style = MonoTextStyle::new(&FONT_6X10, Rgb565::GREEN);
        let dim_style = MonoTextStyle::new(&FONT_6X10, Rgb565::CSS_GRAY);

        self.clear_area(0, 0, self.width, 12);
        let mut header: String<48> = String::new();
        let _ = write!(
            header,
            "Blk:{:3}/{:3} Byte:{} {}",
            selected_block,
            data.block_count,
            selected_byte,
            if edit_mode { "[EDIT]" } else { "      " }
        );
        let _ = Text::new(&header, Point::new(5, 10), header_style).draw(&mut self.driver);

        let visible_rows = 25;
        let half = visible_rows / 2;

        let start = if selected_block <= half {
            0
        } else if selected_block >= data.block_count.saturating_sub(half) {
            data.block_count.saturating_sub(visible_rows)
        } else {
            selected_block - half
        };

        let mut y = 20;

        for i in start..(start + visible_rows).min(data.block_count) {
            let block = &data.blocks[i];
            let is_selected = i == selected_block;

            self.clear_area(0, y - 9, 145, 11);

            let mut idx_str: String<8> = String::new();
            let _ = write!(idx_str, "{:3}:", i);

            let line_style = if is_selected && !edit_mode {
                selected_style
            } else if block == &[0xFF, 0xFF, 0xFF, 0xFF] {
                dim_style
            } else {
                normal_style
            };

            let _ = Text::new(&idx_str, Point::new(2, y), line_style).draw(&mut self.driver);

            for b in 0..4 {
                let x = 28 + (b as i32) * 18;
                
                if is_selected && edit_mode && b == selected_byte {
                    let high_nibble = (block[b] >> 4) & 0x0F;
                    let low_nibble = block[b] & 0x0F;
                    
                    let mut high_str: String<2> = String::new();
                    let mut low_str: String<2> = String::new();
                    let _ = write!(high_str, "{:X}", high_nibble);
                    let _ = write!(low_str, "{:X}", low_nibble);
                    
                    let high_style = if selected_nibble == 0 { edit_style } else { selected_style };
                    let low_style = if selected_nibble == 1 { edit_style } else { selected_style };
                    
                    let _ = Text::new(&high_str, Point::new(x, y), high_style).draw(&mut self.driver);
                    let _ = Text::new(&low_str, Point::new(x + 6, y), low_style).draw(&mut self.driver);
                } else {
                    let mut byte_str: String<4> = String::new();
                    let _ = write!(byte_str, "{:02X}", block[b]);
                    
                    let byte_style = if is_selected && !edit_mode {
                        selected_style
                    } else if block == &[0xFF, 0xFF, 0xFF, 0xFF] {
                        dim_style
                    } else {
                        normal_style
                    };
                    
                    let _ = Text::new(&byte_str, Point::new(x, y), byte_style).draw(&mut self.driver);
                }
            }

            let mut ascii: String<8> = String::new();
            let _ = ascii.push('|');
            for b in 0..4 {
                let ch = block[b];
                let c = if ch >= 0x20 && ch < 0x7F {
                    ch as char
                } else {
                    '.'
                };
                let _ = ascii.push(c);
            }
            let _ = ascii.push('|');
            let _ = Text::new(&ascii, Point::new(105, y), line_style).draw(&mut self.driver);

            y += 11;
        }

        let decoder_y = 300;
        self.clear_area(0, decoder_y - 9, self.width, 12);
        let _ = Text::new("Decode:", Point::new(5, decoder_y), header_style).draw(&mut self.driver);

        let block = &data.blocks[selected_block];
        let mut hex_str: String<20> = String::new();
        let _ = write!(
            hex_str,
            "{:02X} {:02X} {:02X} {:02X}",
            block[0], block[1], block[2], block[3]
        );
        let _ = Text::new(&hex_str, Point::new(50, decoder_y), normal_style).draw(&mut self.driver);

        let mut ascii_str: String<16> = String::new();
        let _ = ascii_str.push_str("\"");
        for b in 0..4 {
            let ch = block[b];
            let c = if ch >= 0x20 && ch < 0x7F {
                ch as char
            } else {
                '.'
            };
            let _ = ascii_str.push(c);
        }
        let _ = ascii_str.push_str("\"");
        let _ = Text::new(&ascii_str, Point::new(130, decoder_y), selected_style)
            .draw(&mut self.driver);

        let u32_val = u32::from_le_bytes(*block);
        let mut dec_str: String<16> = String::new();
        let _ = write!(dec_str, "={:<12}", u32_val);
        let _ = Text::new(&dec_str, Point::new(168, decoder_y), dim_style).draw(&mut self.driver);

        let hint_style = MonoTextStyle::new(&FONT_6X10, Rgb565::CSS_GRAY);
        let hint = if edit_mode {
            "ROT:val BTN:byte BAK:done"
        } else {
            "ROT:blk BTN:edit BAK:menu"
        };
        let _ = Text::new(hint, Point::new(5, self.height as i32 - 3), hint_style)
            .draw(&mut self.driver);
    }

    pub fn show_menu(&mut self, items: &[&str], selected: usize) {
        self.clear();

        let title_style = MonoTextStyle::new(&FONT_6X10, Rgb565::CYAN);
        let _ = Text::new("ST25TB Reader", Point::new(40, 15), title_style).draw(&mut self.driver);

        let normal_style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);
        let selected_style = MonoTextStyle::new(&FONT_6X10, Rgb565::YELLOW);

        let mut y = 40;
        for (i, item) in items.iter().enumerate() {
            let style = if i == selected {
                selected_style
            } else {
                normal_style
            };
            let prefix = if i == selected { "> " } else { "  " };

            let mut line: String<32> = String::new();
            let _ = write!(line, "{}{}", prefix, item);
            let _ = Text::new(&line, Point::new(20, y), style).draw(&mut self.driver);
            y += 16;
        }
    }

    pub fn driver_mut(&mut self) -> &mut D {
        &mut self.driver
    }
}
