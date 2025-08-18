use serde::{Serialize, Deserialize};
use uuid::Uuid;
use tauri::{AppHandle, Emitter};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceStateDto {
    pub is_unlocked: bool,
    pub needs_passphrase: bool,
    pub needs_reset: bool,
    pub is_busy: bool,
    pub current_operation: Option<String>,
    pub pin_cached: bool,
    pub passphrase_cached: bool,
}

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