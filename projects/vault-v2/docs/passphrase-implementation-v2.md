# Passphrase Implementation V2 - State Machine Approach

## Core Architecture

### 1. Device Interaction State Machine

```rust
// src-tauri/src/device/interaction_state.rs

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use std::time::Instant;

#[derive(Debug, Clone, PartialEq)]
pub enum DeviceInteractionState {
    Idle,
    PendingSettings { request_id: Uuid },
    AwaitingPIN { request_id: Uuid, operation: OperationType },
    AwaitingButton { request_id: Uuid, label: Option<String> },
    AwaitingPassphrase { request_id: Uuid, cache_allowed: bool },
    NeedsReconnect { reason: ReconnectReason },
    WaitingForReconnect,
    Reinitializing,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OperationType {
    Settings,
    Transaction,
    Export,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ReconnectReason {
    PassphraseEnabled,
    PassphraseDisabled,
    DeviceReset,
}

#[derive(Debug, Clone)]
pub struct PendingInteraction {
    pub request_id: Uuid,
    pub kind: InteractionKind,
    pub created_at: Instant,
    pub operation_type: OperationType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InteractionKind {
    Pin,
    Button,
    Passphrase,
}

pub struct DeviceSession {
    pub device_id: String,
    pub state: DeviceInteractionState,
    pub pending: Option<PendingInteraction>,
    pub passphrase_cached: bool,
    pub passphrase_cache_expiry: Option<Instant>,
}

lazy_static::lazy_static! {
    pub static ref DEVICE_SESSIONS: Arc<RwLock<HashMap<String, DeviceSession>>> = 
        Arc::new(RwLock::new(HashMap::new()));
}

impl DeviceSession {
    pub fn transition(&mut self, new_state: DeviceInteractionState) -> Result<(), String> {
        // Validate state transition
        let valid = match (&self.state, &new_state) {
            (DeviceInteractionState::Idle, DeviceInteractionState::PendingSettings { .. }) => true,
            (DeviceInteractionState::PendingSettings { .. }, DeviceInteractionState::AwaitingPIN { .. }) => true,
            (DeviceInteractionState::PendingSettings { .. }, DeviceInteractionState::AwaitingButton { .. }) => true,
            (DeviceInteractionState::PendingSettings { .. }, DeviceInteractionState::NeedsReconnect { .. }) => true,
            (DeviceInteractionState::AwaitingPIN { .. }, DeviceInteractionState::PendingSettings { .. }) => true,
            (DeviceInteractionState::AwaitingButton { .. }, DeviceInteractionState::PendingSettings { .. }) => true,
            (DeviceInteractionState::NeedsReconnect { .. }, DeviceInteractionState::WaitingForReconnect) => true,
            (DeviceInteractionState::WaitingForReconnect, DeviceInteractionState::Reinitializing) => true,
            (DeviceInteractionState::Reinitializing, DeviceInteractionState::Idle) => true,
            // Allow cancellation back to Idle from any state
            (_, DeviceInteractionState::Idle) => true,
            _ => false,
        };

        if !valid {
            return Err(format!("Invalid state transition from {:?} to {:?}", self.state, new_state));
        }

        log::info!("Device {} transitioning from {:?} to {:?}", self.device_id, self.state, new_state);
        self.state = new_state;
        Ok(())
    }

    pub fn begin_interaction(&mut self, kind: InteractionKind, operation: OperationType) -> Uuid {
        let id = Uuid::new_v4();
        self.pending = Some(PendingInteraction {
            request_id: id,
            kind,
            created_at: Instant::now(),
            operation_type: operation,
        });
        id
    }

    pub fn clear_interaction(&mut self) {
        self.pending = None;
    }

    pub fn validate_interaction(&self, request_id: &Uuid, kind: InteractionKind) -> Result<(), String> {
        match &self.pending {
            Some(p) if p.request_id == *request_id && p.kind == kind => Ok(()),
            Some(p) if p.request_id != *request_id => Err("Mismatched request ID".to_string()),
            Some(p) if p.kind != kind => Err("Mismatched interaction type".to_string()),
            None => Err("No pending interaction".to_string()),
        }
    }
}
```

### 2. Typed Event System

```rust
// src-tauri/src/device/events.rs

use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DeviceEvent {
    DeviceState {
        device_id: String,
        state: DeviceStateDto,
    },
    DeviceNeedsReconnect {
        device_id: String,
        reason: String,
    },
    DeviceAwaitingPin {
        device_id: String,
        request_id: Uuid,
        kind: String, // "settings" | "tx" | "export"
    },
    DeviceAwaitingButton {
        device_id: String,
        request_id: Uuid,
        label: Option<String>,
    },
    DeviceAwaitingPassphrase {
        device_id: String,
        request_id: Uuid,
        cache_allowed: bool,
    },
    DeviceError {
        device_id: String,
        request_id: Option<Uuid>,
        code: String,
        message: String,
    },
    DeviceConnected {
        device_id: String,
    },
    DeviceDisconnected {
        device_id: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UICommand {
    PinSubmit {
        device_id: String,
        request_id: Uuid,
        pin: String,
    },
    PinCancel {
        device_id: String,
        request_id: Uuid,
    },
    ButtonAck {
        device_id: String,
        request_id: Uuid,
    },
    PassphraseSubmit {
        device_id: String,
        request_id: Uuid,
        passphrase: String,
    },
    PassphraseCancel {
        device_id: String,
        request_id: Uuid,
    },
}

pub async fn emit_device_event(app: &AppHandle, event: DeviceEvent) -> Result<(), String> {
    let event_name = match &event {
        DeviceEvent::DeviceState { .. } => "device:state",
        DeviceEvent::DeviceNeedsReconnect { .. } => "device:needs_reconnect",
        DeviceEvent::DeviceAwaitingPin { .. } => "device:awaiting_pin",
        DeviceEvent::DeviceAwaitingButton { .. } => "device:awaiting_button",
        DeviceEvent::DeviceAwaitingPassphrase { .. } => "device:awaiting_passphrase",
        DeviceEvent::DeviceError { .. } => "device:error",
        DeviceEvent::DeviceConnected { .. } => "device:connected",
        DeviceEvent::DeviceDisconnected { .. } => "device:disconnected",
    };

    app.emit(event_name, &event)
        .map_err(|e| format!("Failed to emit event: {}", e))
}
```

### 3. Improved Passphrase Enable/Disable Flow

```rust
// src-tauri/src/commands.rs

#[tauri::command]
pub async fn enable_passphrase_protection(
    device_id: String,
    enabled: bool,
    queue_manager: State<'_, DeviceQueueManager>,
    app: AppHandle,
) -> Result<(), String> {
    // Check if device is already in an operation
    {
        let sessions = DEVICE_SESSIONS.read().await;
        if let Some(session) = sessions.get(&device_id) {
            if session.state != DeviceInteractionState::Idle {
                return Err("Device is busy with another operation".to_string());
            }
        }
    }

    // Create request ID for correlation
    let request_id = Uuid::new_v4();
    
    // Update session state
    {
        let mut sessions = DEVICE_SESSIONS.write().await;
        let session = sessions.entry(device_id.clone()).or_insert_with(|| {
            DeviceSession {
                device_id: device_id.clone(),
                state: DeviceInteractionState::Idle,
                pending: None,
                passphrase_cached: false,
                passphrase_cache_expiry: None,
            }
        });
        
        session.transition(DeviceInteractionState::PendingSettings { request_id })?;
    }

    // Get device queue handle
    let queue_handle = {
        let manager = queue_manager.lock().await;
        manager.get(&device_id).cloned()
    }.ok_or_else(|| "No device queue found".to_string())?;

    // Send ApplySettings
    let apply_settings = keepkey_rust::messages::ApplySettings {
        use_passphrase: Some(enabled),
        ..Default::default()
    };

    // Process the response with proper state transitions
    match queue_handle.send_raw(Message::ApplySettings(apply_settings), true).await {
        Ok(Message::ButtonRequest(br)) => {
            // Update state to awaiting button
            {
                let mut sessions = DEVICE_SESSIONS.write().await;
                if let Some(session) = sessions.get_mut(&device_id) {
                    session.transition(DeviceInteractionState::AwaitingButton { 
                        request_id,
                        label: br.data,
                    })?;
                }
            }

            // Emit event
            emit_device_event(&app, DeviceEvent::DeviceAwaitingButton {
                device_id: device_id.clone(),
                request_id,
                label: br.data,
            }).await?;

            // Send ButtonAck
            queue_handle.send_raw(Message::ButtonAck(Default::default()), true).await
                .map_err(|e| format!("Failed to send button ack: {}", e))?;

            // Continue processing...
            handle_settings_response(device_id, request_id, queue_handle, app, enabled).await
        }
        Ok(Message::PinMatrixRequest(_)) => {
            // Update state to awaiting PIN
            {
                let mut sessions = DEVICE_SESSIONS.write().await;
                if let Some(session) = sessions.get_mut(&device_id) {
                    let interaction_id = session.begin_interaction(
                        InteractionKind::Pin,
                        OperationType::Settings
                    );
                    session.transition(DeviceInteractionState::AwaitingPIN { 
                        request_id: interaction_id,
                        operation: OperationType::Settings,
                    })?;
                }
            }

            // Emit event
            emit_device_event(&app, DeviceEvent::DeviceAwaitingPin {
                device_id: device_id.clone(),
                request_id,
                kind: "settings".to_string(),
            }).await?;

            Ok(())
        }
        Ok(Message::Success(_)) => {
            // Update state to needs reconnect
            let reason = if enabled {
                ReconnectReason::PassphraseEnabled
            } else {
                ReconnectReason::PassphraseDisabled
            };

            {
                let mut sessions = DEVICE_SESSIONS.write().await;
                if let Some(session) = sessions.get_mut(&device_id) {
                    session.transition(DeviceInteractionState::NeedsReconnect { reason: reason.clone() })?;
                }
            }

            // Update device state
            crate::device::state::set_device_needs_reset(&device_id, true).await;
            crate::device::state::set_device_passphrase_state(&device_id, enabled, false).await;

            // Emit event
            emit_device_event(&app, DeviceEvent::DeviceNeedsReconnect {
                device_id: device_id.clone(),
                reason: format!("{:?}", reason),
            }).await?;

            Ok(())
        }
        Ok(Message::Failure(f)) => {
            // Reset to idle
            {
                let mut sessions = DEVICE_SESSIONS.write().await;
                if let Some(session) = sessions.get_mut(&device_id) {
                    session.transition(DeviceInteractionState::Idle)?;
                }
            }

            Err(format!("Device error: {}", f.message.unwrap_or_default()))
        }
        _ => Err("Unexpected response from device".to_string())
    }
}

#[tauri::command]
pub async fn pin_submit(
    device_id: String,
    request_id: Uuid,
    pin: String,
    queue_manager: State<'_, DeviceQueueManager>,
    app: AppHandle,
) -> Result<(), String> {
    // Validate the interaction
    {
        let sessions = DEVICE_SESSIONS.read().await;
        if let Some(session) = sessions.get(&device_id) {
            session.validate_interaction(&request_id, InteractionKind::Pin)?;
        } else {
            return Err("No active session for device".to_string());
        }
    }

    // Validate non-empty PIN
    if pin.is_empty() {
        return Err("PIN cannot be empty".to_string());
    }

    // Get queue handle
    let queue_handle = {
        let manager = queue_manager.lock().await;
        manager.get(&device_id).cloned()
    }.ok_or_else(|| "No device queue found".to_string())?;

    // Parse PIN positions
    let positions: Vec<u32> = pin.chars()
        .filter_map(|c| c.to_digit(10))
        .collect();

    // Send PinMatrixAck
    let pin_ack = keepkey_rust::messages::PinMatrixAck {
        pin: positions.iter().map(|&p| p.to_string()).collect::<String>(),
    };

    match queue_handle.send_raw(Message::PinMatrixAck(pin_ack), true).await {
        Ok(response) => {
            // Clear the interaction
            {
                let mut sessions = DEVICE_SESSIONS.write().await;
                if let Some(session) = sessions.get_mut(&device_id) {
                    session.clear_interaction();
                    session.transition(DeviceInteractionState::PendingSettings { request_id })?;
                }
            }

            // Continue processing the original operation
            handle_pin_response(device_id, request_id, response, app).await
        }
        Err(e) => {
            // Reset to idle on error
            {
                let mut sessions = DEVICE_SESSIONS.write().await;
                if let Some(session) = sessions.get_mut(&device_id) {
                    session.transition(DeviceInteractionState::Idle)?;
                }
            }
            Err(format!("Failed to submit PIN: {}", e))
        }
    }
}

#[tauri::command]
pub async fn pin_cancel(
    device_id: String,
    request_id: Uuid,
) -> Result<(), String> {
    let mut sessions = DEVICE_SESSIONS.write().await;
    if let Some(session) = sessions.get_mut(&device_id) {
        session.validate_interaction(&request_id, InteractionKind::Pin)?;
        session.clear_interaction();
        session.transition(DeviceInteractionState::Idle)?;
    }
    Ok(())
}
```

### 4. USB Hotplug Detection

```rust
// src-tauri/src/device/usb_monitor.rs

use std::sync::Arc;
use tokio::sync::RwLock;
use tauri::AppHandle;

pub struct UsbMonitor {
    app: AppHandle,
    last_devices: Arc<RwLock<Vec<String>>>,
}

impl UsbMonitor {
    pub fn new(app: AppHandle) -> Self {
        Self {
            app,
            last_devices: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn start(self: Arc<Self>) {
        // Use platform-specific USB monitoring
        #[cfg(target_os = "linux")]
        self.start_udev_monitor().await;
        
        #[cfg(target_os = "macos")]
        self.start_iokit_monitor().await;
        
        #[cfg(target_os = "windows")]
        self.start_windows_monitor().await;
    }

    async fn check_device_changes(&self) {
        let current_devices = keepkey_rust::features::list_connected_devices();
        let current_ids: Vec<String> = current_devices
            .iter()
            .map(|d| d.unique_id.clone())
            .collect();

        let mut last = self.last_devices.write().await;
        
        // Check for disconnections
        for old_id in last.iter() {
            if !current_ids.contains(old_id) {
                log::info!("Device disconnected: {}", old_id);
                
                // Update session state
                {
                    let mut sessions = DEVICE_SESSIONS.write().await;
                    if let Some(session) = sessions.get_mut(old_id) {
                        if matches!(session.state, DeviceInteractionState::NeedsReconnect { .. }) {
                            session.transition(DeviceInteractionState::WaitingForReconnect).ok();
                        }
                    }
                }

                // Emit event
                emit_device_event(&self.app, DeviceEvent::DeviceDisconnected {
                    device_id: old_id.clone(),
                }).await.ok();
            }
        }

        // Check for connections
        for new_id in &current_ids {
            if !last.contains(new_id) {
                log::info!("Device connected: {}", new_id);
                
                // Handle reconnection
                {
                    let mut sessions = DEVICE_SESSIONS.write().await;
                    if let Some(session) = sessions.get_mut(new_id) {
                        if session.state == DeviceInteractionState::WaitingForReconnect {
                            session.transition(DeviceInteractionState::Reinitializing).ok();
                            
                            // Spawn reinitialization task
                            let device_id = new_id.clone();
                            let app = self.app.clone();
                            tokio::spawn(async move {
                                reinitialize_device(device_id, app).await;
                            });
                        }
                    }
                }

                // Emit event
                emit_device_event(&self.app, DeviceEvent::DeviceConnected {
                    device_id: new_id.clone(),
                }).await.ok();
            }
        }

        *last = current_ids;
    }

    #[cfg(target_os = "linux")]
    async fn start_udev_monitor(&self) {
        // Use udev crate for Linux USB monitoring
        // This is more efficient than polling
    }

    #[cfg(target_os = "macos")]
    async fn start_iokit_monitor(&self) {
        // Use IOKit for macOS USB monitoring
    }

    #[cfg(target_os = "windows")]
    async fn start_windows_monitor(&self) {
        // Use Windows device notifications
    }
}

async fn reinitialize_device(device_id: String, app: AppHandle) -> Result<(), String> {
    // Initialize device
    // ... device initialization code ...

    // Get features
    // ... get features code ...

    // Update state
    {
        let mut sessions = DEVICE_SESSIONS.write().await;
        if let Some(session) = sessions.get_mut(&device_id) {
            session.transition(DeviceInteractionState::Idle)?;
            session.passphrase_cached = false;
            session.passphrase_cache_expiry = None;
        }
    }

    // Update device state
    crate::device::state::set_device_needs_reset(&device_id, false).await;

    Ok(())
}
```

### 5. Frontend Integration

```typescript
// src/hooks/useDeviceInteraction.ts

import { useEffect, useCallback, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

interface DeviceEvent {
  type: string;
  device_id: string;
  request_id?: string;
  [key: string]: any;
}

export function useDeviceInteraction() {
  const activeRequests = useRef<Map<string, string>>(new Map());

  useEffect(() => {
    const unlisteners: Array<() => void> = [];

    // Listen for PIN requests
    listen<DeviceEvent>('device:awaiting_pin', (event) => {
      const { device_id, request_id, kind } = event.payload;
      activeRequests.current.set(request_id, device_id);
      
      // Show PIN modal with correlation
      showPinModal({
        deviceId: device_id,
        requestId: request_id,
        operationType: kind,
        onSubmit: (pin: string) => {
          invoke('pin_submit', { 
            deviceId: device_id, 
            requestId: request_id, 
            pin 
          });
        },
        onCancel: () => {
          invoke('pin_cancel', { 
            deviceId: device_id, 
            requestId: request_id 
          });
        }
      });
    }).then(u => unlisteners.push(u));

    // Listen for reconnect needs
    listen<DeviceEvent>('device:needs_reconnect', (event) => {
      const { device_id, reason } = event.payload;
      
      showReconnectDialog({
        deviceId: device_id,
        reason,
        // No polling - wait for device:connected event
      });
    }).then(u => unlisteners.push(u));

    // Listen for device reconnection
    listen<DeviceEvent>('device:connected', (event) => {
      const { device_id } = event.payload;
      
      // Close reconnect dialog if open for this device
      closeReconnectDialog(device_id);
      
      // Refresh device state
      refreshDeviceState(device_id);
    }).then(u => unlisteners.push(u));

    return () => {
      unlisteners.forEach(u => u());
    };
  }, []);

  const submitPin = useCallback(async (deviceId: string, requestId: string, pin: string) => {
    if (!activeRequests.current.has(requestId)) {
      console.error('Invalid request ID for PIN submission');
      return;
    }

    try {
      await invoke('pin_submit', { deviceId, requestId, pin });
      activeRequests.current.delete(requestId);
    } catch (error) {
      console.error('PIN submission failed:', error);
      // Show error to user
    }
  }, []);

  const cancelPin = useCallback(async (deviceId: string, requestId: string) => {
    if (!activeRequests.current.has(requestId)) {
      console.error('Invalid request ID for PIN cancellation');
      return;
    }

    try {
      await invoke('pin_cancel', { deviceId, requestId });
      activeRequests.current.delete(requestId);
    } catch (error) {
      console.error('PIN cancellation failed:', error);
    }
  }, []);

  return {
    submitPin,
    cancelPin,
  };
}
```

## Key Improvements

1. **State Machine**: Enforces valid transitions, prevents race conditions
2. **Request Correlation**: Every interaction has a unique UUID
3. **No Polling**: USB hotplug events drive reconnection detection
4. **Typed Events**: Strict schema prevents ambiguity
5. **Idempotency**: Duplicate requests rejected with proper error
6. **Session Management**: Per-device state with interaction tracking
7. **Proper Cleanup**: Timeouts and cancellation paths defined

## Migration Path

1. Add state machine module alongside existing code
2. Wire up USB monitoring in parallel with current flow
3. Update PassphraseSettings to use new commands
4. Remove app restart logic
5. Test with correlation validation
6. Remove old passphrase flow code

## Testing Matrix

- [ ] Enable passphrase with PIN → PIN modal shows with correct requestId
- [ ] Cancel PIN during enable → Toggle reverts, state returns to Idle
- [ ] Submit empty PIN → Error shown, no empty ack sent
- [ ] Disconnect during PIN entry → Modal closes, state resets
- [ ] Different device connected → Separate session, no cross-talk
- [ ] Double-click toggle → Second request rejected as busy
- [ ] Reconnect detected < 200ms → Via USB hotplug, not polling