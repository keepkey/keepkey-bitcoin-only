# Passphrase Dialog Not Showing After PIN Entry - Resolution

## Issue Description
When the device requested a passphrase after PIN entry (not at startup), the PIN dialog remained visible instead of transitioning to the passphrase dialog. The device screen showed "Enter Passphrase" but the frontend continued showing the PIN entry screen.

## Root Cause
1. **Delayed PIN Dialog Closure**: The PIN dialog had a 1-second delay before closing after successful PIN submission (for visual feedback)
2. **Dialog Priority System**: The PIN dialog had absolute priority in the dialog queue, preventing the passphrase dialog from being shown even when added to the queue

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

## Testing
To verify the fix:
1. Start the application with a device that has both PIN and passphrase protection
2. Navigate to Settings and trigger a PIN-required operation (e.g., toggle passphrase setting)
3. Enter PIN when prompted
4. Verify that the passphrase dialog appears immediately after PIN is accepted
5. The device screen and frontend should both show passphrase entry

## Related Files
- `projects/vault-v2/src/components/PinUnlockDialog.tsx` - PIN dialog component
- `projects/vault-v2/src/contexts/DialogContext.tsx` - Dialog queue management
- `projects/vault-v2/src/App.tsx` - Event listener for passphrase_request events
- `projects/vault-v2/src/components/SimplePassphraseModal.tsx` - Passphrase dialog component