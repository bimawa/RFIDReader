# EBS Ink Jet Cartridge Research

## Hardware Info

**Printer:** Boltmark EBS-6600
**Type:** Industrial Continuous Inkjet (CIJ) marking printer
**Manufacturer:** EBS Ink Jet Systems

**Cartridge:**
- Part: XI32001-000
- Lot: EB1004MS21A
- Ink: Black
- Date on label: 2023-01-04

**RFID Chip:**
- Type: ST25TB04K (or compatible SRIX4K)
- Protocol: ISO14443-B
- Capacity: 128 blocks × 4 bytes = 512 bytes
- UID: `D0 02 3F 66 79 FB 5A F3` (LSB first in dump)

---

## Dump Analysis

### Block Layout

| Block | Data | Interpretation |
|-------|------|----------------|
| 0-4 | System area | ST25TB system blocks |
| 5-13 | `FF FF FF FF` | Empty/unused |
| 14 | `4F FF FF FF` | OTP area? |
| 15 | `1F FF FF FF` | OTP area? |
| 16-19 | Variable | Cartridge ID / Serial |
| 20 | `27 29 FF 01` | Flags + counter? |
| 21 | `05 36 68 28` | Volume / level? |
| 22 | `2C 90 00 00` | Date? (see below) |
| 23-127 | `FF FF FF FF` | Empty/unused |

### Raw Data (Non-empty blocks)

```
B000: 0F FF FF FF  ← System
B001: 9F FF FF FF  ← System  
B002: 0F FF FF FF  ← System
B005: FE FF FF FF  ← OTP
B014: 4F FF FF FF  ← OTP
B015: 1F FF FF FF  ← OTP
B016: 15 0C 3F 13  ← Cartridge data start
B017: CC 2A 15 48
B018: 39 43 10 17
B019: 52 65 0F 19
B020: 27 29 FF 01
B021: 05 36 68 28
B022: 2C 90 00 00  ← Possible date
```

### Date Hypothesis (Block 22)

`2C 90` as big-endian = **11408**

If this is days since epoch (1992-01-01):
- 11408 ÷ 365.25 ≈ 31.2 years
- 1992 + 31 = **2023** ✓

This matches the 2023-01-04 date on the cartridge label!

### Lot Number Decode

**EB1004MS21A:**
- `EB` = EBS (manufacturer)
- `1004` = Unknown (date code? batch?)
- `MS21A` = Series/batch identifier

---

## Purpose of Chip

Industrial inkjet cartridges use RFID chips for:

1. **Usage Counter** - Track ink consumption
2. **Authentication** - Prevent non-genuine cartridges
3. **Expiration** - Enforce shelf life
4. **Refill Prevention** - Block reuse after empty
5. **Region Lock** - Restrict geographic use

---

## TODO: Further Research

- [ ] Dump a **NEW** cartridge (compare with used)
- [ ] Dump **EMPTY** cartridge (find counter changes)
- [ ] Identify counter block (likely B020 or B021)
- [ ] Test writing modified values
- [ ] Find EBS protocol documentation

---

## Related Projects

No open-source EBS cartridge tools found. This may be the first!

Similar industrial inkjet systems:
- Domino
- Videojet
- Markem-Imaje
- Hitachi

These likely use similar ST25TB chips with proprietary data formats.
