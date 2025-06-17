use anyhow::{anyhow, Result};
use hex;
use rusb::{Device, GlobalContext};
use serde::{Deserialize, Serialize};

use crate::messages::{Initialize, Message};
use crate::transport::{ProtocolAdapter, UsbTransport, HidTransport};
use crate::friendly_usb::FriendlyUsbDevice;


const TAG: &str = " | features | ";
const DEVICE_IDS: &[(u16, u16)] = &[(0x2b24, 0x0001), (0x2b24, 0x0002)];

// List usb devices
// This is kept internal to this module for now.
//
// Device state/fallback heuristics are based on:
//   - /docs/usb/oob_mode_detection.md
//   - /docs/usb/hid_fallback_implementation.md
//   - Vault backend (src-tauri/src/features/mod.rs)
pub fn list_devices() -> Box<[Device<GlobalContext>]> {
    rusb::devices()
        .unwrap()
        .iter()
        .filter(|device| {
            let device_desc = device.device_descriptor().unwrap();
            DEVICE_IDS.contains(&(device_desc.vendor_id(), device_desc.product_id()))
        })
        .collect()
}

/// Structure representing device features returned by the KeepKey
/// This is a simplified version that includes the most commonly used fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceFeatures {
    /// Device label or name
    pub label: Option<String>,
    /// Device vendor
    pub vendor: Option<String>,
    /// Device model
    pub model: Option<String>,
    /// Firmware variant (if any)
    pub firmware_variant: Option<String>,
    /// Unique device identifier
    pub device_id: Option<String>,
    /// Device language setting
    pub language: Option<String>,
    /// Whether the device is in bootloader mode
    pub bootloader_mode: bool,
    /// Firmware version components
    pub version: String,
    /// Firmware hash (hex encoded)
    pub firmware_hash: Option<String>,
    /// Bootloader hash (hex encoded)
    pub bootloader_hash: Option<String>,
    /// Bootloader version derived from hash
    pub bootloader_version: Option<String>,
    /// Whether the device has been initialized
    pub initialized: bool,
    /// Whether keys were imported from a computer
    pub imported: Option<bool>,
    /// Whether keys were not backed up during setup
    pub no_backup: bool,
    /// Whether PIN protection is enabled
    pub pin_protection: bool,
    /// Whether the device is currently unlocked
    pub pin_cached: bool,
    /// Whether passphrase protection is enabled
    pub passphrase_protection: bool,
    /// Whether the passphrase is currently cached
    pub passphrase_cached: bool,
    /// Whether wipe code protection is enabled
    pub wipe_code_protection: bool,
    /// Auto-lock delay in milliseconds
    pub auto_lock_delay_ms: Option<u64>,
    /// Enabled policies
    pub policies: Vec<String>,
}

/// Get device features from a specific KeepKey device
///
/// This function connects to a specific KeepKey device by matching the unique ID,
/// sends a GetFeatures request, and returns a structured representation
/// of the device's features and settings.
///
/// # Arguments
/// * `target_device` - The specific device to get features for
///
/// # Returns
/// - `Ok(DeviceFeatures)` if successful with all device information
/// - `Err` if device connection fails or the device doesn't respond properly
/// Detect the device state using Vault-style heuristics.
/// See: /docs/usb/oob_mode_detection.md
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DetectedDeviceState {
    WalletMode,
    BootloaderMode,
    OobWalletMode,
    OobBootloaderMode,
    Unknown,
}

/// Heuristic device state detection
pub fn detect_device_state(features: &DeviceFeatures, raw_len: Option<usize>) -> DetectedDeviceState {
    // 1. Bootloader flag
    if features.bootloader_mode {
        // If features response is very short, it's OOB bootloader
        if let Some(len) = raw_len {
            if len < 32 {
                return DetectedDeviceState::OobBootloaderMode;
            }
        }
        return DetectedDeviceState::BootloaderMode;
    }
    // 2. Firmware string
    if features.version == "Legacy Bootloader" {
        return DetectedDeviceState::BootloaderMode;
    }
    // 3. Uninitialized + short response = OOB wallet
    if !features.initialized {
        if let Some(len) = raw_len {
            if len < 64 {
                return DetectedDeviceState::OobWalletMode;
            }
        }
        return DetectedDeviceState::OobWalletMode;
    }
    // 4. Default
    DetectedDeviceState::WalletMode
}

pub fn get_device_features_for_device(target_device: &FriendlyUsbDevice) -> Result<DeviceFeatures> {
    log::info!("{TAG} Getting features for device: {} ({})", target_device.name, target_device.unique_id);
    
    let devices = list_devices();
    
    let device = if let Some(serial) = &target_device.serial_number {
        devices.iter().find(|d| {
            if let Ok(handle) = d.open() {
                let timeout = std::time::Duration::from_millis(100);
                if let Ok(langs) = handle.read_languages(timeout) {
                    if let Some(lang) = langs.first() {
                        if let Ok(desc) = d.device_descriptor() {
                            if let Ok(device_serial) = handle.read_serial_number_string(*lang, &desc, timeout) {
                                return device_serial == *serial;
                            }
                        }
                    }
                }
            }
            false
        })
    } else {
        let parts: Vec<&str> = target_device.unique_id.split('_').collect();
        if parts.len() >= 2 {
            let bus_str = parts[0].strip_prefix("bus").unwrap_or("");
            let addr_str = parts[1].strip_prefix("addr").unwrap_or("");
            
            if let (Ok(bus), Ok(addr)) = (bus_str.parse::<u8>(), addr_str.parse::<u8>()) {
                devices.iter().find(|d| d.bus_number() == bus && d.address() == addr)
            } else {
                None
            }
        } else {
            None
        }
    };

    let device = device
        .ok_or_else(|| anyhow!("Specific KeepKey device not found: {}", target_device.unique_id))?
        .to_owned();

    let (mut transport, _, _) = UsbTransport::new(&device, 0)
        .map_err(|e| anyhow!("Failed to initialize USB transport for device {}: {}", target_device.unique_id, e))?;

    let features_msg = transport
        .handle(Initialize::default().into())
        .map_err(|e| anyhow!("Failed to communicate with device {}: {}", target_device.unique_id, e))?;

    // Extract features from response
    let features = match features_msg {
        Message::Features(f) => f,
        _ => return Err(anyhow!("Unexpected response from device {}", target_device.unique_id)),
    };

    // Convert to our DeviceFeatures struct
    let device_features = DeviceFeatures {
        label: features.label,
        vendor: features.vendor,
        model: features.model,
        firmware_variant: features.firmware_variant,
        device_id: features.device_id,
        language: features.language,
        bootloader_mode: features.bootloader_mode.unwrap_or(false),
        version: format!(
            "{}.{}.{}",
            features.major_version.unwrap_or(0),
            features.minor_version.unwrap_or(0),
            features.patch_version.unwrap_or(0)
        ),
        firmware_hash: features.firmware_hash.map(hex::encode),
        bootloader_hash: features.bootloader_hash.clone().map(hex::encode),
        bootloader_version: features.bootloader_hash
            .map(hex::encode)
            // Bootloader version mapping removed (was device_update::bootloader_version_from_hash)
            // .and_then(|hash| bootloader_version_from_hash(&hash)),
            // Optionally just pass through the hash or leave as None
            .and_then(|hash| Some(hash)),

        initialized: features.initialized.unwrap_or(false),
        imported: features.imported,
        no_backup: features.no_backup.unwrap_or(false),
        pin_protection: features.pin_protection.unwrap_or(false),
        pin_cached: features.pin_cached.unwrap_or(false),
        passphrase_protection: features.passphrase_protection.unwrap_or(false),
        passphrase_cached: features.passphrase_cached.unwrap_or(false),
        wipe_code_protection: features.wipe_code_protection.unwrap_or(false),
        auto_lock_delay_ms: features.auto_lock_delay_ms.map(|ms| ms as u64),
        policies: features
            .policies
            .into_iter()
            .filter(|p| p.enabled())
            .map(|p| p.policy_name().to_string())
            .collect(),
    };
    log::info!("{TAG} Successfully got features for device {}: firmware v{}", target_device.unique_id, device_features.version);
    Ok(device_features)
}

/// Get device features from a connected KeepKey
///
/// This function connects to the first available KeepKey device,
/// sends a GetFeatures request, and returns a structured representation
/// of the device's features and settings.
///
/// # Returns
/// - `Ok(DeviceFeatures)` if successful with all device information
/// - `Err` if device connection fails or the device doesn't respond properly
pub fn get_device_features_impl() -> Result<DeviceFeatures> {
    // Find and connect to device
    let device = list_devices()
        .iter()
        .next()
        .ok_or_else(|| anyhow!("No KeepKey device found"))?
        .to_owned();

    let (mut transport, _, _) = UsbTransport::new(&device, 0)
        .map_err(|e| anyhow!("Failed to initialize USB transport: {}", e))?;

    // Send Initialize message and get response
    let features_msg = transport
        .handle(Initialize::default().into())
        .map_err(|e| anyhow!("Failed to communicate with device: {}", e))?;

    // Extract features from response
    let features = match features_msg {
        Message::Features(f) => f,
        _ => return Err(anyhow!("Unexpected response from device")),
    };

    // Convert to our DeviceFeatures struct
    let device_features = DeviceFeatures {
        label: features.label,
        vendor: features.vendor,
        model: features.model,
        firmware_variant: features.firmware_variant,
        device_id: features.device_id,
        language: features.language,
        bootloader_mode: features.bootloader_mode.unwrap_or(false),
        version: format!(
            "{}.{}.{}",
            features.major_version.unwrap_or(0),
            features.minor_version.unwrap_or(0),
            features.patch_version.unwrap_or(0)
        ),
        firmware_hash: features.firmware_hash.map(hex::encode),
        bootloader_hash: features.bootloader_hash.clone().map(hex::encode),
        bootloader_version: features.bootloader_hash
            .map(hex::encode)
            // Bootloader version mapping removed (was device_update::bootloader_version_from_hash)
            // .and_then(|hash| bootloader_version_from_hash(&hash)),
            // Optionally just pass through the hash or leave as None
            .and_then(|hash| Some(hash)),

        initialized: features.initialized.unwrap_or(false),
        imported: features.imported,
        no_backup: features.no_backup.unwrap_or(false),
        pin_protection: features.pin_protection.unwrap_or(false),
        pin_cached: features.pin_cached.unwrap_or(false),
        passphrase_protection: features.passphrase_protection.unwrap_or(false),
        passphrase_cached: features.passphrase_cached.unwrap_or(false),
        wipe_code_protection: features.wipe_code_protection.unwrap_or(false),
        auto_lock_delay_ms: features.auto_lock_delay_ms.map(|ms| ms as u64),
        policies: features
            .policies
            .into_iter()
            .filter(|p| p.enabled())
            .map(|p| p.policy_name().to_string())
            .collect(),
    };
    println!("{TAG} device_features: {:#?}", device_features);
    Ok(device_features)
}

/// Get device features from a specific KeepKey device with HID fallback
///
/// This function first tries USB transport, and if it fails with permission errors,
/// falls back to HID API which often has better permissions handling.
/// For older KeepKey devices (PID 0x0001), it will try HID directly as they often
/// work better with HID than with WebUSB.
///
/// # Arguments
/// * `target_device` - The specific device to get features for
///
/// # Returns
/// - `Ok(DeviceFeatures)` if successful with all device information
/// - `Err` if both USB and HID connections fail
pub fn get_device_features_with_fallback(target_device: &FriendlyUsbDevice) -> Result<DeviceFeatures> {
    log::info!("{TAG} Getting features for device with fallback: {} ({})", target_device.name, target_device.unique_id);
    
    // For older KeepKey devices (PID 0x0001), try HID directly
    if target_device.pid == 0x0001 {
        log::info!("{TAG} Detected older KeepKey device (PID 0x0001), trying HID directly");
        match get_device_features_via_hid(target_device) {
            Ok(features) => {
                log::info!("{TAG} Successfully got features via HID for older device {}", target_device.unique_id);
                return Ok(features);
            }
            Err(hid_err) => {
                log::warn!("{TAG} HID direct attempt failed for older device {}: {}, falling back to USB", 
                          target_device.unique_id, hid_err);
                // Fall through to try USB as well
            }
        }
    }
    
    // Try USB transport
    match get_device_features_for_device(target_device) {
        Ok(features) => Ok(features),
        Err(e) => {
            let error_str = e.to_string();
            
            // Check if it's a permission error or any other error
            log::warn!("{TAG} USB failed for device {}: {}, trying HID fallback", 
                      target_device.unique_id, error_str);
            
            // Try HID transport as fallback
            match get_device_features_via_hid(target_device) {
                Ok(features) => {
                    log::info!("{TAG} Successfully got features via HID for device {}", target_device.unique_id);
                    return Ok(features);
                }
                Err(hid_err) => {
                    log::error!("{TAG} HID fallback also failed for device {}: {}", target_device.unique_id, hid_err);
                    return Err(anyhow!("Failed with both USB ({}) and HID ({})", e, hid_err));
                }
            }
        }
    }
}

/// Get device features from a specific KeepKey device via HID
///
/// This function connects to the specific KeepKey device using HID API,
/// sends a GetFeatures request, and returns a structured representation
/// of the device's features and settings.
///
/// # Arguments
/// * `target_device` - The specific device to get features for
///
/// # Returns
/// - `Ok(DeviceFeatures)` if successful with all device information
/// - `Err` if device connection fails or the device doesn't respond properly
pub fn get_device_features_via_hid(target_device: &FriendlyUsbDevice) -> Result<DeviceFeatures> {
    use hidapi::HidApi;
    log::info!("{TAG} Getting features for device via HID: {} ({})", target_device.name, target_device.unique_id);

    // If serial is present, use it; otherwise, enumerate all HID KeepKey devices and try each
    let mut errors = vec![];
    if let Some(serial) = &target_device.serial_number {
        match HidTransport::new_for_device(Some(serial.as_str())) {
            Ok(mut transport) => {
                let adapter = &mut transport as &mut dyn ProtocolAdapter;
                let init_msg = Initialize::default().into();
                match adapter.handle(init_msg) {
                    Ok(features_msg) => {
                        let features = match features_msg {
                            Message::Features(f) => f,
                            _ => return Err(anyhow!("Unexpected response from device {} via HID", target_device.unique_id)),
                        };
                        let device_features = DeviceFeatures {
                            label: features.label,
                            vendor: features.vendor,
                            model: features.model,
                            firmware_variant: features.firmware_variant,
                            device_id: features.device_id,
                            language: features.language,
                            bootloader_mode: features.bootloader_mode.unwrap_or(false),
                            version: format!(
                                "{}.{}.{}",
                                features.major_version.unwrap_or(0),
                                features.minor_version.unwrap_or(0),
                                features.patch_version.unwrap_or(0)
                            ),
                            firmware_hash: features.firmware_hash.map(hex::encode),
                            bootloader_hash: features.bootloader_hash.clone().map(hex::encode),
                            bootloader_version: features.bootloader_hash
                                .map(hex::encode)
                                .and_then(|hash| Some(hash)),
                            initialized: features.initialized.unwrap_or(false),
                            imported: features.imported,
                            no_backup: features.no_backup.unwrap_or(false),
                            pin_protection: features.pin_protection.unwrap_or(false),
                            pin_cached: features.pin_cached.unwrap_or(false),
                            passphrase_protection: features.passphrase_protection.unwrap_or(false),
                            passphrase_cached: features.passphrase_cached.unwrap_or(false),
                            wipe_code_protection: features.wipe_code_protection.unwrap_or(false),
                            auto_lock_delay_ms: features.auto_lock_delay_ms.map(|ms| ms as u64),
                            policies: features
                                .policies
                                .into_iter()
                                .filter(|p| p.enabled())
                                .map(|p| p.policy_name().to_string())
                                .collect(),
                        };
                        log::info!("{TAG} Successfully got features via HID for device {}: firmware v{}", target_device.unique_id, device_features.version);
                        return Ok(device_features);
                    }
                    Err(e) => errors.push(format!("HID (serial match) error: {}", e)),
                }
            }
            Err(e) => errors.push(format!("HID (serial match) transport error: {}", e)),
        }
    } else {
        // No serial: enumerate all HID KeepKey devices and try each
        let api = HidApi::new().map_err(|e| anyhow!("Failed to initialize HID API: {}", e))?;
        let mut found_any = false;
        for device_info in api.device_list() {
            if device_info.vendor_id() == 0x2b24 && (device_info.product_id() == 0x0001 || device_info.product_id() == 0x0002) {
                found_any = true;
                match HidTransport::new_for_device(device_info.serial_number()) {
                    Ok(mut transport) => {
                        let adapter = &mut transport as &mut dyn ProtocolAdapter;
                        let init_msg = Initialize::default().into();
                        match adapter.handle(init_msg) {
                            Ok(features_msg) => {
                                let features = match features_msg {
                                    Message::Features(f) => f,
                                    _ => continue, // try next
                                };
                                let device_features = DeviceFeatures {
                                    label: features.label,
                                    vendor: features.vendor,
                                    model: features.model,
                                    firmware_variant: features.firmware_variant,
                                    device_id: features.device_id,
                                    language: features.language,
                                    bootloader_mode: features.bootloader_mode.unwrap_or(false),
                                    version: format!(
                                        "{}.{}.{}",
                                        features.major_version.unwrap_or(0),
                                        features.minor_version.unwrap_or(0),
                                        features.patch_version.unwrap_or(0)
                                    ),
                                    firmware_hash: features.firmware_hash.map(hex::encode),
                                    bootloader_hash: features.bootloader_hash.clone().map(hex::encode),
                                    bootloader_version: features.bootloader_hash
                                        .map(hex::encode)
                                        .and_then(|hash| Some(hash)),
                                    initialized: features.initialized.unwrap_or(false),
                                    imported: features.imported,
                                    no_backup: features.no_backup.unwrap_or(false),
                                    pin_protection: features.pin_protection.unwrap_or(false),
                                    pin_cached: features.pin_cached.unwrap_or(false),
                                    passphrase_protection: features.passphrase_protection.unwrap_or(false),
                                    passphrase_cached: features.passphrase_cached.unwrap_or(false),
                                    wipe_code_protection: features.wipe_code_protection.unwrap_or(false),
                                    auto_lock_delay_ms: features.auto_lock_delay_ms.map(|ms| ms as u64),
                                    policies: features
                                        .policies
                                        .into_iter()
                                        .filter(|p| p.enabled())
                                        .map(|p| p.policy_name().to_string())
                                        .collect(),
                                };
                                log::info!("{TAG} Successfully got features via HID (enumerate) for device: firmware v{}", device_features.version);
                                return Ok(device_features);
                            }
                            Err(e) => errors.push(format!("HID (enumerate) comm error for serial {:?}: {}", device_info.serial_number(), e)),
                        }
                    }
                    Err(e) => errors.push(format!("HID (enumerate) transport error for serial {:?}: {}", device_info.serial_number(), e)),
                }
            }
        }
        if !found_any {
            errors.push("No HID KeepKey devices found".to_string());
        }
    }
    Err(anyhow!("All HID attempts failed for device {}. Errors: {}", target_device.unique_id, errors.join(" | ")))
}

