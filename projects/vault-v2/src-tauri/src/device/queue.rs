use tauri::{State, AppHandle, Emitter};
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;


// Import types needed for DeviceRequestWrapper
use crate::commands::{DeviceRequestWrapper, DeviceRequest, DeviceResponse, DeviceQueueManager, parse_transaction_from_hex};

// PIN Request tracking for new event-driven PIN flow
#[derive(Clone, Debug)]
pub struct PendingPinRequest {
    pub request_id: String,
    pub device_id: String,
    pub original_request: DeviceRequestWrapper,
    pub session_id: String,
    pub pin_request_type: String,
}

// PIN-aware message handler for emitting events instead of auto-responding
fn pin_aware_message_handler(msg: &keepkey_rust::messages::Message, app_handle: &tauri::AppHandle, device_id: &str, request_id: &str) -> anyhow::Result<Option<keepkey_rust::messages::Message>> {
    match msg {
        keepkey_rust::messages::Message::PinMatrixRequest(pin_req) => {
            // Don't auto-respond to PIN requests - emit an event instead
            let pin_type = match pin_req.r#type {
                Some(t) => match keepkey_rust::messages::PinMatrixRequestType::from_i32(t) {
                    Some(keepkey_rust::messages::PinMatrixRequestType::Current) => "current",
                    Some(keepkey_rust::messages::PinMatrixRequestType::NewFirst) => "new_first", 
                    Some(keepkey_rust::messages::PinMatrixRequestType::NewSecond) => "new_second",
                    _ => "unknown"
                },
                None => "unknown"
            };
            
            let session_id = format!("pin_request_{}_{}", device_id, std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis());
            
            let pin_event_payload = serde_json::json!({
                "device_id": device_id,
                "request_id": request_id,
                "session_id": session_id,
                "pin_type": pin_type,
                "message": match pin_type {
                    "current" => "Enter your current PIN",
                    "new_first" => "Enter a new PIN",
                    "new_second" => "Re-enter the new PIN",
                    _ => "Enter PIN"
                }
            });
            
            println!("📡 Emitting device:pin-request event: {:?}", pin_event_payload);
            
            if let Err(e) = app_handle.emit("device:pin-request", &pin_event_payload) {
                eprintln!("Failed to emit PIN request event: {}", e);
            }
            
            // Store this as a pending request
            let pending_request = PendingPinRequest {
                request_id: request_id.to_string(),
                device_id: device_id.to_string(),
                original_request: DeviceRequestWrapper {
                    device_id: device_id.to_string(),
                    request_id: request_id.to_string(),
                    request: DeviceRequest::GetFeatures, // Placeholder
                },
                session_id: session_id.clone(),
                pin_request_type: pin_type.to_string(),
            };
            
            tokio::spawn(async move {
                let mut pending_pins = PENDING_PIN_REQUESTS.lock().await;
                pending_pins.insert(session_id.clone(), pending_request);
                println!("📋 Stored pending PIN request session: {}", session_id);
            });
            
            // Return None to stop the message flow - frontend will respond
            Ok(None)
        }
        _ => {
            // For other messages, use default handling
            Ok(None)
        }
    }
}

// Global tracking for pending PIN requests
lazy_static::lazy_static! {
    pub static ref PENDING_PIN_REQUESTS: Arc<tokio::sync::Mutex<HashMap<String, PendingPinRequest>>> = Arc::new(tokio::sync::Mutex::new(HashMap::new()));
}

// Create a cache for device states to remember OOB bootloader status
lazy_static::lazy_static! {
    static ref DEVICE_STATE_CACHE: Arc<RwLock<HashMap<String, DeviceStateCache>>> = Arc::new(RwLock::new(HashMap::new()));
}

#[derive(Debug, Clone)]
struct DeviceStateCache {
    is_oob_bootloader: bool,
    last_features: Option<keepkey_rust::messages::Features>,
    last_update: std::time::Instant,
}

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

    // --------------------------------------------------------------
    // Get or create (and cache) the per-device queue handle
    // --------------------------------------------------------------
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

    // ------------------------------------------------------------------
    // --------------------------------------------------------------
    // Pre-flight status check – ensure the device can service this request
    // ------------------------------------------------------------------
    // We fetch the current features via the queue (which opens a temporary
    // transport) so that we have accurate mode/version information.
    let raw_features_opt = match keepkey_rust::device_queue::DeviceQueueHandle::get_features(&queue_handle).await {
        Ok(f) => {
            // Successfully got features, update cache
            let mut cache = DEVICE_STATE_CACHE.write().await;
            cache.insert(request.device_id.clone(), DeviceStateCache {
                is_oob_bootloader: false,
                last_features: Some(f.clone()),
                last_update: std::time::Instant::now(),
            });
            Some(f)
        },
        Err(e) => {
            eprintln!("⚠️  Unable to fetch features for status check: {e}");
            
            // Check if we have cached state for this device
            let cache = DEVICE_STATE_CACHE.read().await;
            if let Some(cached_state) = cache.get(&request.device_id) {
                // If we know this is an OOB bootloader from a previous successful check
                if cached_state.is_oob_bootloader {
                    println!("📋 Using cached OOB bootloader state for device {}", request.device_id);
                    cached_state.last_features.clone()
                } else {
                    None
                }
            } else {
                None
            }
        }
    };

    let status = if let Some(raw) = &raw_features_opt {
        // Convert to the simplified struct used by the evaluator
        let converted = crate::commands::convert_features_to_device_features(raw.clone());
        crate::commands::evaluate_device_status(request.device_id.clone(), Some(&converted))
    } else {
        // Fallback – we couldn't grab features, assume unknown status
        crate::commands::evaluate_device_status(request.device_id.clone(), None)
    };

    // Special handling for devices that might be in OOB bootloader mode
    let is_likely_oob_bootloader = raw_features_opt.is_none() && request_type != "GetFeatures";
    
    // If we couldn't fetch features AND this isn't a plain GetFeatures request
    if raw_features_opt.is_none() && request_type != "GetFeatures" {
        // Check if the device was successfully detected during initial connection
        // (which would have used Initialize for OOB bootloaders)
        let devices = keepkey_rust::features::list_connected_devices();
        let device_exists = devices.iter().any(|d| d.unique_id == request.device_id);
        
        if device_exists {
            println!("🔧 Device {} exists but GetFeatures failed - likely OOB bootloader, allowing request to proceed", request.device_id);
            // Mark this device as OOB bootloader in cache
            let mut cache = DEVICE_STATE_CACHE.write().await;
            cache.insert(request.device_id.clone(), DeviceStateCache {
                is_oob_bootloader: true,
                last_features: None,
                last_update: std::time::Instant::now(),
            });
        } else {
            println!("🚫 Rejecting {request_type} request – unable to determine device state (features fetch failed)");
            return Err("Device state unknown – cannot service request until communication succeeds.".to_string());
        }
    }

    // Check if device is in interactive PIN flow and block other operations
    if crate::commands::is_device_in_pin_flow(&request.device_id) {
        println!("🚫 Rejecting {request_type} request – device is in PIN creation flow");
        return Err("Device is currently in PIN creation flow. Please complete PIN setup before making other requests.".to_string());
    }

    // Only block requests if we have confirmed the device needs updates
    // Don't block if we simply can't determine the state (OOB bootloader case)
    if raw_features_opt.is_some() && (status.needs_bootloader_update || status.needs_firmware_update || status.needs_initialization) {
        let mut reasons = Vec::new();
        if status.needs_bootloader_update { reasons.push("bootloader update"); }
        if status.needs_firmware_update { reasons.push("firmware update"); }
        if status.needs_initialization   { reasons.push("initialization"); }
        let reason_str = reasons.join(", ");
        println!("🚫 Rejecting {request_type} request – device requires {reason_str}");
        return Err(format!("Device cannot process requests until {} is completed.", reason_str));
    }


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
            
            // Create GetAddress message with PIN-aware handling
            let get_address_msg = keepkey_rust::messages::Message::GetAddress(
                keepkey_rust::messages::GetAddress {
                    address_n: path_parts,
                    coin_name: Some(coin_name.clone()),
                    script_type: script_type_int,
                    show_display,
                    multisig: None,
                }
            );
            
            // Use send_raw to get address (now uses PIN flow handler that doesn't auto-respond)
            match queue_handle.send_raw(get_address_msg, false).await {
                Ok(keepkey_rust::messages::Message::Address(addr)) => {
                    Ok(addr.address)
                }
                Ok(keepkey_rust::messages::Message::PinMatrixRequest(pin_req)) => {
                    // Handle PIN request by emitting event
                    let pin_type = match pin_req.r#type {
                        Some(t) => match keepkey_rust::messages::PinMatrixRequestType::from_i32(t) {
                            Some(keepkey_rust::messages::PinMatrixRequestType::Current) => "current",
                            Some(keepkey_rust::messages::PinMatrixRequestType::NewFirst) => "new_first", 
                            Some(keepkey_rust::messages::PinMatrixRequestType::NewSecond) => "new_second",
                            _ => "unknown"
                        },
                        None => "unknown"
                    };
                    
                    let session_id = format!("pin_request_{}_{}", request.device_id, std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis());
                    
                    let pin_event_payload = serde_json::json!({
                        "device_id": request.device_id,
                        "request_id": request.request_id,
                        "session_id": session_id,
                        "pin_type": pin_type,
                        "message": match pin_type {
                            "current" => "Enter your current PIN",
                            "new_first" => "Enter a new PIN",
                            "new_second" => "Re-enter the new PIN",
                            _ => "Enter PIN"
                        }
                    });
                    
                    println!("📡 Emitting device:pin-request event: {:?}", pin_event_payload);
                    
                    if let Err(e) = app.emit("device:pin-request", &pin_event_payload) {
                        eprintln!("Failed to emit PIN request event: {}", e);
                    }
                    
                    // Store the pending request
                    let pending_request = PendingPinRequest {
                        request_id: request.request_id.clone(),
                        device_id: request.device_id.clone(),
                        original_request: request.clone(),
                        session_id: session_id.clone(),
                        pin_request_type: pin_type.to_string(),
                    };
                    
                    tokio::spawn(async move {
                        let mut pending_pins = PENDING_PIN_REQUESTS.lock().await;
                        pending_pins.insert(session_id.clone(), pending_request);
                        println!("📋 Stored pending PIN request session: {}", session_id);
                    });
                    
                    // Return a special error that indicates PIN is required
                    Err("PIN required - please enter PIN in dialog".to_string())
                }
                Ok(keepkey_rust::messages::Message::Failure(failure)) => {
                    Err(format!("Device returned error: {}", failure.message.unwrap_or_default()))
                }
                Ok(_) => {
                    Err("Unexpected response from device for address request".to_string())
                }
                Err(e) => {
                    Err(format!("Failed to get address: {}", e))
                }
            }
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
            // Build transaction map with previous transactions and unsigned transaction
            let mut tx_map = std::collections::HashMap::new();
            
            // Cache previous transactions
            for (idx, input) in inputs.iter().enumerate() {
                if let Some(hex_data) = &input.prev_tx_hex {
                    let tx_hash = hex::decode(&input.txid).map_err(|e| format!("Invalid txid hex: {}", e))?;
                    let tx_hash_hex = hex::encode(&tx_hash);
                    
                    // Parse the previous transaction from hex
                    match parse_transaction_from_hex(hex_data) {
                        Ok((metadata, tx_inputs, tx_outputs)) => {
                            let tx = keepkey_rust::messages::TransactionType {
                                version: Some(metadata.0),
                                lock_time: Some(metadata.3),
                                inputs_cnt: Some(metadata.1),
                                outputs_cnt: Some(metadata.2),
                                inputs: tx_inputs,
                                bin_outputs: tx_outputs,
                                outputs: vec![],
                                extra_data: None,
                                extra_data_len: Some(0),
                                ..Default::default()
                            };
                            tx_map.insert(tx_hash_hex.clone(), tx);
                            println!("✅ Cached previous transaction: {} (v{}, {} inputs, {} outputs)", 
                                   tx_hash_hex, metadata.0, metadata.1, metadata.2);
                        }
                        Err(e) => {
                            eprintln!("⚠️ Failed to parse previous transaction for input {}: {}", idx, e);
                            return Err(format!("Failed to parse previous transaction for input {}: {}", idx, e));
                        }
                    }
                } else {
                    return Err(format!("Input {} missing previous transaction hex", idx));
                }
            }

            // Build the unsigned transaction
            let mut new_tx_inputs = Vec::new();
            for input in inputs {
                let script_type = match input.script_type.as_str() {
                    "p2pkh" => keepkey_rust::messages::InputScriptType::Spendaddress,
                    "p2sh-p2wpkh" => keepkey_rust::messages::InputScriptType::Spendp2shwitness,
                    "p2wpkh" => keepkey_rust::messages::InputScriptType::Spendwitness,
                    _ => keepkey_rust::messages::InputScriptType::Spendaddress,
                };

                new_tx_inputs.push(keepkey_rust::messages::TxInputType {
                    address_n: input.address_n_list.clone(),
                    prev_hash: hex::decode(&input.txid).map_err(|e| format!("Invalid txid hex: {}", e))?,
                    prev_index: input.vout,
                    script_sig: None,
                    sequence: Some(0xffffffff),
                    script_type: Some(script_type as i32),
                    amount: Some(input.amount.parse::<u64>().map_err(|_| "Invalid amount")?),
                    ..Default::default()
                });
            }

            let mut new_tx_outputs = Vec::new();
            for output in outputs {
                let script_type = match output.address_type.as_str() {
                    "change" => {
                        // For change outputs, use address_n and appropriate script type
                        match output.script_type.as_deref().unwrap_or("p2pkh") {
                            "p2pkh" => keepkey_rust::messages::OutputScriptType::Paytoaddress,
                            "p2sh" => keepkey_rust::messages::OutputScriptType::Paytoscripthash,
                            "p2wpkh" => keepkey_rust::messages::OutputScriptType::Paytowitness,
                            _ => keepkey_rust::messages::OutputScriptType::Paytoaddress,
                        }
                    },
                    _ => {
                        // For spend outputs
                        keepkey_rust::messages::OutputScriptType::Paytoaddress
                    }
                };

                new_tx_outputs.push(keepkey_rust::messages::TxOutputType {
                    address: if output.address_type == "change" { None } else { Some(output.address.clone()) },
                    address_n: if output.address_type == "change" { 
                        output.address_n_list.clone().unwrap_or_default() 
                    } else { 
                        vec![] 
                    },
                    amount: output.amount,
                    script_type: script_type as i32,
                    address_type: Some(if output.address_type == "change" {
                        keepkey_rust::messages::OutputAddressType::Change as i32
                    } else {
                        keepkey_rust::messages::OutputAddressType::Spend as i32
                    }),
                    ..Default::default()
                });
            }

            let unsigned_tx = keepkey_rust::messages::TransactionType {
                version: Some(version),
                lock_time: Some(lock_time),
                inputs_cnt: Some(inputs.len() as u32),
                outputs_cnt: Some(outputs.len() as u32),
                inputs: new_tx_inputs,
                bin_outputs: vec![],
                outputs: new_tx_outputs,
                extra_data: None,
                extra_data_len: Some(0),
                ..Default::default()
            };

            tx_map.insert("unsigned".to_string(), unsigned_tx);

            // Start the Bitcoin signing protocol
            let sign_tx = keepkey_rust::messages::Message::SignTx(
                keepkey_rust::messages::SignTx {
                    coin_name: Some(coin.clone()),
                    inputs_count: inputs.len() as u32,
                    outputs_count: outputs.len() as u32,
                    version: Some(version),
                    lock_time: Some(lock_time),
                    ..Default::default()
                }
            );

            println!("📤 Sending SignTx message to device");
            
            // Execute the signing protocol
            let mut current_message = sign_tx;
            let mut signatures = Vec::new();
            let mut serialized_tx_parts = Vec::new();
            
            let signing_result = loop {
                let response = queue_handle.send_raw(current_message, false).await
                    .map_err(|e| format!("Device communication error: {}", e))?;
                
                match response {
                    keepkey_rust::messages::Message::TxRequest(tx_req) => {
                        // Handle serialized data if present
                        if let Some(serialized) = &tx_req.serialized {
                            if let Some(serialized_tx) = &serialized.serialized_tx {
                                serialized_tx_parts.push(serialized_tx.clone());
                            }
                            if let Some(signature) = &serialized.signature {
                                if let Some(sig_index) = serialized.signature_index {
                                    signatures.push((sig_index, hex::encode(signature)));
                                }
                            }
                        }
                        
                        // Handle the transaction request
                        match handle_tx_request(tx_req, &tx_map) {
                            Ok(Some(next_msg)) => current_message = next_msg,
                            Ok(None) => {
                                // Transaction finished
                                let mut serialized_tx = Vec::new();
                                for part in &serialized_tx_parts {
                                    serialized_tx.extend_from_slice(part);
                                }
                                
                                let signed_tx_hex = hex::encode(&serialized_tx);
                                
                                println!("✅ Transaction signed successfully!");
                                println!("   Signatures: {}", signatures.len());
                                println!("   Serialized TX: {} bytes", serialized_tx.len());
                                println!("📦 Raw Transaction Hex:");
                                println!("   {}", signed_tx_hex);
                                
                                // Log individual signatures
                                if !signatures.is_empty() {
                                    println!("📝 Individual Signatures:");
                                    for (idx, sig) in &signatures {
                                        println!("   Input {}: {}", idx, sig);
                                    }
                                }
                                
                                // Don't return early - let the function continue to response creation
                                break Ok(signed_tx_hex);
                            }
                            Err(e) => break Err(e),
                        }
                    }
                    keepkey_rust::messages::Message::Failure(failure) => {
                        let error = format!("Device returned error: {}", failure.message.unwrap_or_default());
                        println!("❌ Failed to sign transaction: {}", error);
                        break Err(error);
                    }
                    _ => {
                        let error = format!("Unexpected response from device: {:?}", response);
                        println!("❌ Failed to sign transaction: {}", error);
                        break Err(error);
                    }
                }
            };
            
            signing_result
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
            println!("🔐 Creating SignedTransaction DeviceResponse");
            println!("    request_id: {}", request.request_id);
            println!("    device_id: {}", request.device_id);
            println!("    signed_tx length: {}", signed_tx.len());
            println!("    signed_tx preview: {}...", if signed_tx.len() > 40 { &signed_tx[..40] } else { signed_tx });
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
        println!("🗄️ Inserting DeviceResponse into last_responses: device_id={}, request_id={}", request.device_id, request.request_id);
        responses.insert(request.request_id.clone(), device_response.clone());
    }
    
    // Emit event to frontend with the response
    let event_payload = serde_json::json!({
        "device_id": request.device_id,
        "request_id": request.request_id,
        "response": device_response
    });
    
    // EXPLICIT LOGGING FOR SIGNING EVENTS
    if let DeviceResponse::SignedTransaction { ref signed_tx, .. } = device_response {
        println!("🚀 Emitting SignedTransaction event to frontend!");
        println!("    device_id: {}", request.device_id);
        println!("    request_id: {}", request.request_id);
        println!("    signed_tx length: {}", signed_tx.len());
        println!("    event_payload: {}", serde_json::to_string_pretty(&event_payload).unwrap_or_else(|_| "failed to serialize".to_string()));
    }
    
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

/// Handle transaction request from device during Bitcoin signing protocol
fn handle_tx_request(
    tx_req: keepkey_rust::messages::TxRequest,
    tx_map: &std::collections::HashMap<String, keepkey_rust::messages::TransactionType>,
) -> Result<Option<keepkey_rust::messages::Message>, String> {
    // Extract transaction hash if provided
    let tx_hash_hex = if let Some(details) = &tx_req.details {
        if let Some(tx_hash) = &details.tx_hash {
            hex::encode(tx_hash)
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    // Handle request type
    match tx_req.request_type {
        Some(rt) if rt == keepkey_rust::messages::RequestType::Txinput as i32 => {
            let details = tx_req.details.as_ref()
                .ok_or_else(|| "Missing details in TXINPUT request".to_string())?;
            let req_index = details.request_index
                .ok_or_else(|| "Missing request_index".to_string())? as usize;
            
            let current_tx = if tx_hash_hex.is_empty() {
                tx_map.get("unsigned")
                    .ok_or_else(|| "Unsigned transaction not found".to_string())?
            } else {
                tx_map.get(&tx_hash_hex)
                    .ok_or_else(|| format!("Previous transaction {} not found", tx_hash_hex))?
            };
            
            let input = current_tx.inputs.get(req_index)
                .ok_or_else(|| format!("Input {} not found", req_index))?;
            
            let mut tx_ack_msg = keepkey_rust::messages::TransactionType::default();
            tx_ack_msg.inputs = vec![input.clone()];
            
            Ok(Some(keepkey_rust::messages::Message::TxAck(
                keepkey_rust::messages::TxAck { tx: Some(tx_ack_msg) }
            )))
        }
        Some(rt) if rt == keepkey_rust::messages::RequestType::Txoutput as i32 => {
            let details = tx_req.details.as_ref()
                .ok_or_else(|| "Missing details in TXOUTPUT request".to_string())?;
            let req_index = details.request_index
                .ok_or_else(|| "Missing request_index".to_string())? as usize;
            
            let current_tx = if tx_hash_hex.is_empty() {
                tx_map.get("unsigned")
                    .ok_or_else(|| "Unsigned transaction not found".to_string())?
            } else {
                tx_map.get(&tx_hash_hex)
                    .ok_or_else(|| format!("Previous transaction {} not found", tx_hash_hex))?
            };
            
            if tx_hash_hex.is_empty() {
                // For unsigned transaction, use outputs
                let output = current_tx.outputs.get(req_index)
                    .ok_or_else(|| format!("Output {} not found", req_index))?;
                
                let mut tx_ack_msg = keepkey_rust::messages::TransactionType::default();
                tx_ack_msg.outputs = vec![output.clone()];
                
                Ok(Some(keepkey_rust::messages::Message::TxAck(
                    keepkey_rust::messages::TxAck { tx: Some(tx_ack_msg) }
                )))
            } else {
                // For previous transactions, use bin_outputs
                let bin_output = current_tx.bin_outputs.get(req_index)
                    .ok_or_else(|| format!("Binary output {} not found", req_index))?;
                
                let mut tx_ack_msg = keepkey_rust::messages::TransactionType::default();
                tx_ack_msg.bin_outputs = vec![bin_output.clone()];
                
                Ok(Some(keepkey_rust::messages::Message::TxAck(
                    keepkey_rust::messages::TxAck { tx: Some(tx_ack_msg) }
                )))
            }
        }
        Some(rt) if rt == keepkey_rust::messages::RequestType::Txmeta as i32 => {
            let current_tx = if tx_hash_hex.is_empty() {
                tx_map.get("unsigned")
                    .ok_or_else(|| "Unsigned transaction not found".to_string())?
            } else {
                tx_map.get(&tx_hash_hex)
                    .ok_or_else(|| format!("Previous transaction {} not found", tx_hash_hex))?
            };
            
            let tx_meta = keepkey_rust::messages::TransactionType {
                version: current_tx.version,
                inputs_cnt: current_tx.inputs_cnt,
                outputs_cnt: if tx_hash_hex.is_empty() {
                    Some(current_tx.outputs.len() as u32)
                } else {
                    current_tx.outputs_cnt
                },
                lock_time: current_tx.lock_time,
                extra_data_len: current_tx.extra_data_len,
                inputs: vec![],
                bin_outputs: vec![],
                outputs: vec![],
                extra_data: None,
                expiry: current_tx.expiry,
                overwintered: current_tx.overwintered,
                version_group_id: current_tx.version_group_id,
                branch_id: current_tx.branch_id,
            };
            
            Ok(Some(keepkey_rust::messages::Message::TxAck(
                keepkey_rust::messages::TxAck { tx: Some(tx_meta) }
            )))
        }
        Some(rt) if rt == keepkey_rust::messages::RequestType::Txfinished as i32 => {
            Ok(None) // Signal completion
        }
        _ => Err(format!("Unknown request type: {:?}", tx_req.request_type)),
    }
}
