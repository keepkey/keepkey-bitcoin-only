# Passphrase Protection PIN Dialog Bug Resolution

## Bug Summary
**Critical Issue**: Users were unable to disable passphrase protection on PIN-protected devices due to multiple interconnected issues in the PIN flow.

## Root Causes Identified

### 1. Ghost PIN Submissions
The device queue was automatically sending empty PIN responses when it received a `PinMatrixRequest` from the device. This happened because:
- The CLI-oriented `standard_message_handler` was being used in desktop builds
- This handler auto-responds to `PinMatrixRequest` by reading from stdin (resulting in empty PIN)
- No state tracking existed to block other commands during PIN entry

### 2. GetFeatures Interruption
While the PIN dialog was open on the device:
- Background status polling would send `GetFeatures` commands
- These commands would interrupt the PIN entry flow
- The PIN dialog would disappear from the device screen

### 3. Incorrect PIN Position Mapping
The PIN positions sent from the frontend didn't match what the KeepKey expected:
- KeepKey uses a standard scrambled PIN layout (7-8-9/4-5-6/1-2-3)
- Frontend was sending button index + 1 instead of the actual position value
- Example: Clicking top-right button sent "3" when it should send "9"

## Comprehensive Fix Set

### 1. Backend Protection (Prompt State Tracking)
- Added a `PromptState` enum (`None`, `AwaitingPin`, `AwaitingPassphrase`)
- Added tracking in `DeviceWorker` to set state on receiving `PinMatrixRequest`/`PassphraseRequest`
- Added blocking logic to prevent any messages except `PinMatrixAck`/`PassphraseAck`/`Cancel` during prompt states
- Added a desktop mode check to always force the `pin_flow_handler` in desktop builds
- Added empty PIN guard to prevent sending empty `PinMatrixAck`

### 2. GetFeatures Protection
- Modified `get_connected_devices_with_features` to check device interaction state
- If device is in `AwaitingPIN` or `AwaitingButton` state, it skips `GetFeatures` call
- Uses cached features instead to avoid interrupting PIN flow
- Modified `get_device_status` to use cached state for devices in interactive states

### 3. Frontend Protection
- Added check in `PassphraseSettings` component to prevent calling parent refresh callback during PIN flow
- Deferred refresh until after device reconnects (when passphrase change completes)
- Fixed PIN position mapping to send correct values based on KeepKey's standard layout

### 4. PIN Mapping Fix
- Changed PIN button click handler from `handlePinButtonClick(index + 1)` to `handlePinButtonClick(position)`
- Position values now correctly match KeepKey's expected PIN layout:
  ```
  7 8 9
  4 5 6
  1 2 3
  ```

## Technical Implementation Details

### Prompt State Tracking
```rust
// New enum for tracking prompt states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PromptState {
    None,
    AwaitingPin,
    AwaitingPassphrase,
}

// Added to DeviceWorker struct
pub struct DeviceWorker {
    // ... existing fields ...
    prompt_state: PromptState,
}

// Update prompt state on message receipt
match &response {
    Message::PinMatrixRequest(_) => {
        info!("🔐 Device requesting PIN, setting prompt state to AwaitingPin");
        self.prompt_state = PromptState::AwaitingPin;
    }
    Message::PassphraseRequest(_) => {
        info!("🔐 Device requesting passphrase, setting prompt state to AwaitingPassphrase");
        self.prompt_state = PromptState::AwaitingPassphrase;
    }
    Message::Success(_) | Message::Failure(_) => {
        if self.prompt_state != PromptState::None {
            info!("✅ Clearing prompt state after {:?}", response.message_type());
            self.prompt_state = PromptState::None;
        }
    }
    _ => {}
}
```

### Message Blocking During PIN Entry
```rust
// Block non-PIN messages during PIN entry
match self.prompt_state {
    PromptState::AwaitingPin => {
        if !matches!(&message, Message::PinMatrixAck(_) | Message::Cancel(_)) {
            warn!("🚫 Blocking message {:?} while awaiting PIN", message.message_type());
            return Err(anyhow!("Device is awaiting PIN input"));
        }
        // Clear prompt state when sending PIN ack
        if matches!(&message, Message::PinMatrixAck(_)) {
            info!("📤 Sending PinMatrixAck, clearing AwaitingPin state");
            self.prompt_state = PromptState::None;
        }
    }
    // Similar for AwaitingPassphrase...
}
```

### GetFeatures Protection
```rust
// Check if device is in an interaction state that we shouldn't interrupt
let device_interaction_state = {
    let sessions = crate::device::interaction_state::DEVICE_SESSIONS.read().await;
    sessions.get(&device_id).map(|s| s.state.clone())
};

// If device is awaiting PIN/Button, skip feature fetching
if matches!(device_interaction_state, 
    Some(crate::device::interaction_state::DeviceInteractionState::AwaitingPIN { .. }) |
    Some(crate::device::interaction_state::DeviceInteractionState::AwaitingButton { .. })
) {
    println!("⏭️ Skipping features for device {} - awaiting user interaction", device_id);
    
    // Try to use cached features
    let cache = crate::device::queue::DEVICE_STATE_CACHE.read().await;
    if let Some(cached_state) = cache.get(&device_id) {
        if let Some(cached_features) = &cached_state.last_features {
            let device_features = convert_features_to_device_features(cached_features.clone());
            return Some(serde_json::json!({
                "device": device,
                "features": device_features,
                "interaction_state": format!("{:?}", device_interaction_state)
            }));
        }
    }
    
    // Return device without features if no cache
    return Some(serde_json::json!({
        "device": device,
        "features": null,
        "interaction_state": format!("{:?}", device_interaction_state)
    }));
}
```

### Frontend Refresh Deferral
```typescript
// Check if device is in PIN/interaction state before notifying parent
const interactionState = await invoke<string>('get_device_interaction_state', { deviceId });
console.log(`[PassphraseSettings] Device interaction state after operation: ${interactionState}`);

// Only notify parent if device is not waiting for user interaction
if (onPassphraseToggle && !interactionState.includes('AwaitingPIN') && !interactionState.includes('AwaitingButton')) {
  console.log('[PassphraseSettings] Device is idle, notifying parent to refresh');
  onPassphraseToggle(newState);
} else if (interactionState.includes('AwaitingPIN') || interactionState.includes('AwaitingButton')) {
  console.log('[PassphraseSettings] Device is awaiting interaction, skipping parent notification to avoid interruption');
}
```

### PIN Position Mapping Fix
```tsx
<Grid templateColumns="repeat(3, 1fr)" gap={2}>
  {/* KeepKey standard PIN layout:
      7 8 9
      4 5 6
      1 2 3
     Grid positions 1-9 map to these values */}
  {[7, 8, 9, 4, 5, 6, 1, 2, 3].map((position, index) => (
    <Button
      key={index}
      onClick={() => handlePinButtonClick(position)}
      isDisabled={isSubmitting || pinPositions.length >= 9}
      size="lg"
      h="60px"
      w="60px"
      fontSize="xl"
      colorScheme="gray"
      variant="outline"
      _hover={{ bg: 'gray.100' }}
    >
      •
    </Button>
  ))}
</Grid>
```

## Conclusion
This was a complex, multi-layered bug that required fixes at several levels of the application stack. The combination of prompt state tracking, message blocking, cached features, and correct PIN mapping has resolved the issue, allowing users to successfully disable passphrase protection on PIN-protected devices.

The fix approach demonstrates the importance of:
1. Understanding the complete flow across frontend and backend
2. Proper state management for device interactions
3. Blocking potentially disruptive operations during critical flows
4. Correct mapping of UI elements to device expectations

These changes significantly improve the reliability of device security settings management in the application.
