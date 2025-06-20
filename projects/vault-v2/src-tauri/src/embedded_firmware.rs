use std::collections::HashMap;

/// Embedded firmware files using include_bytes! for distribution
/// This ensures firmware files are bundled directly into the executable
/// Only includes the latest versions to minimize binary size

// Latest bootloader binary
static BOOTLOADER_LATEST: &[u8] = include_bytes!("../firmware/bl_v2.1.4/blupdater.bin");

// Latest firmware binary
static FIRMWARE_LATEST: &[u8] = include_bytes!("../firmware/v7.10.0/firmware.keepkey.bin");

/// Get embedded bootloader binary by version
pub fn get_bootloader_bytes(version: &str) -> Option<&'static [u8]> {
    match version {
        "2.1.4" => Some(BOOTLOADER_LATEST),
        _ => None,
    }
}

/// Get embedded firmware binary by version
pub fn get_firmware_bytes(version: &str) -> Option<&'static [u8]> {
    match version {
        "7.10.0" => Some(FIRMWARE_LATEST),
        _ => None,
    }
}

/// Get all available bootloader versions (embedded)
pub fn get_available_bootloader_versions() -> Vec<&'static str> {
    vec!["2.1.4"]
}

/// Get all available firmware versions (embedded)
pub fn get_available_firmware_versions() -> Vec<&'static str> {
    vec!["7.10.0"]
}

/// Get the latest bootloader version
pub fn get_latest_bootloader_version() -> &'static str {
    "2.1.4"
}

/// Get the latest firmware version  
pub fn get_latest_firmware_version() -> &'static str {
    "7.10.0"
}

/// Build a mapping of version to bootloader bytes for debugging
pub fn get_bootloader_info() -> HashMap<&'static str, usize> {
    let mut info = HashMap::new();
    
    if let Some(bytes) = get_bootloader_bytes("2.1.4") {
        info.insert("2.1.4", bytes.len());
    }
    
    info
}

/// Build a mapping of version to firmware bytes for debugging
pub fn get_firmware_info() -> HashMap<&'static str, usize> {
    let mut info = HashMap::new();
    
    if let Some(bytes) = get_firmware_bytes("7.10.0") {
        info.insert("7.10.0", bytes.len());
    }
    
    info
} 