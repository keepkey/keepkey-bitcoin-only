# Vault Onboarding Implementation Notes

## Executive Summary

This document captures critical implementation details discovered while fixing bootloader update and device recovery issues in vault-v2. These findings are essential for implementing robust device onboarding in future vault versions.

## Key Discoveries

### 1. USB Protocol Transition is Critical

**Finding**: KeepKey devices undergo a fundamental USB protocol change when updating from legacy bootloaders (v1.x) to modern bootloaders (v2.x).

**Impact**: 
- Device identity changes (PID 0x0001 → 0x0002)
- Transport layer must switch (HID → WebUSB/USB)
- Existing connections become invalid

**Implementation Requirements**:
- Device tracking must use serial numbers, not PID
- Transport creation must be dynamic and adaptable
- Reconnection logic must handle protocol changes

### 2. Device State Corruption is Common

**Finding**: Devices frequently have corrupted policy data after bootloader updates.

**Symptoms**:
```
failed to decode Protobuf message: PolicyType.policy_name: Features.policies: 
invalid string value: data is not UTF-8 encoded
```

**Solution**: Automatic device wipe and re-initialization must be part of the update flow.

### 3. Transport Detection Must Be Flexible

**Current Implementation Gaps**:
- Hard-coded PID expectations
- Rigid transport type assumptions
- No fallback mechanisms

**Required Flexibility**:
```rust
// Don't assume PID remains constant
// Don't assume transport type
// Always have fallback strategies
```

## Onboarding Flow Requirements

### Phase 1: Device Detection
1. Scan for all KeepKey devices (VID 0x2b24)
2. Don't filter by PID initially
3. Determine device state (bootloader mode, firmware version, initialized)
4. Track devices by serial number

### Phase 2: Bootloader Update
1. Detect legacy bootloader (v1.x with PID 0x0001)
2. Prepare for USB protocol change
3. Execute update
4. Handle disconnection gracefully
5. Wait for reconnection with new PID
6. Update internal device tracking

### Phase 3: Firmware Update
1. Ensure device in bootloader mode
2. Handle potential corruption from previous state
3. Upload firmware
4. Wait for device restart
5. Verify successful update

### Phase 4: Device Initialization
1. Check for corrupted data (protobuf errors)
2. Wipe if necessary
3. Initialize device
4. Set up PIN
5. Generate or restore seed

## Critical Code Patterns

### Pattern 1: Flexible Device Discovery
```rust
// Good: Find by serial, adapt to PID changes
fn find_device(serial: &str) -> Result<Device> {
    for device in list_all_devices() {
        if device.serial == serial {
            // Found it, even if PID changed
            return Ok(device);
        }
    }
}

// Bad: Rigid PID expectations
fn find_device(pid: u16) -> Result<Device> {
    // Will fail after bootloader update
}
```

### Pattern 2: Transport Adaptation
```rust
// Good: Detect and adapt to transport type
fn create_transport(device: &Device) -> Result<Transport> {
    match device.pid {
        0x0001 => create_hid_transport(),
        0x0002 => create_webusb_transport(),
        _ => detect_and_create_transport()
    }
}

// Bad: Hard-coded transport assumptions
fn create_transport() -> HidTransport {
    // Fails for modern devices
}
```

### Pattern 3: Error Recovery
```rust
// Good: Detect corruption and recover
match get_features() {
    Err(e) if e.contains("invalid UTF-8") => {
        wipe_device()?;
        get_features()
    }
    result => result
}

// Bad: Propagate errors without recovery
get_features()?  // User stuck with corrupted device
```

## Testing Checklist

### Hardware Scenarios
- [ ] Legacy bootloader (v1.0.3) to modern (v2.1.4) update
- [ ] Device with corrupted policy data
- [ ] Multiple devices connected simultaneously
- [ ] Device disconnection during update
- [ ] Power loss during update

### Software Scenarios
- [ ] Transport type detection
- [ ] PID change handling
- [ ] Serial number tracking
- [ ] Error recovery mechanisms
- [ ] Timeout and retry logic

## Common Pitfalls to Avoid

1. **Assuming Static Device Properties**
   - PIDs change during updates
   - Transport types vary by bootloader version
   - Device paths change on reconnection

2. **Insufficient Error Handling**
   - Protobuf decode errors are common
   - Devices often need wiping after updates
   - Communication timeouts are expected

3. **Rigid Transport Layer**
   - Must support HID, WebUSB, and USB
   - Must handle transitions between types
   - Must have fallback mechanisms

4. **Poor User Communication**
   - Users need to know when to wait
   - Users need to know when to act (button presses)
   - Users need clear error messages

## Recommendations for Future Implementations

### Architecture
1. Implement device tracking by serial number from the start
2. Design transport layer to be protocol-agnostic
3. Build in automatic recovery mechanisms
4. Separate device identity from USB properties

### User Experience
1. Show clear progress during updates
2. Explain disconnections are expected
3. Provide recovery instructions for failures
4. Auto-detect and fix common issues

### Code Organization
1. Centralize USB protocol knowledge
2. Abstract transport differences
3. Implement comprehensive logging
4. Create reusable recovery utilities

## File References

### Modified Files (vault-v2)
- `/projects/keepkey-rust/device_queue.rs` - Core fixes for USB transitions

### Documentation Created
- `/docs/bootloader-update-usb-transition.md` - Technical details
- `/docs/vault-onboarding-implementation-notes.md` - This document

### Existing Documentation
- `/docs/usb/docs/usb-rules.md` - USB device rules
- `/docs/usb/docs/keepkey-webusb-requirements.md` - WebUSB requirements
- `/docs/keepkey/transport_history.md` - Transport evolution

## Conclusion

Successful vault onboarding requires understanding and handling the complex USB protocol transitions that occur during device updates. The key is flexibility - don't assume device properties remain static, always have fallback mechanisms, and implement automatic recovery for common issues. These lessons learned from vault-v2 should be applied to all future vault implementations to ensure robust device onboarding.