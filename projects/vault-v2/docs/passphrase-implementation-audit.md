# Passphrase Implementation Audit & Improvement Plan

## Current State Analysis

### 1. Passphrase Enable/Disable Flow

#### Current Implementation (PROBLEMATIC)
- **Location**: `src/components/PassphraseSettings.tsx`
- **Backend**: `src-tauri/src/commands.rs::enable_passphrase_protection()`

**Current Flow:**
1. User toggles switch in Settings
2. Backend sends `ApplySettings` message to device
3. Device confirms with button press
4. **App automatically restarts** (using `relaunch()` from Tauri)
5. On disable: Shows message "Please unplug your KeepKey and reconnect it"
6. **App restarts again after 3 seconds**

#### Problems with Current Implementation

1. **App Restart Doesn't Work Properly**
   - If device isn't reconnected, app restart is pointless
   - Creates poor UX with unnecessary app closure
   - User loses context and any unsaved state
   - Device state becomes out of sync

2. **PIN Error on Enable**
   ```
   -> ApplySettings
   <- ButtonRequest
   -> ButtonAck
   <- PinMatrixRequest
   -> PinMatrixAck
   <- Failure: PIN must be at least 1 digit
   ```
   - Device is requesting PIN but getting empty PIN response
   - This suggests the PIN flow is not being handled correctly

3. **No State Synchronization**
   - After passphrase enable/disable, device state isn't updated
   - Frontend doesn't know if passphrase is actually needed
   - Can lead to stuck states where passphrase is enabled but app doesn't know

### 2. Passphrase Request Handling

#### Current Implementation
- **Backend**: `src-tauri/src/device/queue.rs`
- **Frontend**: `src/components/SimplePassphraseModal.tsx`

**Flow:**
1. Device returns `PassphraseRequest` during operations
2. Backend emits `passphrase_request` event
3. Frontend shows passphrase modal
4. User enters passphrase
5. Backend sends `PassphraseAck` with passphrase
6. Operation continues

#### Issues
- Passphrase state tracking exists but isn't integrated with device state
- No way to know if device needs passphrase without trying an operation
- Duplicate passphrase requests can occur

### 3. Device State Integration

**Current State Object** (newly added):
```typescript
{
  isUnlocked: boolean,       // PIN state
  needsPassphrase: boolean,   // Should show passphrase dialog
  needsReset: boolean,        // Device needs reconnect
  isBusy: boolean,           // Currently processing
  currentOperation: string,   // What's running
  pinCached: boolean,        // PIN cached in device
  passphraseCached: boolean  // Passphrase cached
}
```

**Problem**: Passphrase state updates aren't happening when:
- Passphrase protection is enabled/disabled
- Passphrase is entered
- Device is reconnected

## Root Cause Analysis

### Why App Restart Was Used
1. **Device Cache Reset**: Device caches passphrase state internally
2. **Queue State Reset**: Backend queue might hold stale state
3. **Frontend State Reset**: React state might be out of sync
4. **Lazy Solution**: Easier than proper state management

### Why It Doesn't Work
1. **Device Must Be Reconnected**: Passphrase changes require device power cycle
2. **App Restart Doesn't Reset Device**: Device keeps its state
3. **No Sync Mechanism**: No way to detect device was reconnected
4. **Poor UX**: User confusion when app closes unexpectedly

## Proposed Solution

### 1. Replace App Restart with Reconnect Flow

#### New Enable Flow
```
1. User toggles passphrase ON
2. Send ApplySettings to device
3. Handle PIN request properly (device needs PIN to change settings)
4. On success:
   - Update device state: needsReset = true
   - Show reconnect dialog: "Please unplug and reconnect your KeepKey"
   - Monitor for device reconnection
   - Update state when device returns
```

#### New Disable Flow
```
1. User toggles passphrase OFF
2. Send ApplySettings to device
3. Handle PIN request if needed
4. On success:
   - Update device state: needsReset = true
   - Show reconnect dialog
   - Monitor for reconnection
   - Clear passphrase cache on reconnect
```

### 2. Implement Proper PIN Handling for Settings Changes

```rust
// In enable_passphrase_protection
match queue_handle.send_raw(message, true).await {
    Ok(Message::PinMatrixRequest(_)) => {
        // Emit PIN request event
        app.emit("pin-request", ...);
        // Wait for PIN entry
        // Send PinMatrixAck
        // Continue with ApplySettings
    }
    Ok(Message::ButtonRequest(_)) => {
        // Send ButtonAck
        // Wait for Success
    }
    Ok(Message::Success(_)) => {
        // Update device state
        set_device_needs_reset(&device_id, true).await;
        Ok(())
    }
    _ => Err("Unexpected response")
}
```

### 3. Add Device Reconnection Detection

```typescript
// In WalletContext
const monitorDeviceReconnection = async (deviceId: string) => {
  const checkInterval = setInterval(async () => {
    const devices = await invoke('get_connected_devices');
    const device = devices.find(d => d.device.unique_id === deviceId);
    
    if (device) {
      // Device reconnected
      clearInterval(checkInterval);
      
      // Get fresh features
      const state = await invoke('get_device_state', { deviceId });
      
      // Update local state
      setDeviceState(state);
      
      // Hide reconnect dialog
      hideReconnectDialog();
    }
  }, 1000);
};
```

### 4. Update Device State Properly

```rust
// After passphrase enable/disable success
async fn update_passphrase_state(device_id: &str, enabled: bool) {
    // Mark device needs reset
    set_device_needs_reset(device_id, true).await;
    
    // Update passphrase state
    set_device_passphrase_state(
        device_id, 
        enabled,  // needs_passphrase
        false     // passphrase_cached (will be false after reconnect)
    ).await;
}
```

### 5. Create Reconnect Dialog Component

```typescript
// New component: DeviceReconnectDialog.tsx
export const DeviceReconnectDialog = ({ 
  isOpen, 
  onReconnected,
  reason 
}: Props) => {
  return (
    <Modal isOpen={isOpen}>
      <ModalContent>
        <ModalHeader>Device Reconnection Required</ModalHeader>
        <ModalBody>
          <VStack spacing={4}>
            <Icon as={FaUsb} boxSize={12} color="blue.500" />
            <Text fontWeight="bold">
              {reason === 'passphrase_enabled' 
                ? 'Passphrase protection has been enabled'
                : 'Passphrase protection has been disabled'}
            </Text>
            <Text textAlign="center">
              Please unplug your KeepKey and reconnect it to apply the changes.
            </Text>
            <Text fontSize="sm" color="gray.600">
              Waiting for device reconnection...
            </Text>
            <Spinner size="lg" color="blue.500" />
          </VStack>
        </ModalBody>
      </ModalContent>
    </Modal>
  );
};
```

## Implementation Steps

### Phase 1: Fix PIN Handling (Immediate)
1. ✅ Update `enable_passphrase_protection` to handle PIN requests
2. ✅ Add PIN flow support for settings changes
3. ✅ Test with actual device

### Phase 2: Remove App Restart (Priority)
1. ✅ Remove `relaunch()` calls from PassphraseSettings.tsx
2. ✅ Replace with reconnect dialog trigger
3. ✅ Update success/error messaging

### Phase 3: Add Reconnect Detection (Core)
1. ✅ Create DeviceReconnectDialog component
2. ✅ Add device reconnection monitoring
3. ✅ Update device state on reconnection
4. ✅ Clear stale cache states

### Phase 4: State Synchronization (Polish)
1. ✅ Update passphrase state in device tracker
2. ✅ Emit proper events for state changes
3. ✅ Sync frontend with backend state
4. ✅ Test full flow end-to-end

## Success Criteria

1. **No App Restarts**: Passphrase toggle doesn't close the app
2. **Clear User Guidance**: User knows to unplug/replug device
3. **Automatic Detection**: App detects when device is reconnected
4. **State Consistency**: Frontend and backend stay in sync
5. **PIN Handling**: Settings changes work with PIN-protected devices
6. **Error Recovery**: Graceful handling of all error cases

## Testing Scenarios

1. **Enable Passphrase**
   - [ ] Without PIN protection
   - [ ] With PIN protection
   - [ ] Cancel during PIN entry
   - [ ] Cancel during button press
   - [ ] Device disconnected during process

2. **Disable Passphrase**
   - [ ] With passphrase cached
   - [ ] Without passphrase cached
   - [ ] With PIN protection
   - [ ] During active operations

3. **Reconnection**
   - [ ] Quick reconnect (<5 seconds)
   - [ ] Slow reconnect (>30 seconds)
   - [ ] Different device connected
   - [ ] Multiple devices connected

## Notes

- The current implementation's app restart is a workaround that doesn't actually solve the problem
- Proper state management and reconnection detection is the correct solution
- This aligns with the device state sync plan already implemented
- The PIN error suggests the device is in PIN flow but frontend doesn't know
- The restart delay timers (2-3 seconds) were attempts to give users time to see messages