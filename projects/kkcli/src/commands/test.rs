use anyhow::Result;
use clap::Args;
use colored::*;
use keepkey_rust::features::{list_connected_devices, get_device_features_with_fallback};
use keepkey_rust::friendly_usb::FriendlyUsbDevice;

#[derive(Args)]
pub struct TestCommand {
    /// Specific device ID to test (if not provided, tests all KeepKey devices)
    #[arg(long)]
    device_id: Option<String>,
    
    /// Test bootloader detection specifically
    #[arg(long)]
    bootloader_test: bool,
    
    /// Test both USB and HID transports
    #[arg(long)]
    transport_test: bool,
}

impl TestCommand {
    pub async fn execute(self) -> Result<()> {
        println!("{}", "🧪 KeepKey Device Communication Test".bright_cyan().bold());
        println!();
        
        let devices = list_connected_devices();
        let keepkey_devices: Vec<_> = devices.into_iter().filter(|d| d.is_keepkey).collect();
        
        if keepkey_devices.is_empty() {
            println!("{}", "❌ No KeepKey devices found".red());
            return Ok(());
        }
        
        let test_devices = if let Some(device_id) = &self.device_id {
            keepkey_devices.into_iter()
                .filter(|d| d.unique_id == *device_id)
                .collect()
        } else {
            keepkey_devices
        };
        
        if test_devices.is_empty() {
            println!("{}", format!("❌ Device {} not found", self.device_id.unwrap_or_default()).red());
            return Ok(());
        }
        
        for device in test_devices {
            self.test_device(&device).await?;
        }
        
        Ok(())
    }
    
    async fn test_device(&self, device: &FriendlyUsbDevice) -> Result<()> {
        println!("{}", format!("🔍 Testing device: {} ({})", device.name, device.unique_id).cyan().bold());
        println!("{}", format!("   VID:PID: {:04x}:{:04x}", device.vid, device.pid).dimmed());
        println!("{}", format!("   Serial: {}", device.serial_number.as_deref().unwrap_or("N/A")).dimmed());
        println!();
        
        // Test 1: Basic device communication with fallback
        println!("{}", "📡 Test 1: Basic device communication (USB → HID fallback)".yellow());
        match get_device_features_with_fallback(device) {
            Ok(features) => {
                println!("{}", "   ✅ Communication successful".green());
                self.print_features(&features);
            }
            Err(e) => {
                println!("{}", format!("   ❌ Communication failed: {}", e).red());
                
                // If bootloader test is enabled, try direct HID
                if self.bootloader_test {
                    println!("{}", "   🔧 Attempting OOB bootloader detection...".yellow());
                    match self.test_oob_bootloader_detection(device).await {
                        Ok(features) => {
                            println!("{}", "   ✅ OOB bootloader detection successful".green());
                            self.print_features(&features);
                        }
                        Err(oob_err) => {
                            println!("{}", format!("   ❌ OOB bootloader detection failed: {}", oob_err).red());
                        }
                    }
                }
            }
        }
        
        // Test 2: Transport-specific tests
        if self.transport_test {
            println!();
            self.test_transports(device).await?;
        }
        
        println!();
        println!("{}", "─".repeat(80).dimmed());
        println!();
        
        Ok(())
    }
    
    fn print_features(&self, features: &keepkey_rust::features::DeviceFeatures) {
        println!("{}", "   📋 Device Features:".blue());
        println!("{}", format!("      Label: {}", features.label.as_deref().unwrap_or("Unnamed")).dimmed());
        println!("{}", format!("      Version: {}", features.version).dimmed());
        println!("{}", format!("      Bootloader Mode: {}", if features.bootloader_mode { "YES".red() } else { "NO".green() }).dimmed());
        println!("{}", format!("      Initialized: {}", if features.initialized { "YES".green() } else { "NO".yellow() }).dimmed());
        println!("{}", format!("      Model: {}", features.model.as_deref().unwrap_or("Unknown")).dimmed());
        println!("{}", format!("      Vendor: {}", features.vendor.as_deref().unwrap_or("Unknown")).dimmed());
        
        if let Some(device_id) = &features.device_id {
            println!("{}", format!("      Device ID: {}", device_id).dimmed());
        }
        
        // Additional bootloader-specific info
        if features.bootloader_mode {
            println!("{}", "      🔧 BOOTLOADER MODE DETECTED".red().bold());
            if features.version == "0.0.0" || features.version.contains("Legacy") {
                println!("{}", "      ⚠️  Legacy bootloader - needs update".yellow());
            }
        }
    }
    
    async fn test_oob_bootloader_detection(&self, device: &FriendlyUsbDevice) -> Result<keepkey_rust::features::DeviceFeatures> {
        tokio::task::spawn_blocking({
            let device = device.clone();
            move || {
                keepkey_rust::features::get_device_features_via_hid(&device)
            }
        }).await?
    }
    
    async fn test_transports(&self, device: &FriendlyUsbDevice) -> Result<()> {
        println!("{}", "🚌 Test 2: Transport-specific tests".yellow());
        
        // Test USB transport
        println!("{}", "   🔌 Testing USB transport...".cyan());
        match self.test_usb_transport(device).await {
            Ok(features) => {
                println!("{}", "      ✅ USB transport successful".green());
                self.print_transport_features("USB", &features);
            }
            Err(e) => {
                println!("{}", format!("      ❌ USB transport failed: {}", e).red());
            }
        }
        
        // Test HID transport
        println!("{}", "   🎛️  Testing HID transport...".cyan());
        match self.test_hid_transport(device).await {
            Ok(features) => {
                println!("{}", "      ✅ HID transport successful".green());
                self.print_transport_features("HID", &features);
            }
            Err(e) => {
                println!("{}", format!("      ❌ HID transport failed: {}", e).red());
            }
        }
        
        Ok(())
    }
    
    async fn test_usb_transport(&self, device: &FriendlyUsbDevice) -> Result<keepkey_rust::features::DeviceFeatures> {
        tokio::task::spawn_blocking({
            let device = device.clone();
            move || {
                keepkey_rust::features::get_device_features_for_device(&device)
            }
        }).await?
    }
    
    async fn test_hid_transport(&self, device: &FriendlyUsbDevice) -> Result<keepkey_rust::features::DeviceFeatures> {
        tokio::task::spawn_blocking({
            let device = device.clone();
            move || {
                keepkey_rust::features::get_device_features_via_hid(&device)
            }
        }).await?
    }
    
    fn print_transport_features(&self, transport_name: &str, features: &keepkey_rust::features::DeviceFeatures) {
        println!("{}", format!("      📋 {} Features:", transport_name).blue());
        println!("{}", format!("         Version: {}", features.version).dimmed());
        println!("{}", format!("         Bootloader: {}", if features.bootloader_mode { "YES".red() } else { "NO".green() }).dimmed());
        println!("{}", format!("         Initialized: {}", if features.initialized { "YES".green() } else { "NO".yellow() }).dimmed());
    }
} 