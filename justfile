# RFIDReader - ESP32-S3 T-Embed CC1101
# Commands for building, flashing, and monitoring

# Default port (change if needed)
port := "/dev/cu.usbmodem1201"

# Setup environment and build
build:
    source ~/export-esp.sh && cargo build --release

# Flash and monitor (most common command)
flash:
    source ~/export-esp.sh && cargo espflash flash --port {{port}} --release && espflash monitor --port {{port}} --non-interactive

# Flash only (no monitor)
flash-only:
    source ~/export-esp.sh && cargo espflash flash --port {{port}} --release

# Monitor only (device already flashed)
monitor:
    espflash monitor --port {{port}} --non-interactive

# Clean build
clean:
    cargo clean

# Check for errors without building
check:
    source ~/export-esp.sh && cargo check --release

# Build and show size
size:
    source ~/export-esp.sh && cargo size --release

# List available serial ports
ports:
    ls /dev/cu.usb* 2>/dev/null || echo "No USB serial ports found"

# Full rebuild (clean + build)
rebuild: clean build
