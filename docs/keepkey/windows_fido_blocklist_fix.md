# Windows FIDO/U2F Blocklist Fix for KeepKey

## Problem Description

Windows 10 (build 1903+) includes a CTAP-HID filter driver that automatically claims any USB device with a FIDO usage page (0xF1D0). Early KeepKey bootloaders advertised this page for Chrome U2F compatibility, causing Windows to block direct USB access with "Access denied" errors.

This results in:
- "Feature object false" errors in logs
- `evaluate_device_status` receiving `None` features
- Device appearing disconnected when it's actually connected

## Solution Implemented

We've implemented a permanent fix that:
1. **Forces HID transport on Windows** for all KeepKey devices
2. **Bypasses the FIDO filter** completely
3. **Works for all device PIDs** (0x0001, 0x0002, etc.)
4. **Doesn't affect macOS/Linux** behavior

### Code Changes

In `projects/keepkey-rust/device_queue.rs`:

```rust
// WINDOWS FIDO BLOCKLIST FIX: Always use HID on Windows to avoid FIDO filter driver
#[cfg(target_os = "windows")]
{
    info!("🪟 Windows detected - forcing HID transport to avoid FIDO/U2F blocklist issues");
    return Ok(TransportType::HidOnly);
}
```

## Why This Works

1. **HID bypasses FIDO filter**: Windows allows HID access without elevated permissions
2. **Same protocol support**: Both USB and HID transports implement the same KeepKey protocol
3. **No driver changes needed**: Uses built-in Windows HID drivers
4. **Automatic fallback**: If HID fails, error is properly reported

## Testing

To verify the fix:
1. Connect a KeepKey on Windows
2. Check logs for "🪟 Windows detected - forcing HID transport"
3. Verify device features are retrieved successfully
4. Confirm no "Access denied" errors

## Alternative Solutions (Not Recommended)

1. **Zadig/WinUSB**: Requires admin rights and driver replacement
2. **Native messaging helper**: Extra process complexity
3. **Registry edits**: System-wide changes affecting other apps

## References

- Windows FIDO filter documentation
- Chrome HID blocklist: https://source.chromium.org/chromium/chromium/src/+/main:services/device/public/cpp/hid/hid_blocklist.cc
- Original issue: Windows CTAP-HID filter claiming KeepKey devices 