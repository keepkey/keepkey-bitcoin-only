use semver::Version;
use inquire::{Confirm, Select, Text};
use rusb::{GlobalContext, Device as RusbDevice};

use super::device_detection::{DeviceState, detect_device_impl};
use kkcli::firmware_manager::FirmwareManager;
use crate::transport::{UsbTransport, ProtocolAdapter}; // Corrected StdUsbTransport
use crate::messages;
use anyhow::anyhow; // Removed unused Result import
use sha2::{Digest, Sha256};

#[derive(Default, Debug, Clone)]
struct WizardCtx {
    remembered_fw: Option<Version>,
    remembered_bl: Option<Version>,
}

fn run_troubleshooter_impl() -> anyhow::Result<()> {
    println!("Troubleshooting steps would go here...");
    println!("Press Enter to continue...");
    let mut _buffer = String::new();
    std::io::stdin().read_line(&mut _buffer)?;
    Ok(())
}

fn wait_for_replug_impl(message: &str) -> anyhow::Result<()> {
    println!("{}", message);
    println!("➡️ Please ensure your KeepKey is connected and in the desired mode.");
    println!("   Press Enter after re-plugging or changing mode, or Ctrl+C to abort...");
    let mut _buffer = String::new();
    std::io::stdin().read_line(&mut _buffer)?;
    std::thread::sleep(std::time::Duration::from_secs(2)); // Allow device to re-enumerate
    Ok(())
}

// --- Action Implementations ---
// These now take FirmwareManager to fetch bytes if needed for an update.

fn do_bootloader_update_impl(
    firmware_manager: &FirmwareManager,
    device_handle: &RusbDevice<GlobalContext>,
) -> anyhow::Result<()> {
    let info = firmware_manager.get_latest_bootloader_info()
        .ok_or_else(|| anyhow!("Latest bootloader information not available."))?;

    println!(
        "Attempting to update bootloader to version: {} from {}",
        info.version,
        info.url
    );

    println!("Fetching bootloader bytes...");
    let bootloader_bytes = firmware_manager.get_firmware_bytes(info)
        .map_err(|e| anyhow!("Failed to get bootloader bytes: {}", e))?;
    println!("Successfully fetched {} bytes for bootloader update.", bootloader_bytes.len());

    println!("Preparing device for bootloader update...");
    let (mut transport, _config_desc, _device_h_for_transport) = match UsbTransport::new(device_handle, 0) {
        Ok(t) => t,
        Err(e) => {
            return Err(anyhow!("Failed to create USB transport for bootloader update. Ensure device is in bootloader mode and drivers are correctly configured. Error: {}", e));
        }
    };
    
    println!("Uploading bootloader ({} bytes)...", bootloader_bytes.len());
    let mut handler = transport.with_standard_handler();
    match handler.handle(
        messages::FirmwareUpload {
            payload_hash: Sha256::digest(&bootloader_bytes).to_vec(),
            payload: bootloader_bytes,
        }
        .into()
    ) {
        Ok(messages::Message::Success(s)) => {
            println!("✅ Bootloader update successful: {}", s.message());
            println!("Device may reboot. Please wait a moment and then re-scan.");
            Ok(())
        }
        Ok(messages::Message::Failure(f)) => {
            Err(anyhow!("Bootloader update failed: {}", f.message()))
        }
        Ok(other) => {
            Err(anyhow!("Unexpected response during bootloader upload: {:?}", other))
        }
        Err(e) => Err(anyhow!("Error during bootloader upload: {}", e)),
    }
}

fn do_firmware_update_impl(
    firmware_manager: &FirmwareManager,
    device_handle: &RusbDevice<GlobalContext>,
) -> anyhow::Result<()> {
    let fw_info = firmware_manager.get_latest_firmware_info()
        .ok_or_else(|| anyhow!("Latest firmware information not available."))?;
    println!(
        "Attempting to update firmware to version: {} from {}",
        fw_info.version,
        fw_info.url
    );
    println!("Fetching firmware bytes...");
    let firmware_bytes = firmware_manager.get_firmware_bytes(fw_info)
        .map_err(|e| anyhow!("Failed to get firmware bytes: {}", e))?;
    println!("Successfully fetched {} bytes of firmware.", firmware_bytes.len());

    println!("Preparing device for firmware update...");
    let (mut transport, _config_desc, _device_h_for_transport) = match UsbTransport::new(device_handle, 0) {
        Ok(t) => t,
        Err(e) => {
             return Err(anyhow!("Failed to create USB transport for firmware update. Ensure device is in bootloader mode and drivers are correctly configured. Error: {}", e));
        }
    };

    {
        // Scope for erase operation and its handler
        println!("Erasing firmware sectors...");
        let mut erase_handler = transport.with_standard_handler();
        match erase_handler.handle(messages::FirmwareErase::default().into()) {
            Ok(messages::Message::Success(s)) => {
                println!("Firmware erase successful: {}", s.message());
            }
            Ok(messages::Message::Failure(f)) => {
                return Err(anyhow!("Firmware erase command failed: {}. Aborting update.", f.message()));
            }
            Ok(other) => {
                return Err(anyhow!("Unexpected response during firmware erase: {:?}. Aborting update.", other));
            }
            Err(e) => return Err(anyhow!("Error during firmware erase: {}. Aborting update.", e)),
        }
    } // erase_handler is dropped here
    
    println!("Uploading firmware ({} bytes)...", firmware_bytes.len());
    let mut upload_handler = transport.with_standard_handler();
    match upload_handler.handle(
        messages::FirmwareUpload {
            payload_hash: Sha256::digest(&firmware_bytes).to_vec(),
            payload: firmware_bytes,
        }
        .into()
    ) {
        Ok(messages::Message::Success(s)) => {
            println!("✅ Firmware update successful: {}", s.message());
            println!("Device may reboot. Please wait a moment and then re-scan.");
            Ok(())
        }
        Ok(messages::Message::Failure(f)) => {
            Err(anyhow!("Firmware update failed: {}", f.message()))
        }
        Ok(other) => {
            Err(anyhow!("Unexpected response during firmware upload: {:?}", other))
        }
        Err(e) => Err(anyhow!("Error during firmware upload: {}", e)),
    }
}

fn do_firmware_update_from_file_impl(
    path: &str,
    _device_handle: &RusbDevice<GlobalContext>, // Prefixed with underscore as it's not used yet
) -> anyhow::Result<()> {
    println!(
        "Attempting to update firmware from custom file: {}",
        path
    );
    // TODO: Read bytes from path, then implement actual firmware update logic
    // This will be similar to do_firmware_update_impl but with bytes from a file.
    // It will need its own UsbTransport creation etc.
    println!("ACTION: Firmware update from file {} would happen here... (Not yet fully implemented)", path);
    Err(anyhow!("Firmware update from file is not fully implemented yet.")) // Fail explicitly
}


// --- UI Step Functions (using inquire) ---

fn step_no_device() -> anyhow::Result<bool> { // Returns true if wizard should quit
    println!("🔌 No KeepKey detected, or it's unresponsive.");
    let options = vec!["Retry scan", "Run troubleshooter", "Quit wizard"];
    let choice = Select::new("What would you like to do?", options).prompt()?;
    match choice {
        "Retry scan" => Ok(false),
        "Run troubleshooter" => { run_troubleshooter_impl()?; Ok(false) }
        "Quit wizard" => Ok(true),
        _ => unreachable!(),
    }
}

fn step_access_error(vid: u16, pid: u16, error_message: &str, underlying_error: &str) -> anyhow::Result<bool> { // Returns true if wizard should quit
    println!("⚠️ Found KeepKey (VID: {:04x}, PID: {:04x}), but could not establish communication.", vid, pid);
    println!("   Error: {}", error_message);
    println!("   Details: {}", underlying_error);
    let options = vec!["Retry scan", "Run troubleshooter", "Quit wizard"];
    let choice = Select::new("What would you like to do?", options).prompt()?;
    match choice {
        "Retry scan" => Ok(false),
        "Run troubleshooter" => { run_troubleshooter_impl()?; Ok(false) }
        "Quit wizard" => Ok(true),
        _ => unreachable!(),
    }
}

fn step_normal(
    ctx: &mut WizardCtx,
    fw: Option<Version>,
    _bl_from_features: Option<Version>,
    _device_handle: &RusbDevice<GlobalContext>,
    firmware_manager: &FirmwareManager,
) -> anyhow::Result<bool> { // Returns true if wizard should quit
    let fw_str = fw.as_ref().map_or_else(|| "unknown".to_string(), |v| v.to_string());
    let bl_str = ctx.remembered_bl.as_ref().map_or_else(|| "unknown".to_string(), |v| v.to_string());
    println!("🟢 Device in NORMAL mode. Firmware: {}, Bootloader: {} (last known)", fw_str, bl_str);

    if let Some(current_fw) = fw {
        ctx.remembered_fw = Some(current_fw);
    }

    let latest_fw_ver_res = firmware_manager.get_latest_firmware_version();
    let latest_bl_ver_res = firmware_manager.get_latest_bootloader_version();

    match (&latest_fw_ver_res, &latest_bl_ver_res) {
        (Ok(latest_fw), Ok(latest_bl)) => {
            // Create message for update prompt that will be displayed to the user
            let mut update_prompt_message = String::new();
            let mut needs_fw_update = false;
            let mut needs_bl_update = false;

            if let Some(current_fw) = &ctx.remembered_fw {
                if current_fw < latest_fw {
                    update_prompt_message = format!("Firmware update available ({} -> {}).", current_fw, latest_fw);
                    needs_fw_update = true;
                } else {
                    update_prompt_message = "Firmware is up-to-date.".to_string();
                }
            } else {
                update_prompt_message = format!("Firmware version unknown. Latest available is {}.", latest_fw);
                needs_fw_update = true; // Recommend update if current is unknown
            }

            if let Some(current_bl) = &ctx.remembered_bl {
                if current_bl < latest_bl {
                    if !update_prompt_message.is_empty() { update_prompt_message.push_str(" "); }
                    update_prompt_message.push_str(&format!("Bootloader update also available ({} -> {}).", current_bl, latest_bl));
                    needs_bl_update = true;
                }
            } else if needs_fw_update || ctx.remembered_bl.is_none() {
                 if !update_prompt_message.is_empty() { update_prompt_message.push_str(" "); }
                 update_prompt_message.push_str(&format!("Bootloader version unknown (latest available: {}).", latest_bl));
                 needs_bl_update = true; // Recommend checking if unknown
            }
            
            if needs_fw_update || needs_bl_update {
                if Confirm::new(&format!("{} Would you like to switch to updater mode to apply updates?", update_prompt_message))
                    .with_default(true).prompt()? {
                    wait_for_replug_impl("Please put your KeepKey into bootloader (updater) mode.")?;
                } else {
                    println!("Update declined. Continuing in normal mode.");
                }
            } else {
                println!("✅ Your KeepKey firmware and bootloader (last known) appear up to date!");
            }
        }
        _ => {
            println!("⚠️ Could not determine latest firmware/bootloader versions from manager.");
            if Confirm::new("Switch to updater mode anyway for manual options or to re-check?").prompt()? {
                 wait_for_replug_impl("Please put your KeepKey into bootloader (updater) mode.")?;
            }
        }
    }
    Ok(false) // Don't quit wizard from normal step unless user explicitly quits via Ctrl+C or future option
}

fn step_updater(
    ctx: &mut WizardCtx,
    bl_version_from_updater: Option<Version>,
    device_handle: &RusbDevice<GlobalContext>,
    firmware_manager: &FirmwareManager,
) -> anyhow::Result<bool> { // Returns true if wizard should quit
    let bl_version_str = bl_version_from_updater.as_ref().map_or_else(|| "unknown".to_string(), |v| v.to_string());
    println!("🛠 Device is in UPDATER mode. Bootloader Version: {}", bl_version_str);

    if let Some(bl) = bl_version_from_updater {
        ctx.remembered_bl = Some(bl);
    }
    if let Some(fw) = &ctx.remembered_fw {
        println!("   Application Firmware Version (last known from normal mode): {}", fw);
    } else {
        println!("   Application Firmware Version (last known from normal mode): Unknown");
    }

    let mut options = Vec::new();
    let latest_fw_info = firmware_manager.get_latest_firmware_info();
    let latest_bl_info = firmware_manager.get_latest_bootloader_info();

    if let Some(info) = latest_fw_info {
        options.push(format!("Update Firmware to latest ({})", info.version));
    } else {
        options.push("Update Firmware to latest (version info unavailable)".to_string());
    }
    options.push("Update Firmware from custom file".to_string());

    if let Some(info) = latest_bl_info {
        options.push(format!("Update Bootloader to latest ({})", info.version));
    } else {
        options.push("Update Bootloader to latest (version info unavailable)".to_string());
    }
    options.push("Re-scan / Return to normal mode check".to_string());
    options.push("Quit wizard".to_string());

    let choice = Select::new("Select an action for updater mode:", options).prompt()?;

    match choice.as_str() {
        s if s.starts_with("Update Firmware to latest") => {
            if latest_fw_info.is_some() {
                do_firmware_update_impl(firmware_manager, device_handle)?;
            } else {
                println!("Cannot update to latest firmware: version information is unavailable.");
            }
        }
        "Update Firmware from custom file" => {
            let path = Text::new("Path to signed firmware file:").prompt()?;
            do_firmware_update_from_file_impl(&path, device_handle)?;
        }
        s if s.starts_with("Update Bootloader to latest") => {
            if latest_bl_info.is_some() {
                do_bootloader_update_impl(firmware_manager, device_handle)?;
            } else {
                println!("Cannot update to latest bootloader: version information is unavailable.");
            }
        }
        "Re-scan / Return to normal mode check" => {
            wait_for_replug_impl("If device was in updater mode, re-plug in normal mode or wait for timeout.")?;
            return Ok(false); // Don't quit, just rescan
        }
        "Quit wizard" => return Ok(true),
        _ => println!("Unknown action selected."),
    }

    // After an action, confirm before re-scan or next step
    if Confirm::new("Operation attempted. Re-scan device?").with_default(true).prompt()? {
        // Just fall through to let the main loop re-scan
    } else {
        println!("Exiting updater actions. You may need to re-plug your device in normal mode.");
        // Consider if quitting wizard is more appropriate if they don't want to rescan
    }
    Ok(false) // Don't quit by default after an action
}

pub fn run_onboarding_wizard() -> anyhow::Result<()> {
    let mut ctx = WizardCtx::default();
    let firmware_manager = FirmwareManager::new()?;
    println!("Welcome to the KeepKey Onboarding Wizard!");

    loop {
        println!("\n-------------------------------------\n");
        println!("[DEBUG] Top of loop. WizardCtx: {:?}", ctx);
        let device_state = match detect_device_impl() {
            Ok(state) => state,
            Err(e) => {
                println!("🚨 Error detecting device: {}. Please ensure your USB setup is correct and try again.", e);
                // Offer to retry or quit after a detection-level error
                let options = vec!["Retry scan", "Quit wizard"];
                match Select::new("What would you like to do?", options).prompt()? {
                    "Retry scan" => continue, // Restart loop
                    "Quit wizard" => return Ok(()),
                    _ => unreachable!(),
                }
            }
        };

        let quit_wizard = match device_state {
            DeviceState::NoDevice => step_no_device()?,
            DeviceState::AccessError { vid, pid, error_message, underlying_error } => {
                step_access_error(vid, pid, &error_message, &underlying_error)?
            }
            DeviceState::Normal { fw, bl, device_handle } => {
                step_normal(&mut ctx, fw, bl, &device_handle, &firmware_manager)?
            }
            DeviceState::Updater { bl_version_from_updater, ref device_handle } => {
                step_updater(&mut ctx, bl_version_from_updater, device_handle, &firmware_manager)?
            }
        };

        if quit_wizard {
            break;
        }
        
        println!("\nScanning for device status changes... (Press Ctrl+C to exit wizard)");
        std::thread::sleep(std::time::Duration::from_millis(2000)); // Slightly shorter delay
        println!("\n-------------------------------------\n");
    }
    println!("Exiting KeepKey Onboarding Wizard.");
    Ok(())
} 