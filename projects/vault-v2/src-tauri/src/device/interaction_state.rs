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
            _ => Err("Unknown validation error".to_string()),
        }
    }
}