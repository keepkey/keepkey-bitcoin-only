use super::Message;
use core::time::Duration;

const TIMEOUT: Duration = Duration::from_millis(5000); // Increased for better device compatibility
const LONG_TIMEOUT: Duration = Duration::from_millis(5 * 60 * 1000);
const LEGACY_DEVICE_TIMEOUT: Duration = Duration::from_millis(10000); // 10 seconds for older devices
const QUICK_TIMEOUT: Duration = Duration::from_millis(5000); // 5 seconds for device communication (was causing timeouts)

// Windows-specific timeouts - HID API is slower than USB on Windows
#[cfg(target_os = "windows")]
const WINDOWS_HID_TIMEOUT: Duration = Duration::from_millis(30000); // 30 seconds for Windows HID (increased from 15)
#[cfg(target_os = "windows")]
const WINDOWS_HID_QUICK_TIMEOUT: Duration = Duration::from_millis(20000); // 20 seconds for Initialize/GetFeatures on Windows HID (increased from 10)

// Thread-local storage to track if we're communicating with an older device
thread_local! {
    static USING_LEGACY_DEVICE: std::cell::RefCell<bool> = std::cell::RefCell::new(false);
}

// Thread-local storage to track if we're using HID transport (Windows-specific)
thread_local! {
    static USING_HID_TRANSPORT: std::cell::RefCell<bool> = std::cell::RefCell::new(false);
}

impl Message {
    /// Set whether we're communicating with a legacy device (PID 0x0001)
    /// This affects timeouts for all messages in the current thread
    pub fn set_legacy_device_mode(enabled: bool) {
        USING_LEGACY_DEVICE.with(|mode| {
            *mode.borrow_mut() = enabled;
        });
    }
    
    /// Get whether we're in legacy device mode
    pub fn is_legacy_device_mode() -> bool {
        USING_LEGACY_DEVICE.with(|mode| *mode.borrow())
    }
    
    /// Set whether we're using HID transport (affects Windows timeouts)
    #[cfg(target_os = "windows")]
    pub fn set_hid_transport_mode(enabled: bool) {
        USING_HID_TRANSPORT.with(|mode| {
            *mode.borrow_mut() = enabled;
        });
    }
    
    /// Get whether we're using HID transport
    #[cfg(target_os = "windows")]
    pub fn is_hid_transport_mode() -> bool {
        USING_HID_TRANSPORT.with(|mode| *mode.borrow())
    }
    
    pub fn read_timeout(&self) -> Duration {
        // Windows HID fix: Use longer timeouts for Windows HID communication
        #[cfg(target_os = "windows")]
        {
            if Message::is_hid_transport_mode() {
                return match self {
                    Message::ButtonAck(_) => LONG_TIMEOUT,
                    Message::FirmwareUpload(_) => LONG_TIMEOUT,
                    Message::Initialize(_) => WINDOWS_HID_QUICK_TIMEOUT, // 10 seconds for Windows HID
                    Message::GetFeatures(_) => WINDOWS_HID_QUICK_TIMEOUT, // 10 seconds for Windows HID
                    _ if Message::is_legacy_device_mode() => WINDOWS_HID_TIMEOUT, // 15 seconds for legacy on Windows HID
                    _ => WINDOWS_HID_TIMEOUT, // 15 seconds default for Windows HID
                };
            }
        }
        
        // For Initialize messages or when in legacy device mode, use appropriate timeout
        match self {
            Message::ButtonAck(_) => LONG_TIMEOUT,
            Message::FirmwareUpload(_) => LONG_TIMEOUT, // Bootloader updates need more time
            Message::Initialize(_) if Message::is_legacy_device_mode() => LEGACY_DEVICE_TIMEOUT,
            Message::Initialize(_) => QUICK_TIMEOUT, // Initialize is usually very fast
            Message::GetFeatures(_) => QUICK_TIMEOUT, // GetFeatures is also fast
            _ if Message::is_legacy_device_mode() => LEGACY_DEVICE_TIMEOUT,
            _ => TIMEOUT,
        }
    }

    pub fn write_timeout(&self) -> Duration {
        // Windows HID fix: Use longer timeouts for Windows HID communication
        #[cfg(target_os = "windows")]
        {
            if Message::is_hid_transport_mode() {
                return match self {
                    Message::FirmwareUpload(_) => LONG_TIMEOUT,
                    Message::Initialize(_) => WINDOWS_HID_QUICK_TIMEOUT, // 10 seconds for Windows HID
                    Message::GetFeatures(_) => WINDOWS_HID_QUICK_TIMEOUT, // 10 seconds for Windows HID
                    _ if Message::is_legacy_device_mode() => WINDOWS_HID_TIMEOUT, // 15 seconds for legacy on Windows HID
                    _ => WINDOWS_HID_TIMEOUT, // 15 seconds default for Windows HID
                };
            }
        }
        
        match self {
            Message::FirmwareUpload(_) => LONG_TIMEOUT,
            Message::Initialize(_) if Message::is_legacy_device_mode() => LEGACY_DEVICE_TIMEOUT,
            Message::Initialize(_) => QUICK_TIMEOUT,
            Message::GetFeatures(_) => QUICK_TIMEOUT,
            _ if Message::is_legacy_device_mode() => LEGACY_DEVICE_TIMEOUT,
            _ => TIMEOUT,
        }
    }
}
