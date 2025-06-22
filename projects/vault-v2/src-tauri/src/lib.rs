use tauri::{Emitter, Manager};

// Modules for better organization

mod commands;
mod device;
mod event_controller;
mod logging;
mod slip132;
mod embedded_firmware;
mod server;

// Re-export commonly used types

use std::sync::Arc;

// Learn more about Tauri commands at https://tauri.app/develop/rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// Onboarding related commands moved to commands.rs



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
fn vault_open_support(app: tauri::AppHandle) -> Result<(), String> {
    println!("Opening support");
    
    // Switch to browser view and navigate to support
    app.emit("vault:change_view", serde_json::json!({
        "view": "browser"
    })).map_err(|e| format!("Failed to emit view change event: {}", e))?;
    
    app.emit("browser:navigate", serde_json::json!({
        "url": "https://support.keepkey.com"
    })).map_err(|e| format!("Failed to emit navigation event: {}", e))?;
    
    Ok(())
}

// Add the missing vault_open_app command to open external URLs
#[tauri::command]
async fn vault_open_app(app_handle: tauri::AppHandle, app_id: String, app_name: String, url: String) -> Result<(), String> {
    println!("Opening app: {} ({}) -> {}", app_name, app_id, url);
    
    // Use Tauri's opener plugin to open the URL in the system browser
    use tauri_plugin_opener::OpenerExt;
    app_handle.opener().open_url(url, None::<&str>)
        .map_err(|e| format!("Failed to open URL: {}", e))?;
    
    Ok(())
}

// Add a general command to open any URL in the system browser
#[tauri::command]
async fn open_url(app_handle: tauri::AppHandle, url: String) -> Result<(), String> {
    println!("Opening URL in system browser: {}", url);
    
    // Use Tauri's opener plugin to open the URL in the system browser
    use tauri_plugin_opener::OpenerExt;
    app_handle.opener().open_url(url, None::<&str>)
        .map_err(|e| format!("Failed to open URL: {}", e))?;
    
    Ok(())
}

#[tauri::command]
async fn restart_backend_startup(app: tauri::AppHandle) -> Result<(), String> {
    println!("🔄 Restarting backend startup process...");
    
    // Emit initial restart status
    let _ = app.emit("status:update", serde_json::json!({
        "status": "Restarting backend..."
    }));
    
    // 1. Stop the current event controller
    if let Some(controller_state) = app.try_state::<std::sync::Arc<std::sync::Mutex<event_controller::EventController>>>() {
        println!("🛑 Stopping current event controller...");
        let controller_arc = controller_state.inner().clone();
        
        // Stop the controller in a separate scope to ensure proper cleanup
        {
            if let Ok(mut controller) = controller_arc.lock() {
                controller.stop();
                println!("✅ Event controller stopped successfully");
            } else {
                println!("⚠️ Failed to lock controller for stop");
            };
        } // Mutex guard is dropped here
    } else {
        println!("⚠️ No event controller found in app state");
    }
    
    // 2. Clear device queue manager state
    if let Some(queue_manager_state) = app.try_state::<std::sync::Arc<tokio::sync::Mutex<std::collections::HashMap<String, keepkey_rust::device_queue::DeviceQueueHandle>>>>() {
        println!("🧹 Cleaning up device queues...");
        let queue_manager_arc = queue_manager_state.inner().clone();
        let mut manager = queue_manager_arc.lock().await;
        
        // Shutdown all existing device queues
        for (device_id, handle) in manager.drain() {
            println!("🛑 Shutting down queue for device: {}", device_id);
            let _ = handle.shutdown().await;
        }
        println!("✅ All device queues cleaned up");
    } else {
        println!("⚠️ No device queue manager found in app state");
    }
    
    // 3. Clear last responses cache
    if let Some(responses_state) = app.try_state::<std::sync::Arc<tokio::sync::Mutex<std::collections::HashMap<String, commands::DeviceResponse>>>>() {
        println!("🧹 Clearing response cache...");
        let responses_arc = responses_state.inner().clone();
        let mut responses = responses_arc.lock().await;
        responses.clear();
        println!("✅ Response cache cleared");
    }
    
    // 4. Give a moment for cleanup to complete
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    
    // 5. Start a new event controller
    println!("🚀 Starting new event controller...");
    let _new_controller = event_controller::spawn_event_controller(&app);
    
    // Emit completion status
    let _ = app.emit("status:update", serde_json::json!({
        "status": "Backend restarted - scanning for devices..."
    }));
    
    println!("✅ Backend restart completed successfully");
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_sql::Builder::default().build())
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            // Initialize device logging system
            if let Err(e) = logging::init_device_logger() {
                eprintln!("Failed to initialize device logger: {}", e);
            } else {
                println!("✅ Device logging initialized - logs will be written to ~/.keepkey/logs/");
            }
            
            // Initialize real device system using keepkey_rust
            let device_queue_manager = Arc::new(tokio::sync::Mutex::new(
                std::collections::HashMap::<String, keepkey_rust::device_queue::DeviceQueueHandle>::new()
            ));
            
            // Initialize response tracking
            let last_responses = Arc::new(tokio::sync::Mutex::new(
                std::collections::HashMap::<String, commands::DeviceResponse>::new()
            ));
            
            app.manage(device_queue_manager.clone());
            app.manage(last_responses);
            
            // Start event controller with proper management
            let _event_controller = event_controller::spawn_event_controller(&app.handle());
            
            // Start background log cleanup task
            tauri::async_runtime::spawn(async move {
                let mut interval = tokio::time::interval(std::time::Duration::from_secs(86400)); // 24 hours
                loop {
                    interval.tick().await;
                    if let Err(e) = logging::get_device_logger().cleanup_old_logs().await {
                        eprintln!("Failed to cleanup old logs: {}", e);
                    }
                }
            });
            
            // Start REST/MCP server in background (only if enabled in preferences)
            let server_handle = app.handle().clone();
            let server_queue_manager = device_queue_manager.clone();
            tauri::async_runtime::spawn(async move {
                // Add a small delay to ensure config system is ready
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                
                // Check if API is enabled in preferences
                let api_enabled = match commands::get_api_enabled().await {
                    Ok(enabled) => enabled,
                    Err(e) => {
                        log::debug!("Could not check API status: {} - defaulting to disabled", e);
                        false // Default to disabled if error
                    }
                };
                
                if api_enabled {
                    log::info!("🚀 API is enabled in preferences, starting server...");
                    
                    if let Err(e) = server::start_server(server_queue_manager).await {
                        log::error!("❌ Server error: {}", e);
                        // Optionally emit error event to frontend
                        let _ = server_handle.emit("server:error", serde_json::json!({
                            "error": format!("Server failed to start: {}", e)
                        }));
                    }
                } else {
                    log::info!("🔒 API is disabled in preferences, skipping server startup");
                }
            });
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            vault_change_view,
            vault_open_support,
            vault_open_app,
            open_url,
            restart_backend_startup,
            // Frontend readiness
            commands::frontend_ready,
            // Device operations - unified queue interface
            device::queue::add_to_device_queue,
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
            // Update commands
            device::updates::update_device_bootloader,
            device::updates::update_device_firmware,
            // PIN creation commands
            commands::initialize_device_pin,
            commands::send_pin_matrix_response,
            commands::get_pin_session_status,
            commands::cancel_pin_creation,
            commands::initialize_device_wallet,
            commands::complete_wallet_creation,
            // PIN unlock commands  
            commands::start_pin_unlock,
            commands::send_pin_unlock_response,
            commands::send_pin_matrix_ack,
            commands::trigger_pin_request,
            commands::check_device_pin_ready,
            // Logging commands
            commands::get_device_log_path,
            commands::get_recent_device_logs,
            commands::cleanup_device_logs,
            // Configuration and onboarding commands
            commands::is_first_time_install,
            commands::is_onboarded,
            commands::set_onboarding_completed,
            commands::get_preference,
            commands::set_preference,
            commands::debug_onboarding_state,
            // API control commands
            commands::get_api_enabled,
            commands::set_api_enabled,
            commands::get_api_status,
            commands::restart_app,
            // Test commands
            commands::test_device_queue,
            commands::test_status_emission,
            commands::test_bootloader_mode_device_status,
            commands::test_oob_device_status_evaluation,
            // Recovery commands - delegated to keepkey_rust
            commands::start_device_recovery,
            commands::send_recovery_character,
            commands::send_recovery_pin_response,
            commands::get_recovery_status,
            commands::cancel_recovery_session,
            // Seed verification commands (dry run recovery)
            commands::start_seed_verification,
            commands::send_verification_character,
            commands::send_verification_pin,
            commands::get_verification_status,
            commands::cancel_seed_verification,
            commands::force_cleanup_seed_verification
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
