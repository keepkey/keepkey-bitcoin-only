pub mod queue;
pub mod updates;
pub mod pending_operations;
pub mod state;
pub mod interaction_state;
pub mod events;
pub mod usb_monitor;

// Re-export the bootloader update tracker
pub use updates::BootloaderUpdateTracker;
// Re-export device state types
pub use state::{DeviceState, DEVICE_STATE_TRACKER};
// Re-export interaction state types
pub use interaction_state::{DeviceSession, DeviceInteractionState, DEVICE_SESSIONS};
// Re-export event types
pub use events::{DeviceEvent, UICommand, emit_device_event};
// Re-export USB monitor
pub use usb_monitor::UsbMonitor;
// Re-export queue types
pub use queue::{DEVICE_STATE_CACHE, DeviceStateCache}; 