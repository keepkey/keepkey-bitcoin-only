# USB/HID Driver Issues: Summary & Action Items

## Quick Summary

**Problem**: Vault-v2 has HID connectivity issues that don't exist in KeepKey Desktop v5.

**Root Cause**: Vault-v2 uses `keepkey_rust` library which has aggressive error handling that fails when devices are busy or claimed by other applications. V5 has its own implementation that gracefully handles these scenarios.

## Key Differences

| Aspect | KeepKey Desktop v5 | Vault-v2 |
|--------|-------------------|----------|
| USB/HID Implementation | Direct, self-contained | Delegated to keepkey_rust |
| Error Handling | Graceful degradation | Fail-fast approach |
| Device Busy/Claimed | Continues with warnings | Stops with error |
| USB→HID Fallback | Automatic | Relies on keepkey_rust |
| Windows Support | Custom polling logic | Standard hidapi |

## Immediate Action Items

### 1. Quick Fix in keepkey_rust (Priority: HIGH)
**File**: `keepkey-bitcoin-only/projects/keepkey-rust/transport/hid.rs`

Change the `handle_device_open_error` function:
```rust
// FROM: Returning error on device claim issues
return Err(anyhow!("Device already in use..."));

// TO: Log warning and continue
warn!("Device may be in use by another application, attempting connection anyway...");
Ok(()) // Continue trying other connection methods
```

### 2. Add Retry Logic in vault-v2 (Priority: HIGH)
**File**: `keepkey-bitcoin-only/projects/vault-v2/src-tauri/src/commands.rs`

Add retry wrapper for device connections:
```rust
async fn connect_with_retry(device_id: &str, max_attempts: u32) -> Result<DeviceHandle, String> {
    for attempt in 1..=max_attempts {
        match connect_device(device_id).await {
            Ok(handle) => return Ok(handle),
            Err(e) if attempt < max_attempts => {
                warn!("Connection attempt {} failed: {}, retrying...", attempt, e);
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
            Err(e) => return Err(e),
        }
    }
    unreachable!()
}
```

### 3. Implement Direct HID Fallback (Priority: MEDIUM)
**File**: Create `keepkey-bitcoin-only/projects/vault-v2/src-tauri/src/transport_fallback.rs`

```rust
pub async fn connect_with_fallback(device: &DeviceInfo) -> Result<Connection, String> {
    // Try USB first
    match try_usb_connection(device).await {
        Ok(conn) => Ok(conn),
        Err(usb_err) => {
            warn!("USB connection failed: {}, trying HID", usb_err);
            // Try HID as fallback
            try_hid_connection(device).await
                .map_err(|hid_err| format!("Both USB and HID failed: USB({}), HID({})", usb_err, hid_err))
        }
    }
}
```

## Testing Checklist

- [ ] Test with KeepKey Desktop v5 running simultaneously
- [ ] Test on Windows without admin privileges
- [ ] Test rapid connect/disconnect (10 times in succession)
- [ ] Test with device already claimed by another app
- [ ] Test recovery from "device busy" state
- [ ] Test with multiple KeepKey devices connected

## Long-term Recommendations

1. **Consider adopting v5's transport implementation** - It's proven to work reliably across platforms
2. **Create unified transport library** - Share code between all KeepKey applications
3. **Add transport selection UI** - Let users manually choose USB or HID if auto-detection fails

## Expected Outcome

After implementing these changes:
- Vault-v2 should connect to devices even when other apps are running
- Windows users should see fewer "device in use" errors
- Automatic fallback from USB to HID should work seamlessly
- Overall reliability should match KeepKey Desktop v5 