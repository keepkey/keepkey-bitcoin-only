# Device Communication Failure Analysis & Fix Plan

## Executive Summary
Critical device communication failures observed with KeepKey device (ID: 333303834174733303000200) showing transport contention, state management bugs, and polling race conditions.

## Observed Issues

### 1. Device State Inconsistencies
- **Current State**: FW 4.0.0, BL 1.0.3 (very outdated)
- **Polling Frequency**: ~2s constant polling causing transport contention
- **Critical Bug**: Service returns `success: true` with `features: null` after timeouts
- **State Mismatch**: `initialized: false` but `hasBackup: true` (impossible state)

### 2. Failure Timeline
```
21:34-21:35: Intermittent "Device not found" errors
21:47:36: Device in bootloader mode (bootloader_mode: true)
21:51+: Device back in app mode but still initialized: false
Continuous: Polling continues with intermittent failures
```

### 3. Root Causes (Ranked by Likelihood)

#### A. Transport Contention (PRIMARY)
- Multiple overlapping polls/HID calls every ~2s
- No mutex/locking on device access
- Race conditions between poller and updater
- **Evidence**: Timeouts during continuous polling

#### B. Ancient Firmware Stack
- Bootloader 1.0.3 + FW 4.0.0 are extremely outdated
- Mode transitions (bootloader ↔ app) cause races
- **Evidence**: Device appears/disappears during transitions

#### C. USB Environment Issues
- Possible hub/cable/power issues
- Windows HID driver conflicts
- Other apps grabbing device
- **Evidence**: Intermittent "Device not found"

#### D. State Management Bugs
- Mixing stale/partial features into status
- No validation of feature completeness
- **Evidence**: `success: true` with `features: null`

## Fix Implementation Plan

### Phase 1: Verbose Logging (Immediate)

#### 1.1 Enhanced Device Operation Logging
```rust
// Every device operation should log:
- Timestamp (ms precision)
- Device ID & path
- Operation type & parameters
- Transport state (open/closed/error)
- Response time
- Full error context
- USB environment (hub depth, power state)
```

#### 1.2 State Transition Logging
```rust
// Log all state changes:
- Previous state
- New state
- Trigger event
- Validation results
- Feature differences
```

#### 1.3 Transport Debug Logging
```rust
// Log transport layer:
- HID open/close events
- Read/write operations with sizes
- Timeout occurrences
- Retry attempts
- Concurrent access attempts
```

### Phase 2: Durability Improvements

#### 2.1 Single-Flight Polling
```rust
struct DevicePoller {
    active_poll: Option<JoinHandle>,
    poll_mutex: Arc<Mutex<()>>,
}

impl DevicePoller {
    async fn poll(&self, device_id: String) -> Result<Features> {
        // Cancel any existing poll
        if let Some(handle) = &self.active_poll {
            handle.abort();
        }
        
        // Acquire mutex for this device
        let _guard = self.poll_mutex.lock().await;
        
        // Perform poll with timeout
        tokio::time::timeout(Duration::from_secs(5), 
            self.get_features(device_id)
        ).await
    }
}
```

#### 2.2 Exponential Backoff
```rust
struct BackoffStrategy {
    base_delay: Duration,
    max_delay: Duration,
    current_delay: Duration,
    consecutive_failures: u32,
}

impl BackoffStrategy {
    fn on_failure(&mut self) {
        self.consecutive_failures += 1;
        self.current_delay = min(
            self.current_delay * 2,
            self.max_delay
        );
    }
    
    fn on_success(&mut self) {
        self.consecutive_failures = 0;
        self.current_delay = self.base_delay;
    }
    
    fn next_poll_delay(&self) -> Duration {
        self.current_delay
    }
}
```

#### 2.3 Robust Error Handling
```rust
// Never return success with null features
async fn get_device_status(device_id: String) -> DeviceStatus {
    match timeout(Duration::from_secs(5), get_features(device_id)).await {
        Ok(Ok(features)) => DeviceStatus {
            success: true,
            features: Some(features),
            error: None,
        },
        Ok(Err(e)) => DeviceStatus {
            success: false,
            features: None,
            error: Some(format!("Transport error: {}", e)),
        },
        Err(_) => DeviceStatus {
            success: false,
            features: None,
            error: Some("Operation timed out".to_string()),
        },
    }
}
```

#### 2.4 Transport Recovery
```rust
async fn with_transport_recovery<T>(
    device_id: String,
    operation: impl Fn() -> Result<T>
) -> Result<T> {
    let mut attempts = 0;
    loop {
        match operation() {
            Ok(result) => return Ok(result),
            Err(e) if is_transport_error(&e) && attempts < 3 => {
                log::warn!("Transport error, attempt {}/3: {}", attempts + 1, e);
                
                // Close and reopen device
                close_device(&device_id).await;
                tokio::time::sleep(Duration::from_millis(500)).await;
                open_device(&device_id).await?;
                
                attempts += 1;
            },
            Err(e) => return Err(e),
        }
    }
}
```

### Phase 3: Core Issue Fixes

#### 3.1 Mode-Aware State Machine
```rust
enum DeviceMode {
    Application { features: Features },
    Bootloader { version: String },
    Transitioning,
    Disconnected,
}

struct DeviceStateMachine {
    mode: DeviceMode,
    last_valid_features: Option<Features>,
    
    fn handle_poll_result(&mut self, result: PollResult) {
        match (&self.mode, result) {
            (_, PollResult::Bootloader(version)) => {
                log::info!("Device in bootloader mode v{}", version);
                self.mode = DeviceMode::Bootloader { version };
                // Suspend normal polling
                // Show bootloader UI
            },
            (DeviceMode::Bootloader { .. }, PollResult::Application(features)) => {
                log::info!("Device transitioned to application mode");
                self.mode = DeviceMode::Application { features };
                self.last_valid_features = Some(features);
            },
            (_, PollResult::NotFound) => {
                log::warn!("Device not found, marking as disconnected");
                self.mode = DeviceMode::Disconnected;
            },
            _ => {}
        }
    }
}
```

#### 3.2 Validated Status Builder
```rust
fn build_device_status(features: Option<Features>) -> DeviceStatus {
    let Some(features) = features else {
        return DeviceStatus {
            success: false,
            error: Some("No features available".to_string()),
            ..Default::default()
        };
    };
    
    // Validate feature consistency
    if !features.initialized && features.has_backup {
        log::error!("Invalid state: uninitialized device claims backup");
        return DeviceStatus {
            success: false,
            error: Some("Inconsistent device state".to_string()),
            ..Default::default()
        };
    }
    
    DeviceStatus {
        success: true,
        features: Some(features),
        needs_firmware_update: check_firmware_version(&features),
        needs_bootloader_update: check_bootloader_version(&features),
        needs_initialization: !features.initialized,
        ..Default::default()
    }
}
```

#### 3.3 Coordinated Operations
```rust
struct DeviceOperationCoordinator {
    active_operations: Arc<Mutex<HashMap<String, OperationType>>>,
    
    async fn begin_operation(&self, device_id: String, op_type: OperationType) -> OperationGuard {
        let mut ops = self.active_operations.lock().await;
        
        // Cancel polling if update/wipe operation
        if matches!(op_type, OperationType::Update | OperationType::Wipe) {
            if let Some(OperationType::Poll) = ops.get(&device_id) {
                log::info!("Cancelling poll for {} operation", op_type);
                ops.remove(&device_id);
            }
        }
        
        ops.insert(device_id.clone(), op_type);
        OperationGuard { device_id, coordinator: self.clone() }
    }
}
```

## Implementation Priority

### Immediate (Today)
1. ✅ Add verbose logging to all device operations
2. ✅ Fix `success: true` with `features: null` bug
3. ✅ Add operation timeouts (5s max)

### Short-term (This Week)
1. Implement single-flight polling
2. Add exponential backoff
3. Mode-aware state machine
4. Transport recovery logic

### Medium-term (Next Week)
1. Operation coordinator
2. Comprehensive state validation
3. USB diagnostics collection
4. Performance monitoring

## Testing Strategy

### Unit Tests
- Mock HID transport for failure scenarios
- Test state machine transitions
- Verify backoff behavior

### Integration Tests
- Simulate device disconnection
- Test bootloader ↔ app transitions
- Concurrent operation handling

### Manual Testing
- Physical device disconnect/reconnect
- Update flows with old firmware
- USB hub/cable variations

## Monitoring & Metrics

### Key Metrics to Track
- Transport error rate
- Average poll latency
- State transition frequency
- Recovery success rate
- Concurrent operation conflicts

### Alert Thresholds
- >10% transport errors → Alert
- >5s poll latency → Warning
- >3 retries needed → Log
- State validation failures → Error

## User Communication

### Error Messages
```typescript
const ERROR_MESSAGES = {
  TRANSPORT_TIMEOUT: "Device communication timed out. Please check your cable and try again.",
  DEVICE_BUSY: "Device is busy with another operation. Please wait...",
  INCONSISTENT_STATE: "Device is in an unexpected state. Please unplug and reconnect.",
  UPDATE_REQUIRED: "Your device firmware is severely outdated. Update required for stability.",
};
```

### Recovery Instructions
1. Check cable and USB port
2. Close other wallet applications
3. Update firmware if outdated
4. Contact support if issues persist

## Success Criteria

- Zero `success: true` with `null` features
- <1% transport error rate
- <2s average poll latency
- 100% recovery from transient failures
- Clear error messages for users

## Timeline

- **Day 1**: Verbose logging + critical bug fixes
- **Day 2-3**: Durability improvements
- **Day 4-5**: Core state machine fixes
- **Day 6-7**: Testing and validation
- **Week 2**: Monitoring and optimization