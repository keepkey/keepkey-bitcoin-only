use anyhow::{Result, anyhow};
use tracing::{info, error, warn};
use hex;
use std::collections::HashMap;
use std::time::Duration;

use crate::transport::{UsbTransport, ProtocolAdapter};
use crate::messages::{self, Message};
use crate::server::routes;

// Local timeout constant since the vault project doesn't have DEVICE_OPERATION_TIMEOUT
const DEVICE_OPERATION_TIMEOUT: Duration = Duration::from_secs(30);

// Helper function to get the first available KeepKey device
fn try_get_device() -> Result<rusb::Device<rusb::GlobalContext>, anyhow::Error> {
    const DEVICE_IDS: &[(u16, u16)] = &[(0x2b24, 0x0001), (0x2b24, 0x0002)];
    
    let devices = rusb::devices()?;
    for device in devices.iter() {
        let device_desc = device.device_descriptor()?;
        let vid = device_desc.vendor_id();
        let pid = device_desc.product_id();
        
        if DEVICE_IDS.contains(&(vid, pid)) {
            return Ok(device);
        }
    }
    Err(anyhow::anyhow!("No KeepKey device found"))
}

// Bitcoin transaction signing implementation
// pub(crate) async fn bitcoin_sign_tx_impl(state: &ServerState, request: routes::BitcoinSignRequest) -> Result<routes::BitcoinSignResponse> {
//     // SECURITY: No transaction data or signing-related information is ever persisted to disk.
//     // All transaction data exists only in memory for the duration of this signing operation
//     // and is cleared when the function returns.
//     
//     info!("🚀 Starting Bitcoin transaction signing");
//     info!("📋 Request: {} inputs, {} outputs", request.inputs.len(), request.outputs.len());
//     
//     // Wrap device communication in timeout
//     let result = timeout(DEVICE_OPERATION_TIMEOUT, async {
//         // Get a mutable reference to the active transport
//         let mut transport_guard = state.active_transport.lock().await;
//         let transport = transport_guard.as_mut().ok_or_else(|| {
//             error!("🚨 No active USB transport found in ServerState");
//             anyhow::anyhow!("No active USB transport available")
//         })?;
//         
//         // Create SignTx message to initiate Bitcoin signing
//         let sign_tx = messages::SignTx {
//             outputs_count: request.outputs.len() as u32,
//             inputs_count: request.inputs.len() as u32,
//             coin_name: Some("Bitcoin".to_string()),
//             version: Some(1),
//             lock_time: Some(0),
//             expiry: None,
//             overwintered: None,
//             version_group_id: None,
//             branch_id: None,
//         };
//         
//         // Track transaction state
//         let mut signatures = Vec::new();
//         let mut serialized_tx_parts = Vec::new();
//         
// 
//         
//         // Start the signing process
//         let mut current_message: Message = sign_tx.into();
//         
//         info!("📤 Sending SignTx message to device");
//         
//         loop {
//             let response = transport
//                 .with_standard_handler()
//                 .handle(current_message)?;
//             
//             match response {
//                 Message::TxRequest(tx_req) => {
//                     match tx_req.request_type {
//                         Some(rt) if rt == messages::RequestType::Txinput as i32 => {
//                             // Get the requested index from details
//                             let requested_index = if let Some(details) = &tx_req.details {
//                                 details.request_index.unwrap_or(0) as usize
//                             } else {
//                                 0
//                             };
//                             
//                             // Check if this is a request for a previous transaction input
//                             if let Some(details) = &tx_req.details {
//                                 if let Some(tx_hash) = &details.tx_hash {
//                                     // This is asking for an input from a previous transaction
//                                     let tx_hash_hex = hex::encode(tx_hash);
//                                     info!("📥 Device requesting input #{} from previous tx: {}", requested_index, tx_hash_hex);
//                                     
//                                     // For previous transaction inputs, we need to parse the actual transaction
//                                     // to get the input at the requested index.
//                                     // Find the corresponding input in our request.inputs that uses this prev_tx_hash
//                                     // to get the prev_tx_hex data.
//                                     let mut found_prev_tx_hex: Option<&String> = None;
//                                     for input_param in &request.inputs {
//                                         if hex::encode(&input_param.prev_hash) == tx_hash_hex {
//                                             found_prev_tx_hex = input_param.hex.as_ref();
//                                             break;
//                                         }
//                                     }
// 
//                                     if let Some(hex_data) = found_prev_tx_hex {
//                                         match parse_tx_input_from_hex(hex_data, requested_index) {
//                                             Ok((prev_hash_bytes, prev_index_val, script_sig_bytes, sequence_val)) => {
//                                                 info!("📊 Found previous input {}: prev_hash={}, prev_index={}",
//                                                     requested_index, hex::encode(&prev_hash_bytes), prev_index_val);
// 
//                                                 let tx_ack_input = messages::TxInputType {
//                                                     address_n: vec![], // Empty for previous tx input
//                                                     prev_hash: prev_hash_bytes.into(),
//                                                     prev_index: prev_index_val,
//                                                     script_sig: Some(script_sig_bytes.into()),
//                                                     sequence: Some(sequence_val),
//                                                     script_type: None, // Not needed for prev input
//                                                     multisig: None,    // Not needed for prev input
//                                                     amount: None,      // Amount is not part of prev tx input for TxAck
//                                                     decred_tree: None,
//                                                     decred_script_version: None,
//                                                 };
//                                                 current_message = messages::TxAck { tx: Some(messages::TransactionType {
//                                                     inputs: vec![tx_ack_input],
//                                                     ..Default::default() // Only sending the input
//                                                 })}.into();
//                                             }
//                                             Err(e) => {
//                                                 error!("🚨 Failed to parse previous transaction input {}: {}", requested_index, e);
//                                                 current_message = messages::Failure{ code: Some(messages::FailureType::FailureUnexpectedMessage as i32), message: Some(format!("Failed to parse prev tx input: {}", e)) }.into();
//                                             }
//                                         }
//                                     } else {
//                                         error!("🚨 Previous transaction hex not found in request for hash: {}", tx_hash_hex);
//                                         current_message = messages::Failure{ code: Some(messages::FailureType::FailureUnexpectedMessage as i32), message: Some(format!("Prev tx data not found for hash: {}", tx_hash_hex)) }.into();
//                                     }
//                                 } else {
//                                     // This is asking for an input from the new transaction being signed
//                                     info!("📥 Device requesting input #{}", requested_index);
//                                     
//                                     if requested_index >= request.inputs.len() {
//                                         return Err(anyhow::anyhow!("Device requested input {} out of range (have {} inputs)", 
//                                             requested_index, request.inputs.len()));
//                                     }
//                                     
//                                     let input = &request.inputs[requested_index];
//                                     
//                                     // Parse the prev_hash from hex string
//                                     let prev_hash = hex::decode(&input.prev_hash)?;
//                                     
//                                     // Create TxInputType
//                                     let tx_input = messages::TxInputType {
//                                         address_n: input.address_n.clone(),
//                                         prev_hash: prev_hash.into(),
//                                         prev_index: input.prev_index,
//                                         script_sig: None,
//                                         sequence: Some(0xffffffff),
//                                         script_type: Some(parse_bitcoin_input_script_type(&input.script_type)? as i32),
//                                         multisig: None,
//                                         amount: Some(input.amount.parse()?),
//                                         decred_tree: None,
//                                         decred_script_version: None,
//                                     };
//                                     
//                                     // Create TxAck with the input
//                                     let tx_ack = messages::TxAck {
//                                         tx: Some(messages::TransactionType {
//                                             version: None,
//                                             inputs: vec![tx_input],
//                                             bin_outputs: vec![],
//                                             outputs: vec![],
//                                             lock_time: None,
//                                             inputs_cnt: None,
//                                             outputs_cnt: None,
//                                             extra_data: None,
//                                             extra_data_len: None,
//                                             expiry: None,
//                                             overwintered: None,
//                                             version_group_id: None,
//                                             branch_id: None,
//                                         }),
//                                     };
//                                     
//                                     current_message = tx_ack.into();
//                                 }
//                             } else {
//                                 return Err(anyhow::anyhow!("TXINPUT request missing details"));
//                             }
//                         },
//                         Some(rt) if rt == messages::RequestType::Txoutput as i32 => {
//                             if let Some(details) = &tx_req.details {
//                                 let requested_index = details.request_index.unwrap_or(0) as usize;
//                                 
//                                 // Add detailed logging to understand the request
//                                 info!("📥 Device requesting output #{}", requested_index);
//                                 if let Some(tx_hash) = &details.tx_hash {
//                                     let tx_hash_hex = hex::encode(tx_hash);
//                                     info!("   🔍 TX Hash present: {} - This is a PREVIOUS transaction output request", tx_hash_hex);
//                                 } else {
//                                     info!("   📌 No TX Hash - This is a NEW transaction output request");
//                                 }
//                                 
//                                 // Check if this is a request for outputs from a previous transaction
//                                 if let Some(tx_hash) = &details.tx_hash {
//                                     // This is asking for an output from a previous transaction
//                                     let tx_hash_hex = hex::encode(tx_hash);
//                                     info!("📥 Device requesting output #{} from previous tx: {}", requested_index, tx_hash_hex);
//                                     
//                                     // Get the hex_data from the request.inputs that matches the tx_hash_hex
//                                     let mut found_prev_tx_hex: Option<&String> = None;
//                                     for input_param in &request.inputs {
//                                         // A bit indirect: an input to the *current* tx uses an output from a *previous* tx.
//                                         // The prev_hash of the *current* input is the hash of the *previous* tx.
//                                         if input_param.prev_hash == tx_hash_hex {
//                                             found_prev_tx_hex = input_param.hex.as_ref();
//                                             break;
//                                         }
//                                     }
// 
//                                     if let Some(hex_data) = found_prev_tx_hex {
//                                         match parse_tx_output_from_hex(hex_data, requested_index) {
//                                             Ok((amount, script_pubkey)) => {
//                                                 info!("📊 Found previous output {}: {} satoshis", requested_index, amount);
//                                                 let bin_output = messages::TxOutputBinType {
//                                                     amount,
//                                                     script_pubkey: script_pubkey.into(),
//                                                     decred_script_version: None, // Not applicable for standard BTC
//                                                 };
//                                                 current_message = messages::TxAck { tx: Some(messages::TransactionType {
//                                                     bin_outputs: vec![bin_output],
//                                                     ..Default::default() // Only sending this bin_output
//                                                 })}.into();
//                                             }
//                                             Err(e) => {
//                                                 error!("🚨 Failed to parse previous transaction output {}: {}", requested_index, e);
//                                                 current_message = messages::Failure{ code: Some(messages::FailureType::FailureUnexpectedMessage as i32), message: Some(format!("Failed to parse prev tx output: {}", e)) }.into();
//                                             }
//                                         }
//                                     } else {
//                                         error!("🚨 Previous transaction hex not found in request for hash: {}", tx_hash_hex);
//                                         current_message = messages::Failure{ code: Some(messages::FailureType::FailureUnexpectedMessage as i32), message: Some(format!("Prev tx data not found for hash: {}", tx_hash_hex)) }.into();
//                                     }
//                                 } else {
//                                     // This is asking for an output from the new transaction being signed
//                                     info!("📥 Device requesting output #{} for NEW transaction", requested_index);
//                                     info!("📊 We have {} outputs available in NEW transaction", request.outputs.len());
//                                     
//                                     if requested_index >= request.outputs.len() {
//                                         warn!("⚠️  Device requested output {} out of range (NEW transaction has {} outputs)", 
//                                             requested_index, request.outputs.len());
//                                         // Log all outputs for debugging
//                                         for (idx, out) in request.outputs.iter().enumerate() {
//                                             info!("   Output {}: {} satoshis to {:?}", idx, out.amount, out.address);
//                                         }
//                                         
//                                         // WORKAROUND: Send an empty output type to see what happens
//                                         info!("🔧 WORKAROUND: Sending empty TXOUTPUT response for out-of-range request");
//                                         
//                                         // Create an empty TxAck with no outputs
//                                         let tx_ack = messages::TxAck {
//                                             tx: Some(messages::TransactionType {
//                                                 version: None,
//                                                 inputs: vec![],
//                                                 bin_outputs: vec![],
//                                                 outputs: vec![], // Empty outputs array
//                                                 lock_time: None,
//                                                 inputs_cnt: None,
//                                                 outputs_cnt: Some(0), // Indicate 0 outputs
//                                                 extra_data: None,
//                                                 extra_data_len: None,
//                                                 expiry: None,
//                                                 overwintered: None,
//                                                 version_group_id: None,
//                                                 branch_id: None,
//                                             }),
//                                         };
//                                         
//                                         current_message = tx_ack.into();
//                                         continue;
//                                     }
//                                     
//                                     let output = &request.outputs[requested_index];
//                                     
//                                     // Create TxOutputType
//                                     let tx_output = messages::TxOutputType {
//                                         address: output.address.clone(),
//                                         address_n: output.address_n.clone().unwrap_or_default(),
//                                         amount: output.amount.parse()?,
//                                         script_type: parse_bitcoin_output_script_type(&output.script_type)? as i32,
//                                         multisig: None,
//                                         op_return_data: None,
//                                         address_type: None,
//                                         decred_script_version: None,
//                                     };
//                                     
//                                     // Create TxAck with the output
//                                     let tx_ack = messages::TxAck {
//                                         tx: Some(messages::TransactionType {
//                                             version: None,
//                                             inputs: vec![],
//                                             bin_outputs: vec![],
//                                             outputs: vec![tx_output],
//                                             lock_time: None,
//                                             inputs_cnt: None,
//                                             outputs_cnt: Some(1), // Set output count to 1 like hdwallet does
//                                             extra_data: None,
//                                             extra_data_len: None,
//                                             expiry: None,
//                                             overwintered: None,
//                                             version_group_id: None,
//                                             branch_id: None,
//                                         }),
//                                     };
//                                     
//                                     current_message = tx_ack.into();
//                                 }
//                             } else {
//                                 return Err(anyhow::anyhow!("TXOUTPUT request missing details"));
//                             }
//                         },
//                         Some(rt) if rt == messages::RequestType::Txmeta as i32 => {
//                             info!("📥 Device requesting transaction metadata");
//                             
//                             // The device wants metadata about a previous transaction
//                             // Check if the request has tx_hash to know which transaction
//                             if let Some(details) = &tx_req.details {
//                                 if let Some(tx_hash) = &details.tx_hash {
//                                     let tx_hash_hex = hex::encode(tx_hash);
//                                     info!("📋 Device wants metadata for tx: {}", tx_hash_hex);
//                                     
//                                     // Get the hex_data from the request.inputs that matches the tx_hash_hex
//                                     let mut found_prev_tx_hex: Option<&String> = None;
//                                     for input_param in &request.inputs {
//                                         // An input to the *current* tx uses an output from a *previous* tx.
//                                         // The prev_hash of the *current* input is the hash of the *previous* tx.
//                                         if input_param.prev_hash == tx_hash_hex {
//                                             found_prev_tx_hex = input_param.hex.as_ref();
//                                             break;
//                                         }
//                                     }
// 
//                                     if let Some(hex_data) = found_prev_tx_hex {
//                                         match parse_tx_metadata_from_hex(hex_data) {
//                                             Ok((version, lock_time, inputs_count, outputs_count)) => {
//                                                 info!("📋 Parsed metadata: v{}, lock_time={}, inputs={}, outputs={}", version, lock_time, inputs_count, outputs_count);
//                                                 current_message = messages::TxAck { tx: Some(messages::TransactionType {
//                                                     version: Some(version),
//                                                     lock_time: Some(lock_time),
//                                                     inputs_cnt: Some(inputs_count),
//                                                     outputs_cnt: Some(outputs_count),
//                                                     ..Default::default() // Only sending metadata fields
//                                                 })}.into();
//                                             }
//                                             Err(e) => {
//                                                 error!("🚨 Failed to parse previous transaction metadata: {}", e);
//                                                 current_message = messages::Failure{ code: Some(messages::FailureType::FailureUnexpectedMessage as i32), message: Some(format!("Failed to parse prev tx metadata: {}", e)) }.into();
//                                             }
//                                         }
//                                     } else {
//                                         error!("🚨 Previous transaction hex not found in request for hash: {}", tx_hash_hex);
//                                         current_message = messages::Failure{ code: Some(messages::FailureType::FailureUnexpectedMessage as i32), message: Some(format!("Prev tx data not found for hash: {}", tx_hash_hex)) }.into();
//                                     }
//                                 } else {
//                                     // No tx_hash means it wants metadata for the unsigned transaction
//                                     let tx_meta = messages::TransactionType {
//                                         version: Some(1),
//                                         inputs: vec![],
//                                         bin_outputs: vec![],
//                                         outputs: vec![],
//                                         lock_time: Some(0),
//                                         inputs_cnt: Some(request.inputs.len() as u32),
//                                         outputs_cnt: Some(request.outputs.len() as u32),
//                                         extra_data: None,
//                                         extra_data_len: Some(0),
//                                         expiry: None,
//                                         overwintered: None,
//                                         version_group_id: None,
//                                         branch_id: None,
//                                     };
//                                     
//                                     let tx_ack = messages::TxAck {
//                                         tx: Some(tx_meta),
//                                     };
//                                     
//                                     current_message = tx_ack.into();
//                                 }
//                             } else {
//                                 return Err(anyhow::anyhow!("TXMETA request missing details"));
//                             }
//                         },
//                         Some(rt) if rt == messages::RequestType::Txextradata as i32 => {
//                             info!("📥 Device requesting extra data");
//                             // For Bitcoin, there's typically no extra data
//                             let tx_ack = messages::TxAck {
//                                 tx: Some(messages::TransactionType {
//                                     version: None,
//                                     inputs: vec![],
//                                     bin_outputs: vec![],
//                                     outputs: vec![],
//                                     lock_time: None,
//                                     inputs_cnt: None,
//                                     outputs_cnt: None,
//                                     extra_data: Some(vec![].into()),
//                                     extra_data_len: None,
//                                     expiry: None,
//                                     overwintered: None,
//                                     version_group_id: None,
//                                     branch_id: None,
//                                 }),
//                             };
//                             current_message = tx_ack.into();
//                         },
//                         Some(rt) if rt == messages::RequestType::Txfinished as i32 => {
//                             info!("✅ Device finished signing transaction");
//                             
//                             // Collect the serialized transaction if provided
//                             if let Some(serialized) = &tx_req.serialized {
//                                 if let Some(serialized_tx) = &serialized.serialized_tx {
//                                     serialized_tx_parts.push(serialized_tx.clone());
//                                 }
//                                 if let Some(signature) = &serialized.signature {
//                                     if let Some(sig_index) = serialized.signature_index {
//                                         info!("📝 Got signature for input {}", sig_index);
//                                         signatures.push(hex::encode(signature));
//                                     }
//                                 }
//                             }
//                             
//                             // Combine all serialized parts
//                             let mut serialized_tx = Vec::new();
//                             for part in serialized_tx_parts {
//                                 serialized_tx.extend_from_slice(&part);
//                             }
//                             
//                             return Ok(routes::BitcoinSignResponse {
//                                 signatures,
//                                 serialized_tx: hex::encode(serialized_tx),
//                             });
//                         },
//                         _ => {
//                             error!("❌ Unknown request type: {:?}", tx_req.request_type);
//                             return Err(anyhow::anyhow!("Unknown request type from device"));
//                         }
//                     }
//                     
//                     // Check if we have serialized data in this response
//                     if let Some(serialized) = &tx_req.serialized {
//                         if let Some(serialized_tx) = &serialized.serialized_tx {
//                             serialized_tx_parts.push(serialized_tx.clone());
//                         }
//                         if let Some(signature) = &serialized.signature {
//                             if let Some(sig_index) = serialized.signature_index {
//                                 info!("📝 Got signature for input {}", sig_index);
//                                 signatures.push(hex::encode(signature));
//                             }
//                         }
//                     }
//                 },
//                 Message::Failure(failure) => {
//                     let error_msg = failure.message.unwrap_or_else(|| "Unknown error".to_string());
//                     error!("❌ Bitcoin signing failed: {}", error_msg);
//                     return Err(anyhow::anyhow!("Failure: {}", error_msg));
//                 },
//                 _ => {
//                     error!("❌ Unexpected message type during Bitcoin signing");
//                     return Err(anyhow::anyhow!("Unexpected message type"));
//                 }
//             }
//         }
//     }).await??;
//     
//     Ok(result)
// }
// 
// // Helper function to parse transaction from hex
fn parse_transaction_from_hex(hex_str: &str) -> Result<((u32, u32, u32, u32), Vec<messages::TxInputType>, Vec<messages::TxOutputBinType>)> {
    let tx_bytes = hex::decode(hex_str)?;
    let mut cursor = 0;
    
    // Version (4 bytes)
    if tx_bytes.len() < 4 {
        return Err(anyhow::anyhow!("Transaction too short for version"));
    }
    let version = u32::from_le_bytes([tx_bytes[0], tx_bytes[1], tx_bytes[2], tx_bytes[3]]);
    cursor += 4;
    
    // Check for witness marker (BIP144)
    let has_witness = if tx_bytes.len() > cursor + 1 && tx_bytes[cursor] == 0x00 && tx_bytes[cursor + 1] == 0x01 {
        cursor += 2; // Skip marker and flag
        true
    } else {
        false
    };
    
    // Input count (varint)
    let (input_count, bytes_read) = read_varint(&tx_bytes[cursor..])?;
    cursor += bytes_read;
    
    // Parse inputs
    let mut inputs = Vec::new();
    for i in 0..input_count {
        // Previous hash (32 bytes)
        if cursor + 32 > tx_bytes.len() {
            return Err(anyhow::anyhow!("Transaction too short for input {} prev_hash", i));
        }
        let mut prev_hash = tx_bytes[cursor..cursor + 32].to_vec();
        prev_hash.reverse(); // Bitcoin uses little-endian for hashes
        cursor += 32;
        
        // Previous index (4 bytes)
        if cursor + 4 > tx_bytes.len() {
            return Err(anyhow::anyhow!("Transaction too short for input {} prev_index", i));
        }
        let prev_index = u32::from_le_bytes([tx_bytes[cursor], tx_bytes[cursor + 1], tx_bytes[cursor + 2], tx_bytes[cursor + 3]]);
        cursor += 4;
        
        // Script length (varint)
        let (script_len, bytes_read) = read_varint(&tx_bytes[cursor..])?;
        cursor += bytes_read;
        
        // Script sig
        if cursor + script_len as usize > tx_bytes.len() {
            return Err(anyhow::anyhow!("Transaction too short for input {} script", i));
        }
        let script_sig = tx_bytes[cursor..cursor + script_len as usize].to_vec();
        cursor += script_len as usize;
        
        // Sequence (4 bytes)
        if cursor + 4 > tx_bytes.len() {
            return Err(anyhow::anyhow!("Transaction too short for input {} sequence", i));
        }
        let sequence = u32::from_le_bytes([tx_bytes[cursor], tx_bytes[cursor + 1], tx_bytes[cursor + 2], tx_bytes[cursor + 3]]);
        cursor += 4;
        
        inputs.push(messages::TxInputType {
            address_n: vec![],
            prev_hash,
            prev_index,
            script_sig: Some(script_sig),
            sequence: Some(sequence),
            script_type: Some(messages::InputScriptType::Spendaddress as i32),
            multisig: None,
            amount: None,
            decred_tree: None,
            decred_script_version: None,
        });
    }
    
    // Output count (varint)
    let (output_count, bytes_read) = read_varint(&tx_bytes[cursor..])?;
    cursor += bytes_read;
    
    // Parse outputs
    let mut outputs = Vec::new();
    for i in 0..output_count {
        // Amount (8 bytes)
        if cursor + 8 > tx_bytes.len() {
            return Err(anyhow::anyhow!("Transaction too short for output {} amount", i));
        }
        let amount = u64::from_le_bytes([
            tx_bytes[cursor], tx_bytes[cursor + 1], tx_bytes[cursor + 2], tx_bytes[cursor + 3],
            tx_bytes[cursor + 4], tx_bytes[cursor + 5], tx_bytes[cursor + 6], tx_bytes[cursor + 7]
        ]);
        cursor += 8;
        
        // Script length (varint)
        let (script_len, bytes_read) = read_varint(&tx_bytes[cursor..])?;
        cursor += bytes_read;
        
        // Script pubkey
        if cursor + script_len as usize > tx_bytes.len() {
            return Err(anyhow::anyhow!("Transaction too short for output {} script", i));
        }
        let script_pubkey = tx_bytes[cursor..cursor + script_len as usize].to_vec();
        cursor += script_len as usize;
        
        outputs.push(messages::TxOutputBinType {
            amount,
            script_pubkey,
            decred_script_version: None,
        });
    }
    
    // If this is a witness transaction, skip witness data
    if has_witness {
        // For each input, skip witness data
        for _ in 0..input_count {
            // Witness item count (varint)
            let (witness_count, bytes_read) = read_varint(&tx_bytes[cursor..])?;
            cursor += bytes_read;
            
            // Skip each witness item
            for _ in 0..witness_count {
                let (item_len, bytes_read) = read_varint(&tx_bytes[cursor..])?;
                cursor += bytes_read + item_len as usize;
            }
        }
    }
    
    // Lock time (4 bytes)
    if cursor + 4 > tx_bytes.len() {
        return Err(anyhow::anyhow!("Transaction too short for lock_time"));
    }
    let lock_time = u32::from_le_bytes([tx_bytes[cursor], tx_bytes[cursor + 1], tx_bytes[cursor + 2], tx_bytes[cursor + 3]]);
    
    Ok(((version, input_count as u32, output_count as u32, lock_time), inputs, outputs))
}

// Helper function to parse Bitcoin input script type
fn parse_bitcoin_input_script_type(script_type: &str) -> Result<messages::InputScriptType> {
    match script_type.to_lowercase().as_str() {
        "p2pkh" => Ok(messages::InputScriptType::Spendaddress),
        "p2sh" => Ok(messages::InputScriptType::Spendmultisig),
        "p2wpkh" => Ok(messages::InputScriptType::Spendwitness),
        "p2sh-p2wpkh" => Ok(messages::InputScriptType::Spendp2shwitness),
        "p2tr" => Err(anyhow::anyhow!("Taproot not supported")),
        _ => {
            warn!("Unknown input script type '{}', defaulting to p2pkh", script_type);
            Ok(messages::InputScriptType::Spendaddress)
        }
    }
}

// Helper function to parse Bitcoin output script type
fn parse_bitcoin_output_script_type(script_type: &str) -> Result<messages::OutputScriptType> {
    match script_type.to_lowercase().as_str() {
        "p2pkh" => Ok(messages::OutputScriptType::Paytoaddress),
        "p2wpkh" => Ok(messages::OutputScriptType::Paytowitness),
        "p2sh" => Ok(messages::OutputScriptType::Paytoscripthash),
        "p2sh-p2wpkh" => Ok(messages::OutputScriptType::Paytop2shwitness),
        "p2tr" => Err(anyhow::anyhow!("Taproot not supported")),
        "multisig" => Ok(messages::OutputScriptType::Paytomultisig),
        "op_return" => Ok(messages::OutputScriptType::Paytoopreturn),
        _ => {
            warn!("Unknown output script type '{}', defaulting to PAYTOADDRESS", script_type);
            Ok(messages::OutputScriptType::Paytoaddress)
        }
    }
}

// Helper function to parse basic transaction metadata from hex
fn parse_tx_metadata_from_hex(hex_str: &str) -> Result<(u32, u32, u32, u32)> {
    // Basic Bitcoin transaction parsing to extract version, input count, and output count
    // This is a simplified parser - in production you'd use a proper Bitcoin library
    
    let tx_bytes = hex::decode(hex_str)?;
    if tx_bytes.len() < 10 {
        return Err(anyhow::anyhow!("Transaction hex too short"));
    }
    
    let mut cursor = 0;
    
    // Read version (4 bytes, little-endian)
    let version = u32::from_le_bytes([
        tx_bytes[cursor],
        tx_bytes[cursor + 1],
        tx_bytes[cursor + 2],
        tx_bytes[cursor + 3],
    ]);
    cursor += 4;
    
    // Check for witness marker (BIP144)
    let has_witness = if tx_bytes.len() > cursor + 1 && tx_bytes[cursor] == 0x00 && tx_bytes[cursor + 1] == 0x01 {
        cursor += 2; // Skip marker and flag
        true
    } else {
        false
    };
    
    // Read input count (varint)
    let (input_count, bytes_read) = read_varint(&tx_bytes[cursor..])?;
    cursor += bytes_read;
    
    // Skip through inputs to find output count
    // Each input has: txid (32 bytes) + vout (4 bytes) + script_len (varint) + script + sequence (4 bytes)
    for _ in 0..input_count {
        cursor += 36; // txid + vout
        let (script_len, bytes_read) = read_varint(&tx_bytes[cursor..])?;
        cursor += bytes_read + script_len as usize + 4; // script + sequence
    }
    
    // Read output count (varint)
    let (output_count, bytes_read) = read_varint(&tx_bytes[cursor..])?;
    cursor += bytes_read;

    // Skip through outputs
    // Each output has: value (8 bytes) + script_len (varint) + script
    for _ in 0..output_count {
        cursor += 8; // value
        let (script_len, bytes_read) = read_varint(&tx_bytes[cursor..])?;
        cursor += bytes_read + script_len as usize; // script
    }

    // If this is a witness transaction, skip witness data before reading lock_time
    if has_witness {
        // For each input, skip witness data
        for _ in 0..input_count {
            // Witness item count (varint)
            let (witness_count, bytes_read) = read_varint(&tx_bytes[cursor..])?;
            cursor += bytes_read;
            
            // Skip each witness item
            for _ in 0..witness_count {
                let (item_len, bytes_read) = read_varint(&tx_bytes[cursor..])?;
                cursor += bytes_read + item_len as usize;
            }
        }
    }
    
    // Read lock_time (4 bytes, little-endian)
    if tx_bytes.len() < cursor + 4 {
        return Err(anyhow::anyhow!("Transaction hex too short to read lock_time"));
    }
    let lock_time = u32::from_le_bytes([
        tx_bytes[cursor],
        tx_bytes[cursor + 1],
        tx_bytes[cursor + 2],
        tx_bytes[cursor + 3],
    ]);

    Ok((version, lock_time, input_count as u32, output_count as u32))
}

// Helper function to read Bitcoin varint
fn read_varint(data: &[u8]) -> Result<(u64, usize)> {
    if data.is_empty() {
        return Err(anyhow::anyhow!("No data for varint"));
    }
    
    match data[0] {
        0..=252 => Ok((data[0] as u64, 1)),
        253 => {
            if data.len() < 3 {
                return Err(anyhow::anyhow!("Insufficient data for varint"));
            }
            Ok((u16::from_le_bytes([data[1], data[2]]) as u64, 3))
        }
        254 => {
            if data.len() < 5 {
                return Err(anyhow::anyhow!("Insufficient data for varint"));
            }
            Ok((u32::from_le_bytes([data[1], data[2], data[3], data[4]]) as u64, 5))
        }
        255 => {
            if data.len() < 9 {
                return Err(anyhow::anyhow!("Insufficient data for varint"));
            }
            Ok((u64::from_le_bytes([
                data[1], data[2], data[3], data[4],
                data[5], data[6], data[7], data[8]
            ]), 9))
        }
    }
}

// Helper function to parse a specific input from transaction hex
fn parse_tx_input_from_hex(hex_str: &str, input_index: usize) -> Result<(Vec<u8>, u32, Vec<u8>, u32)> {
    let tx_bytes = hex::decode(hex_str)?;
    
    if tx_bytes.len() < 10 {
        return Err(anyhow::anyhow!("Transaction hex too short"));
    }
    
    let mut cursor = 0;
    
    // Skip version (4 bytes)
    cursor += 4;
    
    // Check for witness marker (BIP144)
    if tx_bytes.len() > cursor + 1 && tx_bytes[cursor] == 0x00 && tx_bytes[cursor + 1] == 0x01 {
        cursor += 2; // Skip marker and flag
    }
    
    // Read input count
    let (input_count, varint_size) = read_varint(&tx_bytes[cursor..])?;
    cursor += varint_size;
    
    // Iterate through inputs to find the requested one
    for i in 0..input_count {
        // Read previous transaction hash (32 bytes)
        if cursor + 32 > tx_bytes.len() {
            return Err(anyhow::anyhow!("Unexpected end of transaction data while reading prev hash"));
        }
        let prev_hash = tx_bytes[cursor..cursor + 32].to_vec();
        cursor += 32;
        
        // Read previous output index (4 bytes)
        if cursor + 4 > tx_bytes.len() {
            return Err(anyhow::anyhow!("Unexpected end of transaction data while reading prev index"));
        }
        let prev_index = u32::from_le_bytes([
            tx_bytes[cursor],
            tx_bytes[cursor + 1],
            tx_bytes[cursor + 2],
            tx_bytes[cursor + 3],
        ]);
        cursor += 4;
        
        // Read script length
        let (script_len, varint_size) = read_varint(&tx_bytes[cursor..])?;
        cursor += varint_size;
        
        // Read script
        if cursor + script_len as usize > tx_bytes.len() {
            return Err(anyhow::anyhow!("Unexpected end of transaction data while reading script"));
        }
        let script_sig = tx_bytes[cursor..cursor + script_len as usize].to_vec();
        cursor += script_len as usize;
        
        // Read sequence (4 bytes)
        if cursor + 4 > tx_bytes.len() {
            return Err(anyhow::anyhow!("Unexpected end of transaction data while reading sequence"));
        }
        let sequence = u32::from_le_bytes([
            tx_bytes[cursor],
            tx_bytes[cursor + 1],
            tx_bytes[cursor + 2],
            tx_bytes[cursor + 3],
        ]);
        cursor += 4;
        
        // If this is the requested input, return it
        if i as usize == input_index {
            // Reverse the hash back to normal byte order (Bitcoin uses little-endian)
            let mut normal_hash = prev_hash.clone();
            normal_hash.reverse();
            return Ok((normal_hash, prev_index, script_sig, sequence));
        }
    }
    
    Err(anyhow::anyhow!("Input index {} not found in transaction (has {} inputs)", input_index, input_count))
}

// Helper function to parse a specific output from transaction hex
fn parse_tx_output_from_hex(hex_str: &str, output_index: usize) -> Result<(u64, Vec<u8>)> {
    let tx_bytes = hex::decode(hex_str)?;
    if tx_bytes.len() < 10 {
        return Err(anyhow::anyhow!("Transaction hex too short"));
    }
    
    let mut cursor = 0;
    
    // Read version (4 bytes, little-endian)
    cursor += 4;
    
    // Check for witness marker (BIP144)
    let has_witness = tx_bytes.len() > cursor + 1 && 
                     tx_bytes[cursor] == 0x00 && 
                     tx_bytes[cursor + 1] == 0x01;
    if has_witness {
        cursor += 2; // Skip marker and flag
    }
    
    // Read input count (varint)
    let (input_count, bytes_read) = read_varint(&tx_bytes[cursor..])?;
    cursor += bytes_read;
    
    // Skip through inputs
    for _ in 0..input_count {
        // Skip txid (32 bytes) + vout (4 bytes)
        cursor += 36;
        
        // Read and skip script length
        let (script_len, bytes_read) = read_varint(&tx_bytes[cursor..])?;
        cursor += bytes_read + script_len as usize;
        
        // Skip sequence (4 bytes)
        cursor += 4;
    }
    
    // Read output count
    let (output_count, bytes_read) = read_varint(&tx_bytes[cursor..])?;
    cursor += bytes_read;
    
    if output_index >= output_count as usize {
        return Err(anyhow::anyhow!("Output index {} out of range (tx has {} outputs)", 
                                   output_index, output_count));
    }
    
    // Skip to the requested output
    for i in 0..=output_index {
        if i < output_index {
            // Skip this output
            cursor += 8; // amount
            let (script_len, bytes_read) = read_varint(&tx_bytes[cursor..])?;
            cursor += bytes_read + script_len as usize;
        } else {
            // This is the output we want
            // Read amount (8 bytes, little-endian)
            if cursor + 8 > tx_bytes.len() {
                return Err(anyhow::anyhow!("Insufficient data for output amount"));
            }
            let amount = u64::from_le_bytes([
                tx_bytes[cursor], tx_bytes[cursor + 1], tx_bytes[cursor + 2], tx_bytes[cursor + 3],
                tx_bytes[cursor + 4], tx_bytes[cursor + 5], tx_bytes[cursor + 6], tx_bytes[cursor + 7],
            ]);
            cursor += 8;
            
            // Read script length
            let (script_len, bytes_read) = read_varint(&tx_bytes[cursor..])?;
            cursor += bytes_read;
            
            // Read script
            if cursor + script_len as usize > tx_bytes.len() {
                return Err(anyhow::anyhow!("Insufficient data for output script"));
            }
            let script = tx_bytes[cursor..cursor + script_len as usize].to_vec();
            
            return Ok((amount, script));
        }
    }
    
    Err(anyhow::anyhow!("Failed to parse output"))
}

// Bitcoin message signing
pub(crate) async fn bitcoin_sign_message_impl(_request: routes::BitcoinSignMessageRequest) -> Result<routes::BitcoinSignMessageResponse> {
    error!("Bitcoin sign message not implemented");
    Err(anyhow::anyhow!("Not implemented"))
}

// Bitcoin message verification
pub(crate) async fn bitcoin_verify_message_impl(_request: routes::BitcoinVerifyMessageRequest) -> Result<routes::BitcoinVerifyMessageResponse> {
    error!("Bitcoin verify message not implemented");
    Err(anyhow::anyhow!("Not implemented"))
}

// Add this new implementation that creates a fresh connection
pub async fn bitcoin_sign_tx_fresh_impl(
    request: routes::BitcoinSignRequest,
) -> Result<routes::BitcoinSignResponse> {
    info!("🚀 Starting Bitcoin transaction signing with FRESH connection");
    info!("📋 Request: {} inputs, {} outputs", request.inputs.len(), request.outputs.len());
    
    // Create a fresh USB connection (like the CLI does)
    // Get USB device first
    let device = try_get_device()?;
    
    // Create new transport (exactly like CLI does)
    let (transport, _config_descriptor, _handle) = UsbTransport::new(&device, 0)?;
    
    info!("✅ Created fresh USB connection");
    
    // Create mutable transport for signing
    let mut transport = transport;
    
    // Now run the same signing logic as the regular implementation
    // but using our fresh protocol_adapter
    
    // Build transaction metadata map
    let mut tx_map = HashMap::new();
    
    // Add previous transactions to the map
    for (idx, input) in request.inputs.iter().enumerate() {
        if let Some(hex_data) = &input.hex {
            let tx_hash = hex::decode(&input.prev_hash)?;
            let tx_hash_hex = hex::encode(&tx_hash);
            
            // Parse the previous transaction
            match parse_transaction_from_hex(hex_data) {
                Ok((metadata, inputs, outputs)) => {
                    let tx = messages::TransactionType {
                        version: Some(metadata.0),
                        lock_time: Some(metadata.3),
                        inputs_cnt: Some(metadata.1),
                        outputs_cnt: Some(metadata.2),
                        inputs,
                        bin_outputs: outputs,
                        outputs: vec![],
                        extra_data: None,
                        extra_data_len: Some(0),
                        expiry: None,
                        overwintered: None,
                        version_group_id: None,
                        branch_id: None,
                    };
                    tx_map.insert(tx_hash_hex.clone(), tx);
                    info!("   ✅ Cached previous transaction: {} (v{}, {} inputs, {} outputs)", 
                         tx_hash_hex, metadata.0, metadata.1, metadata.2);
                }
                Err(e) => {
                    warn!("   ⚠️  Failed to parse previous transaction for input {}: {}", idx, e);
                }
            }
        }
    }
    
    // Build the new transaction structure for tx_map["unsigned"]
    let mut new_tx_inputs = Vec::new();
    for input in &request.inputs {
        let script_type = match input.script_type.as_str() {
            "p2pkh" => messages::InputScriptType::Spendaddress,
            "p2sh-p2wpkh" => messages::InputScriptType::Spendp2shwitness,
            "p2wpkh" => messages::InputScriptType::Spendwitness,
            _ => messages::InputScriptType::Spendaddress,
        };
        
        new_tx_inputs.push(messages::TxInputType {
            address_n: input.address_n.clone(),
            prev_hash: hex::decode(&input.prev_hash)?,
            prev_index: input.prev_index,
            script_sig: None,
            sequence: Some(0xffffffff),
            script_type: Some(script_type as i32),
            multisig: None,
            amount: Some(input.amount.parse::<u64>()?),
            decred_tree: None,
            decred_script_version: None,
        });
    }
    
    let mut new_tx_outputs = Vec::new();
    for output in &request.outputs {
        let script_type = match output.script_type.as_str() {
            "p2pkh" => messages::OutputScriptType::Paytoaddress,
            "p2sh" => messages::OutputScriptType::Paytoscripthash,
            "p2wpkh" => messages::OutputScriptType::Paytowitness,
            _ => messages::OutputScriptType::Paytoaddress,
        };
        
        new_tx_outputs.push(messages::TxOutputType {
            address: output.address.clone(),
            address_n: output.address_n.clone().unwrap_or_default(),
            amount: output.amount.parse::<u64>()?,
            script_type: script_type as i32,
            multisig: None,
            op_return_data: None,
            address_type: Some(messages::OutputAddressType::Spend as i32),
            decred_script_version: None,
        });
    }
    
    let unsigned_tx = messages::TransactionType {
        version: Some(1),
        lock_time: Some(0),
        inputs_cnt: Some(request.inputs.len() as u32),
        outputs_cnt: Some(request.outputs.len() as u32),
        inputs: new_tx_inputs,
        bin_outputs: vec![],
        outputs: new_tx_outputs,
        extra_data: None,
        extra_data_len: Some(0),
        expiry: None,
        overwintered: None,
        version_group_id: None,
        branch_id: None,
    };
    
    tx_map.insert("unsigned".to_string(), unsigned_tx);
    
    // Create SignTx message
    let sign_tx = messages::SignTx {
        outputs_count: request.outputs.len() as u32,
        inputs_count: request.inputs.len() as u32,
        coin_name: Some("Bitcoin".to_string()),
        version: Some(1),
        lock_time: Some(0),
        expiry: None,
        overwintered: None,
        version_group_id: None,
        branch_id: None,
    };
    
    info!("📤 Sending SignTx message to device");
    
    // Start the signing flow
    let mut current_message = Message::SignTx(sign_tx);
    let mut signatures = Vec::new();
    let mut serialized_tx_parts = Vec::new();
    
    loop {
        let response = transport
            .with_standard_handler()
            .handle(current_message)?;
        
        match response {
            Message::TxRequest(tx_req) => {
                // Handle the transaction request (same logic as regular implementation)
                match handle_tx_request_for_fresh(tx_req, &tx_map, &mut signatures, &mut serialized_tx_parts) {
                    Ok(Some(next_msg)) => current_message = next_msg,
                    Ok(None) => {
                        // Transaction finished
                        let mut serialized_tx = Vec::new();
                        for part in &serialized_tx_parts {
                            serialized_tx.extend_from_slice(part);
                        }
                        
                        info!("✅ Transaction signed successfully with fresh connection!");
                        info!("   Signatures: {}", signatures.len());
                        info!("   Serialized TX: {} bytes", serialized_tx.len());
                        
                        return Ok(routes::BitcoinSignResponse {
                            signatures: signatures.into_iter().map(|(_, sig)| sig).collect(),
                            serialized_tx: hex::encode(serialized_tx),
                        });
                    }
                    Err(e) => return Err(e),
                }
            }
            Message::Failure(failure) => {
                return Err(anyhow!("Device failure: {:?}", failure));
            }
            _ => {
                return Err(anyhow!("Unexpected response: {:?}", response));
            }
        }
    }
}

// Helper function to handle TxRequest for fresh connection implementation
fn handle_tx_request_for_fresh(
    tx_req: messages::TxRequest,
    tx_map: &HashMap<String, messages::TransactionType>,
    signatures: &mut Vec<(u32, String)>,
    serialized_tx_parts: &mut Vec<Vec<u8>>,
) -> Result<Option<Message>> {
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
    
    // Handle request type
    match tx_req.request_type {
        Some(rt) if rt == messages::RequestType::Txinput as i32 => {
            let details = tx_req.details.as_ref()
                .ok_or_else(|| anyhow!("Missing details in TXINPUT request"))?;
            let req_index = details.request_index
                .ok_or_else(|| anyhow!("Missing request_index"))? as usize;
            
            let current_tx = if tx_hash_hex.is_empty() {
                tx_map.get("unsigned")
                    .ok_or_else(|| anyhow!("Unsigned transaction not found"))?
            } else {
                tx_map.get(&tx_hash_hex)
                    .ok_or_else(|| anyhow!("Previous transaction {} not found", tx_hash_hex))?
            };
            
            let input = current_tx.inputs.get(req_index)
                .ok_or_else(|| anyhow!("Input {} not found", req_index))?;
            
            let mut tx_ack_msg = messages::TransactionType::default();
            tx_ack_msg.inputs = vec![input.clone()];
            
            Ok(Some(Message::TxAck(messages::TxAck { tx: Some(tx_ack_msg) })))
        }
        Some(rt) if rt == messages::RequestType::Txoutput as i32 => {
            let details = tx_req.details.as_ref()
                .ok_or_else(|| anyhow!("Missing details in TXOUTPUT request"))?;
            let req_index = details.request_index
                .ok_or_else(|| anyhow!("Missing request_index"))? as usize;
            
            let current_tx = if tx_hash_hex.is_empty() {
                tx_map.get("unsigned")
                    .ok_or_else(|| anyhow!("Unsigned transaction not found"))?
            } else {
                tx_map.get(&tx_hash_hex)
                    .ok_or_else(|| anyhow!("Previous transaction {} not found", tx_hash_hex))?
            };
            
            let mut tx_ack_msg = messages::TransactionType::default();
            
            if tx_hash_hex.is_empty() {
                // New transaction - check if output exists
                if let Some(output) = current_tx.outputs.get(req_index) {
                    tx_ack_msg.outputs = vec![output.clone()];
                    tx_ack_msg.outputs_cnt = Some(1);
                } else {
                    // Output doesn't exist - send empty response
                    warn!("⚠️  Device requested output {} but only {} outputs exist", 
                          req_index, current_tx.outputs.len());
                    tx_ack_msg.outputs = vec![];
                    tx_ack_msg.outputs_cnt = Some(1);
                }
            } else {
                // Previous transaction
                let output = current_tx.bin_outputs.get(req_index)
                    .ok_or_else(|| anyhow!("Output {} not found in previous transaction", req_index))?;
                tx_ack_msg.bin_outputs = vec![output.clone()];
            }
            
            Ok(Some(Message::TxAck(messages::TxAck { tx: Some(tx_ack_msg) })))
        }
        Some(rt) if rt == messages::RequestType::Txmeta as i32 => {
            let current_tx = if tx_hash_hex.is_empty() {
                tx_map.get("unsigned")
                    .ok_or_else(|| anyhow!("Unsigned transaction not found"))?
            } else {
                tx_map.get(&tx_hash_hex)
                    .ok_or_else(|| anyhow!("Previous transaction {} not found", tx_hash_hex))?
            };
            
            let tx_meta = messages::TransactionType {
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
            
            Ok(Some(Message::TxAck(messages::TxAck { tx: Some(tx_meta) })))
        }
        Some(rt) if rt == messages::RequestType::Txfinished as i32 => {
            Ok(None) // Signal completion
        }
        _ => Err(anyhow!("Unknown request type: {:?}", tx_req.request_type)),
    }
} 