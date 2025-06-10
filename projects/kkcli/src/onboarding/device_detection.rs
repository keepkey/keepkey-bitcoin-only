use rusb::{GlobalContext, Device as RusbDevice};
use semver::Version;

use crate::transport::{UsbTransport, ProtocolAdapter}; // Assuming these can be used or adapted
use crate::messages::{self, Message};

// KeepKey Vendor ID and Product IDs (examples, verify with actual device)
const KEEPKEY_VID: u16 = 0x2b24;
// PID that KeepKey uses for its primary interface (might be same for normal and bootloader)
const KEEPKEY_PID_INTERFACE: u16 = 0x0002; // Updater/Bootloader mode
const KEEPKEY_PID_NORMAL: u16 = 0x0001; // Normal operational mode

#[derive(Clone, Debug)]
pub enum DeviceState {
    NoDevice,
    Normal {
        fw: Option<Version>,
        bl: Option<Version>,
        device_handle: RusbDevice<GlobalContext>,
    },
    Updater {
        bl_version_from_updater: Option<Version>,
        device_handle: RusbDevice<GlobalContext>,
    },
    AccessError {
        vid: u16,
        pid: u16,
        error_message: String,
        underlying_error: String,
    },
}

// Renamed and implemented to attempt real communication and mode detection
pub fn get_device_info_and_mode(device: &RusbDevice<GlobalContext>) -> anyhow::Result<DeviceState> {
    let device_vid = device.device_descriptor()?.vendor_id();
    let device_pid = device.device_descriptor()?.product_id();
    println!("get_device_info_and_mode: Attempting to connect to VID={:04x}, PID={:04x} on interface 0...", device_vid, device_pid);

    let transport_result = UsbTransport::new(device, 0);

    let mut transport = match transport_result {
        Ok((transport, _config_desc, _handle)) => transport,
        Err(e) => {
            let err_msg = format!("Failed to create UsbTransport on interface 0: {}", e);
            println!("get_device_info_and_mode: {}", err_msg);
            return Ok(DeviceState::AccessError {
                vid: device_vid,
                pid: device_pid,
                error_message: "Failed to initialize USB transport on interface 0.".to_string(),
                underlying_error: e.to_string(),
            });
        }
    };

    // Send Initialize first
    println!("get_device_info_and_mode: UsbTransport created on interface 0. Sending Initialize message...");
    match transport.handle(messages::Initialize::default().into()) {
        Ok(initialize_response) => {
            println!("get_device_info_and_mode: Initialize response: {:?}", initialize_response);

            if let Message::Features(features_from_initialize) = initialize_response {
                println!("get_device_info_and_mode: Using Features from Initialize response directly.");
                // Process these features (copied and adapted from below GetFeatures handling)
                if features_from_initialize.bootloader_mode.unwrap_or(false) {
                    println!("get_device_info_and_mode: Device is in UPDATER mode (from Initialize).");
                    let mut bl_updater_version = None;
                    if let (Some(maj), Some(min), Some(pat)) = (features_from_initialize.major_version, features_from_initialize.minor_version, features_from_initialize.patch_version) {
                        let version_str = format!("{}.{}.{}", maj, min, pat);
                        match Version::parse(&version_str) {
                            Ok(v) => bl_updater_version = Some(v),
                            Err(e) => println!("get_device_info_and_mode: Failed to parse BL version (from Initialize) '{}': {}", version_str, e),
                        }
                    }
                    return Ok(DeviceState::Updater {
                        bl_version_from_updater: bl_updater_version,
                        device_handle: device.clone()
                    });
                } else {
                    println!("get_device_info_and_mode: Device is in NORMAL mode (from Initialize).");
                    let mut fw_version = None;
                    if let (Some(maj), Some(min), Some(pat)) = (features_from_initialize.major_version, features_from_initialize.minor_version, features_from_initialize.patch_version) {
                        let version_str = format!("{}.{}.{}", maj, min, pat);
                        match Version::parse(&version_str) {
                            Ok(v) => fw_version = Some(v),
                            Err(e) => println!("get_device_info_and_mode: Failed to parse FW version (from Initialize) '{}': {}", version_str, e),
                        }
                    }
                    return Ok(DeviceState::Normal {
                        fw: fw_version,
                        bl: None, 
                        device_handle: device.clone(),
                    });
                }
            } else {
                // Initialize response was not Features (e.g., could be Success). Proceed to send GetFeatures.
                println!("get_device_info_and_mode: Initialize response was not Features ({:?}). Proceeding to send GetFeatures.", initialize_response);
            }
        }
        Err(e_init) => {
            println!("get_device_info_and_mode: Error sending Initialize: {}. Proceeding to GetFeatures as fallback.", e_init);
        }
    }

    // If Initialize didn't return Features directly, or if Initialize errored, now send GetFeatures
    println!("get_device_info_and_mode: Sending GetFeatures message (either as primary or fallback)...");
    match transport.handle(messages::GetFeatures::default().into()) {
        Ok(response_message) => {
            if let Message::Features(features_msg) = response_message {
                println!(
                    "get_device_info_and_mode: Received Features. bootloader_mode: {:?}, version_fields: {:?}.{:?}.{:?}",
                    features_msg.bootloader_mode,
                    features_msg.major_version,
                    features_msg.minor_version,
                    features_msg.patch_version
                );

                if features_msg.bootloader_mode.unwrap_or(false) {
                    println!("get_device_info_and_mode: Device is in UPDATER mode.");
                    let mut bl_updater_version = None;
                    if let (Some(maj), Some(min), Some(pat)) = (features_msg.major_version, features_msg.minor_version, features_msg.patch_version) {
                        let version_str = format!("{}.{}.{}", maj, min, pat);
                        match Version::parse(&version_str) {
                            Ok(v) => bl_updater_version = Some(v),
                            Err(e) => println!("get_device_info_and_mode: Failed to parse bootloader version from updater mode '{}': {}", version_str, e),
                        }
                    }
                    return Ok(DeviceState::Updater {
                        bl_version_from_updater: bl_updater_version,
                        device_handle: device.clone()
                    });
                } else {
                    println!("get_device_info_and_mode: Device is in NORMAL mode.");
                    let mut fw_version = None;
                    if let (Some(maj), Some(min), Some(pat)) = (features_msg.major_version, features_msg.minor_version, features_msg.patch_version) {
                        let version_str = format!("{}.{}.{}", maj, min, pat);
                        match Version::parse(&version_str) {
                            Ok(v) => fw_version = Some(v),
                            Err(e) => println!("get_device_info_and_mode: Failed to parse firmware version '{}': {}", version_str, e),
                        }
                    }
                    return Ok(DeviceState::Normal {
                        fw: fw_version,
                        bl: None, 
                        device_handle: device.clone(),
                    });
                }
            } else {
                println!("get_device_info_and_mode: Did not receive Features message after Initialize. Received: {:?}", response_message);
                return Ok(DeviceState::AccessError {
                    vid: device_vid,
                    pid: device_pid,
                    error_message: "Communication failed: Did not receive Features message after Initialize.".to_string(),
                    underlying_error: format!("Unexpected GetFeatures response: {:?}", response_message)
                });
            }
        }
        Err(e_get_features) => {
            println!("get_device_info_and_mode: Error calling GetFeatures after Initialize: {}.", e_get_features);
            return Ok(DeviceState::AccessError {
                vid: device_vid,
                pid: device_pid,
                error_message: "Communication failed: Error calling GetFeatures after Initialize.".to_string(),
                underlying_error: e_get_features.to_string(),
            });
        }
    }
}

pub fn detect_device_impl() -> anyhow::Result<DeviceState> {
    println!("detect_device_impl: Scanning for USB devices...");
    match rusb::devices() {
        Ok(devices) => {
            for device_candidate in devices.iter() {
                let device_desc = match device_candidate.device_descriptor() {
                    Ok(d) => d,
                    Err(_e) => { 
                        continue;
                    }
                };

                if device_desc.vendor_id() == KEEPKEY_VID {
                    println!(
                        "detect_device_impl: Found device with KeepKey VID: {:04x}, PID: {:04x}",
                        device_desc.vendor_id(),
                        device_desc.product_id()
                    );
                    if device_desc.product_id() == KEEPKEY_PID_INTERFACE || device_desc.product_id() == KEEPKEY_PID_NORMAL {
                        println!("detect_device_impl: KeepKey VID and a recognized PID matched. Calling get_device_info_and_mode.");
                        return get_device_info_and_mode(&device_candidate);
                    } else {
                        println!("detect_device_impl: KeepKey VID matched, but PID {:04x} does not match KEEPKEY_PID_INTERFACE {:04x} or KEEPKEY_PID_NORMAL {:04x}.", device_desc.product_id(), KEEPKEY_PID_INTERFACE, KEEPKEY_PID_NORMAL);
                    }
                }
            }
        }
        Err(e) => {
            println!("detect_device_impl: Failed to list rusb devices: {}", e);
            return Err(anyhow::anyhow!("Failed to list rusb devices: {}", e));
        }
    }
    Ok(DeviceState::NoDevice)
} 