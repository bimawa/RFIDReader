# Fix Observer Effect Bug

## TL;DR

> **Quick Summary**: Fix timing bug where RFID chip reading only works with serial monitor connected. Replace unreliable spin_loop delays with precise ROM-based ets_delay_us function.
> 
> **Deliverables**:
> - Fixed `delay_ms()` in `src/drivers/pn532.rs`
> - Fixed `delay_ms()` in `src/protocol/st25tb.rs`
> - Device reads chips without serial monitor attached
> 
> **Estimated Effort**: Quick
> **Parallel Execution**: NO - sequential (2 tasks)
> **Critical Path**: Task 1 → Task 2 → Verification

---

## Context

### Original Request
Device only reads RFID chips when serial monitor is connected. Without monitor, reading fails. This "Observer Effect" bug needs to be fixed.

### Interview Summary
**Key Discussions**:
- Root cause: `delay_ms()` uses unreliable `spin_loop` with magic number `ms * 10000`
- When monitor connected, `log::info!()` adds real USB I/O delays that mask the bug
- Solution: Use ESP32 ROM function `ets_delay_us()` for precise microsecond delays

**Research Findings**:
- `esp_hal::rom::ets_delay_us` is available in esp-hal 1.0.0
- `unstable` feature already enabled in Cargo.toml
- ROM function is CPU-frequency aware and precise
- Already used in esp-hal ecosystem for this purpose

### Metis Review
**Identified Gaps** (addressed):
- Verification cannot use serial monitor (would mask the bug) → Hardware-based verification via beep
- Three spin_loops exist but only 2 need fixing → Explicit guardrail to NOT touch main.rs
- Could inject Delay struct instead → Quick fix with ets_delay_us is sufficient for this bug

---

## Work Objectives

### Core Objective
Replace unreliable spin_loop delays with precise ROM-based timing to fix RFID reading without serial monitor.

### Concrete Deliverables
- `src/drivers/pn532.rs` - Fixed `delay_ms()` function (lines 126-130)
- `src/protocol/st25tb.rs` - Fixed `delay_ms()` function (lines 46-50)

### Definition of Done
- [ ] Device reads ST25TB chip WITHOUT serial monitor attached (verified by beep)
- [ ] Device still reads chips WITH serial monitor attached (no regression)
- [ ] `cargo build --release` succeeds

### Must Have
- Replace spin_loop with `esp_hal::rom::ets_delay_us(ms * 1000)` in both files
- Preserve exact delay durations (only timing accuracy changes)

### Must NOT Have (Guardrails)
- DO NOT touch `main.rs` (has intentional spin_loop in error handler)
- DO NOT change function signatures
- DO NOT change delay duration values
- DO NOT add new features or refactoring
- DO NOT add logging/instrumentation
- DO NOT modify Pn532/St25tb struct APIs

---

## Verification Strategy (MANDATORY)

### Test Decision
- **Infrastructure exists**: NO (no automated tests)
- **User wants tests**: Manual-only
- **QA approach**: Hardware-based verification

### Hardware-Based Verification

**CRITICAL**: Cannot use serial monitor to verify fix for bug that manifests WITHOUT serial monitor.

**Primary Test (Without Monitor):**
1. Flash device: `cargo espflash flash --port /dev/cu.usbmodem1201 --release`
2. Wait 3 seconds for boot
3. Place ST25TB chip on antenna
4. **EXPECTED**: Device beeps indicating successful read
5. If no beep after 10 seconds with chip present → FAIL

**Regression Test (With Monitor):**
1. Flash and attach monitor: `just flash`
2. Place ST25TB chip on antenna
3. **EXPECTED**: Serial output shows chip read, device beeps

---

## TODOs

- [ ] 1. Fix delay_ms in pn532.rs

  **What to do**:
  - Replace spin_loop implementation with ROM delay function
  - Change from:
    ```rust
    fn delay_ms(&self, ms: u32) {
        for _ in 0..(ms * 10000) {
            core::hint::spin_loop();
        }
    }
    ```
  - To:
    ```rust
    fn delay_ms(&self, ms: u32) {
        esp_hal::rom::ets_delay_us(ms * 1000);
    }
    ```

  **Must NOT do**:
  - Change delay duration values anywhere in the file
  - Modify function signature
  - Touch any other functions
  - Add imports beyond what's needed

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: `[]`

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Blocks**: Task 2 (same pattern applied)
  - **Blocked By**: None

  **References**:
  - `src/drivers/pn532.rs:126-130` - Current spin_loop implementation to replace
  - `esp_hal::rom::ets_delay_us` - ROM function for precise microsecond delays

  **Acceptance Criteria**:
  - [ ] `delay_ms()` function uses `esp_hal::rom::ets_delay_us(ms * 1000)`
  - [ ] No other changes in the file
  - [ ] `cargo build --release` succeeds without new errors

  **Commit**: YES
  - Message: `fix(pn532): use ROM delay instead of spin_loop`
  - Files: `src/drivers/pn532.rs`

---

- [ ] 2. Fix delay_ms in st25tb.rs

  **What to do**:
  - Apply same fix pattern as Task 1
  - Change from:
    ```rust
    fn delay_ms(&self, ms: u32) {
        for _ in 0..(ms * 10000) {
            core::hint::spin_loop();
        }
    }
    ```
  - To:
    ```rust
    fn delay_ms(&self, ms: u32) {
        esp_hal::rom::ets_delay_us(ms * 1000);
    }
    ```

  **Must NOT do**:
  - Change delay duration values anywhere in the file
  - Modify function signature
  - Touch any other functions

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: `[]`

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Blocks**: Verification
  - **Blocked By**: Task 1

  **References**:
  - `src/protocol/st25tb.rs:46-50` - Current spin_loop implementation to replace
  - Task 1 - Same pattern applied

  **Acceptance Criteria**:
  - [ ] `delay_ms()` function uses `esp_hal::rom::ets_delay_us(ms * 1000)`
  - [ ] No other changes in the file
  - [ ] `cargo build --release` succeeds without new errors/warnings related to this change

  **Commit**: YES
  - Message: `fix(st25tb): use ROM delay instead of spin_loop`
  - Files: `src/protocol/st25tb.rs`

---

- [ ] 3. Verify fix with hardware test

  **What to do**:
  - Build and flash firmware
  - Test WITHOUT serial monitor (primary fix verification)
  - Test WITH serial monitor (regression check)

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: `[]`

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Blocks**: None (final task)
  - **Blocked By**: Tasks 1, 2

  **References**:
  - `just flash` - Build and flash command

  **Acceptance Criteria**:
  
  **Build & Flash:**
  ```bash
  cargo espflash flash --port /dev/cu.usbmodem1201 --release
  # Assert: Exit code 0
  ```

  **Without Monitor Test (PRIMARY):**
  - Wait 3 seconds after flash
  - Place ST25TB chip on PN532 antenna
  - Assert: Device produces audible beep within 10 seconds
  - This confirms fix works without serial monitor masking timing issues

  **With Monitor Test (REGRESSION):**
  ```bash
  just flash
  # Place chip on antenna
  # Assert: Serial output shows "Chip read OK"
  # Assert: Device beeps
  ```

  **Commit**: YES
  - Message: `fix: resolve observer effect - RFID works without serial monitor`
  - Files: (amend previous commits into single commit)

---

## Commit Strategy

| After Task | Message | Files | Verification |
|------------|---------|-------|--------------|
| 1 | `fix(pn532): use ROM delay instead of spin_loop` | src/drivers/pn532.rs | cargo build |
| 2 | `fix(st25tb): use ROM delay instead of spin_loop` | src/protocol/st25tb.rs | cargo build |
| 3 | Squash into: `fix: resolve observer effect bug` | both files | hardware test |

---

## Success Criteria

### Verification Commands
```bash
# Build check
cargo build --release  # Expected: success, no new errors

# Flash only (for without-monitor test)
cargo espflash flash --port /dev/cu.usbmodem1201 --release

# Flash with monitor (for regression test)
just flash
```

### Final Checklist
- [ ] Device reads chips WITHOUT serial monitor (beep heard)
- [ ] Device reads chips WITH serial monitor (no regression)
- [ ] No changes to main.rs
- [ ] No changes to delay duration values
- [ ] Build succeeds
