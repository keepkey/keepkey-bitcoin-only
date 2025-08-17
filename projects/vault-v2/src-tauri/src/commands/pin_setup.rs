use keepkey_rust::messages::{Message, ChangePin, ButtonAck, PinMatrixAck};
use tauri::{State, AppHandle, Emitter};
use crate::commands::DeviceQueueManager;
use uuid::Uuid;

/// Start PIN setup for a device (for already initialized devices)
#[tauri::command]
pub async fn start_pin_setup(
    device_id: String,
    app: AppHandle,
    queue_manager: State<'_, DeviceQueueManager>,
) -> Result<String, String> {
    log::info!("Starting PIN setup for device {}", device_id);
    
    // Generate a session ID
    let session_id = Uuid::new_v4().to_string();
    
    // Get the device queue handle
    let queue_handle = {
        let manager = queue_manager.lock().await;
        manager.get(&device_id).cloned()
    };
    
    let queue_handle = queue_handle
        .ok_or_else(|| format!("No device queue found for device {}", device_id))?;
    
    // Send ChangePin message to start PIN setup
    let change_pin = ChangePin {
        remove: Some(false), // false means set/change PIN
    };
    let message = Message::ChangePin(change_pin);
    
    match queue_handle.send_raw(message, true).await {
        Ok(response) => {
            match response {
                Message::ButtonRequest(_) => {
                    log::info!("PIN setup: Got ButtonRequest, sending ButtonAck");
                    
                    // Send ButtonAck
                    let button_ack = Message::ButtonAck(ButtonAck {});
                    match queue_handle.send_raw(button_ack, true).await {
                        Ok(Message::PinMatrixRequest(req)) => {
                            log::info!("PIN setup: Got first PinMatrixRequest for new PIN");
                            
                            // Emit event to frontend to show PIN matrix UI
                            let payload = serde_json::json!({
                                "sessionId": session_id,
                                "deviceId": device_id,
                                "step": "new_pin",
                                "type": req.r#type
                            });
                            
                            if let Err(e) = app.emit("pin_matrix_request", payload) {
                                log::error!("Failed to emit pin_matrix_request event: {}", e);
                                return Err(format!("Failed to emit event: {}", e));
                            }
                            
                            Ok(session_id)
                        }
                        Ok(msg) => {
                            let error = format!("Unexpected response after ButtonAck: {:?}", msg);
                            log::error!("{}", error);
                            Err(error)
                        }
                        Err(e) => {
                            let error = format!("Failed to send ButtonAck: {}", e);
                            log::error!("{}", error);
                            Err(error)
                        }
                    }
                }
                Message::PinMatrixRequest(req) => {
                    log::info!("PIN setup: Got direct PinMatrixRequest for new PIN");
                    
                    // Emit event to frontend to show PIN matrix UI
                    let payload = serde_json::json!({
                        "sessionId": session_id,
                        "deviceId": device_id,
                        "step": "new_pin",
                        "type": req.r#type
                    });
                    
                    if let Err(e) = app.emit("pin_matrix_request", payload) {
                        log::error!("Failed to emit pin_matrix_request event: {}", e);
                        return Err(format!("Failed to emit event: {}", e));
                    }
                    
                    Ok(session_id)
                }
                Message::Failure(failure) => {
                    let error = format!("Device rejected PIN setup: {}", failure.message.unwrap_or_default());
                    log::error!("{}", error);
                    Err(error)
                }
                _ => {
                    let error = format!("Unexpected response from device: {:?}", response);
                    log::error!("{}", error);
                    Err(error)
                }
            }
        }
        Err(e) => {
            let error = format!("Failed to start PIN setup: {}", e);
            log::error!("{}", error);
            Err(error)
        }
    }
}

/// Send PIN response during PIN setup
#[tauri::command]
pub async fn send_pin_setup_response(
    device_id: String,
    session_id: String,
    pin: String,
    step: String,
    app: AppHandle,
    queue_manager: State<'_, DeviceQueueManager>,
) -> Result<String, String> {
    log::info!("Sending PIN response for device {} (session: {}, step: {})", device_id, session_id, step);
    
    // Get the device queue handle
    let queue_handle = {
        let manager = queue_manager.lock().await;
        manager.get(&device_id).cloned()
    };
    
    let queue_handle = queue_handle
        .ok_or_else(|| format!("No device queue found for device {}", device_id))?;
    
    // Send PinMatrixAck with the PIN
    let pin_ack = PinMatrixAck {
        pin,
    };
    let message = Message::PinMatrixAck(pin_ack);
    
    match queue_handle.send_raw(message, true).await {
        Ok(response) => {
            match response {
                Message::PinMatrixRequest(req) => {
                    // This is the confirmation request
                    log::info!("PIN setup: Got confirmation PinMatrixRequest");
                    
                    // Emit event to frontend to show PIN matrix UI for confirmation
                    let payload = serde_json::json!({
                        "sessionId": session_id,
                        "deviceId": device_id,
                        "step": "confirm_pin",
                        "type": req.r#type
                    });
                    
                    if let Err(e) = app.emit("pin_matrix_request", payload) {
                        log::error!("Failed to emit pin_matrix_request event: {}", e);
                        return Err(format!("Failed to emit event: {}", e));
                    }
                    
                    Ok("confirm".to_string())
                }
                Message::Success(_) => {
                    log::info!("PIN setup completed successfully");
                    
                    // Emit success event
                    let payload = serde_json::json!({
                        "sessionId": session_id,
                        "deviceId": device_id,
                        "success": true,
                    });
                    
                    if let Err(e) = app.emit("pin_setup_complete", payload) {
                        log::error!("Failed to emit pin_setup_complete event: {}", e);
                    }
                    
                    Ok("success".to_string())
                }
                Message::Failure(failure) => {
                    let error = format!("PIN setup failed: {}", failure.message.unwrap_or_default());
                    log::error!("{}", error);
                    
                    // Emit failure event
                    let payload = serde_json::json!({
                        "sessionId": session_id,
                        "deviceId": device_id,
                        "success": false,
                        "error": error.clone(),
                    });
                    
                    if let Err(e) = app.emit("pin_setup_complete", payload) {
                        log::error!("Failed to emit pin_setup_complete event: {}", e);
                    }
                    
                    Err(error)
                }
                _ => {
                    let error = format!("Unexpected response: {:?}", response);
                    log::error!("{}", error);
                    Err(error)
                }
            }
        }
        Err(e) => {
            let error = format!("Failed to send PIN response: {}", e);
            log::error!("{}", error);
            Err(error)
        }
    }
}