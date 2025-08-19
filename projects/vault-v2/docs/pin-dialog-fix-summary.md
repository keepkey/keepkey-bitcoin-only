# PIN Dialog Fix for Passphrase Settings

## Issue
When trying to disable passphrase protection, the device was requesting PIN verification but the PIN dialog wasn't appearing. This resulted in an empty PIN being sent, causing the error "PIN must be at least 1 digit consisting of numbers from 1 to 9".

## Root Cause
The `useDeviceInteraction` hook that listens for `device:awaiting_pin` events and shows PIN dialogs was defined but never actually mounted in the application.

## Fix Applied

### 1. Enabled Global Device Interaction Handler
Added `useDeviceInteraction()` to the App component to enable global event listening for device interactions.

### 2. Created Generic PIN Dialog Component
Created `DevicePinDialog.tsx` that handles PIN requests during device operations with:
- Visual PIN matrix matching device screen layout
- Operation-specific descriptions (settings, transactions, exports)
- Request ID correlation for proper state management
- Error handling and loading states

### 3. Updated Dialog System Integration
Modified `useDeviceInteraction` hook to:
- Use the proper dialog system with unique dialog IDs
- Pass correct props including request ID for correlation
- Handle PIN submission via `pin_submit` command
- Properly clean up dialogs on completion or error

## How It Works Now

1. User toggles passphrase setting
2. Device responds with `ButtonRequest` → Backend sends `ButtonAck`
3. Device responds with `PinMatrixRequest`
4. Backend emits `device:awaiting_pin` event with request ID
5. `useDeviceInteraction` hook catches event and shows PIN dialog
6. User enters PIN → Dialog calls `pin_submit` with request ID
7. Backend validates PIN and continues operation
8. Device applies setting change and requests reconnection

## Note on PIN Requirement
The PIN verification for changing passphrase settings is a security feature of the KeepKey device, not a bug. This ensures that only authorized users can modify critical security settings.
