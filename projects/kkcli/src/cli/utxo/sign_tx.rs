use crate::{
    cli::{parsers::{Bip32PathParser, HexParser, FromStringParser}, CliCommand},
    messages::{self, Message},
    transport::ProtocolAdapter,
};
use anyhow::{Result, anyhow};
use clap::Args;
use std::collections::HashMap;

/// Sign UTXO (Bitcoin) transaction
/// 
/// This command handles the complex TxRequest/TxAck protocol flow for signing Bitcoin transactions.
/// It supports providing previous transaction data needed for non-SegWit inputs.
#[derive(Debug, Clone, Args)]
pub struct SignTx {
    /// Coin name (e.g., Bitcoin, Testnet)
    #[clap(short, long, default_value = "Bitcoin")]
    coin_name: String,
    
    /// Transaction version
    #[clap(short = 'v', long, default_value = "1")]
    version: u32,
    
    /// Transaction lock_time
    #[clap(short, long, default_value = "0")]
    lock_time: u32,
    
    /// Transaction inputs (format: "path:prev_hash:prev_index:amount:script_type")
    /// Example: "m/44'/0'/0'/0/0:abc123...:0:100000:p2pkh"
    /// Script types: p2pkh, p2sh-p2wpkh, p2wpkh
    #[clap(short = 'i', long, value_delimiter = ' ', required = true)]
    inputs: Vec<String>,
    
    /// Transaction outputs (format: "address:amount" or "path:amount:change" for change outputs)
    /// Example: "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa:50000" or "m/44'/0'/0'/0/1:45000:change"
    #[clap(short = 'o', long, value_delimiter = ' ', required = true)]
    outputs: Vec<String>,
    
    /// Previous transactions hex (format: "txid:hex")
    /// Required for non-SegWit inputs
    #[clap(short = 'p', long, value_delimiter = ' ')]
    prev_txs: Vec<String>,
}

/// Parsed input data
#[derive(Debug, Clone)]
struct ParsedInput {
    path: Vec<u32>,
    prev_hash: Vec<u8>,
    prev_index: u32,
    amount: u64,
    script_type: messages::InputScriptType,
}

/// Parsed output data
#[derive(Debug, Clone)]
struct ParsedOutput {
    address: Option<String>,
    address_n: Option<Vec<u32>>,
    amount: u64,
    script_type: messages::OutputScriptType,
    is_change: bool,
}

impl SignTx {
    fn parse_inputs(&self) -> Result<Vec<ParsedInput>> {
        let mut parsed_inputs = Vec::new();
        
        for input_str in &self.inputs {
            let parts: Vec<&str> = input_str.split(':').collect();
            if parts.len() != 5 {
                return Err(anyhow!("Invalid input format. Expected: path:prev_hash:prev_index:amount:script_type"));
            }
            
            let path = Bip32PathParser.parse_str(parts[0])?;
            let prev_hash = HexParser.parse_str(parts[1])?;
            let prev_index: u32 = parts[2].parse()?;
            let amount: u64 = parts[3].parse()?;
            let script_type = match parts[4] {
                "p2pkh" => messages::InputScriptType::Spendaddress,
                "p2sh-p2wpkh" => messages::InputScriptType::Spendp2shwitness,
                "p2wpkh" => messages::InputScriptType::Spendwitness,
                _ => return Err(anyhow!("Invalid script type: {}", parts[4])),
            };
            
            parsed_inputs.push(ParsedInput {
                path: path.into(),
                prev_hash,
                prev_index,
                amount,
                script_type,
            });
        }
        
        Ok(parsed_inputs)
    }
    
    fn parse_outputs(&self) -> Result<Vec<ParsedOutput>> {
        let mut parsed_outputs = Vec::new();
        
        for output_str in &self.outputs {
            if output_str.ends_with(":change") {
                // Change output with path
                let parts: Vec<&str> = output_str.split(':').collect();
                if parts.len() != 3 {
                    return Err(anyhow!("Invalid change output format. Expected: path:amount:change"));
                }
                
                let path = Bip32PathParser.parse_str(parts[0])?;
                let amount: u64 = parts[1].parse()?;
                
                parsed_outputs.push(ParsedOutput {
                    address: None,
                    address_n: Some(path.into()),
                    amount,
                    script_type: messages::OutputScriptType::Paytoaddress,
                    is_change: true,
                });
            } else {
                // Regular output with address
                let parts: Vec<&str> = output_str.split(':').collect();
                if parts.len() != 2 {
                    return Err(anyhow!("Invalid output format. Expected: address:amount"));
                }
                
                let address = parts[0].to_string();
                let amount: u64 = parts[1].parse()?;
                
                parsed_outputs.push(ParsedOutput {
                    address: Some(address),
                    address_n: None,
                    amount,
                    script_type: messages::OutputScriptType::Paytoaddress,
                    is_change: false,
                });
            }
        }
        
        Ok(parsed_outputs)
    }
    
    fn build_prev_tx_map(&self) -> Result<HashMap<String, messages::TransactionType>> {
        let mut tx_map = HashMap::new();
        
        for prev_tx_str in &self.prev_txs {
            let parts: Vec<&str> = prev_tx_str.split(':').collect();
            if parts.len() != 2 {
                return Err(anyhow!("Invalid previous transaction format. Expected: txid:hex"));
            }
            
            let txid = parts[0];
            let hex_data = HexParser.parse_str(parts[1])?;
            
            // Parse the raw transaction hex
            let (tx_meta, inputs, outputs) = parse_raw_tx(&hex_data)?;
            
            let tx = messages::TransactionType {
                version: Some(tx_meta.0),
                lock_time: Some(tx_meta.3),
                inputs_cnt: Some(tx_meta.1),
                outputs_cnt: Some(tx_meta.2),
                inputs: inputs,
                bin_outputs: outputs,
                outputs: vec![],
                extra_data: None,
                extra_data_len: Some(0),
                expiry: None,
                overwintered: None,
                version_group_id: None,
                branch_id: None,
            };
            
            tx_map.insert(txid.to_string(), tx);
        }
        
        Ok(tx_map)
    }
}

/// Parse raw transaction hex to extract metadata, inputs, and outputs
fn parse_raw_tx(data: &[u8]) -> Result<((u32, u32, u32, u32), Vec<messages::TxInputType>, Vec<messages::TxOutputBinType>)> {
    let mut cursor = 0;
    
    // Version (4 bytes)
    if data.len() < 4 {
        return Err(anyhow!("Transaction too short for version"));
    }
    let version = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
    cursor += 4;
    
    // Check for witness marker (BIP144)
    let has_witness = if data.len() > cursor + 1 && data[cursor] == 0x00 && data[cursor + 1] == 0x01 {
        cursor += 2; // Skip marker and flag
        true
    } else {
        false
    };
    
    // Input count (varint)
    let (input_count, bytes_read) = read_varint(&data[cursor..])?;
    cursor += bytes_read;
    
    // Parse inputs
    let mut inputs = Vec::new();
    for i in 0..input_count {
        // Previous hash (32 bytes)
        if cursor + 32 > data.len() {
            return Err(anyhow!("Transaction too short for input {} prev_hash", i));
        }
        let mut prev_hash = data[cursor..cursor + 32].to_vec();
        prev_hash.reverse(); // Bitcoin uses little-endian for hashes
        cursor += 32;
        
        // Previous index (4 bytes)
        if cursor + 4 > data.len() {
            return Err(anyhow!("Transaction too short for input {} prev_index", i));
        }
        let prev_index = u32::from_le_bytes([data[cursor], data[cursor + 1], data[cursor + 2], data[cursor + 3]]);
        cursor += 4;
        
        // Script length (varint)
        let (script_len, bytes_read) = read_varint(&data[cursor..])?;
        cursor += bytes_read;
        
        // Script sig
        if cursor + script_len as usize > data.len() {
            return Err(anyhow!("Transaction too short for input {} script", i));
        }
        let script_sig = data[cursor..cursor + script_len as usize].to_vec();
        cursor += script_len as usize;
        
        // Sequence (4 bytes)
        if cursor + 4 > data.len() {
            return Err(anyhow!("Transaction too short for input {} sequence", i));
        }
        let sequence = u32::from_le_bytes([data[cursor], data[cursor + 1], data[cursor + 2], data[cursor + 3]]);
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
    let (output_count, bytes_read) = read_varint(&data[cursor..])?;
    cursor += bytes_read;
    
    // Parse outputs
    let mut outputs = Vec::new();
    for i in 0..output_count {
        // Amount (8 bytes)
        if cursor + 8 > data.len() {
            return Err(anyhow!("Transaction too short for output {} amount", i));
        }
        let amount = u64::from_le_bytes([
            data[cursor], data[cursor + 1], data[cursor + 2], data[cursor + 3],
            data[cursor + 4], data[cursor + 5], data[cursor + 6], data[cursor + 7]
        ]);
        cursor += 8;
        
        // Script length (varint)
        let (script_len, bytes_read) = read_varint(&data[cursor..])?;
        cursor += bytes_read;
        
        // Script pubkey
        if cursor + script_len as usize > data.len() {
            return Err(anyhow!("Transaction too short for output {} script", i));
        }
        let script_pubkey = data[cursor..cursor + script_len as usize].to_vec();
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
            let (witness_count, bytes_read) = read_varint(&data[cursor..])?;
            cursor += bytes_read;
            
            // Skip each witness item
            for _ in 0..witness_count {
                let (item_len, bytes_read) = read_varint(&data[cursor..])?;
                cursor += bytes_read + item_len as usize;
            }
        }
    }
    
    // Lock time (4 bytes)
    if cursor + 4 > data.len() {
        return Err(anyhow!("Transaction too short for lock_time"));
    }
    let lock_time = u32::from_le_bytes([data[cursor], data[cursor + 1], data[cursor + 2], data[cursor + 3]]);
    
    Ok(((version, input_count as u32, output_count as u32, lock_time), inputs, outputs))
}

/// Read a variable-length integer from the data
fn read_varint(data: &[u8]) -> Result<(u64, usize)> {
    if data.is_empty() {
        return Err(anyhow!("Unexpected end of data while reading varint"));
    }
    
    match data[0] {
        0xfd => {
            if data.len() < 3 {
                return Err(anyhow!("Unexpected end of data while reading varint"));
            }
            Ok((u16::from_le_bytes([data[1], data[2]]) as u64, 3))
        }
        0xfe => {
            if data.len() < 5 {
                return Err(anyhow!("Unexpected end of data while reading varint"));
            }
            Ok((u32::from_le_bytes([data[1], data[2], data[3], data[4]]) as u64, 5))
        }
        0xff => {
            if data.len() < 9 {
                return Err(anyhow!("Unexpected end of data while reading varint"));
            }
            Ok((u64::from_le_bytes([data[1], data[2], data[3], data[4], data[5], data[6], data[7], data[8]]), 9))
        }
        n => Ok((n as u64, 1))
    }
}

impl CliCommand for SignTx {
    fn handle(self, protocol_adapter: &mut dyn ProtocolAdapter) -> Result<()> {
        println!("╔══════════════════════════════════════════════════════════════╗");
        println!("║          Bitcoin Transaction Signing Protocol Debug          ║");
        println!("╚══════════════════════════════════════════════════════════════╝");
        println!();
        
        // Parse inputs and outputs
        let parsed_inputs = self.parse_inputs()?;
        let parsed_outputs = self.parse_outputs()?;
        let mut prev_tx_map = self.build_prev_tx_map()?;
        
        println!("📋 Transaction Parameters:");
        println!("   Coin: {}", self.coin_name);
        println!("   Version: {}", self.version);
        println!("   Lock Time: {}", self.lock_time);
        println!("   Inputs: {}", parsed_inputs.len());
        println!("   Outputs: {}", parsed_outputs.len());
        println!();
        
        // Build the new transaction structure for txmap["unsigned"]
        let mut new_tx_inputs = Vec::new();
        for input in &parsed_inputs {
            new_tx_inputs.push(messages::TxInputType {
                address_n: input.path.clone(),
                prev_hash: input.prev_hash.clone(),
                prev_index: input.prev_index,
                script_sig: None,
                sequence: Some(0xffffffff),
                script_type: Some(input.script_type as i32),
                multisig: None,
                amount: Some(input.amount),
                decred_tree: None,
                decred_script_version: None,
            });
        }
        
        let mut new_tx_outputs = Vec::new();
        for output in &parsed_outputs {
            new_tx_outputs.push(messages::TxOutputType {
                address: output.address.clone(),
                address_n: output.address_n.clone().unwrap_or_default(),
                amount: output.amount,
                script_type: output.script_type as i32,
                multisig: None,
                op_return_data: None,
                address_type: if output.is_change { 
                    Some(messages::OutputAddressType::Change as i32) 
                } else { 
                    Some(messages::OutputAddressType::Spend as i32) 
                },
                decred_script_version: None,
            });
        }
        
        let unsigned_tx = messages::TransactionType {
            version: Some(self.version),
            lock_time: Some(self.lock_time),
            inputs_cnt: Some(parsed_inputs.len() as u32),
            outputs_cnt: Some(parsed_outputs.len() as u32),
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
        
        prev_tx_map.insert("unsigned".to_string(), unsigned_tx);
        
        // Prepare to track transaction data
        let mut signatures = Vec::new();
        let mut serialized_tx_parts = Vec::new();
        let mut request_counter = 0;
        
        // Send initial SignTx message
        let sign_tx = messages::SignTx {
            outputs_count: parsed_outputs.len() as u32,
            inputs_count: parsed_inputs.len() as u32,
            coin_name: Some(self.coin_name.clone()),
            version: Some(self.version),
            lock_time: Some(self.lock_time),
            expiry: None,
            overwintered: None,
            version_group_id: None,
            branch_id: None,
        };
        
        println!("═══════════════════════════════════════════════════════════════");
        println!("📤 SENDING: SignTx message");
        println!("   outputs_count: {}", parsed_outputs.len());
        println!("   inputs_count: {}", parsed_inputs.len());
        println!("   coin_name: {:?}", sign_tx.coin_name);
        println!("   version: {:?}", sign_tx.version);
        println!("   lock_time: {:?}", sign_tx.lock_time);
        println!("═══════════════════════════════════════════════════════════════");
        println!();
        
        // Start the protocol flow
        let mut current_message = Message::SignTx(sign_tx);
        
        loop {
            request_counter += 1;
            
            // Send message and get response
            println!("⏳ Waiting for device response #{}", request_counter);
            let response = protocol_adapter
                .with_standard_handler()
                .handle(current_message)?;
            
            match response {
                Message::TxRequest(tx_req) => {
                    println!("╔══════════════════════════════════════════════════════════════╗");
                    println!("║ 📥 RECEIVED: TxRequest #{}                                   ║", request_counter);
                    println!("╚══════════════════════════════════════════════════════════════╝");
                    
                    // Decode request type
                    let request_type_str = match tx_req.request_type {
                        Some(0) => "TXINPUT (0)",
                        Some(1) => "TXOUTPUT (1)",
                        Some(2) => "TXMETA (2)",
                        Some(3) => "TXFINISHED (3)",
                        Some(4) => "TXEXTRADATA (4)",
                        Some(n) => &format!("UNKNOWN ({})", n),
                        None => "NONE",
                    };
                    
                    println!("   Request Type: {}", request_type_str);
                    
                    // Check for details
                    let mut tx_hash_hex = String::new();
                    if let Some(details) = &tx_req.details {
                        println!("   📎 Details:");
                        if let Some(idx) = details.request_index {
                            println!("      - Request Index: {}", idx);
                        }
                        if let Some(tx_hash) = &details.tx_hash {
                            tx_hash_hex = hex::encode(tx_hash);
                            println!("      - TX Hash: {}", tx_hash_hex);
                        }
                        if let Some(extra_len) = details.extra_data_len {
                            println!("      - Extra Data Length: {}", extra_len);
                        }
                        if let Some(extra_offset) = details.extra_data_offset {
                            println!("      - Extra Data Offset: {}", extra_offset);
                        }
                    } else {
                        println!("   📎 Details: None");
                    }
                    
                    // Check if we have serialized data in this response
                    if let Some(serialized) = &tx_req.serialized {
                        println!("   📦 Serialized Data:");
                        if let Some(serialized_tx) = &serialized.serialized_tx {
                            let hex_part = hex::encode(serialized_tx);
                            println!("      - TX Part ({} bytes): {}", serialized_tx.len(), 
                                if hex_part.len() > 64 { format!("{}...", &hex_part[..64]) } else { hex_part });
                            serialized_tx_parts.push(serialized_tx.clone());
                        }
                        if let Some(signature) = &serialized.signature {
                            if let Some(sig_index) = serialized.signature_index {
                                let sig_hex = hex::encode(signature);
                                println!("      - Signature for input {}: {}", sig_index, sig_hex);
                                signatures.push((sig_index, sig_hex));
                            }
                        }
                    }
                    
                    println!();
                    
                    // Handle the request type
                    match tx_req.request_type {
                        Some(rt) if rt == messages::RequestType::Txinput as i32 => {
                            println!("🔧 ACTION: Sending transaction INPUT data");
                            
                            let details = tx_req.details.as_ref()
                                .ok_or_else(|| anyhow!("Missing details in TXINPUT request"))?;
                            let req_index = details.request_index
                                .ok_or_else(|| anyhow!("Missing request_index in TXINPUT"))? as usize;
                            
                            println!("   Device wants input at index: {}", req_index);
                            
                            // Get the transaction to read from
                            let current_tx = if tx_hash_hex.is_empty() {
                                prev_tx_map.get("unsigned")
                                    .ok_or_else(|| anyhow!("Unsigned transaction not found"))?
                            } else {
                                prev_tx_map.get(&tx_hash_hex)
                                    .ok_or_else(|| anyhow!("Previous transaction {} not found", tx_hash_hex))?
                            };
                            
                            // Get the requested input
                            let input = current_tx.inputs.get(req_index)
                                .ok_or_else(|| anyhow!("Input {} not found in transaction", req_index))?;
                            
                            // Create TxAck with the input
                            let mut tx_ack_msg = messages::TransactionType::default();
                            tx_ack_msg.inputs = vec![input.clone()];
                            
                            println!("📤 SENDING: TxAck with input {}", req_index);
                            if !input.address_n.is_empty() {
                                println!("   Path: m/{}", input.address_n.iter()
                                    .map(|n| if n & 0x80000000 != 0 { 
                                        format!("{}'", n & 0x7fffffff) 
                                    } else { 
                                        n.to_string() 
                                    })
                                    .collect::<Vec<_>>()
                                    .join("/"));
                            }
                            println!("   Prev hash: {}", hex::encode(&input.prev_hash));
                            println!("   Prev index: {}", input.prev_index);
                            if let Some(amount) = input.amount {
                                println!("   Amount: {} sats", amount);
                            }
                            println!();
                            
                            current_message = Message::TxAck(messages::TxAck { tx: Some(tx_ack_msg) });
                        },
                        Some(rt) if rt == messages::RequestType::Txoutput as i32 => {
                            println!("🔧 ACTION: Sending transaction OUTPUT data");
                            
                            let details = tx_req.details.as_ref()
                                .ok_or_else(|| anyhow!("Missing details in TXOUTPUT request"))?;
                            let req_index = details.request_index
                                .ok_or_else(|| anyhow!("Missing request_index in TXOUTPUT"))? as usize;
                            
                            println!("   Device wants output at index: {}", req_index);
                            
                            // Determine which transaction to read from
                            let current_tx = if tx_hash_hex.is_empty() {
                                println!("   📌 This is for the NEW transaction being signed");
                                prev_tx_map.get("unsigned")
                                    .ok_or_else(|| anyhow!("Unsigned transaction not found"))?
                            } else {
                                println!("   ⚠️  This is for PREVIOUS transaction: {}", tx_hash_hex);
                                prev_tx_map.get(&tx_hash_hex)
                                    .ok_or_else(|| anyhow!("Previous transaction {} not found", tx_hash_hex))?
                            };
                            
                            // Create TxAck with the output
                            let mut tx_ack_msg = messages::TransactionType::default();
                            
                            if tx_hash_hex.is_empty() {
                                // New transaction - use regular outputs
                                let output = current_tx.outputs.get(req_index)
                                    .ok_or_else(|| anyhow!("Output {} not found in new transaction", req_index))?;
                                tx_ack_msg.outputs = vec![output.clone()];
                                
                                println!("📤 SENDING: TxAck with output {}", req_index);
                                if let Some(addr) = &output.address {
                                    println!("   Address: {}", addr);
                                } else if !output.address_n.is_empty() {
                                    println!("   Path (change): m/{}", output.address_n.iter()
                                        .map(|n| if n & 0x80000000 != 0 { 
                                            format!("{}'", n & 0x7fffffff) 
                                        } else { 
                                            n.to_string() 
                                        })
                                        .collect::<Vec<_>>()
                                        .join("/"));
                                }
                                println!("   Amount: {} sats", output.amount);
                            } else {
                                // Previous transaction - use bin_outputs
                                let output = current_tx.bin_outputs.get(req_index)
                                    .ok_or_else(|| anyhow!("Output {} not found in previous transaction {}", req_index, tx_hash_hex))?;
                                tx_ack_msg.bin_outputs = vec![output.clone()];
                                
                                println!("📤 SENDING: TxAck with bin_output {}", req_index);
                                println!("   Amount: {} sats", output.amount);
                                println!("   Script: {}", hex::encode(&output.script_pubkey));
                            }
                            println!();
                            
                            current_message = Message::TxAck(messages::TxAck { tx: Some(tx_ack_msg) });
                        },
                        Some(rt) if rt == messages::RequestType::Txmeta as i32 => {
                            println!("🔧 ACTION: Sending transaction metadata");
                            
                            // Determine which transaction metadata to send
                            let current_tx = if tx_hash_hex.is_empty() {
                                println!("   📌 Metadata for NEW transaction");
                                prev_tx_map.get("unsigned")
                                    .ok_or_else(|| anyhow!("Unsigned transaction not found"))?
                            } else {
                                println!("   📌 Metadata for PREVIOUS transaction: {}", tx_hash_hex);
                                prev_tx_map.get(&tx_hash_hex)
                                    .ok_or_else(|| anyhow!("Previous transaction {} not found", tx_hash_hex))?
                            };
                            
                            let tx_meta = messages::TransactionType {
                                version: current_tx.version,
                                inputs_cnt: current_tx.inputs_cnt,
                                outputs_cnt: if tx_hash_hex.is_empty() {
                                    Some(current_tx.outputs.len() as u32)
                                } else {
                                    current_tx.outputs_cnt // Use the parsed output count from metadata
                                },
                                lock_time: current_tx.lock_time,
                                extra_data_len: current_tx.extra_data_len,
                                // Don't send actual inputs/outputs, just metadata
                                inputs: vec![],
                                bin_outputs: vec![],
                                outputs: vec![],
                                extra_data: None,
                                expiry: current_tx.expiry,
                                overwintered: current_tx.overwintered,
                                version_group_id: current_tx.version_group_id,
                                branch_id: current_tx.branch_id,
                            };
                            
                            println!("📤 SENDING: TxAck with metadata");
                            println!("   version: {:?}", tx_meta.version);
                            println!("   inputs_cnt: {:?}", tx_meta.inputs_cnt);
                            println!("   outputs_cnt: {:?}", tx_meta.outputs_cnt);
                            println!("   lock_time: {:?}", tx_meta.lock_time);
                            println!();
                            
                            current_message = Message::TxAck(messages::TxAck { tx: Some(tx_meta) });
                        },
                        Some(rt) if rt == messages::RequestType::Txfinished as i32 => {
                            println!("✅ TRANSACTION SIGNING COMPLETED!");
                            println!();
                            
                            // Combine all serialized parts
                            let mut serialized_tx = Vec::new();
                            for part in &serialized_tx_parts {
                                serialized_tx.extend_from_slice(part);
                            }
                            
                            println!("📊 Final Results:");
                            println!("═══════════════════════════════════════════════════════════════");
                            
                            if !signatures.is_empty() {
                                println!("📝 Signatures:");
                                for (idx, sig) in &signatures {
                                    println!("   Input {}: {}", idx, sig);
                                }
                            } else {
                                println!("⚠️  No signatures received (transaction may not have been fully signed)");
                            }
                            
                            if !serialized_tx.is_empty() {
                                println!();
                                println!("📦 Serialized Transaction ({} bytes):", serialized_tx.len());
                                println!("   {}", hex::encode(&serialized_tx));
                            } else {
                                println!();
                                println!("⚠️  No serialized transaction data received");
                            }
                            
                            println!("═══════════════════════════════════════════════════════════════");
                            
                            return Ok(());
                        },
                        _ => {
                            println!("❓ Unknown request type: {:?}", tx_req.request_type);
                            return Err(anyhow!("Unknown or unhandled request type"));
                        }
                    }
                },
                Message::ButtonRequest(btn_req) => {
                    println!("╔══════════════════════════════════════════════════════════════╗");
                    println!("║ 🔘 BUTTON REQUEST                                            ║");
                    println!("╚══════════════════════════════════════════════════════════════╝");
                    
                    let code_str = match btn_req.code {
                        Some(8) => "SignTx - Confirm transaction on device",
                        Some(3) => "ConfirmOutput - Confirm output on device",
                        Some(2) => "FeeOverThreshold - High fee warning",
                        _ => "Unknown button request",
                    };
                    
                    println!("   Action Required: {}", code_str);
                    println!("   ⚠️  Please check your KeepKey device!");
                    println!();
                    
                    // Send ButtonAck
                    current_message = Message::ButtonAck(messages::ButtonAck {});
                },
                Message::Failure(failure) => {
                    println!("╔══════════════════════════════════════════════════════════════╗");
                    println!("║ ❌ DEVICE RETURNED FAILURE                                   ║");
                    println!("╚══════════════════════════════════════════════════════════════╝");
                    
                    if let Some(code) = failure.code {
                        let code_str = match code {
                            1 => "UnexpectedMessage",
                            2 => "ButtonExpected",
                            3 => "SyntaxError",
                            4 => "ActionCancelled",
                            5 => "PinExpected",
                            6 => "PinCancelled",
                            7 => "PinInvalid",
                            8 => "InvalidSignature",
                            9 => "Other",
                            10 => "NotEnoughFunds",
                            11 => "NotInitialized",
                            12 => "PinMismatch",
                            99 => "FirmwareError",
                            _ => "Unknown",
                        };
                        println!("   Code: {} ({})", code, code_str);
                    }
                    
                    let error_msg = failure.message.unwrap_or_else(|| "Unknown error".to_string());
                    println!("   Message: {}", error_msg);
                    println!();
                    
                    return Err(anyhow!("Device failure: {}", error_msg));
                },
                other => {
                    println!("╔══════════════════════════════════════════════════════════════╗");
                    println!("║ ⚠️  UNEXPECTED MESSAGE TYPE                                   ║");
                    println!("╚══════════════════════════════════════════════════════════════╝");
                    println!("   Received: {:?}", std::mem::discriminant(&other));
                    println!();
                    
                    return Err(anyhow!("Unexpected message type from device"));
                }
            }
        }
    }
}
