# Passphrase and PIN Flow Fixes Summary

## Issues Fixed

### 1. PIN Recognition Issue
**Problem**: Device was rejecting PIN with "Failure: Unexpected message"
**Root Cause**: `get_device_status` was sending `GetFeatures` and `Initialize` commands while the device was waiting for `PinMatrixAck`, which reset the device's state machine.

### 2. Passphrase Dialog Not Appearing
**Problem**: Passphrase dialog wasn't appearing after successful PIN entry
**Root Cause**: Two issues:
- `send_pin_matrix_ack` wasn't emitting the passphrase request event when device responded with `PassphraseRequest`
- `trigger_pin_request` wasn't emitting passphrase event when device was already PIN unlocked

## Changes Made

### 1. PIN Flow Protection (`commands.rs`)
```rust
// In get_device_status - Check if device is in PIN flow before sending commands
if is_device_in_pin_flow(&device_id) {
    // Use cached state instead of sending commands that would interrupt PIN entry
    // Return cached features or minimal status
}
```

### 2. Event Controller Protection (`event_controller.rs`)
```rust
// In try_get_device_features - Added PIN flow check before OOB bootloader detection
if crate::commands::is_device_in_pin_flow(&device.unique_id) {
    // Skip OOB bootloader detection to avoid sending Initialize
}
```

### 3. Passphrase Event Emission After PIN (`commands.rs`)
```rust
// In send_pin_matrix_ack - When device responds with PassphraseRequest after PIN
keepkey_rust::messages::Message::PassphraseRequest(_req) => {
    // Emit passphrase_request event to frontend
    // Mark passphrase request as active in state
}
```

### 4. Passphrase Event for Already Unlocked Devices (`commands.rs`)
```rust
// In trigger_pin_request - When device is already PIN unlocked
Ok(keepkey_rust::messages::Message::PassphraseRequest(_)) => {
    // Device is already PIN unlocked but needs passphrase
    // Emit passphrase_request event
    // Mark passphrase request as active
}
```

### 5. Type Exports (`device/mod.rs`)
- Made `DEVICE_STATE_CACHE` and `DeviceStateCache` public
- Exported `PASSPHRASE_REQUEST_STATE` and `PassphraseRequestState`

## Flow Summary

1. **PIN Entry Flow**:
   - User triggers PIN request → Device shows PIN matrix
   - Backend marks device as "in PIN flow"
   - All status checks use cached state to avoid interrupting
   - User enters PIN → Device accepts → Returns PassphraseRequest
   - Backend emits passphrase event → Frontend shows passphrase dialog

2. **Already PIN Unlocked Flow**:
   - Device is already PIN unlocked (cached) but needs passphrase
   - Frontend triggers PIN request → Device returns PassphraseRequest
   - Backend recognizes device is already unlocked
   - Backend emits passphrase event → Frontend shows passphrase dialog

## Key Improvements

1. **State Machine Integrity**: Device state is no longer reset during PIN entry
2. **Event Flow**: Proper event emission ensures UI dialogs appear when needed
3. **Cache Usage**: Cached features prevent unnecessary device communication
4. **Flow Protection**: PIN flow state prevents interference from other operations