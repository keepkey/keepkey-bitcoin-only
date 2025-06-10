# KeepKey Firmware Update Procedure

Based on the GitHub repository structure you provided (https://github.com/keepkey/keepkey-desktop/tree/master/firmware), if a manifest.json is hosted there, a more direct URL to access its raw content would typically be:
https://raw.githubusercontent.com/keepkey/keepkey-desktop/master/firmware/manifest.json
I will update the document to use this more specific and conventional URL for accessing raw files from a GitHub repository. I'll also add a note emphasizing that this URL is based on common practice and should be verified with official KeepKey documentation or sources.


## 1. Introduction

Firmware is the software that runs directly on the KeepKey hardware. Keeping it up-to-date is crucial for security, new features, and bug fixes. This document outlines the general procedure for how a client application (like a desktop app or CLI tool) can manage KeepKey firmware updates.

The process involves checking for the latest firmware versions from a trusted source, comparing with the device's current version, securely downloading the new firmware, verifying its integrity, and then guiding the user through the installation process.

## 2. Overview of the Update Process

The firmware update process can be broken down into these main phases:

1.  **Discovery:** Determine the latest available firmware versions.
2.  **Comparison:** Compare latest versions with the device's current firmware.
3.  **Download & Verification:** Securely download and verify the integrity of the new firmware binary.
4.  **Installation:** Put the KeepKey into bootloader mode and upload the new firmware.
5.  **Completion:** The device verifies and applies the update, then reboots.

## 3. Key Components

### 3.1. `manifest.json`

A central piece of the update mechanism is a `manifest.json` file, typically hosted on a secure server (e.g., `https://github.com/keepkey/keepkey-desktop/tree/master/firmware/manifest.json`). This file contains metadata about available firmware versions.

A likely structure for `manifest.json`:

```json
{
  "firmwares": [
    {
      "type": "main", // "main" or "updater"
      "version": "7.3.0", // Semantic version string
      "url": "https://path/to/keepkey_main_v7.3.0.bin",
      "sha256sum": "abcdef123456...", // SHA256 hash of the .bin file
      "notesUrl": "https://path/to/release_notes_v7.3.0.md", // Optional
      "minBootloaderVersion": "1.1.0" // Optional minimum bootloader required
    },
    {
      "type": "updater",
      "version": "1.2.0",
      "url": "https://path/to/keepkey_updater_v1.2.0.bin",
      "sha256sum": "fedcba654321...",
      "notesUrl": "https://path/to/updater_release_notes_v1.2.0.md"
    }
    // ... other firmware versions
  ],
  "latest": { // Optional: quick lookup for the newest of each type
    "main": "7.3.0",
    "updater": "1.2.0"
  }
}
```

### 3.2. Firmware Binaries (`.bin` files)

These are the actual firmware images (e.g., `keepkey_main_vX.Y.Z.bin`, `keepkey_updater_vX.Y.Z.bin`) that get flashed onto the device. They are typically stored alongside the `manifest.json` or at URLs specified within it.

### 3.3. KeepKey Device

The KeepKey device has different operational modes:
*   **Normal Mode:** Standard operation.
*   **Bootloader Mode:** A special mode for firmware updates. The device's existing firmware can typically command it to reboot into bootloader mode, or it might be entered by holding a button during power-on.

### 3.4. Client Application

This is the software (desktop app, web app, CLI) that orchestrates the update. It communicates with both the firmware source (server) and the KeepKey device.

## 4. Detailed Steps

### 4.1. Fetch Latest Firmware Manifest

*   The client application periodically (or on user request) fetches `manifest.json` from its known secure URL.
*   This MUST be done over HTTPS to prevent tampering.
*   The JSON is parsed to extract information about available firmware versions.

### 4.2. Determine Current Device Firmware Version

*   The client application communicates with the connected KeepKey to get its current firmware versions (main firmware and bootloader version).
*   This is typically done using a command like `GetFeatures`, which returns a `Features` message containing `major_version`, `minor_version`, `patch_version`, and `bootloader_hash` (which can imply bootloader version or be compared directly if the manifest provides hashes for bootloaders).

### 4.3. Compare Versions and Decide on Update

*   The client compares the device's current versions with the latest versions listed in the `manifest.json`.
*   Semantic versioning comparison (e.g., using a library) should be used to determine if an update is available (e.g., `7.3.0` > `7.2.1`).
*   The client might also check for bootloader updates. Sometimes a main firmware update requires a newer bootloader. The `minBootloaderVersion` field in the manifest can assist with this.
*   If an update is available, the user is typically prompted to proceed.

### 4.4. Download Firmware Binary

*   If the user agrees to update, the client downloads the appropriate `.bin` file from the URL specified in the `manifest.json` for the target version.
*   This download MUST also occur over HTTPS.
*   The binary should be stored temporarily.

### 4.5. Verify Firmware Integrity (Hashing) - CRITICAL STEP

*   After downloading, the client computes the SHA256 hash of the downloaded `.bin` file.
*   This computed hash is then compared against the `sha256sum` provided in the `manifest.json` for that specific firmware version.
*   **If the hashes do not match, the firmware binary is considered compromised or corrupted, and the update process MUST be aborted.** This step is vital to prevent flashing malicious or damaged firmware.

### 4.6. Prepare Device for Update (Enter Bootloader Mode)

*   The client application instructs the user on how to put their KeepKey into bootloader mode. This might involve:
    *   Sending a specific command to the device (e.g., `BootloaderMode` if supported by the current firmware).
    *   Guiding the user to disconnect, hold a button, and reconnect the device.
*   Once in bootloader mode, the device will typically present a different USB interface.

### 4.7. Upload Firmware to Device

*   The client application communicates with the KeepKey in bootloader mode.
*   The firmware update protocol usually involves:
    1.  **Erase Firmware:** Sending a command like `FirmwareErase` (or similar) to prepare the device's flash memory. The device may do this automatically.
    2.  **Upload Firmware:** Sending the firmware binary in chunks using a command like `FirmwareUpload`. Each chunk might be acknowledged by the device.
*   The KeepKey bootloader itself performs cryptographic signature verification on the received firmware before committing it to flash. This ensures the firmware is authentically signed by KeepKey/ShapeShift.

### 4.8. Finalize Update and Reboot

*   After the entire binary is uploaded and verified by the bootloader, the device flashes the new firmware.
*   Upon successful flashing, the KeepKey automatically reboots into the new firmware.
*   The client application should confirm the new firmware version by re-connecting and calling `GetFeatures` again.

## 5. Security Considerations

*   **HTTPS:** All communications for fetching the manifest and firmware binaries must use HTTPS to prevent man-in-the-middle attacks.
*   **Manifest Integrity:** While HTTPS helps, the source of the manifest itself should be highly trusted.
*   **Binary Integrity (SHA256):** Client-side hash verification is a crucial defense against corrupted or tampered downloads.
*   **Device-Side Signature Verification:** The KeepKey bootloader's signature check is the ultimate gatekeeper, ensuring only officially signed firmware can run. This protects against malicious firmware even if the download/verification process on the client was compromised.
*   **User Prompts:** Clearly inform the user about the update, version numbers, and any risks. Obtain explicit consent before proceeding.

## 6. Conceptual Code Examples (Rust-like)

```rust
// Note: This is conceptual pseudo-code. Actual implementation details will vary.
// Assumes use of libraries like serde (for JSON), reqwest (for HTTP), sha2 (for hashing).

use serde::{Deserialize, Serialize};
use semver::Version; // For version comparison

#[derive(Serialize, Deserialize, Debug, Clone)]
struct FirmwareEntry {
    #[serde(rename = "type")]
    firmware_type: String, // "main" or "updater"
    version: String,
    url: String,
    sha256sum: String,
    #[serde(rename = "notesUrl")]
    notes_url: Option<String>,
    #[serde(rename = "minBootloaderVersion")]
    min_bootloader_version: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct FirmwareManifest {
    firmwares: Vec<FirmwareEntry>,
    latest: Option<std::collections::HashMap<String, String>>,
}

// --- Assume KeepKey communication functions exist ---
// fn get_device_features() -> Result<DeviceFeatures, Error>;
// struct DeviceFeatures { main_version: Version, bootloader_version: Version }
// fn trigger_bootloader_mode() -> Result<(), Error>;
// fn upload_firmware_to_bootloader(firmware_data: &[u8]) -> Result<(), Error>;
// ----------------------------------------------------

async fn fetch_manifest(url: &str) -> Result<FirmwareManifest, String> {
    // In a real app, use a robust HTTP client like reqwest
    // For example:
    // let response = reqwest::get(url).await.map_err(|e| e.to_string())?;
    // let manifest: FirmwareManifest = response.json().await.map_err(|e| e.to_string())?;
    // Ok(manifest)
    Err(format!("Fetching from {} not implemented in pseudo-code", url))
}

fn find_latest_firmware<'a>(
    manifest: &'a FirmwareManifest,
    firmware_type: &str,
) -> Option<&'a FirmwareEntry> {
    manifest
        .firmwares
        .iter()
        .filter(|entry| entry.firmware_type == firmware_type)
        .max_by_key(|entry| Version::parse(&entry.version).ok())
}

async fn check_for_updates(
    manifest_url: &str,
    current_main_version_str: &str,
    // current_bootloader_version_str: &str,
) -> Result<Option<FirmwareEntry>, String> {
    let manifest = fetch_manifest(manifest_url).await?;
    let current_main_version = Version::parse(current_main_version_str)
        .map_err(|e| format!("Invalid current main version: {}", e))?;

    if let Some(latest_main_fw) = find_latest_firmware(&manifest, "main") {
        let latest_version = Version::parse(&latest_main_fw.version)
            .map_err(|e| format!("Invalid latest main version in manifest: {}", e))?;
        if latest_version > current_main_version {
            // Optionally, check minBootloaderVersion here against current_bootloader_version
            return Ok(Some(latest_main_fw.clone()));
        }
    }
    Ok(None)
}

async fn download_and_verify_firmware(firmware_entry: &FirmwareEntry) -> Result<Vec<u8>, String> {
    // Download firmware_entry.url using an HTTP client
    // let firmware_data = reqwest::get(&firmware_entry.url).await?
    //    .bytes().await?.to_vec();
    let firmware_data: Vec<u8> = Vec::new(); // Placeholder for downloaded data

    // Verify SHA256 hash
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(&firmware_data);
    let computed_hash = format!("{:x}", hasher.finalize());

    if computed_hash.to_lowercase() == firmware_entry.sha256sum.to_lowercase() {
        Ok(firmware_data)
    } else {
        Err(format!(
            "Hash mismatch! Expected: {}, Computed: {}",
            firmware_entry.sha256sum, computed_hash
        ))
    }
}

// Example usage flow (conceptual)
// async fn perform_update_if_needed() {
//     let device_features = get_device_features().unwrap(); // Get from device
//     let current_version_str = format!("{}.{}.{}",
//         device_features.main_major,
//         device_features.main_minor,
//         device_features.main_patch);
//
//     match check_for_updates("https://example.com/manifest.json", &current_version_str).await {
//         Ok(Some(update_info)) => {
//             println!("Update available: {} -> {}", current_version_str, update_info.version);
//             // Prompt user to update...
//             match download_and_verify_firmware(&update_info).await {
//                 Ok(firmware_binary) => {
//                     println!("Firmware downloaded and verified. Preparing to update device.");
//                     // trigger_bootloader_mode().unwrap();
//                     // Wait for device to re-enumerate in bootloader mode...
//                     // upload_firmware_to_bootloader(&firmware_binary).unwrap();
//                     println!("Update process initiated.");
//                 }
//                 Err(e) => eprintln!("Error downloading/verifying firmware: {}", e),
//             }
//         }
//         Ok(None) => println!("Firmware is up to date."),
//         Err(e) => eprintln!("Error checking for updates: {}", e),
//     }
// }

```

## 7. Error Handling and Rollback

*   **Error Handling:** The client application must gracefully handle errors at each step (network issues, hash mismatches, device communication failures, etc.).
*   **Rollback:** True firmware rollback is typically not a client-side managed process. If a firmware update fails critically on the device, the bootloader might attempt to revert to a previous known-good firmware if such a mechanism exists on the device, or it might remain in bootloader mode requiring a manual re-flash. The client's main role is to prevent bad firmware from being sent in the first place via verification.

This document provides a high-level guide. Specific KeepKey models or firmware versions might have variations in their update protocols. Always refer to official KeepKey documentation if available for precise details. 