# KeepKey Passphrase UX Audit and Fix

## Current Issues Identified

### 1. Incorrect Passphrase Entry Form
**Problem**: The current `SimplePassphraseModal.tsx` has both "Passphrase" and "Confirm Passphrase" fields.

**Correct Pattern**: The passphrase entry should be a single field with the message "After submitting your passphrase, use the button on the KeepKey to approve it." The confirmation happens on the device, not in the UI.

**Reference Image 1**: Shows the correct single-field pattern:
- Single passphrase input field
- Button to submit passphrase 
- Device confirmation message

### 2. Settings Toggle UX Issue
**Problem**: The settings toggle doesn't properly show the waiting state for device confirmation.

**Correct Pattern**: When toggling passphrase protection in settings:
1. Show spinner during "Enable Passphrase" operation
2. Display "Press and hold the button on your KeepKey to change the device Passphrase" message
3. Wait for device confirmation before proceeding
4. Application should reboot after successful enablement

**Reference Image 2**: Shows the correct waiting pattern with spinner and device instruction.

### 3. Missing Application Reboot
**Problem**: After enabling passphrase protection, the device state changes but the application doesn't properly detect this.

**Correct Pattern**: The entire application should reboot/restart to properly detect that the device now has passphrase protection enabled and requires passphrase entry for operations.

## Technical Implementation Requirements

### 1. Fix Passphrase Entry Modal
- Remove "Confirm Passphrase" field
- Update text to match the correct pattern
- Simplify validation logic
- Update styling to match reference design

### 2. Fix Settings Toggle Flow
- Add proper spinner during device operation
- Show correct instructional text
- Implement proper device confirmation waiting
- Trigger application reboot after successful enable

### 3. Improve Device State Detection
- Detect passphrase protection status on app startup
- Handle device reconnection after passphrase enable
- Properly manage application lifecycle during device state changes

## Security Considerations

1. **Single Entry Point**: Having only one passphrase field is actually more secure as it matches the device confirmation pattern
2. **No Local Confirmation**: The device handles confirmation, reducing attack surface
3. **Clear Memory**: Passphrase should still be cleared from memory immediately after submission
4. **Device Authority**: Device is the authoritative source for passphrase validation

## Implementation Status

### ✅ COMPLETED
1. **`src/components/SimplePassphraseModal.tsx`** - Fixed passphrase entry form
   - ✅ Removed "Confirm Passphrase" field 
   - ✅ Updated header to "Enter Your Passphrase"
   - ✅ Added correct instruction: "After submitting your passphrase, use the button on the KeepKey to approve it"
   - ✅ Simplified validation logic
   - ✅ Updated styling to match reference design

2. **`src/components/PassphraseSettings.tsx`** - Fixed settings toggle UX
   - ✅ Updated description text to match reference
   - ✅ Added spinner during device operation
   - ✅ Shows correct instructional text: "Press and hold the button on your KeepKey to change the device Passphrase"
   - ✅ Implemented proper application reboot after successful enable

3. **Application Lifecycle Management**
   - ✅ Uses `restart_backend_startup` command to properly reset device state
   - ✅ Implements app reload after passphrase enable
   - ✅ Proper timing for backend restart and app reload

4. **Device State Detection**
   - ✅ Device features already include `passphraseProtection: boolean` field
   - ✅ `KeepKeyDeviceList.tsx` correctly detects and passes passphrase status
   - ✅ System refreshes device list after passphrase toggle changes

## Implementation Notes

- The passphrase system now correctly follows the device-confirmation pattern
- Single passphrase entry field matches KeepKey's security model
- Application properly restarts to detect new device state
- Device passphrase protection status is correctly detected on startup via device features