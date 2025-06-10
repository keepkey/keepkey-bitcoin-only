# v1.0.3 Bootloader ButtonRequest Fix

## Problem
The v1.0.3 bootloader update was failing with "No data received from device" after a 10-second timeout when using keepkey-desktop-v5, but worked fine with kkcli.

## Root Cause
The v1.0.3 bootloader sends a `ButtonRequest` message before accepting the firmware upload, expecting a `ButtonAck` response. Without this acknowledgment, the device waits indefinitely (or times out).

## The Fix
The issue was that keepkey-desktop-v5 was calling `adapter.handle()` directly instead of using `adapter.with_standard_handler().handle()`.

### Before (broken):
```rust
adapter.handle(
    messages::FirmwareUpload { ... }.into()
)
```

### After (fixed):
```rust
let mut handler = adapter.with_standard_handler();
handler.handle(
    messages::FirmwareUpload { ... }.into()
)
```

## What `with_standard_handler()` Does
The standard message handler automatically responds to common protocol messages:
- `ButtonRequest` → responds with `ButtonAck`
- `PinMatrixRequest` → prompts for PIN and responds with `PinMatrixAck`
- `PassphraseRequest` → prompts for passphrase and responds with `PassphraseAck`

## Why This Matters for v1.0.3
The v1.0.3 bootloader has a different protocol flow than newer versions:
1. Device receives `FirmwareUpload` message
2. Device sends `ButtonRequest` (shows "Upload" on screen)
3. Device waits for `ButtonAck` 
4. User physically presses button to confirm
5. Upload proceeds

Without the `ButtonAck` response in step 3, the device never gets to step 4 where the user can confirm.

## Lessons Learned
1. Always use `with_standard_handler()` for firmware operations
2. The v1.0.3 bootloader requires both software acknowledgment (ButtonAck) AND physical button press
3. Protocol differences between bootloader versions need careful handling
4. kkcli's implementation was correct and served as a good reference 