use hidapi::HidApi;
use std::time::Duration;

const KEEPKEY_VID: u16 = 0x2B24;
const KEEPKEY_PIDS: &[u16] = &[0x0001, 0x0002];

fn main() {
    println!("Testing raw HID communication without report ID...\n");
    
    let api = match HidApi::new() {
        Ok(api) => api,
        Err(e) => {
            println!("❌ Failed to initialize HID API: {}", e);
            return;
        }
    };
    
    // Find KeepKey device
    let mut keepkey_device = None;
    for device_info in api.device_list() {
        if device_info.vendor_id() == KEEPKEY_VID && 
           KEEPKEY_PIDS.contains(&device_info.product_id()) {
            println!("✅ Found KeepKey device:");
            println!("   VID:PID: {:04x}:{:04x}", device_info.vendor_id(), device_info.product_id());
            println!("   Serial: {:?}", device_info.serial_number());
            println!("   Path: {}", device_info.path().to_string_lossy());
            keepkey_device = Some(device_info);
            break;
        }
    }
    
    let device_info = match keepkey_device {
        Some(info) => info,
        None => {
            println!("❌ No KeepKey devices found");
            return;
        }
    };
    
    // Try to open the device
    let device = match device_info.open_device(&api) {
        Ok(device) => {
            println!("✅ Successfully opened KeepKey device");
            device
        },
        Err(e) => {
            println!("❌ Failed to open device: {}", e);
            println!("   This might be a device access issue (another app using it)");
            return;
        }
    };
    
    println!("\n🔬 Testing different HID packet formats...\n");
    
    // Test 1: Current keepkey_rust format (with report ID 0x00)
    println!("Test 1: WITH report ID (0x00) - current keepkey_rust format");
    test_hid_write_format(&device, true);
    
    // Test 2: Without report ID prefix
    println!("\nTest 2: WITHOUT report ID - raw packet");
    test_hid_write_format(&device, false);
    
    println!("\nTest completed. Check which format works without Windows error 87!");
}

fn test_hid_write_format(device: &hidapi::HidDevice, with_report_id: bool) {
    // Create a simple Initialize message
    // Initialize message: type 0x0000, no data, length 0
    let msg_type: [u8; 2] = [0x00, 0x00];  // Initialize = 0
    let msg_length: [u8; 4] = [0x00, 0x00, 0x00, 0x00];  // No data
    
    let mut packet = if with_report_id {
        // Current keepkey_rust format: [Report ID][0x3f][0x23][0x23][msg_type(2)][length(4)][data...]
        let mut pkt = vec![0u8; 64];
        pkt[0] = 0x00;  // Report ID
        pkt[1] = 0x3f;
        pkt[2] = 0x23;
        pkt[3] = 0x23;
        pkt[4..6].copy_from_slice(&msg_type);
        pkt[6..10].copy_from_slice(&msg_length);
        pkt
    } else {
        // Without report ID: [0x3f][0x23][0x23][msg_type(2)][length(4)][data...]
        let mut pkt = vec![0u8; 64];
        pkt[0] = 0x3f;
        pkt[1] = 0x23;
        pkt[2] = 0x23;
        pkt[3..5].copy_from_slice(&msg_type);
        pkt[5..9].copy_from_slice(&msg_length);
        pkt
    };
    
    // Show packet format
    let preview: Vec<String> = packet[..16].iter()
        .map(|b| format!("{:02x}", b))
        .collect();
    println!("   Packet format (first 16 bytes): {}", preview.join(" "));
    
    // Try to write
    match device.write(&packet) {
        Ok(bytes_written) => {
            println!("   ✅ SUCCESS! Wrote {} bytes to device", bytes_written);
            
            // Try to read response
            let mut response = vec![0u8; 64];
            match device.read_timeout(&mut response, 1000) {
                Ok(bytes_read) if bytes_read > 0 => {
                    let resp_preview: Vec<String> = response[..bytes_read.min(16)].iter()
                        .map(|b| format!("{:02x}", b))
                        .collect();
                    println!("   📨 Got response ({} bytes): {}", bytes_read, resp_preview.join(" "));
                }
                Ok(_) => {
                    println!("   ⚠️  No response from device (timeout)");
                }
                Err(e) => {
                    println!("   ⚠️  Read failed: {}", e);
                }
            }
        }
        Err(e) => {
            println!("   ❌ FAILED: {}", e);
            let error_str = e.to_string();
            if error_str.contains("87") || error_str.contains("parameter") {
                println!("      ^ This is Windows error 87 (invalid parameter) - packet format issue!");
            }
        }
    }
} 