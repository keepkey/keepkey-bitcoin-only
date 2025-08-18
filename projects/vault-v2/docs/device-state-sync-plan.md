# Device State Synchronization Plan

## Problem Statement
Backend and frontend are getting out of sync, particularly around:
- PIN entry states (backend thinks PIN is needed, frontend doesn't know)
- Passphrase states (similar desync issues)
- Device connection states (backend loses connection, frontend doesn't know)
- Queue processing states (backend has pending operations, frontend unaware)

## Goals
- **Lightweight**: Small nudge, not a rewrite
- **Non-invasive**: Minimal changes to existing code
- **Reliable**: Keep states in sync without constant polling
- **Recoverable**: Gracefully handle and recover from desync states

## Proposed Solution

### 1. Global Device State Object
Create a unified state structure that both backend and frontend understand:

```typescript
interface DeviceState {
  deviceId: string;
  connected: boolean;
  initialized: boolean;
  
  // Authentication states
  pinState: 'none' | 'locked' | 'unlocked' | 'entering';
  passphraseState: 'none' | 'required' | 'entered' | 'entering';
  
  // Operation states
  busy: boolean;
  currentOperation?: string;
  queueLength: number;
  
  // Health states
  lastHeartbeat: number;
  needsReconnect: boolean;
  errorState?: string;
  
  // Metadata
  firmware: string;
  label: string;
  model: string;
}
```

### 2. Sync Mechanism

#### A. Periodic Sync (every 5 seconds)
```typescript
// In WalletContext or new DeviceStateContext
useEffect(() => {
  const syncInterval = setInterval(async () => {
    if (deviceId) {
      const state = await invoke('get_device_state', { deviceId });
      updateLocalState(state);
      handleStateChanges(state);
    }
  }, 5000);
  
  return () => clearInterval(syncInterval);
}, [deviceId]);
```

#### B. Event-Driven Updates
Keep existing events but add state sync:
- `device:pin-request-triggered` → sync state
- `device:passphrase-request` → sync state
- `device:disconnected` → sync state
- `device:operation-complete` → sync state

#### C. On-Demand Sync
Add a "sync now" capability for critical operations:
```typescript
const syncDeviceState = async () => {
  const state = await invoke('get_device_state', { deviceId });
  return state;
};

// Use before critical operations
const handleSendTransaction = async () => {
  await syncDeviceState(); // Ensure we have latest state
  // ... proceed with operation
};
```

### 3. Backend Changes (Minimal)

#### A. Add State Tracking
```rust
// In device_manager.rs or similar
pub struct DeviceStateTracker {
    states: Arc<Mutex<HashMap<String, DeviceState>>>,
}

impl DeviceStateTracker {
    pub fn update_pin_state(&self, device_id: &str, state: PinState) {
        // Update the state
        self.emit_state_change(device_id);
    }
    
    pub fn get_state(&self, device_id: &str) -> DeviceState {
        // Return current state
    }
}
```

#### B. Add Tauri Command
```rust
#[tauri::command]
pub async fn get_device_state(device_id: String) -> Result<DeviceState, String> {
    let tracker = state.device_tracker.lock().await;
    Ok(tracker.get_state(&device_id))
}
```

### 4. Frontend Integration Points

#### A. WalletContext Enhancement
```typescript
// Add to WalletContext
const [deviceState, setDeviceState] = useState<DeviceState | null>(null);

// Sync function
const syncDeviceState = useCallback(async () => {
  if (!deviceId) return;
  
  try {
    const state = await invoke('get_device_state', { deviceId });
    setDeviceState(state);
    
    // Handle specific state changes
    if (state.pinState === 'locked' && !isPinDialogOpen) {
      showPinDialog();
    }
    
    if (state.needsReconnect) {
      showReconnectPrompt();
    }
  } catch (error) {
    console.error('Failed to sync device state:', error);
  }
}, [deviceId]);
```

#### B. Auto-Recovery Flows
```typescript
// Auto-handle common desync scenarios
useEffect(() => {
  if (deviceState?.needsReconnect) {
    // Show reconnect UI
    showReconnectDialog();
  }
  
  if (deviceState?.pinState === 'locked' && !dialogQueue.includes('pin')) {
    // Backend needs PIN but frontend doesn't know
    pinDialog.show({ deviceId });
  }
  
  if (deviceState?.passphraseState === 'required' && !dialogQueue.includes('passphrase')) {
    // Backend needs passphrase but frontend doesn't know
    passphraseDialog.show({ deviceId });
  }
}, [deviceState]);
```

### 5. Implementation Steps

1. **Phase 1: Backend State Tracking** (1 day)
   - Add DeviceState struct
   - Add state tracking to existing operations
   - Add get_device_state command

2. **Phase 2: Frontend Sync** (1 day)
   - Add periodic sync to WalletContext
   - Add device state to context
   - Wire up state change handlers

3. **Phase 3: Auto-Recovery** (1 day)
   - Add reconnect dialog
   - Add auto-PIN dialog trigger
   - Add auto-passphrase dialog trigger

4. **Phase 4: Testing & Refinement** (1 day)
   - Test various desync scenarios
   - Tune sync intervals
   - Add error recovery

## Benefits
- **Predictable**: Frontend always knows device state
- **Resilient**: Auto-recovers from common issues
- **Lightweight**: Just adds state tracking, no major refactor
- **Debuggable**: Single source of truth for device state

## Risks & Mitigations
- **Risk**: Too much polling
  - **Mitigation**: 5-second interval is reasonable, can be tuned
  
- **Risk**: State gets out of sync
  - **Mitigation**: Event-driven updates + periodic sync as backup
  
- **Risk**: Complex state management
  - **Mitigation**: Keep state flat and simple

## Alternative Approaches Considered
1. **WebSocket for real-time sync**: Too complex for this use case
2. **Redux/Zustand state management**: Would require bigger refactor
3. **Backend-only recovery**: Wouldn't solve UI desync issues

## Success Metrics
- No more stuck PIN dialogs
- Auto-recovery from disconnects
- Reduced "device not ready" errors
- Better user experience during auth flows