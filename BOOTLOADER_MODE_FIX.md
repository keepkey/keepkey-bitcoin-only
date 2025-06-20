# Bootloader Mode Device Status Fix

## Problem
The vault-v2 application was incorrectly marking devices in bootloader mode as "ready", which led to confusing UX where:
- Device was clearly in bootloader mode (`bootloader_mode: true`)
- Bootloader version was `2.1.4` 
- Device was emitting "Device ready" status
- This caused the app to try to show the normal vault interface instead of the bootloader update flow

## Root Cause Analysis

### Issue 1: Incorrect bootloader version detection
In `commands.rs`, the `evaluate_device_status` function had flawed logic:

```rust
let current_bootloader_version = if features.bootloader_mode {
    if features.version.starts_with("1.") {
        features.version.clone() // OOB bootloader versions like 1.0.3
    } else {
        "Unknown bootloader".to_string()  // ❌ WRONG!
    }
} else {
    // ... normal mode logic
};

let needs_bootloader_update = if current_bootloader_version == "Unknown bootloader" {
    false // ❌ WRONG! Assuming no update needed
} else {
    // ... version comparison logic
};
```

**Problem**: When a device was in bootloader mode with version "2.1.4", it was setting the bootloader version to "Unknown bootloader" and then assuming no update was needed.

### Issue 2: Event controller not checking bootloader mode
In `event_controller.rs`, the readiness logic was missing a critical check:

```rust
let is_actually_ready = !status.needs_bootloader_update && 
                       !status.needs_firmware_update && 
                       !status.needs_initialization &&
                       !is_pin_locked;  // ❌ Missing bootloader_mode check!
```

**Problem**: A device in bootloader mode should NEVER be considered "ready" regardless of update status.

## Fixes Implemented

### Fix 1: Proper bootloader version detection
```rust
let current_bootloader_version = if features.bootloader_mode {
    if features.version.starts_with("1.") {
        features.version.clone() // OOB bootloader versions like 1.0.3
    } else {
        // Modern bootloader in bootloader mode - use version directly
        features.version.clone()  // ✅ FIXED!
    }
} else {
    // ... normal mode logic unchanged
};

let needs_bootloader_update = if features.bootloader_mode {
    // CRITICAL: If device is in bootloader mode, it ALWAYS needs update
    // (even if version appears up-to-date, it needs to exit bootloader mode)
    true  // ✅ FIXED!
} else if current_bootloader_version == "Unknown bootloader" {
    false // Can't determine, assume no update needed
} else {
    // ... version comparison logic unchanged
};
```

### Fix 2: Event controller bootloader mode check
```rust
// CRITICAL: Device in bootloader mode is NEVER ready
let is_actually_ready = !features.bootloader_mode &&  // ✅ FIXED!
                       !status.needs_bootloader_update && 
                       !status.needs_firmware_update && 
                       !status.needs_initialization &&
                       !is_pin_locked;
```

### Fix 3: Better status messages
```rust
let status_message = if features.bootloader_mode {
    "Device in bootloader mode"  // ✅ Clear status for bootloader mode
} else if is_pin_locked {
    "Device locked - enter PIN"
} else if status.needs_bootloader_update {
    "Bootloader update needed"
} else if status.needs_firmware_update {
    "Firmware update needed"
} else if status.needs_initialization {
    "Device setup needed"
} else {
    "Device ready"
};
```

### Fix 4: Enhanced debug logging
```rust
println!("⚠️ Device connected but needs updates (bootloader_mode: {}, bootloader: {}, firmware: {}, init: {}, pin_locked: {})", 
         features.bootloader_mode,  // ✅ Show bootloader mode status
         status.needs_bootloader_update, 
         status.needs_firmware_update, 
         status.needs_initialization,
         is_pin_locked);
```

## Test Command Added
Created `test_bootloader_mode_device_status()` command to verify the fix:
- Creates mock device features with `bootloader_mode: true` and `version: "2.1.4"`
- Verifies that `evaluate_device_status()` correctly marks it as needing bootloader update
- Ensures devices in bootloader mode are never considered "ready"

## Expected Behavior After Fix

### Before (Broken)
```
🔧 Bootloader check: Unknown bootloader -> needs update: false (bootloader_mode: true)
✅ Device is fully ready, emitting device:ready event  // ❌ WRONG!
📡 Emitting status: Device ready  // ❌ WRONG!
```

### After (Fixed)
```
🔧 Bootloader check: 2.1.4 -> needs update: true (bootloader_mode: true)
⚠️ Device connected but needs updates (bootloader_mode: true, bootloader: true, firmware: false, init: false, pin_locked: false)
📡 Emitting status: Device in bootloader mode  // ✅ CORRECT!
```

## Impact
- Devices in bootloader mode will now properly trigger the bootloader update flow
- Users will see clear "Device in bootloader mode" status instead of confusing "Device ready"
- DeviceUpdateManager will correctly show bootloader update dialog instead of vault interface
- Prevents user confusion and ensures proper device update workflow

## Files Modified
1. `projects/vault-v2/src-tauri/src/commands.rs` - Fixed `evaluate_device_status()` logic
2. `projects/vault-v2/src-tauri/src/event_controller.rs` - Fixed readiness check and status messages  
3. `projects/vault-v2/src-tauri/src/lib.rs` - Added new test command registration

## Verification
- Code compiles successfully with no errors
- Added comprehensive test command to verify fix
- Enhanced logging to make bootloader mode detection visible
- Maintains backward compatibility with existing functionality 