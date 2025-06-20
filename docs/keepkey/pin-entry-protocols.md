# KeepKey PIN Entry Protocols

## Overview

This document describes the PIN entry protocols for KeepKey devices based on analysis of the reference hdwallet implementation and our vault-v2 implementation.

## Key Findings

### 1. PIN Entry Location
- **Host Computer Entry**: KeepKey devices do NOT have on-device PIN entry
- `hasOnDevicePinEntry(): boolean { return false; }` in hdwallet reference
- PIN is entered on the host computer using a scrambled 3x3 matrix displayed on device

### 2. PIN Matrix Protocol

#### Device Display
- Device shows a 3x3 grid with scrambled numbers (e.g., 5-1-8 / 3-7-2 / 6-4-9)
- Each session uses a different random scrambling for security
- User sees real numbers on device screen, but host only sees positions 1-9

#### Host Interface
- Host computer shows generic 3x3 grid with dots or position indicators
- User clicks positions based on what they see on device screen
- Host sends position numbers (1-9) not actual PIN digits

### 3. Message Flow

#### PIN Request Flow
```
1. Host sends operation requiring PIN (e.g., GetPublicKey, SignTx)
2. Device responds with PinMatrixRequest
3. Host emits PIN_REQUEST event to frontend
4. User enters PIN positions on host interface
5. Host sends PinMatrixAck with position string
6. Device validates PIN and continues operation
```

#### Key Message Types
- `MESSAGETYPE_PINMATRIXREQUEST` - Device requests PIN entry
- `MESSAGETYPE_PINMATRIXACK` - Host responds with PIN positions
- `PIN_REQUEST` event - Frontend notification for PIN entry

### 4. Reference Implementation Analysis

#### Transport Layer (transport.ts)
```typescript
if (msgTypeEnum === Messages.MessageType.MESSAGETYPE_PINMATRIXREQUEST) {
  this.emit(
    core.Events.PIN_REQUEST,
    core.makeEvent({
      message_type: core.Events.PIN_REQUEST,
      from_wallet: true,
    })
  );
  this.userActionRequired = true;
  return this.handleCancellableResponse(); // Wait for user input
}
```

#### Wallet Layer (keepkey.ts)
```typescript
public async sendPin(pin: string): Promise<void> {
  const matrixAck = new Messages.PinMatrixAck();
  matrixAck.setPin(pin);
  await this.transport.call(Messages.MessageType.MESSAGETYPE_PINMATRIXACK, matrixAck, {
    msgTimeout: core.DEFAULT_TIMEOUT,
    omitLock: true,
    noWait: true,
  });
}
```

## Current Implementation Issues

### 1. Automatic PIN Response Problem
- Our `standard_message_handler` in transport/mod.rs automatically responds to PinMatrixRequest
- This causes immediate failures when device is PIN-locked
- Solution: Don't auto-respond, let frontend handle PIN requests

### 2. Device Status vs Operation Handling
- Device status correctly lsdetects PIN lock state
- But individual operations (GetXpub, GetAddress) trigger PIN requests during execution
- These should bubble up to frontend, not auto-respond with empty PINs

### 3. Frontend PIN Dialog Flow
- Backend detects PIN lock → emits `device:pin-unlock-needed`
- Frontend shows PIN unlock dialog
- User enters PIN → frontend calls PIN unlock API
- Device unlocks → operations can proceed

## Recommended Implementation

### 1. Transport Layer Changes
```rust
// Remove automatic PIN response from standard_message_handler
Message::PinMatrixRequest(_) => {
    // Don't handle automatically - let it bubble up as an error
    // Frontend will handle PIN requests via dedicated PIN unlock flow
    None
}
```

### 2. Frontend PIN Handling
```typescript
// On PIN request during operation:
1. Show PIN unlock dialog
2. Get user PIN input
3. Send PinMatrixAck via backend API
4. Resume operation or retry
```

### 3. Operation Flow
```
1. Check device status → needs_pin_unlock: true
2. Show PIN unlock dialog (don't attempt operations)
3. User enters PIN → device unlocks
4. Device status updates → needs_pin_unlock: false
5. Now safe to perform operations (GetXpub, etc.)
```

## Security Considerations

### 1. PIN Scrambling
- Device scrambles PIN matrix each session
- Host never knows actual PIN digits, only positions
- Protects against keyboard loggers and host compromises

### 2. PIN Caching
- Device may cache PIN for session (pin_cached flag)
- Reduces need for repeated PIN entry
- Cache timeout controlled by device settings

### 3. Session Management
- PIN sessions are tied to device connection
- Disconnect/reconnect typically clears PIN cache
- Failed PIN attempts may trigger device lockout

## Implementation Status

### ✅ Working
- Device PIN lock detection
- Backend PIN unlock API
- PIN dialog styling

### ❌ Issues Fixed
- Automatic PIN response loop (FIXED)
- Missing PIN dialog display (FIXED)
- Misleading "device ready" logs (FIXED)

### ✅ Fixed Issues (2024-01-20)

#### Issue 1: Device Not Showing PIN Matrix
- **Problem**: Device wasn't showing PIN matrix when unlock was requested
- **Cause**: Using `GetFeatures` to trigger PIN request doesn't work reliably on locked devices
- **Solution**: Changed to use `GetPublicKey` message which always requires authentication
- **Result**: Device now properly displays PIN matrix when unlock is requested

#### Issue 2: Stale PIN Sessions
- **Problem**: Backend would reject new PIN sessions with "Device is already in PIN flow"
- **Cause**: Previous PIN sessions weren't being cleaned up properly
- **Solution**: Added automatic cleanup of stale sessions in `start_pin_unlock`
- **Result**: PIN unlock works reliably without stale session conflicts

#### Issue 3: Missing Manual Unlock Trigger
- **Problem**: No way to manually trigger PIN unlock from UI
- **Cause**: Only automatic triggers existed (during portfolio load)
- **Solution**: Added lock/unlock button to device settings and resend button to PIN dialog
- **Result**: Users can now manually trigger PIN unlock and retry if needed

### 🔧 Current Implementation
- **Manual Lock Button**: Added to KeepKey device settings - shows "Lock Device" or "Unlock Device" based on PIN cache status
- **Resend PIN Button**: Added to PIN unlock dialog to retry if device doesn't show matrix
- **Stale Session Cleanup**: Automatic cleanup prevents "already in PIN flow" errors
- **Proper PIN Flow**: Using `GetPublicKey` → `PinMatrixRequest` → `PinMatrixAck` flow matching HDwallet reference

## Next Steps

1. Verify PIN unlock backend API works correctly
2. Test PIN unlock flow end-to-end
3. Ensure operations work after PIN unlock
4. Add proper error handling for PIN failures 