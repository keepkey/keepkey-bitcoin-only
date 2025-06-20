use keepkey_rust::features::{list_connected_devices, get_device_features_with_fallback};

fn main() {
    println!("Testing KeepKey device connection and communication...\n");
    
    // First, list devices
    let devices = list_connected_devices();
    
    if devices.is_empty() {
        println!("❌ No KeepKey devices found");
        return;
    }
    
    println!("✅ Found {} KeepKey device(s):", devices.len());
    for device in &devices {
        println!("  • {} ({})", device.name, device.unique_id);
        println!("    VID:PID: {:04x}:{:04x}", device.vid, device.pid);
        println!("    Serial: {:?}\n", device.serial_number);
    }
    
    // Try to connect to each device and get features
    for device in &devices {
        println!("🔌 Attempting to connect to device: {}", device.unique_id);
        println!("   Serial: {:?}", device.serial_number);
        
        match get_device_features_with_fallback(device) {
            Ok(features) => {
                println!("✅ Successfully connected and got features!");
                println!("   Label: {:?}", features.label);
                println!("   Vendor: {:?}", features.vendor);  
                println!("   Model: {:?}", features.model);
                println!("   Version: {}", features.version);
                println!("   Initialized: {}", features.initialized);
                println!("   Bootloader Mode: {}", features.bootloader_mode);
                if let Some(device_id) = &features.device_id {
                    println!("   Device ID: {}", device_id);
                }
                println!();
            }
            Err(e) => {
                println!("❌ Failed to connect to device: {}", e);
                println!("   This is the exact error that causes issues in vault-v2\n");
                
                // Check if this is a "device in use" error
                let error_str = e.to_string().to_lowercase();
                if error_str.contains("in use") || error_str.contains("access") || 
                   error_str.contains("permission") || error_str.contains("busy") ||
                   error_str.contains("claimed") {
                    println!("🔍 This appears to be a 'device in use' error - the exact issue described in the docs!");
                    println!("   According to the analysis, keepkey_rust fails aggressively here");
                    println!("   while KeepKey Desktop v5 would continue with warnings.");
                }
            }
        }
    }
    
    println!("Test completed. If you see connection failures above,");
    println!("these are the exact issues that need to be fixed in keepkey_rust");
    println!("before vault-v2 can work reliably on Windows.");
} 