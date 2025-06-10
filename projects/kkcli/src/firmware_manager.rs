use rust_embed::RustEmbed;
use serde::Deserialize;
use semver::Version;
use anyhow::{Result, anyhow};
use url::Url; // For joining URLs

#[derive(RustEmbed)]
#[folder = "firmware/"]
// If your manifest.json contains URLs like "keepkey_v7.3.0.bin", 
// and these files are directly under "firmware/", then no prefix or prefix="firmware/" is fine.
// If prefix="firmware/", then FirmwareAssets::get("keepkey_v7.3.0.bin") would look for "firmware/keepkey_v7.3.0.bin".
// If no prefix, FirmwareAssets::get("firmware/keepkey_v7.3.0.bin") is needed.
// Let's stick to no prefix for now, meaning paths in manifest should be like "firmware/X.bin" or the get call adjusted.
struct FirmwareAssets;

#[derive(Debug, Clone, Copy, Deserialize)] // Deserialize on UrlType might be optional if always set manually, but harmless.
pub enum UrlType {
    EmbeddedRelative,
    HttpAbsolute,
    HttpRelative, // Relative to a remote base URL
}

impl Default for UrlType {
    fn default() -> Self {
        UrlType::HttpAbsolute // Default value, will be overwritten by manual logic.
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct FirmwareInfo {
    pub version: String,
    pub url: String,
    pub hash: String,
    #[serde(skip_deserializing, default)] // Use Default for this skipped field
    pub url_type: UrlType, 
}

#[derive(Deserialize, Debug, Clone)]
pub struct LatestFirmware {
    pub firmware: FirmwareInfo, // Renamed from main to firmware based on new releases.json
    pub bootloader: FirmwareInfo,
    // pub updater: Option<UpdaterInfo>, // Removed for now as it's not used
}

#[derive(Deserialize, Debug)]
struct ManifestFile {
    pub latest: LatestFirmware,
    // It seems keepkey-desktop/develop/firmware/releases.json has "latest" and "beta"
    // but also top-level "hashes", "links", "strings".
    // For now, we only care about "latest". If we need "beta" or others later,
    // this struct (and LatestFirmware if beta structure is different) will need adjustment.
}

const EMBEDDED_LTS_MANIFEST_PATH: &str = "lts_manifest.json"; // Path within firmware/ folder
// URL for fetching the latest manifest from the network
const REMOTE_MANIFEST_URL: &str = "https://raw.githubusercontent.com/keepkey/keepkey-desktop/master/firmware/releases.json";

pub struct FirmwareManager {
    // Releases becomes an Option to handle cases where no manifest (LTS or remote) can be loaded.
    releases: Option<LatestFirmware>, 
    remote_base_url: Option<String>, // To store the base for relative remote URLs
}

impl FirmwareManager {
    pub fn new() -> Result<Self> {
        let mut loaded_releases: Option<LatestFirmware> = None;
        let mut current_remote_base_url: Option<String> = None;

        // 1. Try to load embedded LTS manifest. Non-critical if not found.
        println!("Attempting to load embedded LTS manifest from: firmware/{}", EMBEDDED_LTS_MANIFEST_PATH);
        if let Some(lts_manifest_asset) = FirmwareAssets::get(EMBEDDED_LTS_MANIFEST_PATH) {
            if let Ok(lts_manifest_data) = std::str::from_utf8(lts_manifest_asset.data.as_ref()) {
                match serde_json::from_str::<ManifestFile>(lts_manifest_data) {
                    Ok(mut lts_parsed_manifest) => {
                        // For embedded, URLs are relative to firmware/ folder if not absolute HTTP
                        Self::set_url_type_for_manifest_entries(&mut lts_parsed_manifest.latest, UrlType::EmbeddedRelative, None);
                        loaded_releases = Some(lts_parsed_manifest.latest);
                        println!("Successfully loaded embedded LTS manifest. Main FW: {}, BL: {}", 
                            loaded_releases.as_ref().unwrap().firmware.version, 
                            loaded_releases.as_ref().unwrap().bootloader.version);
                    },
                    Err(e) => {
                        println!("Warning: Failed to parse embedded LTS manifest (firmware/{}): {}. Continuing without embedded LTS.", EMBEDDED_LTS_MANIFEST_PATH, e);
                    }
                }
            } else {
                println!("Warning: Embedded LTS manifest (firmware/{}) content is not valid UTF-8. Continuing without embedded LTS.", EMBEDDED_LTS_MANIFEST_PATH);
            }
        } else {
            println!("Warning: Embedded LTS manifest (firmware/{}) not found. Continuing without embedded LTS.", EMBEDDED_LTS_MANIFEST_PATH);
        }

        // 2. Try fetching remote manifest. This can overwrite LTS if successful.
        println!("Attempting to fetch remote manifest from: {}", REMOTE_MANIFEST_URL);
        
        // Use a separate thread to avoid nested runtime panic
        let remote_result = std::thread::spawn(move || {
            reqwest::blocking::get(REMOTE_MANIFEST_URL)
        }).join().unwrap();
        
        match remote_result {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<ManifestFile>() {
                        Ok(mut remote_parsed_manifest) => {
                            println!("Successfully fetched and parsed remote manifest. Main FW: {}, BL: {}", 
                                remote_parsed_manifest.latest.firmware.version, remote_parsed_manifest.latest.bootloader.version);
                            
                            // Determine base URL for relative paths in remote manifest
                            if let Ok(parsed_remote_url) = Url::parse(REMOTE_MANIFEST_URL) {
                                if let Some(mut base_path_segments) = parsed_remote_url.path_segments().map(|c| c.collect::<Vec<_>>()) {
                                    if !base_path_segments.is_empty() {
                                        base_path_segments.pop(); // Remove the filename (releases.json)
                                        let mut base_url_for_relative = parsed_remote_url.clone();
                                        let path_prefix = base_path_segments.join("/");
                                        base_url_for_relative.set_path(if path_prefix.is_empty() { "/" } else { &path_prefix });
                                        let base_url_str = format!("{}/", base_url_for_relative.as_str().trim_end_matches('/'));
                                        current_remote_base_url = Some(base_url_str);
                                        Self::set_url_type_for_manifest_entries(&mut remote_parsed_manifest.latest, UrlType::HttpRelative, current_remote_base_url.as_deref());
                                    }
                                }
                            }
                            loaded_releases = Some(remote_parsed_manifest.latest); // Update with remote info
                        }
                        Err(e) => {
                            let msg_prefix = if loaded_releases.is_some() { "Using embedded LTS versions." } else { "No firmware versions available." };
                            println!("Failed to parse remote manifest: {}. {}", e, msg_prefix);
                        }
                    }
                } else {
                    let msg_prefix = if loaded_releases.is_some() { "Using embedded LTS versions." } else { "No firmware versions available." };
                    println!(
                        "Failed to fetch remote manifest: HTTP {}. {}",
                        response.status(), msg_prefix
                    );
                }
            }
            Err(e) => {
                let msg_prefix = if loaded_releases.is_some() { "Using embedded LTS versions." } else { "No firmware versions available." };
                println!("Network error fetching remote manifest: {}. {}", e, msg_prefix);
            }
        }
        
        if loaded_releases.is_none() {
            println!("Warning: No firmware manifest (LTS or remote) could be loaded. Firmware functionality will be unavailable.");
        }

        Ok(Self { releases: loaded_releases, remote_base_url: current_remote_base_url })
    }

    // Helper to set UrlType for firmware and bootloader entries
    fn set_url_type_for_manifest_entries(latest_fw: &mut LatestFirmware, default_type: UrlType, remote_base: Option<&str>) {
        // Firmware
        if latest_fw.firmware.url.starts_with("http://") || latest_fw.firmware.url.starts_with("https://") {
            latest_fw.firmware.url_type = UrlType::HttpAbsolute;
        } else if remote_base.is_some() { // If remote_base is provided, it implies HttpRelative for non-absolute URLs
            latest_fw.firmware.url_type = UrlType::HttpRelative;
        } else {
            latest_fw.firmware.url_type = default_type; // For embedded, typically EmbeddedRelative
        }
        // Bootloader
        if latest_fw.bootloader.url.starts_with("http://") || latest_fw.bootloader.url.starts_with("https://") {
            latest_fw.bootloader.url_type = UrlType::HttpAbsolute;
        } else if remote_base.is_some() {
            latest_fw.bootloader.url_type = UrlType::HttpRelative;
        } else {
            latest_fw.bootloader.url_type = default_type;
        }
    }

    pub fn get_latest_firmware_version(&self) -> Result<Version> {
        self.releases
            .as_ref()
            .and_then(|r| Version::parse(&r.firmware.version).ok())
            .ok_or_else(|| anyhow!("Latest firmware version not available or invalid"))
    }

    pub fn get_latest_bootloader_version(&self) -> Result<Version> {
        self.releases
            .as_ref()
            .and_then(|r| Version::parse(&r.bootloader.version).ok())
            .ok_or_else(|| anyhow!("Latest bootloader version not available or invalid"))
    }

    pub fn get_firmware_bytes(&self, info: &FirmwareInfo) -> Result<Vec<u8>> {
        match info.url_type {
            UrlType::EmbeddedRelative => {
                println!("Attempting to load embedded firmware asset: firmware/{}", info.url);
                FirmwareAssets::get(&info.url) // Assumes info.url is like "X.bin", not "firmware/X.bin"
                    .map(|asset| asset.data.into_owned())
                    .ok_or_else(|| anyhow!("Embedded firmware asset not found: firmware/{}", info.url))
            }
            UrlType::HttpAbsolute => {
                println!("Downloading firmware from absolute URL: {}", info.url);
                let url = info.url.clone();
                let response = std::thread::spawn(move || {
                    reqwest::blocking::get(&url)
                }).join().unwrap()?;
                if response.status().is_success() {
                    Ok(response.bytes()?.to_vec())
                } else {
                    Err(anyhow!("Failed to download firmware from {}: HTTP {}", info.url, response.status()))
                }
            }
            UrlType::HttpRelative => {
                if let Some(base_url_str) = &self.remote_base_url {
                    match Url::parse(base_url_str) {
                        Ok(base_url) => {
                            match base_url.join(&info.url) {
                                Ok(full_url) => {
                                    println!("Downloading firmware from constructed URL: {}", full_url.as_str());
                                    let url_str = full_url.as_str().to_string();
                                    let response = std::thread::spawn(move || {
                                        reqwest::blocking::get(&url_str)
                                    }).join().unwrap()?;
                                    if response.status().is_success() {
                                        Ok(response.bytes()?.to_vec())
                                    } else {
                                        Err(anyhow!("Failed to download firmware from {}: HTTP {}", full_url.as_str(), response.status()))
                                    }
                                }
                                Err(e) => Err(anyhow!("Failed to join base URL '{}' with relative path '{}': {}", base_url_str, info.url, e)),
                            }
                        }
                        Err(e) => Err(anyhow!("Invalid remote base URL stored '{}': {}", base_url_str, e)),
                    }
                } else {
                    Err(anyhow!("Cannot fetch relative HTTP URL '{}': remote base URL is not set.", info.url))
                }
            }
        }
    }
    
    pub fn get_latest_firmware_bytes(&self) -> Result<Vec<u8>> {
        let info = self.releases.as_ref()
            .ok_or_else(|| anyhow!("Firmware information not available to get bytes."))?.firmware.clone();
        self.get_firmware_bytes(&info)
    }

    pub fn get_latest_bootloader_bytes(&self) -> Result<Vec<u8>> {
        let info = self.releases.as_ref()
            .ok_or_else(|| anyhow!("Firmware information not available to get bytes."))?.bootloader.clone();
        self.get_firmware_bytes(&info)
    }

    pub fn get_latest_firmware_info(&self) -> Option<&FirmwareInfo> {
        self.releases.as_ref().map(|r| &r.firmware)
    }

    pub fn get_latest_bootloader_info(&self) -> Option<&FirmwareInfo> {
        self.releases.as_ref().map(|r| &r.bootloader)
    }
} 