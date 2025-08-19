# Passphrase Dialog Not Showing After PIN Entry - Resolution

## Issue Description
When the device requested a passphrase after PIN entry (not at startup), the PIN dialog remained visible instead of transitioning to the passphrase dialog. The device screen showed "Enter Passphrase" but the frontend continued showing the PIN entry screen.

## Root Cause
1. **Delayed PIN Dialog Closure**: The PIN dialog had a 1-second delay before closing after successful PIN submission (for visual feedback)
2. **Dialog Priority System**: The PIN dialog had absolute priority in the dialog queue, preventing the passphrase dialog from being shown even when added to the queue
3. **Multiple Duplicate Event Listeners**: 
   - Both WalletContext and DeviceUpdateManager were listening to `device:pin-request-triggered` event
   - WalletContext had TWO listeners for PIN events (one for address failures, one for pin-request-triggered)
   - useDeviceInteraction hook was listening to `device:awaiting_pin` and showing a different PIN dialog component (DevicePinDialog)
4. **Excessive PIN Triggers**: DeviceUpdateManager was calling `trigger_pin_request` when receiving `device:passphrase-unlock-needed` event, causing 20+ duplicate requests
5. **Auto-triggering in Dialog**: PinUnlockDialog was automatically triggering another PIN request when opened, even though the backend had already triggered one
6. **Multiple PIN Dialog Components**: System had two different PIN dialog components (PinUnlockDialog and DevicePinDialog) being shown by different parts of the code

## Solution Implemented

### 1. Immediate PIN Dialog Closure
**File**: `projects/vault-v2/src/components/PinUnlockDialog.tsx`
- Removed the 1-second timeout delay when PIN is submitted successfully
- PIN dialog now closes immediately, allowing passphrase dialog to show without delay
- This prevents the UI from being stuck showing the wrong dialog while the device is in a different state

### 2. Dialog Priority Adjustment
**File**: `projects/vault-v2/src/contexts/DialogContext.tsx`
- Updated `processQueue` function to handle passphrase dialogs with appropriate priority
- Added explicit check for passphrase dialogs after PIN dialogs
- Ensures passphrase dialog is shown immediately when PIN dialog is removed from queue

### 3. Remove Duplicate Event Listeners
**File**: `projects/vault-v2/src/components/DeviceUpdateManager.tsx`
- Commented out the `device:pin-request-triggered` event listener
- This prevents DeviceUpdateManager from showing its own PIN dialog
- WalletContext is now the single source of truth for PIN dialog management
- Eliminates race conditions and duplicate PIN dialogs

### 4. Remove Unnecessary PIN Triggers
**File**: `projects/vault-v2/src/components/DeviceUpdateManager.tsx`
- Removed `invoke('trigger_pin_request')` call from the `device:passphrase-unlock-needed` handler
- The backend automatically triggers PIN when needed; manual triggers were causing duplicates
- This eliminated the 20+ duplicate PIN request calls

### 5. Prevent Auto-triggering in PIN Dialog
**File**: `projects/vault-v2/src/components/PinUnlockDialog.tsx`
- Added check for existing PIN flow before triggering a new request
- Only triggers PIN request if device is not already in PIN flow
- Prevents duplicate requests when dialog is shown after backend has already triggered PIN

### 6. Centralize PIN Dialog Handling in WalletContext
**File**: `projects/vault-v2/src/contexts/WalletContext.tsx`
- Re-enabled the `device:pin-request-triggered` event listener with duplicate checking
- Added check to prevent showing dialog if already showing for the device
- This is now the SINGLE source of truth for PIN dialog management

### 7. Disable PIN Handling in useDeviceInteraction Hook
**File**: `projects/vault-v2/src/hooks/useDeviceInteraction.ts`
- Disabled the `device:awaiting_pin` event handler that was showing DevicePinDialog
- This was creating a second, different PIN dialog component
- Prevents duplicate PIN dialogs from appearing

## Technical Details

### Before (PinUnlockDialog.tsx)
```typescript
setStep('success')
// Auto-close after brief success display
setTimeout(() => {
  console.log('🔒 PIN dialog auto-closing after success')
  onUnlocked()
}, 1000)
```

### After (PinUnlockDialog.tsx)
```typescript
setStep('success')
// Close immediately to allow passphrase dialog to show
// The backend will send a passphrase_request event if needed
onUnlocked()
```

### Dialog Priority System Update (DialogContext.tsx)
```typescript
// Check for critical security dialogs that must be shown immediately
// PIN dialog has highest priority when present
const pinDialog = queue.find(d => d.id.includes('pin-unlock'));
if (pinDialog) {
  return pinDialog;
}

// Passphrase dialog should be shown immediately after PIN
const passphraseDialog = queue.find(d => d.id.includes('passphrase'));
if (passphraseDialog) {
  return passphraseDialog;
}
```

### DeviceUpdateManager Event Listener Removal
```typescript
// BEFORE: Both WalletContext and DeviceUpdateManager were listening
// DeviceUpdateManager.tsx
listen('device:pin-request-triggered', async (event) => {
  // This was causing duplicate PIN dialogs
  setShowPinUnlock(true)
})

// AFTER: Only WalletContext handles PIN requests
// Event listener commented out in DeviceUpdateManager
```

## Testing
To verify the fix:
1. Start the application with a device that has both PIN and passphrase protection
2. Navigate to Settings and trigger a PIN-required operation (e.g., toggle passphrase setting)
3. Enter PIN when prompted
4. Verify that the passphrase dialog appears immediately after PIN is accepted (no duplicate PIN dialog)
5. The device screen and frontend should both show passphrase entry
6. No extra PIN dialog should appear that needs to be manually closed

## Summary of Final Solution

The duplicate PIN dialog issue was caused by multiple event systems trying to show PIN dialogs simultaneously:
1. `WalletContext` listening to `device:pin-request-triggered` 
2. `useDeviceInteraction` listening to `device:awaiting_pin`
3. `DeviceUpdateManager` triggering PIN requests multiple times

The solution was to:
1. **Centralize PIN dialog management** in `WalletContext` as the single source of truth
2. **Disable duplicate handlers** in `useDeviceInteraction` and `DeviceUpdateManager`
3. **Add duplicate checking** to prevent showing the same dialog twice
4. **Remove delays** in PIN dialog closure to allow immediate passphrase dialog transition
5. **Fix dialog priority** to ensure passphrase dialogs can show after PIN dialogs

Now there is only ONE place that shows PIN dialogs (`WalletContext`), preventing duplicates while ensuring proper PIN → Passphrase dialog flow.

## Related Files
- `projects/vault-v2/src/components/PinUnlockDialog.tsx` - PIN dialog component
- `projects/vault-v2/src/contexts/DialogContext.tsx` - Dialog queue management
- `projects/vault-v2/src/contexts/WalletContext.tsx` - Centralized PIN dialog handling
- `projects/vault-v2/src/hooks/useDeviceInteraction.ts` - Device interaction hooks (PIN handling disabled)
- `projects/vault-v2/src/components/DeviceUpdateManager.tsx` - Device update management
- `projects/vault-v2/src/App.tsx` - Event listener for passphrase_request events
- `projects/vault-v2/src/components/SimplePassphraseModal.tsx` - Passphrase dialog component