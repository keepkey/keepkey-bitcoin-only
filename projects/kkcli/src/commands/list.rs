use anyhow::Result;
use clap::Args;
use colored::*;
use keepkey_rust::features::{list_connected_devices, get_device_features_with_fallback, DeviceFeatures};
use keepkey_rust::friendly_usb::FriendlyUsbDevice;
use tabled::{settings::Style, Table, Tabled};

#[derive(Args)]
pub struct ListCommand {
    /// Show only KeepKey devices
    #[arg(long)]
    keepkey_only: bool,
    
    /// Include detailed device features
    #[arg(long)]
    features: bool,
    
    /// Attempt to detect OOB bootloader mode
    #[arg(long)]
    detect_bootloader: bool,
}

#[derive(Tabled)]
struct DeviceRow {
    #[tabled(rename = "Device ID")]
    device_id: String,
    
    #[tabled(rename = "Name")]
    name: String,
    
    #[tabled(rename = "VID:PID")]
    vid_pid: String,
    
    #[tabled(rename = "Serial")]
    serial: String,
    
    #[tabled(rename = "Manufacturer")]
    manufacturer: String,
    
    #[tabled(rename = "KeepKey")]
    is_keepkey: String,
}

#[derive(Tabled)]
struct DetailedDeviceRow {
    #[tabled(rename = "Device ID")]
    device_id: String,
    
    #[tabled(rename = "Label")]
    label: String,
    
    #[tabled(rename = "Version")]
    version: String,
    
    #[tabled(rename = "Bootloader Mode")]
    bootloader_mode: String,
    
    #[tabled(rename = "Initialized")]
    initialized: String,
    
    #[tabled(rename = "Model")]
    model: String,
    
    #[tabled(rename = "Vendor")]
    vendor: String,
    
    #[tabled(rename = "Status")]
    status: String,
}

impl ListCommand {
    pub async fn execute(self) -> Result<()> {
        println!("{}", "🔍 Scanning for connected devices...".cyan());
        
        let devices = list_connected_devices();
        
        if devices.is_empty() {
            println!("{}", "❌ No devices found".red());
            return Ok(());
        }
        
        let keepkey_devices: Vec<_> = if self.keepkey_only {
            devices.into_iter().filter(|d| d.is_keepkey).collect()
        } else {
            devices
        };
        
        if keepkey_devices.is_empty() {
            if self.keepkey_only {
                println!("{}", "❌ No KeepKey devices found".red());
            } else {
                println!("{}", "❌ No devices found".red());
            }
            return Ok(());
        }
        
        println!("{} Found {} device(s)\n", "✅".green(), keepkey_devices.len());
        
        if self.features || self.detect_bootloader {
            self.show_detailed_devices(keepkey_devices).await
        } else {
            self.show_basic_devices(keepkey_devices)
        }
    }
    
    fn show_basic_devices(&self, devices: Vec<FriendlyUsbDevice>) -> Result<()> {
        let rows: Vec<DeviceRow> = devices.into_iter().map(|device| {
            DeviceRow {
                device_id: device.unique_id.clone(),
                name: device.name.clone(),
                vid_pid: format!("{:04x}:{:04x}", device.vid, device.pid),
                serial: device.serial_number.clone().unwrap_or_else(|| "N/A".to_string()),
                manufacturer: device.manufacturer.clone().unwrap_or_else(|| "Unknown".to_string()),
                is_keepkey: if device.is_keepkey { "✅".to_string() } else { "❌".to_string() },
            }
        }).collect();
        
        let table = Table::new(rows).with(Style::modern()).to_string();
        println!("{}", table);
        
        Ok(())
    }
    
    async fn show_detailed_devices(&self, devices: Vec<FriendlyUsbDevice>) -> Result<()> {
        let mut rows = Vec::new();
        
        for device in devices {
            if !device.is_keepkey {
                continue; // Only show detailed info for KeepKey devices
            }
            
            println!("{} Getting features for: {} ({})", "📡".cyan(), device.name, device.unique_id);
            
            let (features, status) = match get_device_features_with_fallback(&device) {
                Ok(features) => {
                    let status = self.evaluate_device_status(&features);
                    (Some(features), status)
                }
                Err(e) => {
                    if self.detect_bootloader {
                        println!("{} Feature fetch failed, attempting OOB bootloader detection...", "🔧".yellow());
                        match self.try_oob_bootloader_detection(&device).await {
                            Ok(features) => {
                                let status = format!("{} (OOB detected)", self.evaluate_device_status(&features));
                                (Some(features), status)
                            }
                            Err(oob_err) => {
                                let error_msg = format!("Failed: {} (OOB: {})", e, oob_err);
                                println!("{} {}", "❌".red(), error_msg);
                                (None, "Communication Error".to_string())
                            }
                        }
                    } else {
                        let error_msg = format!("Failed: {}", e);
                        println!("{} {}", "❌".red(), error_msg);
                        (None, "Communication Error".to_string())
                    }
                }
            };
            
            let row = if let Some(features) = features {
                DetailedDeviceRow {
                    device_id: device.unique_id.clone(),
                    label: features.label.clone().unwrap_or_else(|| "Unnamed".to_string()),
                    version: features.version.clone(),
                    bootloader_mode: if features.bootloader_mode { "✅ YES".green().to_string() } else { "❌ NO".to_string() },
                    initialized: if features.initialized { "✅ YES".green().to_string() } else { "⚠️ NO".yellow().to_string() },
                    model: features.model.clone().unwrap_or_else(|| "Unknown".to_string()),
                    vendor: features.vendor.clone().unwrap_or_else(|| "Unknown".to_string()),
                    status,
                }
            } else {
                DetailedDeviceRow {
                    device_id: device.unique_id.clone(),
                    label: "Unknown".to_string(),
                    version: "Unknown".to_string(),
                    bootloader_mode: "Unknown".to_string(),
                    initialized: "Unknown".to_string(),
                    model: "Unknown".to_string(),
                    vendor: "Unknown".to_string(),
                    status,
                }
            };
            
            rows.push(row);
        }
        
        if rows.is_empty() {
            println!("{}", "❌ No KeepKey devices found or all failed to communicate".red());
            return Ok(());
        }
        
        println!();
        let table = Table::new(rows).with(Style::modern()).to_string();
        println!("{}", table);
        
        Ok(())
    }
    
    fn evaluate_device_status(&self, features: &DeviceFeatures) -> String {
        if features.bootloader_mode {
            if features.version == "0.0.0" || features.version.contains("Legacy") {
                "🔧 Legacy Bootloader".red().to_string()
            } else {
                "🔧 Bootloader Mode".yellow().to_string()
            }
        } else if !features.initialized {
            "⚙️ Needs Setup".yellow().to_string()
        } else {
            "✅ Ready".green().to_string()
        }
    }
    
    async fn try_oob_bootloader_detection(&self, device: &FriendlyUsbDevice) -> Result<DeviceFeatures> {
        // Use keepkey-rust's proven OOB bootloader detection
        tokio::task::spawn_blocking({
            let device = device.clone();
            move || {
                keepkey_rust::features::get_device_features_via_hid(&device)
            }
        }).await?
    }
} 