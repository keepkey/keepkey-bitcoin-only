# PIN Flow Protection System

## Overview
This document describes the PIN flow protection system implemented in KeepKey Vault v2 to prevent device operations from interrupting PIN entry on the hardware device.

## Problem Statement
When a KeepKey device displays the PIN entry screen, any USB communication can dismiss the screen, causing a poor user experience. Common culprits include:
- Background xpub fetching for portfolio display
- Status polling for device updates
- GetFeatures calls for device state checking

## Solution Architecture

### 1. PIN Flow State Tracking
The system maintains a global registry of devices currently in PIN entry mode:

```rust
// In commands.rs
lazy_static! {
    static ref DEVICES_IN_PIN_FLOW: Arc<Mutex<HashSet<String>>> = 
        Arc::new(Mutex::new(HashSet::new()));
}
```

### 2. Two-Layer Protection

#### Layer 1: Request Blocking at Queue Entry
**Location**: `src/device/queue.rs` lines 82-95

Before ANY request enters the device queue, the system checks if the device is in PIN flow:

```rust
if crate::commands::is_device_in_pin_flow(&request.device_id) {
    match &request.request {
        DeviceRequest::GetXpub { .. } | 
        DeviceRequest::GetAddress { .. } | 
        DeviceRequest::SignTransaction { .. } => {
            return Err("Device is currently in PIN entry mode. Please complete PIN entry first.".to_string());
        },
        _ => {
            // Allow GetFeatures and SendRaw during PIN flow as they might be needed
        }
    }
}
```

**Blocked Operations During PIN Flow:**
- ❌ GetXpub - Would interrupt PIN screen
- ❌ GetAddress - Would interrupt PIN screen  
- ❌ SignTransaction - Would interrupt PIN screen
- ✅ SendRaw - Needed for PIN-related messages
- ✅ GetFeatures - Handled by Layer 2

#### Layer 2: GetFeatures Caching
**Location**: `src/device/queue.rs` lines 101-138

Even allowed operations avoid unnecessary device communication:

```rust
let raw_features_opt = if crate::commands::is_device_in_pin_flow(&request.device_id) {
    // Use cached features instead of querying device
    let cache = DEVICE_STATE_CACHE.read().await;
    cache.get(&request.device_id).and_then(|state| state.last_features.clone())
} else {
    // Normal flow - fetch fresh features from device
    match keepkey_rust::device_queue::DeviceQueueHandle::get_features(&queue_handle).await {
        // ... normal GetFeatures handling
    }
};
```

### 3. PIN Flow Lifecycle

#### Starting PIN Flow
```rust
// In trigger_pin_request() or start_pin_creation()
mark_device_in_pin_flow(&device_id)?;
```

#### During PIN Flow
- All non-essential requests are blocked
- Status checks use cached data
- PIN-related messages (SendRaw) pass through

#### Ending PIN Flow
```rust
// After PIN entry complete or cancelled
unmark_device_in_pin_flow(&device_id)?;
```

## Implementation Details

### Helper Functions

**Check PIN Flow Status**
```rust
pub fn is_device_in_pin_flow(device_id: &str) -> bool {
    if let Ok(devices) = DEVICES_IN_PIN_FLOW.lock() {
        devices.contains(device_id)
    } else {
        false
    }
}
```

**Mark Device in PIN Flow**
```rust
pub fn mark_device_in_pin_flow(device_id: &str) -> Result<(), String> {
    let mut devices = DEVICES_IN_PIN_FLOW.lock()
        .map_err(|_| "Failed to lock PIN flow registry")?;
    devices.insert(device_id.to_string());
    Ok(())
}
```

**Remove from PIN Flow**
```rust
pub fn unmark_device_in_pin_flow(device_id: &str) -> Result<(), String> {
    let mut devices = DEVICES_IN_PIN_FLOW.lock()
        .map_err(|_| "Failed to lock PIN flow registry")?;
    devices.remove(device_id);
    Ok(())
}
```

### Error Handling

When requests are blocked during PIN flow:
1. User-friendly error message returned
2. No device communication attempted
3. Frontend can display appropriate UI feedback

Example error response:
```json
{
  "error": "Device is currently in PIN entry mode. Please complete PIN entry first."
}
```

## Benefits

1. **Uninterrupted PIN Entry**: Users can complete PIN entry without the screen disappearing
2. **Reduced USB Traffic**: Fewer unnecessary device queries during sensitive operations
3. **Better UX**: Clear error messages explain why operations are temporarily blocked
4. **Cache Efficiency**: Features cached and reused during PIN flow

## Testing

### Test Scenarios

1. **Basic PIN Flow Protection**
   - Trigger PIN request
   - Attempt to fetch xpub while PIN screen visible
   - Verify request is blocked with appropriate error

2. **Cache Usage During PIN**
   - Get device features before PIN flow
   - Enter PIN flow
   - Request device status
   - Verify cached features are used (no USB traffic)

3. **Flow Cleanup**
   - Enter PIN flow
   - Complete or cancel PIN entry
   - Verify normal operations resume
   - Verify xpub requests succeed

### Debug Logging

Key log messages to monitor:
```
🚫 Blocking request during PIN flow - device is entering PIN
⚠️ Skipping GetFeatures check - device is in PIN flow
📋 Using cached features for device during PIN flow
```

## Related Files

- `src/commands.rs` - PIN flow state management functions
- `src/device/queue.rs` - Request blocking and caching logic
- `src/event_controller.rs` - Device event handling

## Future Improvements

1. **Timeout Handling**: Auto-clear PIN flow state after timeout (e.g., 5 minutes)
2. **Multi-Device Support**: Better handling when multiple devices are in PIN flow
3. **State Persistence**: Restore PIN flow state after app restart
4. **WebSocket Notifications**: Notify frontend when device enters/exits PIN flow

## Troubleshooting

### Issue: PIN Screen Still Disappearing
- Check logs for any SendRaw messages during PIN flow
- Verify all background polling is using the device queue
- Ensure frontend isn't bypassing the queue system

### Issue: Operations Blocked After PIN Entry
- Check if `unmark_device_in_pin_flow()` is called after PIN completion
- Verify PIN flow cleanup on error conditions
- Check DEVICES_IN_PIN_FLOW state in debugger

### Issue: Features Not Cached
- Ensure device has been queried at least once before PIN flow
- Check DEVICE_STATE_CACHE has entry for device
- Verify cache insertion in GetFeatures success path