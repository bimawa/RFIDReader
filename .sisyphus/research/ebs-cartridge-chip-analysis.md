# EBS-6600 Cartridge Chip Analysis

> Research conducted: January 2026
> Chip: ST25TB04K (128 blocks x 4 bytes = 512 bytes)
> Cartridge: Boltmark EBS-6600, Ink Black, Part# XI32001-000

## Raw Dump

```
UID: F3 5A FB 79 66 3F 02 D0

Block | Hex Data      | Binary (first byte)     | Notes
------|---------------|-------------------------|------------------
B000  | FF FF FF FF   | 1111 1111               | System area
B001  | FF FF FF FF   | 1111 1111               | System area
B002  | FF FF FF FF   | 1111 1111               | System area
B003  | FF FF FF FF   | 1111 1111               | System area
B004  | FF FF FF FF   | 1111 1111               | System area
B005  | FE FF FF FF   | 1111 1110               | OTP - 1 bit used
B006  | FF FF FF FF   | 1111 1111               | OTP area
B007  | FF FF FF FF   | 1111 1111               | OTP area
B008  | FF FF FF FF   | 1111 1111               | OTP area
B009  | FF FF FF FF   | 1111 1111               | OTP area
B010  | FF FF FF FF   | 1111 1111               | OTP area
B011  | FF FF FF FF   | 1111 1111               | OTP area
B012  | FF FF FF FF   | 1111 1111               | OTP area
B013  | FF FF FF FF   | 1111 1111               | OTP area
B014  | 4F FF FF FF   | 0100 1111               | OTP counter? 4 bits used
B015  | 1F FF FF FF   | 0001 1111               | OTP counter? 3 bits used
B016  | 15 0C 3F 13   |                         | User data start
B017  | CC 2A 15 48   |                         | User data
B018  | 39 43 10 17   |                         | User data
B019  | 52 65 0F 19   |                         | User data
B020  | 27 29 FF 01   |                         | User data
B021  | 05 36 68 28   |                         | User data
B022  | 2C 90 00 00   |                         | User data end
B023+ | FF FF FF FF   |                         | Unused
```

## Cartridge Label Information

| Field | Value |
|-------|-------|
| Printer | Boltmark EBS-6600 |
| Ink Type | Black |
| Part Number | XI32001-000 |
| Lot Number | EB1004MS21A |
| Use Before | 2023-01-04 |
| Warning | Danger (flammable) |

---

## ST25TB04K Memory Map (from datasheet)

| Block Range | Size | Description | Properties |
|-------------|------|-------------|------------|
| 0-4 | 20 bytes | System area | Read-only after init |
| 5-15 | 44 bytes | OTP area | One-Time Programmable (1→0 only) |
| 16-126 | 444 bytes | User EEPROM | Read/Write |
| 127 | 4 bytes | UID | Read-only |

### OTP (One-Time Programmable) Behavior

- Bits can only transition from 1 to 0
- Cannot be reset back to 1 (permanent)
- Used for counters, lock bits, authentication

---

## Analysis: OTP Counters (Blocks 5, 14, 15)

### Block 5: FE FF FF FF
```
FE = 1111 1110 (binary)
     ^^^^^^^^
     |||||||└─ Bit 0 = 0 (used)
     ||||||└── Bit 1 = 1
     |||||└─── Bit 2 = 1
     ||||└──── Bit 3 = 1
     |||└───── Bit 4 = 1
     ||└────── Bit 5 = 1
     |└─────── Bit 6 = 1
     └──────── Bit 7 = 1

Interpretation: 1 event/unit counted (cartridge initialized)
```

### Block 14: 4F FF FF FF
```
4F = 0100 1111 (binary)
     ^^^^^^^^
     |||||||└─ Bit 0 = 1
     ||||||└── Bit 1 = 1
     |||||└─── Bit 2 = 1
     ||||└──── Bit 3 = 1
     |||└───── Bit 4 = 0 (used)
     ||└────── Bit 5 = 0 (used)
     |└─────── Bit 6 = 1
     └──────── Bit 7 = 0 (used)

Bits used: 3 (positions 4, 5, 7)
Hypothesis: Ink level counter or print head usage
```

### Block 15: 1F FF FF FF
```
1F = 0001 1111 (binary)
     ^^^^^^^^
     |||||||└─ Bit 0 = 1
     ||||||└── Bit 1 = 1
     |||||└─── Bit 2 = 1
     ||||└──── Bit 3 = 1
     |||└───── Bit 4 = 1
     ||└────── Bit 5 = 0 (used)
     |└─────── Bit 6 = 0 (used)
     └──────── Bit 7 = 0 (used)

Bits used: 3 (positions 5, 6, 7)
Hypothesis: Secondary counter or validation bits
```

### Counter Theory

ST25TB chips have hardware countdown counters. Common patterns:

1. **Unary Counter**: Each bit = 1 unit. Count down by clearing bits.
   - 32 bits total per block = 32 units max
   - Block 14 first byte: 4F = 4 bits cleared = 4 units used
   
2. **Binary Counter**: Standard decrement
   - Full 32-bit value = 4 billion units max
   
3. **Combined**: First bytes = coarse, remaining = fine

---

## Analysis: User Data (Blocks 16-22)

### Raw bytes (28 bytes total):
```
B016: 15 0C 3F 13
B017: CC 2A 15 48
B018: 39 43 10 17
B019: 52 65 0F 19
B020: 27 29 FF 01
B021: 05 36 68 28
B022: 2C 90 00 00
```

### Attempt 1: ASCII Interpretation

```
Hex  | Dec | ASCII | Printable?
-----|-----|-------|----------
0x15 | 21  | NAK   | No
0x0C | 12  | FF    | No
0x3F | 63  | ?     | Yes
0x13 | 19  | DC3   | No
0xCC | 204 | -     | No (extended)
0x2A | 42  | *     | Yes
0x15 | 21  | NAK   | No
0x48 | 72  | H     | Yes
0x39 | 57  | 9     | Yes
0x43 | 67  | C     | Yes
0x10 | 16  | DLE   | No
0x17 | 23  | ETB   | No
0x52 | 82  | R     | Yes
0x65 | 101 | e     | Yes
0x0F | 15  | SI    | No
0x19 | 25  | EM    | No
0x27 | 39  | '     | Yes
0x29 | 41  | )     | Yes
0xFF | 255 | -     | No
0x01 | 1   | SOH   | No
0x05 | 5   | ENQ   | No
0x36 | 54  | 6     | Yes
0x68 | 104 | h     | Yes
0x28 | 40  | (     | Yes
0x2C | 44  | ,     | Yes
0x90 | 144 | -     | No
0x00 | 0   | NUL   | No
0x00 | 0   | NUL   | No
```

**Partial ASCII found**: `?`, `*`, `H`, `9`, `C`, `R`, `e`, `'`, `)`, `6`, `h`, `(`, `,`

Not standard ASCII text. Likely binary encoded.

### Attempt 2: BCD (Binary Coded Decimal)

BCD encodes each decimal digit as 4 bits:
```
0x13 = 1, 3 → "13"
0x15 = 1, 5 → "15"  
0x04 = 0, 4 → "04"
etc.
```

Looking for date 2023-01-04:
- Year: 2023 or 23 → 0x20, 0x23 or 0x23
- Month: 01 → 0x01
- Day: 04 → 0x04

Searching in data:
- 0x04 not found directly
- Could be encoded differently

### Attempt 3: Field Structure Hypothesis

Based on cartridge info, expected fields:
1. Part number: XI32001-000 (11 chars)
2. Lot number: EB1004MS21A (11 chars)
3. Expiry date: 2023-01-04 (could be days since epoch or structured)
4. Ink type: Black (could be 1 byte code)
5. Capacity/Volume: Initial ink amount
6. Checksum: Validation

**Possible structure (28 bytes):**
```
Offset | Size | Field
-------|------|------------------
0      | 2    | Header/Magic
2      | 4    | Part number (encoded)
6      | 4    | Lot number (encoded)  
10     | 4    | Expiry date (timestamp?)
14     | 2    | Ink type code
16     | 4    | Initial capacity
20     | 4    | Manufacturing date?
24     | 2    | Checksum
26     | 2    | Reserved/Padding
```

### Attempt 4: XOR/Simple Encryption

Many cartridge chips use simple XOR with UID:
```
Key candidate: First 4 bytes of UID = F3 5A FB 79

B016 XOR key:
15 XOR F3 = E6
0C XOR 5A = 56  ('V')
3F XOR FB = C4
13 XOR 79 = 6A  ('j')
```

Not obviously meaningful. May need different key or algorithm.

### Attempt 5: Reverse Byte Order (Little Endian)

Block 16 as little-endian uint32: 0x133F0C15 = 322,568,213
Block 22 as little-endian uint32: 0x0000902C = 36,908

Block 22 value (36,908) could be:
- Days since some epoch
- Capacity in some unit
- Counter value

---

## Hypotheses for Data Meaning

### Hypothesis A: Simple Counter System

| Block | Purpose |
|-------|---------|
| 5 | Initialization flag (1 bit used = cartridge activated) |
| 14-15 | Ink level counters (OTP decrement) |
| 16-17 | Encrypted part/lot number |
| 18-19 | Encrypted date info |
| 20-21 | Capacity / manufacturing data |
| 22 | Counter value or checksum |

### Hypothesis B: Challenge-Response System

Some industrial printers use:
1. Read UID + specific blocks
2. Calculate expected response
3. Verify cartridge authenticity

The non-ASCII data may be authentication tokens.

### Hypothesis C: Proprietary Encoding

EBS/Boltmark likely uses proprietary format:
- May require sniffing printer-cartridge communication
- Could involve time-based or sequence-based encoding

---

## OTP Counter Behavior (Key Insight)

### How Ink Level Tracking Works

1. **Fresh cartridge**: OTP blocks = 0xFFFFFFFF (all bits = 1)
2. **As ink used**: Bits flipped 1→0 (cannot reverse)
3. **Empty cartridge**: Most/all bits = 0

**Block 14 analysis**:
```
Fresh:  FF FF FF FF = 32 bits available
Current: 4F FF FF FF = 29 bits remaining (3 used in first byte)

If each bit = ~3% of cartridge:
3 bits used ≈ 9% ink consumed
```

**Block 15 analysis**:
```
Current: 1F FF FF FF = 29 bits remaining (3 used in first byte)
Same consumption pattern as Block 14
```

**Combined counter theory**:
- Block 14 = Primary ink counter
- Block 15 = Verification counter (redundancy)
- Must match for cartridge to be valid

---

## Recommendations for Further Research

### 1. Collect More Dumps

To find patterns, need dumps from:
- [ ] Fresh/new cartridge (same model)
- [ ] Partially used cartridge
- [ ] Empty/rejected cartridge
- [ ] Different ink colors (same printer)
- [ ] Different lot numbers

### 2. Compare Dumps

Create diff table:
```
Block | Fresh | Partial | Empty | Notes
------|-------|---------|-------|------
B005  | FF    | FE?     | ?     | Init flag
B014  | FF    | 4F      | ?     | Counter 1
B015  | FF    | 1F      | ?     | Counter 2
B016  | ?     | 15 0C.. | ?     | User data
```

### 3. Sniff Communication

Use logic analyzer or protocol sniffer to capture:
- Printer → Cartridge commands
- Cartridge → Printer responses
- Timing and sequence

### 4. Test Write Behavior

Carefully test (on expendable cartridge):
- Can user data blocks (16+) be written?
- What happens if OTP counter is decremented?
- Does printer verify checksum?

---

## Tools and References

### Hardware
- PN532 NFC reader (current setup)
- Logic analyzer for SPI/I2C sniffing
- Proxmark3 for advanced RFID analysis

### Software
- [SRIX4K-Reader](https://github.com/ErikPelli/SRIX4K-Reader) - SRIX4K/ST25TB04K tool
- libnfc - NFC library
- Current RFIDReader firmware

### Documentation
- [ST25TB04K Datasheet](https://www.st.com/resource/en/datasheet/st25tb04k.pdf)
- [ST25TB Product Presentation](https://www.st.com/resource/en/product_presentation/st25tb-product-presentation.pdf)
- ISO 14443-3B specification

---

## Appendix: Known Cartridge Info

### Cartridge 1 (This Dump)
```
Printer:     Boltmark EBS-6600
Type:        Ink Black
Part#:       XI32001-000
Lot#:        EB1004MS21A
Use Before:  2023-01-04
UID:         F3 5A FB 79 66 3F 02 D0
Status:      Partially used (estimated ~9% consumed based on OTP)
```

### Future Cartridges
(To be added as more dumps collected)

---

## Conclusion

The ST25TB04K chip on EBS-6600 cartridges uses:

1. **OTP counters** (blocks 5, 14, 15) for ink level tracking
   - Cannot be reset (hardware limitation)
   - Decrementing bits as ink consumed

2. **User data area** (blocks 16-22) for cartridge identification
   - 28 bytes of data
   - Likely encoded/encrypted (not plain ASCII)
   - Contains part number, lot, expiry date

3. **Potential workarounds**:
   - Clone entire chip to fresh ST25TB04K
   - Emulate chip with programmable NFC tag
   - Cannot "refill" same chip due to OTP nature

**Next Steps**: Collect more dumps to establish patterns and decode user data format.
