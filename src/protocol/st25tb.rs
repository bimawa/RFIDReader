use crate::drivers::{pn532::Pn532Error, Pn532};
use embedded_hal::digital::{InputPin, OutputPin};
use embedded_hal::i2c::I2c;

const CMD_INITIATE: u8 = 0x06;
const CMD_SELECT: u8 = 0x0E;
const CMD_GET_UID: u8 = 0x0B;
const CMD_READ_BLOCK: u8 = 0x08;
const CMD_WRITE_BLOCK: u8 = 0x09;
const CMD_COMPLETION: u8 = 0x0F;

#[derive(Debug, Clone)]
pub struct ChipData {
    pub chip_id: u8,
    pub uid: [u8; 8],
    pub blocks: [[u8; 4]; 256],
    pub block_count: usize,
}

impl Default for ChipData {
    fn default() -> Self {
        Self {
            chip_id: 0,
            uid: [0u8; 8],
            blocks: [[0u8; 4]; 256],
            block_count: 0,
        }
    }
}

pub struct St25tb<'a, I2C, IRQ, RST> {
    pn532: &'a mut Pn532<I2C, IRQ, RST>,
    chip_id: u8,
}

impl<'a, I2C, IRQ, RST> St25tb<'a, I2C, IRQ, RST>
where
    I2C: I2c,
    IRQ: InputPin,
    RST: OutputPin,
{
    pub fn new(pn532: &'a mut Pn532<I2C, IRQ, RST>) -> Self {
        Self { pn532, chip_id: 0 }
    }

    fn delay_ms(&self, ms: u32) {
        esp_hal::rom::ets_delay_us(ms * 1000);
    }

    pub fn initiate(&mut self, force: bool) -> Result<u8, Pn532Error> {
        if force {
            self.pn532.rf_configuration_retries(0xFF)?;
        }
        self.pn532.rf_configuration_timing(0x00, 0x0B)?;

        let cmd = [CMD_INITIATE, 0x00];

        for attempt in 0..5 {
            match self.pn532.communicate_thru(&cmd) {
                Ok(response) if response.len() >= 1 => {
                    self.chip_id = response[0];
                    if force {
                        let _ = self.pn532.rf_configuration_retries(0x00);
                    }
                    return Ok(self.chip_id);
                }
                _ => {
                    self.delay_ms(50 * (attempt + 1));
                }
            }
        }

        Err(Pn532Error::InvalidResponse)
    }

    pub fn select(&mut self, chip_id: Option<u8>) -> Result<(), Pn532Error> {
        let id = chip_id.unwrap_or(self.chip_id);
        self.pn532.rf_configuration_timing(0x00, 0x08)?;

        let cmd = [CMD_SELECT, id];
        let response = self.pn532.communicate_thru(&cmd)?;

        if response.len() >= 1 && response[0] == id {
            Ok(())
        } else {
            Err(Pn532Error::InvalidResponse)
        }
    }

    pub fn get_uid(&mut self) -> Result<[u8; 8], Pn532Error> {
        self.pn532.rf_configuration_timing(0x00, 0x07)?;

        let cmd = [CMD_GET_UID];
        let response = self.pn532.communicate_thru(&cmd)?;

        if response.len() >= 8 {
            let mut uid = [0u8; 8];
            uid.copy_from_slice(&response[..8]);
            Ok(uid)
        } else {
            Err(Pn532Error::InvalidResponse)
        }
    }

    pub fn read_block(&mut self, block_idx: u8) -> Result<[u8; 4], Pn532Error> {
        self.pn532.rf_configuration_timing(0x00, 0x07)?;

        let cmd = [CMD_READ_BLOCK, block_idx];
        let response = self.pn532.communicate_thru(&cmd)?;

        if response.len() >= 4 {
            let mut block = [0u8; 4];
            block.copy_from_slice(&response[..4]);
            Ok(block)
        } else {
            Err(Pn532Error::InvalidResponse)
        }
    }

    pub fn write_block(&mut self, block_idx: u8, data: &[u8; 4]) -> Result<(), Pn532Error> {
        self.pn532.rf_configuration_timing(0x00, 0x0E)?;

        let cmd = [
            CMD_WRITE_BLOCK,
            block_idx,
            data[0],
            data[1],
            data[2],
            data[3],
        ];

        let _ = self.pn532.communicate_thru(&cmd);
        self.delay_ms(50);
        Ok(())
    }

    pub fn completion(&mut self) -> Result<(), Pn532Error> {
        self.pn532.rf_configuration_timing(0x00, 0x01)?;
        let cmd = [CMD_COMPLETION];
        let _ = self.pn532.communicate_thru(&cmd);
        Ok(())
    }

    pub fn read_full_chip(&mut self) -> Result<ChipData, Pn532Error> {
        let _ = self.pn532.rf_field(false);
        self.delay_ms(100);
        let _ = self.pn532.rf_field(true);
        self.delay_ms(200);

        let chip_id = self.initiate(true)?;
        self.select(None)?;
        let uid = self.get_uid()?;

        let mut data = ChipData {
            chip_id,
            uid,
            ..Default::default()
        };

        for i in 0u16..256 {
            match self.read_block(i as u8) {
                Ok(block) => {
                    data.blocks[i as usize] = block;
                    data.block_count = (i + 1) as usize;
                    if block != [0xFF, 0xFF, 0xFF, 0xFF] {
                        log::info!("Block {:3}: {:02X?}", i, block);
                    }
                }
                Err(_) => {
                    log::info!("Total blocks: {}", i);
                    break;
                }
            }
        }

        let _ = self.completion();
        let _ = self.pn532.rf_field(false);
        Ok(data)
    }

    pub fn write_full_chip(&mut self, data: &ChipData) -> Result<(), Pn532Error> {
        let _ = self.pn532.rf_field(false);
        self.delay_ms(100);
        let _ = self.pn532.rf_field(true);
        self.delay_ms(200);

        self.initiate(true)?;
        self.select(None)?;

        let mut changed_blocks = alloc::vec::Vec::new();
        for i in 0..data.block_count {
            if i == 0 || i == 5 || i == 6 {
                continue;
            }

            let current = self.read_block(i as u8).unwrap_or([0xFF; 4]);
            if current != data.blocks[i] {
                changed_blocks.push(i);
                log::info!(
                    "Block {} changed: {:02X?} -> {:02X?}",
                    i,
                    current,
                    data.blocks[i]
                );
            }
        }

        if changed_blocks.is_empty() {
            log::info!("No changes to write");
            let _ = self.completion();
            let _ = self.pn532.rf_field(false);
            return Ok(());
        }

        let mut write_errors = 0usize;
        for &i in &changed_blocks {
            log::info!("Writing block {}...", i);
            if self.write_block(i as u8, &data.blocks[i]).is_err() {
                log::warn!("Write block {} failed", i);
                write_errors += 1;
            }
        }

        let _ = self.completion();
        self.delay_ms(200);

        let _ = self.pn532.rf_field(false);
        self.delay_ms(200);
        let _ = self.pn532.rf_field(true);
        self.delay_ms(300);

        self.initiate(true)?;
        self.select(None)?;

        let mut verify_errors = 0usize;
        for &i in &changed_blocks {
            match self.read_block(i as u8) {
                Ok(read_data) => {
                    if read_data != data.blocks[i] {
                        log::warn!(
                            "Verify block {}: wrote {:02X?}, read {:02X?}",
                            i,
                            data.blocks[i],
                            read_data
                        );
                        verify_errors += 1;
                    } else {
                        log::info!("Block {} verified OK", i);
                    }
                }
                Err(_) => {
                    log::warn!("Verify read block {} failed", i);
                    verify_errors += 1;
                }
            }
        }

        let _ = self.completion();
        let _ = self.pn532.rf_field(false);

        if write_errors > 0 || verify_errors > 0 {
            log::error!(
                "Write: {} errors, Verify: {} errors",
                write_errors,
                verify_errors
            );
            return Err(Pn532Error::InvalidResponse);
        }

        log::info!("Write verified OK");
        Ok(())
    }
}
