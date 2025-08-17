# PERSISTENT BUG 02: Multiple Passphrase Request Attempts Before Dialog Opens

## Issue Description
The device is repeatedly requesting passphrase input multiple times (3+ attempts) before finally allowing the passphrase dialog to open. This causes the device screen to blink/flash repeatedly, creating a poor user experience.

## Current Behavior (Problematic)
1. Device requests passphrase for xpub derivation
2. System attempts GetFeatures → fails with "Unknown message"
3. Falls back to Initialize → succeeds
4. Sends GetPublicKey → receives PassphraseRequest
5. **BUG**: System repeats steps 2-4 multiple times (3+ attempts observed)
6. Device screen blinks/flashes with each attempt
7. Finally opens passphrase dialog after multiple failed attempts

## Expected Behavior
1. Device requests passphrase
2. Single attempt to communicate with device
3. Immediately block and open passphrase dialog
4. No screen blinking or repeated attempts
5. Clean, single interaction flow

## Log Evidence
```
🎯 Frontend ready signal received - enabling event emission
✅ No queued events to flush
Adding to device queue: DeviceRequestWrapper { device_id: "343737340F4736331F003B00", request_id: "xpub_1755456366173_xksop5x3e", request: GetXpub { path: "m/49'/0'/0'" } }
-> GetFeatures
<- Failure
🔧 GetFeatures returned Failure: Unknown message, attempting OOB bootloader fallback with Initialize message
-> Initialize
<- Features: Unlabeled v7.10.0 ✅
✅ OOB bootloader Initialize fallback successful for device 343737340F4736331F003B00
🔧 Bootloader check: 2.1.4 -> needs update: false (bootloader_mode: false)
🔧 Firmware check: 7.10.0 vs 7.10.0 -> needs update: false (bootloader_mode: false)
🔧 Initialization check: initialized=true, needs_setup=false, has_pin_protection=false, pin_cached=true
-> GetPublicKey
<- PassphraseRequest
🔐 Device requested passphrase for xpub
⚠️ Passphrase request already active for device 343737340F4736331F003B00, skipping duplicate

[PATTERN REPEATS FOR m/84'/0'/0']

Adding to device queue: DeviceRequestWrapper { device_id: "343737340F4736331F003B00", request_id: "xpub_1755456367920_hz4hh5u8d", request: GetXpub { path: "m/84'/0'/0'" } }
-> GetFeatures
<- Failure
🔧 GetFeatures returned Failure: Unknown message, attempting OOB bootloader fallback with Initialize message
-> Initialize
<- Features: Unlabeled v7.10.0 ✅
✅ OOB bootloader Initialize fallback successful for device 343737340F4736331F003B00
🔧 Bootloader check: 2.1.4 -> needs update: false (bootloader_mode: false)
🔧 Firmware check: 7.10.0 vs 7.10.0 -> needs update: false (bootloader_mode: false)
🔧 Initialization check: initialized=true, needs_setup=false, has_pin_protection=false, pin_cached=true
-> GetPublicKey
<- PassphraseRequest
🔐 Device requested passphrase for xpub
⚠️ Passphrase request already active for device 343737340F4736331F003B00, skipping duplicate
```

## Key Problems
1. **Multiple duplicate requests**: System is queuing multiple xpub requests while passphrase dialog is pending
2. **Screen flashing**: Each attempt causes device screen to update/flash
3. **Poor UX**: User sees device blinking repeatedly before dialog appears
4. **Inefficient flow**: Multiple unnecessary GetFeatures/Initialize cycles

## Potential Root Causes
1. Frontend is not properly blocking/waiting when passphrase is requested
2. Queue processing continues even when passphrase request is active
3. Missing synchronization between device queue and passphrase dialog state
4. Duplicate detection (`⚠️ Passphrase request already active`) is not preventing queue processing

## Suggested Fixes
1. **Implement proper blocking**: When PassphraseRequest is received, immediately block all queue processing
2. **Single attempt pattern**: Only attempt device communication once before showing dialog
3. **Queue suspension**: Suspend device queue processing while passphrase dialog is active
4. **Better state management**: Track passphrase request state globally and prevent duplicate attempts
5. **Frontend coordination**: Ensure frontend properly waits for passphrase before sending additional requests

## Files to Investigate
- `projects/vault-v2/src-tauri/src/commands/pin_setup.rs` - PIN/passphrase handling
- `projects/vault-v2/src/components/PinSetupDialog.tsx` - Frontend dialog management
- Device queue management logic
- Passphrase request state tracking

## Severity
High - Significantly impacts user experience with confusing visual feedback

## Status
**❌ BROKEN - NOT FIXED**

## Attempt Tally
- Attempt #1: ❌ Failed - Device still spamming requests
- Attempt #2: ❌ Failed - Multiple blinks continue
- Attempt #3: ❌ Failed - Queue not blocking properly
- Attempt #4: ❌ Failed - Duplicate detection not preventing issue
- Attempt #5: ❌ Failed - Frontend not coordinating with backend
- Attempt #6: ❌ Failed - Still experiencing 3+ requests before dialog opens
- **Total Attempts: 6**
- **Current State: STILL BROKEN**