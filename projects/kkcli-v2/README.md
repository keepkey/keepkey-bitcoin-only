# KeepKey CLI v2

Enhanced KeepKey device management CLI powered by `keepkey-rust`.

## Features

- 📱 **Device Discovery**: List all connected KeepKey devices
- 🔧 **Bootloader Detection**: Detect OOB (out-of-box) bootloader mode
- 🚌 **Transport Testing**: Test both USB and HID communication paths
- 📊 **Rich Output**: Beautiful tables with device status and features
- 🔍 **Debugging**: Detailed device communication testing

## Installation

From the project root:

```bash
cd projects/kkcli-v2
cargo build --release
```

## Usage

### Basic Commands

List all connected devices:
```bash
./target/release/kkcli-v2 list
```

List only KeepKey devices:
```bash
./target/release/kkcli-v2 list --keepkey-only
```

Show detailed device features:
```bash
./target/release/kkcli-v2 list --features
```

Include bootloader detection:
```bash
./target/release/kkcli-v2 list --features --detect-bootloader
```

### Testing Commands

Test all KeepKey devices:
```bash
./target/release/kkcli-v2 test
```

Test specific device:
```bash
./target/release/kkcli-v2 test --device-id "932313031174732313008100"
```

Test bootloader detection:
```bash
./target/release/kkcli-v2 test --bootloader-test
```

Test both USB and HID transports:
```bash
./target/release/kkcli-v2 test --transport-test
```

Full debugging test:
```bash
./target/release/kkcli-v2 test --bootloader-test --transport-test --verbose
```

## Example Output

### Device List
```
KeepKey CLI v2 - Enhanced Device Management
Powered by keepkey-rust

🔍 Scanning for connected devices...
✅ Found 1 device(s)

┌─────────────────────────────┬─────────┬─────────┬─────────────────────────────┬─────────────────┬─────────┐
│ Device ID                   │ Name    │ VID:PID │ Serial                      │ Manufacturer    │ KeepKey │
├─────────────────────────────┼─────────┼─────────┼─────────────────────────────┼─────────────────┼─────────┤
│ 932313031174732313008100    │ KeepKey │ 2b24:0001│ 932313031174732313008100    │ KeepKey, LLC.   │ ✅      │
└─────────────────────────────┴─────────┴─────────┴─────────────────────────────┴─────────────────┴─────────┘
```

### Detailed Features
```
📡 Getting features for: KeepKey (932313031174732313008100)

┌─────────────────────────────┬─────────┬─────────┬───────────────────┬─────────────┬─────────┬───────────────┬─────────────────────┐
│ Device ID                   │ Label   │ Version │ Bootloader Mode   │ Initialized │ Model   │ Vendor        │ Status              │
├─────────────────────────────┼─────────┼─────────┼───────────────────┼─────────────┼─────────┼───────────────┼─────────────────────┤
│ 932313031174732313008100    │ Unnamed │ 1.0.3   │ ✅ YES           │ ⚠️ NO       │ Unknown │ keepkey.com   │ 🔧 Legacy Bootloader│
└─────────────────────────────┴─────────┴─────────┴───────────────────┴─────────────┴─────────┴───────────────┴─────────────────────┘
```

### Communication Test
```
🧪 KeepKey Device Communication Test

🔍 Testing device: KeepKey (932313031174732313008100)
   VID:PID: 2b24:0001
   Serial: 932313031174732313008100

📡 Test 1: Basic device communication (USB → HID fallback)
   ❌ Communication failed: Failed to get device features
   🔧 Attempting OOB bootloader detection...
   ✅ OOB bootloader detection successful
   📋 Device Features:
      Label: Unnamed
      Version: 1.0.3
      Bootloader Mode: YES
      Initialized: NO
      Model: Unknown
      Vendor: keepkey.com
      🔧 BOOTLOADER MODE DETECTED
      ⚠️  Legacy bootloader - needs update
```

## Troubleshooting OOB Bootloader Issues

If you're experiencing "Device Already In Use" errors with OOB bootloader detection:

1. **Close all other KeepKey applications**:
   - KeepKey Desktop app
   - KeepKey Bridge
   - Chrome extensions using KeepKey
   - Other wallet applications

2. **Test with kkcli-v2**:
   ```bash
   # Basic test
   ./target/release/kkcli-v2 list --features --detect-bootloader
   
   # Full debugging test
   ./target/release/kkcli-v2 test --bootloader-test --transport-test --verbose
   ```

3. **Check device mode**:
   - If "Bootloader Mode: YES" → Device is in bootloader mode
   - If "Version: 1.0.3" or similar → Legacy bootloader needs update
   - If "Status: Legacy Bootloader" → Update required

4. **Transport debugging**:
   ```bash
   ./target/release/kkcli-v2 test --transport-test
   ```
   This will test both USB and HID transports separately to identify which one works.

## Architecture

- **Built on keepkey-rust**: Uses the proven device communication library
- **Async/await**: Modern async Rust for better performance  
- **Multiple Transports**: Automatic USB → HID fallback
- **OOB Detection**: Implements documented bootloader detection heuristics
- **Rich CLI**: Uses `clap` for argument parsing and `tabled` for output formatting

## Device Status Indicators

| Status | Meaning |
|--------|---------|
| ✅ Ready | Device is initialized and ready to use |
| ⚙️ Needs Setup | Device needs initialization |
| 🔧 Bootloader Mode | Device is in bootloader mode |
| 🔧 Legacy Bootloader | Device has legacy bootloader and needs update |
| ❌ Communication Error | Cannot communicate with device |

## Contributing

This CLI is designed to be a debugging and testing tool for KeepKey device communication. When adding features:

1. Use `keepkey-rust` functions whenever possible
2. Add proper error handling and user-friendly messages
3. Include both basic and verbose output modes
4. Test with both USB and HID transports 