# KeepKey Bootloader Update and USB Protocol Transition Guide

## Overview

This document describes the critical issues and solutions for handling KeepKey device bootloader updates, particularly the transition from legacy HID-based bootloaders to modern WebUSB-enabled bootloaders. These insights are crucial for any vault implementation handling device onboarding and updates.

## Problem Statement

When updating a KeepKey device from bootloader v1.x to v2.x, several critical issues occur:

1. **USB Protocol Change**: The device changes its Product ID (PID) from `0x0001` (HID) to `0x0002` (WebUSB/USB)
2. **Transport Incompatibility**: The device disconnects and reconnects with a different USB transport type
3. **Device Identity Loss**: The system loses track of the device due to the PID change
4. **Data Corruption**: Devices may have corrupted policy data after the update

## USB Protocol Details

### Product ID Mapping

| Bootloader Version | Product ID | Transport Type | Capabilities |
|-------------------|------------|----------------|--------------|
| v1.x (Legacy)     | `0x0001`   | HID Only       | Limited API, no debug interface |
| v2.x (Modern)     | `0x0002`   | WebUSB/USB     | Full API, debug interface, bulk transfers |

### Transport Detection Logic

```rust
// Legacy devices (PID 0x0001) must use HID on all platforms
if device_info.pid == 0x0001 {
    return TransportType::HidOnly;
}

// Modern devices (PID 0x0002) use WebUSB/USB transport
if device_info.pid == 0x0002 {
    return TransportType::TraditionalUsb;
}
```

## Implementation Solutions

### 1. Handling PID Transition During Bootloader Update

**Location**: `device_queue.rs::handle_update_bootloader`

**Key Changes**:
- Detect when updating from old bootloader (PID `0x0001`)
- After successful update, automatically update device info to expect PID `0x0002`
- Clear transport to force recreation with new protocol

```rust
// Remember if we started with PID 0x0001 (old bootloader)
let started_with_old_bootloader = self.device_info.pid == 0x0001;

// After successful bootloader update
if started_with_old_bootloader {
    info!("📝 Device will reconnect with PID 0x0002 after bootloader update");
    self.device_info.pid = 0x0002;
    self.transport = None; // Force transport recreation
}
```

### 2. Dynamic Device Reconnection Detection

**Location**: `device_queue.rs::ensure_transport`

**Key Changes**:
- If transport creation fails with expected PID, search for device with same serial but different PID
- Automatically update device info when PID change is detected
- Retry transport creation with updated information

```rust
// If failed and PID is 0x0002, try looking for device with same serial
if transport_result.is_err() && self.device_info.pid == 0x0002 {
    // Search for device by serial number, ignoring PID
    let devices = rusb::devices().unwrap();
    for device in devices.iter() {
        if let Ok(desc) = device.device_descriptor() {
            if desc.vendor_id() == self.device_info.vid {
                // Check serial number match
                if device_serial == expected_serial && desc.product_id() != self.device_info.pid {
                    // Update PID and retry
                    self.device_info.pid = desc.product_id();
                    transport_result = create_transport_for_device(&self.device_info);
                }
            }
        }
    }
}
```

### 3. Flexible Device Discovery

**Location**: `device_queue.rs::find_physical_device_by_info`

**Key Changes**:
- Prioritize serial number matching over PID matching
- Log PID changes for debugging
- Fall back to any KeepKey device if exact match fails

```rust
// Match by serial number (flexible - allows PID change after bootloader update)
if desc.vendor_id() == device_info.vid {
    if device_serial == expected_serial {
        // Log if PID changed (happens after bootloader update)
        if desc.product_id() != device_info.pid {
            info!("📝 Device reconnected with different PID: 0x{:04x} -> 0x{:04x}", 
                  device_info.pid, desc.product_id());
        }
        return Ok(device.clone());
    }
}
```

### 4. Handling Corrupted Policy Data

**Location**: `device_queue.rs::handle_get_features`

**Problem**: After bootloader update, devices may have corrupted policy data causing protobuf decode errors.

**Solution**:
- Detect "invalid UTF-8" errors in policy fields
- Automatically attempt device wipe to clear corruption
- Provide clear user feedback if manual confirmation needed

```rust
if error_str.contains("invalid string value: data is not UTF-8 encoded") || 
   error_str.contains("PolicyType.policy_name") {
    warn!("⚠️ Device appears to have corrupted policy data after bootloader update");
    
    // Try to wipe the device to clear corrupted data
    let wipe_result = transport.handle(WipeDevice {}.into());
    
    match wipe_result {
        Ok(Message::Success(_)) => {
            // Retry GetFeatures after successful wipe
            transport.handle(GetFeatures {}.into())?
        }
        Ok(Message::ButtonRequest(_)) => {
            return Err(anyhow!("Device has corrupted data and needs to be wiped. 
                               Please confirm the wipe on your device."));
        }
        _ => return Err(e);
    }
}
```

## Critical Implementation Considerations

### 1. Device State After Bootloader Update

After a bootloader update from v1.x to v2.x:
- Device will disconnect and reconnect with different PID
- Device may be in bootloader mode requiring firmware update
- Device may have corrupted or uninitialized storage
- Transport type changes from HID to WebUSB/USB

### 2. Timing and Retry Logic

- Allow longer timeouts after bootloader updates (2-5 seconds)
- Implement exponential backoff for reconnection attempts
- Clear transport and force recreation after updates

### 3. Error Recovery Strategies

1. **Transport Creation Failure**: Search for device with different PID
2. **Protobuf Decode Errors**: Attempt automatic device wipe
3. **Communication Timeouts**: Wait for device reboot and retry
4. **Device Not Found**: Check all PIDs for matching serial number

## Testing Scenarios

### Scenario 1: Legacy to Modern Bootloader Update
1. Start with device on bootloader v1.0.3 (PID `0x0001`)
2. Perform bootloader update to v2.1.4
3. Verify device reconnects with PID `0x0002`
4. Confirm transport switches from HID to WebUSB/USB

### Scenario 2: Corrupted Device Recovery
1. Device with corrupted policy data after update
2. System detects protobuf decode error
3. Automatic wipe initiated
4. Device successfully initialized after wipe

### Scenario 3: Multiple Device Handling
1. Multiple KeepKey devices connected
2. Update one device's bootloader
3. Verify correct device tracking through PID change
4. Ensure other devices remain unaffected

## Common Issues and Solutions

| Issue | Symptom | Solution |
|-------|---------|----------|
| Device lost after update | "Device not found" errors | Search by serial number, ignore PID |
| HID read timeout | Communication timeouts after update | Device switched to WebUSB, update transport |
| Protobuf decode error | "invalid UTF-8" in policies | Device needs wipe and re-initialization |
| Wrong transport type | Connection failures | Detect PID and choose appropriate transport |

## Code Locations

All critical changes are in: `/projects/keepkey-bitcoin-only/projects/keepkey-rust/device_queue.rs`

- **Lines 553-631**: `handle_update_bootloader` - PID transition handling
- **Lines 291-371**: `ensure_transport` - Dynamic reconnection logic
- **Lines 375-421**: `handle_get_features` - Corruption detection and recovery
- **Lines 990-1028**: `find_physical_device_by_info` - Flexible device discovery

## Recommendations for Future Vault Versions

1. **Pre-Update Detection**: Check device PID before starting updates
2. **User Communication**: Inform users about expected disconnection/reconnection
3. **Progress Tracking**: Maintain update state across disconnections
4. **Automatic Recovery**: Implement all recovery mechanisms from the start
5. **Testing**: Always test with actual hardware transitions, not just emulators

## References

- [USB Rules Documentation](./usb/docs/usb-rules.md)
- [WebUSB Requirements](./usb/docs/keepkey-webusb-requirements.md)
- [Transport History](./keepkey/transport_history.md)
- [Device Controller Implementation](./usb/docs/device_controller_implementation.md)