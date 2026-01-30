# Mobile Data Transfer Research

> Research conducted: January 2026
> Goal: Find user-friendly way to transfer RFID chip data (~2KB) from T-Embed CC1101 to mobile devices

## Executive Summary

| Option | Complexity | User Experience | Recommendation |
|--------|------------|-----------------|----------------|
| **BLE + Native iOS/Android** | High | Best | For production app |
| **BLE + React Native** | Medium | Best | Fastest cross-platform MVP |
| **BLE + Flutter** | Medium | Best | Alternative cross-platform |
| **WiFi AP + Web** | Low | Good | Simplest, no app install |

**Winner for MVP**: React Native + react-native-ble-plx (fastest development, good UX)
**Winner for Best UX**: Native iOS (SwiftUI) + Native Android (Kotlin/Compose)

---

## Hardware Capabilities

### ESP32-S3 (T-Embed CC1101)

| Interface | Available | Notes |
|-----------|-----------|-------|
| WiFi | Yes | 802.11 b/g/n, 2.4GHz |
| Bluetooth LE | Yes | BLE 5.0 |
| Classic Bluetooth | No | ESP32-S3 doesn't support Classic BT |
| NFC (as tag) | No | PN532 is reader, not emulator |

### Coexistence

- BLE + Display (SPI): Compatible
- BLE + NFC/PN532 (I2C): Compatible
- BLE + WiFi: Supported via time-division multiplexing

---

## Option 1: Native iOS App (SwiftUI + CoreBluetooth)

### Architecture

```
[iPhone App - SwiftUI]
    |
    | CoreBluetooth (BLE Central)
    v
[ESP32-S3 - BLE GATT Server]
    |
    | I2C
    v
[PN532 - RFID Reader]
```

### Key Findings

#### CoreBluetooth Basics

```swift
class BLEManager: NSObject, CBCentralManagerDelegate, CBPeripheralDelegate {
    private var centralManager: CBCentralManager!
    private var connectedPeripheral: CBPeripheral?
    
    override init() {
        super.init()
        centralManager = CBCentralManager(delegate: self, queue: nil)
    }
    
    func startScanning() {
        centralManager.scanForPeripherals(withServices: [serviceUUID])
    }
}
```

#### MTU Limitations

- iOS MTU: Fixed at 512 bytes max (not configurable)
- For 2KB data: Split into 4 chunks of 512 bytes
- Transfer time: ~1 second

#### Background Operation

```xml
<!-- Info.plist -->
<key>UIBackgroundModes</key>
<array>
    <string>bluetooth-central</string>
</array>
```

- Must specify service UUIDs for background scanning
- iOS may terminate after ~30s inactivity

#### Apple Developer Program

- **Free**: Test on own device via Xcode (expires after 7 days)
- **$99/year**: TestFlight, App Store, Ad Hoc distribution

### Open Source Examples

| Project | Stars | Features |
|---------|-------|----------|
| [esp32-ble-ios-demo](https://github.com/marcboeker/esp32-ble-ios-demo) | 72 | Simple ESP32 + iOS BLE |
| [EspBlufiForiOS](https://github.com/EspressifApp/EspBlufiForiOS) | 117 | Official Espressif, BLUFI protocol |
| [BLE_Swift_ESP32_SampleProject](https://github.com/pierdr/BLE_Swift_ESP32_SampleProject) | 50 | SwiftUI + ESP32 |

### Pros/Cons

| Pros | Cons |
|------|------|
| Best iOS integration | iOS only |
| Native performance | Swift learning curve |
| Background BLE support | $99/year for distribution |
| Small app size | |

---

## Option 2: Native Android App (Kotlin + Jetpack Compose)

### Architecture

Same as iOS, but using Android BLE APIs.

### Key Findings

#### Permissions (Android 12+)

```xml
<uses-permission android:name="android.permission.BLUETOOTH_SCAN" 
                 android:usesPermissionFlags="neverForLocation" />
<uses-permission android:name="android.permission.BLUETOOTH_CONNECT" />
```

No location permission required with `neverForLocation` flag!

#### MTU Negotiation

```kotlin
gatt.requestMtu(517)  // Request max MTU

override fun onMtuChanged(gatt: BluetoothGatt, mtu: Int, status: Int) {
    val maxPayload = mtu - 3  // ~514 bytes usable
}
```

- Android can negotiate up to 517 bytes
- Transfer time for 2KB: ~0.3-0.5 seconds

#### Min SDK

- **Minimum**: API 21 (Android 5.0)
- **Recommended**: API 31+ (Android 12) for new BLE permissions

### Open Source Examples

| Project | Stars | Features |
|---------|-------|----------|
| [android-esp32-ble](https://github.com/TanaroSch/android-esp32-ble) | - | ESP32 UART template |
| [ble-starter-android](https://github.com/PunchThrough/ble-starter-android) | - | Production-ready starter |
| [ble-device-scanner](https://github.com/vinaygarg55/ble-device-scanner) | - | Jetpack Compose + MVVM |

### Pros/Cons

| Pros | Cons |
|------|------|
| Best Android integration | Android only |
| Faster MTU than iOS | Permission complexity |
| Free distribution (Play Store $25 one-time) | Fragmentation across devices |
| Kotlin is modern | |

---

## Option 3: Cross-Platform (React Native)

### Recommended Stack

```
react-native + react-native-ble-plx
```

### Key Findings

#### Library Comparison

| Library | Stars | Best For |
|---------|-------|----------|
| react-native-ble-plx | 3,354 | Full-featured, Expo support |
| react-native-ble-manager | 2,296 | Simpler API, very active |

#### Background Support

- **iOS**: Supported with UIBackgroundModes
- **Android**: Requires `react-native-background-actions` (foreground service)

#### Learning Curve

- **From Web Dev**: 2-4 weeks
- Same JavaScript/TypeScript
- Same React patterns

### Pros/Cons

| Pros | Cons |
|------|------|
| Single codebase for iOS + Android | Slight performance overhead |
| Fastest for web developers | Native debugging harder |
| Large community | Bridge issues occasionally |
| Expo support | |

---

## Option 4: Cross-Platform (Flutter)

### Recommended Stack

```
flutter + flutter_reactive_ble (by Philips Hue)
```

### Key Findings

#### Library Comparison

| Library | Maintainer | Notes |
|---------|------------|-------|
| flutter_reactive_ble | Philips Hue | Production-proven, stable |
| flutter_blue_plus | Community | Commercial license for 15+ employees |

#### Background Support

- **iOS**: Good with UIBackgroundModes
- **Android**: Limited, needs foreground service wrapper

#### Learning Curve

- **From Web Dev**: 6-8 weeks
- Must learn Dart
- Widget-based UI (different from React)

### Pros/Cons

| Pros | Cons |
|------|------|
| Single codebase | Dart learning curve |
| flutter_reactive_ble very stable | Background scanning limited on Android |
| Hot reload | Larger app size (~15-20MB) |
| Good platform parity | |

---

## Option 5: ESP32 BLE Implementation (Rust)

### Current Stack

```toml
esp-hal = "1.0.0"  # no_std, bare metal
```

### BLE Options

| Crate | Type | Notes |
|-------|------|-------|
| **esp-wifi** | no_std | Recommended, coex support |
| esp32-nimble | std (esp-idf) | Easier API, requires std |
| trouble | Embassy/async | Pure Rust, requires Embassy |

### Cargo.toml Addition

```toml
[dependencies]
esp-wifi = { version = "0.15", features = ["esp32s3", "ble", "coex"] }

[profile.dev.package.esp-wifi]
opt-level = 3  # Required for BLE
```

### GATT Profile Design

```
Service: RFID Data Transfer
UUID: Custom (e.g., 6E400001-B5A3-F393-E0A9-E50E24DCCA9E)

Characteristics:
  - RX (Write): Phone -> ESP32 commands
  - TX (Notify): ESP32 -> Phone data
```

### Power Considerations

| Mode | Current |
|------|---------|
| BLE Active | 10-20 mA |
| BLE Advertising | 5-15 mA |
| Light Sleep | 1-5 mA |
| Deep Sleep | 10-100 uA |

---

## Recommended Implementation Path

### Phase 1: ESP32 BLE Server (Rust)

1. Add `esp-wifi` with BLE feature
2. Implement GATT server with RX/TX characteristics
3. Commands: `READ_CHIP`, `SEND_DATA`, `GET_STATUS`
4. Test with nRF Connect app (free, iOS/Android)

### Phase 2: Mobile App MVP

**Fastest Option**: React Native + react-native-ble-plx

1. Scan for device by service UUID
2. Connect and discover characteristics
3. Send commands, receive chip data
4. Display hex dump, allow copy/export

### Phase 3: Polish (Optional)

- Native iOS app for best experience
- Native Android app
- Background sync
- Cloud backup

---

## Alternative: WiFi AP + Web Interface

If BLE is too complex, simpler alternative:

```
[Phone Browser]
    |
    | HTTP (WiFi)
    v
[ESP32 as Access Point]
    |
    | Web Server (embedded)
    v
[Simple HTML/JS UI]
```

### Pros

- No app to install
- Works on any device with browser
- Simpler ESP32 code (esp-wifi has HTTP examples)

### Cons

- Must disconnect from main WiFi
- Less polished UX
- No background operation

---

## Decision Matrix

| Factor | Native iOS | Native Android | React Native | Flutter | WiFi Web |
|--------|------------|----------------|--------------|---------|----------|
| Dev Time | 4-6 weeks | 4-6 weeks | 2-4 weeks | 4-6 weeks | 1-2 weeks |
| UX Quality | Excellent | Excellent | Very Good | Very Good | Good |
| Background | Yes | Yes | Partial | Partial | No |
| App Install | Required | Required | Required | Required | No |
| Maintenance | 2 codebases | 2 codebases | 1 codebase | 1 codebase | 1 codebase |

---

## Next Steps

1. **Decide on approach** (BLE app vs WiFi web)
2. **If BLE**: Start with ESP32 GATT server
3. **Test with nRF Connect** before writing mobile app
4. **Choose mobile framework** based on team skills
5. **Implement MVP** with basic read/export

---

## References

### iOS
- [CoreBluetooth Documentation](https://developer.apple.com/documentation/corebluetooth)
- [esp32-ble-ios-demo](https://github.com/marcboeker/esp32-ble-ios-demo)

### Android
- [Android BLE Guide](https://developer.android.com/develop/connectivity/bluetooth/ble)
- [Punch Through BLE Guides](https://punchthrough.com/blog/)

### Cross-Platform
- [react-native-ble-plx](https://github.com/dotintent/react-native-ble-plx)
- [flutter_reactive_ble](https://github.com/PhilipsHue/flutter_reactive_ble)

### ESP32
- [esp-wifi crate](https://docs.rs/esp-wifi)
- [esp32-nimble](https://github.com/taks/esp32-nimble)
