use tauri::{Emitter, Manager};

// Modules for better organization
mod db;
mod commands;
mod event_controller;

// Re-export commonly used types
pub use db::{Database, DeviceInfo, XpubInfo};
use std::sync::Arc;

// Learn more about Tauri commands at https://tauri.app/develop/rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// Onboarding related commands
#[tauri::command]
fn is_first_time_install() -> Result<bool, String> {
    // For now, return false - can be enhanced to check actual installation state
    Ok(false)
}



// Vault interface commands
#[tauri::command]
fn vault_change_view(app: tauri::AppHandle, view: String) -> Result<(), String> {
    println!("View changed to: {}", view);
    // Emit event to frontend if needed
    match app.emit("vault:change_view", serde_json::json!({ "view": view })) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed to emit view change event: {}", e))
    }
}

#[tauri::command]
fn vault_open_support() -> Result<(), String> {
    println!("Opening support");
    // Could open browser or perform other support actions
    Ok(())
}

#[tauri::command]
fn restart_backend_startup(app: tauri::AppHandle) -> Result<(), String> {
    println!("Restarting backend startup process");
    // Emit event to indicate restart
    match app.emit("application:state", serde_json::json!({
        "status": "Restarting...",
        "connected": false,
        "features": null
    })) {
        Ok(_) => {
            // Simulate restart process
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(1000));
                let _ = app.emit("application:state", serde_json::json!({
                    "status": "Device ready",
                    "connected": true,
                    "features": {
                        "label": "KeepKey",
                        "vendor": "KeepKey",
                        "model": "KeepKey",
                        "firmware_variant": "keepkey",
                        "device_id": "keepkey-001",
                        "language": "english",
                        "bootloader_mode": false,
                        "version": "7.7.0",
                        "firmware_hash": null,
                        "bootloader_hash": null,
                        "initialized": true,
                        "imported": false,
                        "no_backup": false,
                        "pin_protection": true,
                        "pin_cached": false,
                        "passphrase_protection": false,
                        "passphrase_cached": false,
                        "wipe_code_protection": false,
                        "auto_lock_delay_ms": null,
                        "policies": []
                    }
                }));
            });
            Ok(())
        },
        Err(e) => Err(format!("Failed to emit restart event: {}", e))
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_sql::Builder::default().build())
        .setup(|app| {
            // Initialize real device system using keepkey_rust
            let device_queue_manager = Arc::new(tokio::sync::Mutex::new(
                std::collections::HashMap::<String, keepkey_rust::device_queue::DeviceQueueHandle>::new()
            ));
            
            app.manage(device_queue_manager);
            
            // Start event controller with proper management
            let _event_controller = event_controller::spawn_event_controller(&app.handle());
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            is_first_time_install,
            vault_change_view,
            vault_open_support,
            restart_backend_startup,
            // Device operations - unified queue interface
            commands::add_to_device_queue,
            commands::get_queue_status,
            // Basic device enumeration (non-queue operations)
            commands::get_connected_devices,
            commands::get_blocking_actions,
            // New device commands (all go through queue)
            commands::get_device_status,
            commands::get_device_info_by_id,
            commands::wipe_device,
            commands::set_device_label,
            commands::get_connected_devices_with_features,
            // Test command
            commands::test_device_queue
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
