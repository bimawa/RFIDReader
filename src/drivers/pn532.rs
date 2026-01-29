//! PN532 NFC driver with ISO14443-B support for ST25TB chips
//!
//! Based on kpn532 library by Benjamin DELPY (gentilkiwi)

use embedded_hal::digital::{InputPin, OutputPin};
use embedded_hal::i2c::I2c;

const PN532_I2C_READY: u8 = 0x01;
const PN532_PREAMBLE: u8 = 0x00;
const PN532_STARTCODE1: u8 = 0x00;
const PN532_STARTCODE2: u8 = 0xFF;
const PN532_POSTAMBLE: u8 = 0x00;

const PN532_HOST_TO_PN532: u8 = 0xD4;
const PN532_PN532_TO_HOST: u8 = 0xD5;

// Commands
const PN532_CMD_GETFIRMWAREVERSION: u8 = 0x02;
const PN532_CMD_SAMCONFIGURATION: u8 = 0x14;
const PN532_CMD_RFCONFIGURATION: u8 = 0x32;
const PN532_CMD_WRITEREGISTER: u8 = 0x08;
const PN532_CMD_INCOMMUNICATETHRU: u8 = 0x42;

// PN532 Registers for ISO14443-B
const REG_CIU_CONTROL: u16 = 0x633C;
const REG_CIU_TX_MODE: u16 = 0x6302;
const REG_CIU_RX_MODE: u16 = 0x6303;
const REG_CIU_CWGSP: u16 = 0x6318;
const REG_CIU_MODGSP: u16 = 0x6319;

pub struct Pn532<I2C, IRQ, RST> {
    i2c: I2C,
    irq: IRQ,
    rst: RST,
    addr: u8,
    buffer: [u8; 265],
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Pn532Error {
    I2cError,
    Timeout,
    InvalidResponse,
    ChecksumError,
    NackReceived,
}

impl<I2C, IRQ, RST> Pn532<I2C, IRQ, RST>
where
    I2C: I2c,
    IRQ: InputPin,
    RST: OutputPin,
{
    pub fn new(i2c: I2C, irq: IRQ, rst: RST, addr: u8) -> Self {
        Self {
            i2c,
            irq,
            rst,
            addr,
            buffer: [0u8; 265],
        }
    }

    pub fn init(&mut self) -> Result<(), Pn532Error> {
        for attempt in 0..3 {
            self.reset()?;

            log::info!("PN532 SAM config (attempt {})...", attempt);
            if let Err(e) = self.sam_configuration() {
                log::warn!("SAM config failed: {:?}, retrying...", e);
                self.delay_ms(200);
                continue;
            }

            log::info!("PN532 ISO14443-B config...");
            if let Err(e) = self.configure_iso14443b() {
                log::warn!("ISO14443-B config failed: {:?}, retrying...", e);
                self.delay_ms(200);
                continue;
            }

            log::info!("PN532 init complete!");
            return Ok(());
        }

        log::error!("PN532 init failed after 3 attempts");
        Err(Pn532Error::Timeout)
    }

    fn reset(&mut self) -> Result<(), Pn532Error> {
        log::info!("PN532 resetting...");
        let _ = self.rst.set_low();
        self.delay_ms(100);
        let _ = self.rst.set_high();
        self.delay_ms(500);

        log::info!("Waking up PN532...");
        for attempt in 0..5 {
            // Send preamble bytes to wake up PN532 from low power mode
            let preamble = [0x55u8; 16];
            let _ = self.i2c.write(self.addr, &preamble);
            self.delay_ms(50);

            // Now send SAMConfiguration command embedded in wakeup sequence
            let wakeup = [0x00, 0x00, 0xFF, 0x03, 0xFD, 0xD4, 0x14, 0x01, 0x17, 0x00];
            if self.i2c.write(self.addr, &wakeup).is_ok() {
                log::info!("Wakeup sent on attempt {}", attempt);
                self.delay_ms(100);

                // Wait for response and drain it
                if self.wait_ready().is_ok() {
                    let mut buf = [0u8; 16];
                    let _ = self.i2c.read(self.addr, &mut buf);
                    self.delay_ms(50);
                    log::info!("PN532 wakeup complete");
                    return Ok(());
                }
            }
            self.delay_ms(200);
        }

        log::warn!("PN532 wakeup failed, continuing anyway...");
        Ok(())
    }

    fn delay_ms(&self, ms: u32) {
        esp_hal::rom::ets_delay_us(ms * 1000);
    }

    pub fn get_firmware_version(&mut self) -> Result<(u8, u8, u8), Pn532Error> {
        self.send_command(&[PN532_CMD_GETFIRMWAREVERSION])?;
        let response = self.read_response()?;

        if response.len() >= 4 {
            Ok((response[0], response[1], response[2]))
        } else {
            Err(Pn532Error::InvalidResponse)
        }
    }

    fn sam_configuration(&mut self) -> Result<(), Pn532Error> {
        self.send_command(&[PN532_CMD_SAMCONFIGURATION, 0x01, 0x00, 0x01])?;
        self.read_response()?;
        Ok(())
    }

    fn configure_iso14443b(&mut self) -> Result<(), Pn532Error> {
        // Configure PN532 registers for ISO14443-B (ST25TB)
        // Based on kpn532 Registers_B_SR_ST25TB
        let registers: [(u16, u8); 5] = [
            (REG_CIU_CONTROL, 0x10), // Initiator mode
            (REG_CIU_TX_MODE, 0x83), // TxCRCEn, 106kbps, ISO14443-B
            (REG_CIU_RX_MODE, 0x83), // RxCRCEn, 106kbps, ISO14443-B
            (REG_CIU_CWGSP, 0x3F),   // Max conductance
            (REG_CIU_MODGSP, 0x12),  // Modulation conductance
        ];

        for (reg, val) in registers {
            self.write_register(reg, val)?;
        }
        Ok(())
    }

    fn write_register(&mut self, reg: u16, value: u8) -> Result<(), Pn532Error> {
        let cmd = [
            PN532_CMD_WRITEREGISTER,
            (reg >> 8) as u8,
            (reg & 0xFF) as u8,
            value,
        ];
        self.send_command(&cmd)?;
        self.read_response()?;
        Ok(())
    }

    pub fn rf_configuration_timing(
        &mut self,
        atr_timeout: u8,
        retry_timeout: u8,
    ) -> Result<(), Pn532Error> {
        self.send_command(&[
            PN532_CMD_RFCONFIGURATION,
            0x02, // Various timings
            0x00,
            atr_timeout,
            retry_timeout,
        ])?;
        self.read_response()?;
        Ok(())
    }

    pub fn rf_configuration_retries(&mut self, max_retries: u8) -> Result<(), Pn532Error> {
        self.send_command(&[
            PN532_CMD_RFCONFIGURATION,
            0x04, // MaxRtyCOM
            max_retries,
        ])?;
        self.read_response()?;
        Ok(())
    }

    pub fn rf_field(&mut self, on: bool) -> Result<(), Pn532Error> {
        self.send_command(&[
            PN532_CMD_RFCONFIGURATION,
            0x01,
            if on { 0x01 } else { 0x00 },
        ])?;
        self.read_response()?;
        Ok(())
    }

    pub fn communicate_thru(&mut self, data: &[u8]) -> Result<&[u8], Pn532Error> {
        let mut cmd = [0u8; 64];
        cmd[0] = PN532_CMD_INCOMMUNICATETHRU;
        cmd[1..1 + data.len()].copy_from_slice(data);

        self.send_command(&cmd[..1 + data.len()])?;
        let response = self.read_response()?;

        // response[0] = InCommunicateThru status (00=OK)
        if response.is_empty() {
            return Err(Pn532Error::InvalidResponse);
        }

        let status = response[0];
        if status != 0x00 {
            log::warn!("InCommunicateThru error status: 0x{:02X}", status);
            return Err(Pn532Error::InvalidResponse);
        }

        Ok(&response[1..])
    }

    fn send_command(&mut self, cmd: &[u8]) -> Result<(), Pn532Error> {
        let len = cmd.len() + 1;
        let mut frame = [0u8; 64];
        let mut idx = 0;

        frame[idx] = PN532_PREAMBLE;
        idx += 1;
        frame[idx] = PN532_STARTCODE1;
        idx += 1;
        frame[idx] = PN532_STARTCODE2;
        idx += 1;
        frame[idx] = len as u8;
        idx += 1;
        frame[idx] = (!len as u8).wrapping_add(1);
        idx += 1;
        frame[idx] = PN532_HOST_TO_PN532;
        idx += 1;

        let mut dcs: u8 = PN532_HOST_TO_PN532;
        for &b in cmd {
            frame[idx] = b;
            idx += 1;
            dcs = dcs.wrapping_add(b);
        }
        frame[idx] = (!dcs).wrapping_add(1);
        idx += 1;
        frame[idx] = PN532_POSTAMBLE;
        idx += 1;

        self.i2c
            .write(self.addr, &frame[..idx])
            .map_err(|_| Pn532Error::I2cError)?;

        self.wait_ready()?;
        self.read_ack()
    }

    fn read_ack(&mut self) -> Result<(), Pn532Error> {
        let mut buf = [0u8; 7];
        self.i2c
            .read(self.addr, &mut buf)
            .map_err(|_| Pn532Error::I2cError)?;

        if buf[1] == 0x00 && buf[2] == 0x00 && buf[3] == 0xFF && buf[4] == 0x00 && buf[5] == 0xFF {
            Ok(())
        } else if buf[1] == 0x00
            && buf[2] == 0x00
            && buf[3] == 0xFF
            && buf[4] == 0xFF
            && buf[5] == 0x00
        {
            Err(Pn532Error::NackReceived)
        } else {
            Err(Pn532Error::InvalidResponse)
        }
    }

    fn read_response(&mut self) -> Result<&[u8], Pn532Error> {
        self.wait_ready()?;

        // Read full response in one go: ready(1) + preamble(3) + len(1) + lcs(1) + data(len) + dcs(1) + postamble(1)
        // Max response = 1 + 3 + 1 + 1 + 255 + 1 + 1 = 263, but we read up to ~32 bytes for typical responses
        let mut frame = [0u8; 32];
        self.i2c
            .read(self.addr, &mut frame)
            .map_err(|_| Pn532Error::I2cError)?;

        if frame[1] != 0x00 || frame[2] != 0x00 || frame[3] != 0xFF {
            return Err(Pn532Error::InvalidResponse);
        }

        let len = frame[4] as usize;
        if len == 0 || len > 252 {
            return Err(Pn532Error::InvalidResponse);
        }

        if frame[4].wrapping_add(frame[5]) != 0 {
            return Err(Pn532Error::ChecksumError);
        }

        // Data starts at frame[6], DCS at frame[6+len]
        let data_start = 6;
        let dcs_pos = data_start + len;

        if frame[data_start] != PN532_PN532_TO_HOST {
            return Err(Pn532Error::InvalidResponse);
        }

        let mut dcs: u8 = 0;
        for i in 0..len {
            dcs = dcs.wrapping_add(frame[data_start + i]);
        }
        if dcs.wrapping_add(frame[dcs_pos]) != 0 {
            return Err(Pn532Error::ChecksumError);
        }

        for i in 0..len {
            self.buffer[i] = frame[data_start + i];
        }
        Ok(&self.buffer[2..len])
    }

    fn wait_ready(&mut self) -> Result<(), Pn532Error> {
        for _ in 0..200 {
            if self.irq.is_low().unwrap_or(false) {
                return Ok(());
            }

            let mut status = [0u8; 1];
            if self.i2c.read(self.addr, &mut status).is_ok() && status[0] == PN532_I2C_READY {
                return Ok(());
            }

            self.delay_ms(5);
        }

        Err(Pn532Error::Timeout)
    }

    pub fn probe(&mut self) -> bool {
        let mut buf = [0u8; 1];
        if self.i2c.read(self.addr, &mut buf).is_ok() {
            log::info!(
                "PN532 probe: addr 0x{:02X} responded with 0x{:02X}",
                self.addr,
                buf[0]
            );
            true
        } else {
            log::warn!("PN532 probe: no response from addr 0x{:02X}", self.addr);
            false
        }
    }

    pub fn hard_reset(&mut self) {
        log::info!("PN532 hard reset...");
        let _ = self.rst.set_low();
        self.delay_ms(200);
        let _ = self.rst.set_high();
        self.delay_ms(500);
    }
}
