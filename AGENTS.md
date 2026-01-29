# RFIDReader Project - Development Notes

## Hardware: LilyGO T-Embed CC1101

### Key Hardware Details

| Component | Description |
|-----------|-------------|
| MCU | ESP32-S3FN16R8 (Dual-core LX7, 240MHz) |
| Flash | 16MB |
| PSRAM | 8MB (Octal SPI) |
| Display | 1.9" IPS TFT ST7789V, 170x320px (vertical) |
| NFC | PN532 (I2C) |
| Sub-GHz | CC1101 |
| Audio | I2S Speaker + ES7210 Microphone |
| Battery | BQ25896 (charging) + BQ27220 (monitoring) |

---

## Critical Pin Configuration

### Power Enable (IMPORTANT!)
```
GPIO15 - BOARD_PWR_EN - MUST be HIGH to enable:
  - Audio amplifier
  - CC1101 module
  - LED strip
```

### Display (ST7789V - SPI)
```
GPIO21 - Backlight (HIGH = on)
GPIO41 - CS
GPIO9  - MOSI
GPIO11 - SCK
GPIO16 - DC
GPIO40 - RST
```

### PN532 NFC (I2C)
```
GPIO8  - SDA
GPIO18 - SCL
GPIO17 - IRQ
GPIO45 - RF_REST (strapping pin - handle carefully!)
I2C Address: 0x24
```

### I2S Audio (Speaker)
```
GPIO46 - BCLK (Bit Clock)
GPIO40 - LRCLK/WS (Word Select)
GPIO7  - DOUT/DIN (Data)
```

### Rotary Encoder
```
GPIO4 - Encoder A
GPIO5 - Encoder B
GPIO0 - Encoder Button (directly connects to BOOT)
```

### Other
```
GPIO6  - Back/User Button
GPIO14 - WS2812 LED Strip (8 LEDs)
GPIO15 - Power Enable
```

### I2C Devices on Bus
```
0x24 - PN532 (NFC)
0x55 - BQ27220 (Battery Monitor)
0x6B - BQ25896 (Battery Charger)
```

---

## Known Issues & Workarounds

### 1. "Observer Effect" - Device Only Works with Serial Monitor
**Problem:** RFID reading only works when serial monitor is connected.

**Workaround:** Always flash with monitor:
```bash
cargo espflash flash --port /dev/cu.usbmodem1201 --release && espflash monitor --port /dev/cu.usbmodem1201 --non-interactive
```

**Root Cause:** Unknown - possibly USB/power initialization quirk.

### 2. PN532 Wakeup Issues
**Problem:** First SAM config often fails after reset.

**Solution:** Implemented retry loop with multiple wakeup attempts:
- Reset PN532 via GPIO45
- Send wakeup command
- Retry SAM config up to 3 times

### 3. GPIO45 is Strapping Pin
**Problem:** GPIO45 controls boot mode and is also used for PN532 reset.

**Solution:** Only toggle after boot is complete. Never hold LOW during reset.

### 4. Audio Quiet Without Power Enable
**Problem:** I2S audio works but is very quiet.

**Solution:** Must set GPIO15 HIGH before using audio:
```rust
let _pwr_en = Output::new(peripherals.GPIO15, Level::High, OutputConfig::default());
```

### 5. Display Flickering When Updating
**Problem:** Full screen clear causes visible flicker during updates.

**Solution:** Clear only individual rows before redrawing them:
```rust
fn clear_area(&mut self, x: i32, y: i32, w: u32, h: u32) {
    let _ = Rectangle::new(Point::new(x, y), Size::new(w, h))
        .into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK))
        .draw(&mut self.driver);
}

// Before drawing each row:
self.clear_area(0, y - 9, 145, 11);
```

**Key points:**
- Use `force_clear=true` only on first draw (full clear)
- Use `force_clear=false` for updates (per-row clear)
- Clear only the exact area being redrawn

---

## I2S Audio Configuration

```rust
// Working I2S setup for T-Embed CC1101 speaker
let i2s = I2s::new(
    peripherals.I2S0,
    peripherals.DMA_CH0,
    I2sConfig::new_tdm_philips()
        .with_sample_rate(Rate::from_hz(16000))
        .with_data_format(DataFormat::Data16Channel16)
        .with_channels(Channels::STEREO),
).unwrap();

let i2s_tx = i2s.i2s_tx
    .with_bclk(peripherals.GPIO46)
    .with_ws(peripherals.GPIO40)
    .with_dout(peripherals.GPIO7)
    .build(unsafe { &mut *core::ptr::addr_of_mut!(TX_DESCRIPTORS) });
```

### Beep Sound Parameters
```rust
const SAMPLE_RATE: u32 = 16000;
const BEEP_FREQ: u32 = 2500;      // Hz - clear audible tone
const BEEP_DURATION_MS: u32 = 150;
const AMPLITUDE: i16 = 32700;     // Max volume
```

---

## ST25TB RFID Chip Details

### Chip: ST25TB04K
- 128 blocks x 4 bytes = 512 bytes total
- OTP memory (bits can only go 1→0, never 0→1)
- ISO14443-B protocol

### Block Layout
```
Blocks 0-4:   System area
Blocks 5-15:  OTP area (One-Time Programmable)
Blocks 16+:   User data
Block 127:    UID (read-only)
```

### Reading
- Use InCommunicateThru command
- 10 retry attempts recommended
- Status 0x01 on last block is normal (no more data)

---

## Build Commands

```bash
# Setup environment
source ~/export-esp.sh

# Build only
cargo build --release

# Flash and monitor (REQUIRED for device to work)
cargo espflash flash --port /dev/cu.usbmodem1201 --release && espflash monitor --port /dev/cu.usbmodem1201 --non-interactive

# Monitor only
espflash monitor --port /dev/cu.usbmodem1201 --non-interactive
```

---

## Dependencies (Cargo.toml)

```toml
esp-hal = { version = "1.0.0", features = ["esp32s3", "unstable"] }
esp-bootloader-esp-idf = "0.1"
esp-backtrace = { version = "0.15", features = ["esp32s3", "panic-handler", "println"] }
esp-println = { version = "0.13", features = ["esp32s3", "log"] }
esp-alloc = "0.7"

embedded-hal = "1.0"
embedded-hal-bus = "0.2"
embedded-graphics = "0.8"
mipidsi = "0.8"
display-interface-spi = "0.5"

heapless = "0.8"
libm = "0.2"
```

---

## References

- [LilyGO T-Embed-CC1101 GitHub](https://github.com/Xinyuan-LilyGO/T-Embed-CC1101)
- [LilyGO Wiki](https://wiki.lilygo.cc/get_started/en/Wearable/T-Embed-CC1101/T-Embed-CC1101.html)
- [ST25TB Datasheet](https://www.st.com/resource/en/datasheet/st25tb04k.pdf)
- [PN532 User Manual](https://www.nxp.com/docs/en/user-guide/141520.pdf)
