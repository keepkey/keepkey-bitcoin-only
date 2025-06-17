use tauri::{State, AppHandle, Emitter};
use std::sync::Arc;


// Import types needed for DeviceRequestWrapper
use crate::commands::{DeviceRequestWrapper, DeviceRequest, DeviceResponse, DeviceQueueManager};

#[tauri::command]
pub async fn add_to_device_queue(
    request: DeviceRequestWrapper,
    queue_manager: State<'_, DeviceQueueManager>,
    last_responses: State<'_, Arc<tokio::sync::Mutex<std::collections::HashMap<String, DeviceResponse>>>>,
    app: AppHandle,
) -> Result<String, String> {
    println!("Adding to device queue: {:?}", request);
    
    // Log the incoming request
    let request_data = serde_json::json!({
        "request": request.request,
        "device_id": request.device_id,
        "request_id": request.request_id
    });
    
    let request_type = match &request.request {
        DeviceRequest::GetXpub { .. } => "GetXpub",
        DeviceRequest::GetAddress { .. } => "GetAddress", 
        DeviceRequest::GetFeatures => "GetFeatures",
        DeviceRequest::SignTransaction { .. } => "SignTransaction",
        DeviceRequest::SendRaw { .. } => "SendRaw",
    };
    
    if let Err(e) = crate::logging::log_device_request(
        &request.device_id,
        &request.request_id,
        request_type,
        &request_data
    ).await {
        eprintln!("Failed to log device request: {}", e);
    }
    
    // Get or create device queue handle
    let queue_handle = {
        let mut manager = queue_manager.lock().await;
        
        if let Some(handle) = manager.get(&request.device_id) {
            handle.clone()
        } else {
            // Find the device by ID using high-level API
            let devices = keepkey_rust::features::list_connected_devices();
            let device_info = devices
                .iter()
                .find(|d| d.unique_id == request.device_id)
                .ok_or_else(|| format!("Device {} not found", request.device_id))?;
                
            // Spawn a new device worker using the real keepkey_rust implementation
            let handle = keepkey_rust::device_queue::DeviceQueueFactory::spawn_worker(request.device_id.clone(), device_info.clone());
            manager.insert(request.device_id.clone(), handle.clone());
            handle
        }
    };
    
    // Process the request based on type
    let result = match request.request {
        DeviceRequest::GetXpub { ref path } => {
            // Parse derivation path
            let path_parts = crate::commands::parse_derivation_path(&path)?;
            
            // Create GetPublicKey message for xpub
            let get_public_key = keepkey_rust::messages::Message::GetPublicKey(
                keepkey_rust::messages::GetPublicKey {
                    address_n: path_parts,
                    coin_name: Some("Bitcoin".to_string()),
                    script_type: None, // Default script type
                    ecdsa_curve_name: Some("secp256k1".to_string()),
                    show_display: Some(false), // Don't show on device for xpub requests
                    ..Default::default()
                }
            );
            
            // Send raw message to get xpub
            let response = queue_handle
                .send_raw(get_public_key, false)
                .await
                .map_err(|e| format!("Failed to get xpub: {}", e))?;
            
            match response {
                keepkey_rust::messages::Message::PublicKey(public_key) => {
                    // Extract xpub from the response
                    let xpub = public_key.xpub.unwrap_or_default();
                    if xpub.is_empty() {
                        Err("Device returned empty xpub".to_string())
                    } else {
                        Ok(xpub)
                    }
                }
                keepkey_rust::messages::Message::Failure(failure) => {
                    Err(format!("Device returned error: {}", failure.message.unwrap_or_default()))
                }
                _ => {
                    Err("Unexpected response from device for xpub request".to_string())
                }
            }
        }
        DeviceRequest::GetAddress { ref path, ref coin_name, ref script_type, show_display } => {
            let path_parts = crate::commands::parse_derivation_path(&path)?;
            let script_type_int = match script_type.as_deref() {
                Some("p2pkh") => Some(0),       // SPENDADDRESS = 0
                Some("p2sh-p2wpkh") => Some(4), // SPENDP2SHWITNESS = 4  
                Some("p2wpkh") => Some(3),      // SPENDWITNESS = 3
                _ => None,
            };
            
            queue_handle
                .get_address(path_parts, coin_name.clone(), script_type_int, show_display)
                .await
                .map_err(|e| format!("Failed to get address: {}", e))
        }
        DeviceRequest::GetFeatures => {
            let features = queue_handle
                .get_features()
                .await
                .map_err(|e| format!("Failed to get features: {}", e))?;
            
            // Create a serializable version of features
            let features_json = serde_json::json!({
                "version": format!("{}.{}.{}", 
                    features.major_version.unwrap_or(0), 
                    features.minor_version.unwrap_or(0), 
                    features.patch_version.unwrap_or(0)),
                "initialized": features.initialized.unwrap_or(false),
                "label": features.label.unwrap_or_default(),
                "vendor": features.vendor.unwrap_or_default(),
                "model": features.model.unwrap_or_default(),
                "bootloader_mode": features.bootloader_mode.unwrap_or(false)
            });
            
            Ok(features_json.to_string())
        }
        DeviceRequest::SignTransaction { ref coin, ref inputs, ref outputs, version, lock_time } => {
            // Convert our structured inputs to the format expected by keepkey_rust
            let mut keepkey_inputs = Vec::new();
            for input in inputs {
                let keepkey_input = keepkey_rust::messages::TxInputType {
                    address_n: input.address_n_list.clone(),
                    prev_hash: hex::decode(&input.txid).map_err(|e| format!("Invalid txid hex: {}", e))?,
                    prev_index: input.vout,
                    script_sig: None, // Will be filled by device
                    sequence: Some(0xffffffff), // Default sequence
                    script_type: match input.script_type.as_str() {
                        "p2pkh" => Some(keepkey_rust::messages::InputScriptType::Spendaddress as i32),
                        "p2sh" => Some(keepkey_rust::messages::InputScriptType::Spendp2shwitness as i32),
                        "p2wpkh" => Some(keepkey_rust::messages::InputScriptType::Spendwitness as i32),
                        _ => Some(keepkey_rust::messages::InputScriptType::Spendaddress as i32),
                    },
                    amount: Some(input.amount.parse::<u64>().map_err(|_| "Invalid amount")?),
                    ..Default::default()
                };
                keepkey_inputs.push(keepkey_input);
            }

            // Convert outputs
            let mut keepkey_outputs = Vec::new();
            for output in outputs {
                let keepkey_output = if output.address_type == "change" {
                    // For change outputs, we need addressNList instead of address
                    // TODO: Calculate and set proper change address derivation path (address_n)
                    // WARNING: Leaving this as an empty vec will result in invalid change outputs!
                    keepkey_rust::messages::TxOutputType {
                        amount: output.amount,
                        script_type: keepkey_rust::messages::OutputScriptType::Paytoaddress as i32,
                        address_n: vec![], // FIXME: Must set correct derivation path
                        ..Default::default()
                    }
                } else {
                    // For spend outputs, use the address
                    keepkey_rust::messages::TxOutputType {
                        address: Some(output.address.clone()),
                        amount: output.amount,
                        script_type: keepkey_rust::messages::OutputScriptType::Paytoaddress as i32,
                        ..Default::default()
                    }
                };
                keepkey_outputs.push(keepkey_output);
            }

            // Create SignTx message
            let sign_tx = keepkey_rust::messages::Message::SignTx(
                keepkey_rust::messages::SignTx {
                    coin_name: Some(coin.clone()),
                    inputs_count: keepkey_inputs.len() as u32,
                    outputs_count: keepkey_outputs.len() as u32,
                    version: Some(version),
                    lock_time: Some(lock_time),
                    ..Default::default()
                }
            );

            // Sign the transaction through the device queue
            // There is no sign_transaction method; use send_raw instead
            match queue_handle.send_raw(sign_tx, false).await {
                Ok(message) => {
                    // Try to extract signed tx string from Message
                    let signed_tx_hex = match message {
                        keepkey_rust::messages::Message::TxAck(ref tx_ack) => {
                            // Use Debug format for TransactionType since it doesn't implement Serialize
                            format!("{:?}", tx_ack.tx)
                        },
                        _ => {
                            format!("Unexpected response: {:?}", message)
                        }
                    };
                    println!("✅ Transaction signed successfully: {:?}", signed_tx_hex);
                    Ok(signed_tx_hex)
                }
                Err(e) => {
                    println!("❌ Failed to sign transaction: {}", e);
                    Err(format!("Failed to sign transaction: {}", e))
                }
            }
        }
        DeviceRequest::SendRaw { ref message_type, ref message_data } => {
            // Log the raw message being sent
            if let Err(e) = crate::logging::log_raw_device_message(
                &request.device_id,
                "SEND",
                &message_type,
                &message_data
            ).await {
                eprintln!("Failed to log raw device message: {}", e);
            }
            
            // For raw messages, we'd need to implement proper message parsing
            Err("Raw message sending not yet implemented".to_string())
        }
    };
    
    // Create and store the response
    let device_response = match (&request.request, &result) {
        (DeviceRequest::GetXpub { path }, Ok(ref xpub)) => {
            // Infer script_type from path
            let script_type = if path.starts_with("m/44'") {
                Some("p2pkh".to_string())
            } else if path.starts_with("m/49'") {
                Some("p2sh-p2wpkh".to_string())
            } else if path.starts_with("m/84'") {
                Some("p2wpkh".to_string())
            } else {
                None
            };
            // Debug logging for xpub conversion
            println!("[slip132-debug] Original xpub: {}", xpub);
            println!("[slip132-debug] Inferred script_type: {:?}", script_type);
            // Convert xpub prefix if possible
            let converted_xpub = if let Some(ref st) = script_type {
                match crate::slip132::convert_xpub_prefix(&xpub, st) {
                    Ok(res) => {
                        println!("[slip132-debug] Converted xpub: {}", res);
                        res
                    },
                    Err(e) => {
                        eprintln!("[slip132] Failed to convert xpub prefix: {}", e);
                        xpub.to_string()
                    }
                }
            } else {
                xpub.to_string()
            };
            DeviceResponse::Xpub {
                request_id: request.request_id.clone(),
                device_id: request.device_id.clone(),
                path: path.clone(),
                xpub: converted_xpub,
                script_type,
                success: true,
                error: None,
            }
        }
        (DeviceRequest::GetXpub { path }, Err(e)) => {
            // Infer script_type from path for error case as well
            let script_type = if path.starts_with("m/44'") {
                Some("p2pkh".to_string())
            } else if path.starts_with("m/49'") {
                Some("p2sh-p2wpkh".to_string())
            } else if path.starts_with("m/84'") {
                Some("p2wpkh".to_string())
            } else {
                None
            };
            DeviceResponse::Xpub {
                request_id: request.request_id.clone(),
                device_id: request.device_id.clone(),
                path: path.clone(),
                xpub: String::new(),
                script_type,
                success: false,
                error: Some(e.clone()),
            }
        }
        (DeviceRequest::GetAddress { path, .. }, Ok(ref address)) => {
            DeviceResponse::Address {
                request_id: request.request_id.clone(),
                device_id: request.device_id.clone(),
                path: path.clone(),
                address: address.clone(),
                success: true,
                error: None,
            }
        }
        (DeviceRequest::GetAddress { path, .. }, Err(e)) => {
            DeviceResponse::Address {
                request_id: request.request_id.clone(),
                device_id: request.device_id.clone(),
                path: path.clone(),
                address: String::new(),
                success: false,
                error: Some(e.clone()),
            }
        }
        (DeviceRequest::SignTransaction { .. }, Ok(ref signed_tx)) => {
            DeviceResponse::SignedTransaction {
                request_id: request.request_id.clone(),
                device_id: request.device_id.clone(),
                signed_tx: signed_tx.clone(),
                txid: None, // TODO: Calculate txid from signed transaction if needed (see v1 implementation)
                // WARNING: If txid is required by the consumer, port calculation logic from v1
                success: true,
                error: None,
            }
        }
        (DeviceRequest::SignTransaction { .. }, Err(e)) => {
            DeviceResponse::SignedTransaction {
                request_id: request.request_id.clone(),
                device_id: request.device_id.clone(),
                signed_tx: String::new(),
                txid: None,
                success: false,
                error: Some(e.clone()),
            }
        }
        _ => {
            // For other request types, create a generic raw response
            DeviceResponse::Raw {
                request_id: request.request_id.clone(),
                device_id: request.device_id.clone(),
                response: match &result {
                    Ok(resp) => serde_json::json!({"response": resp}),
                    Err(e) => serde_json::json!({"error": e}),
                },
                success: result.is_ok(),
                error: result.as_ref().err().cloned(),
            }
        }
    };
    
    // Store the response for queue status queries
    {
        let mut responses = last_responses.lock().await;
        responses.insert(request.device_id.clone(), device_response.clone());
    }
    
    // Emit event to frontend with the response
    let event_payload = serde_json::json!({
        "device_id": request.device_id,
        "request_id": request.request_id,
        "response": device_response
    });
    
    if let Err(e) = app.emit("device:response", &event_payload) {
        eprintln!("Failed to emit device:response event: {}", e);
    } else {
        println!("📡 Emitted device:response event for request {}", request.request_id);
    }
    
    // Log the response
    match &result {
        Ok(response) => {
            println!("✅ Device operation completed: {}", response);
            
            let response_data = serde_json::json!({
                "response": response,
                "request_type": request_type
            });
            
            if let Err(e) = crate::logging::log_device_response(
                &request.device_id,
                &request.request_id,
                true,
                &response_data,
                None
            ).await {
                eprintln!("Failed to log device response: {}", e);
            }
            
            Ok(request.request_id)
        }
        Err(e) => {
            println!("❌ Device operation failed: {}", e);
            
            let error_data = serde_json::json!({
                "error": e,
                "request_type": request_type
            });
            
            if let Err(log_err) = crate::logging::log_device_response(
                &request.device_id,
                &request.request_id,
                false,
                &error_data,
                Some(e)
            ).await {
                eprintln!("Failed to log device error response: {}", log_err);
            }
            
            Err(e.to_string())
        }
    }
}
