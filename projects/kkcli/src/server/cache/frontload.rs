use anyhow::Result;
use tracing::{info, debug, error, warn};
use hex;
use serde_json;
use crate::messages::{self, Message};
use crate::transport::{UsbTransport, ProtocolAdapter};
use crate::server::routes;
use super::device_cache::{DeviceCache, CachedBalance};
use rusb::GlobalContext;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct DeviceFrontloader {
    cache: DeviceCache,
    transport_arc: Arc<Mutex<Option<UsbTransport<GlobalContext>>>>,
    device_obj: rusb::Device<GlobalContext>,
}

impl DeviceFrontloader {
    pub fn new(cache: DeviceCache, transport_arc: Arc<Mutex<Option<UsbTransport<GlobalContext>>>>, device_obj: rusb::Device<GlobalContext>) -> Self {
        Self { cache, transport_arc, device_obj }
    }

    /// Frontload all device data - but only populate what's missing
    pub async fn frontload_all(&self) -> Result<()> {
        info!("🔄 Starting device frontload process...");
        let start_time = std::time::Instant::now();
        
        // Get device features and ID
        let (features, device_id) = self.frontload_features().await?;
        
        // Save features to cache (always update features)
        self.cache.save_features(&features, &device_id).await?;
        
        // Load existing device data into memory cache (critical for address caching)
        self.cache.load_device(&device_id).await?;
        info!("📚 Loaded existing device data into memory cache");
        
        // Always ensure all default paths are loaded (not just if database is empty)
        self.ensure_all_default_paths_loaded().await?;
        
        // Always check for missing addresses from database paths
        info!("📍 Checking for missing addresses from database paths...");
        let mut total_addresses = 0;
        
        total_addresses += self.populate_missing_addresses(&device_id).await?;
        
        // CRITICAL: Fetch balances during frontload - FAIL FAST if Pioneer unavailable
        info!("💰 Fetching balances from Pioneer API during frontload...");
        if let Err(e) = self.frontload_balances(&device_id).await {
            error!("❌ Failed to fetch balances during frontload - FAILING FAST: {}", e);
            return Err(anyhow::anyhow!("Frontload failed: Cannot fetch balances from Pioneer API: {}", e));
        }
        
        let elapsed = start_time.elapsed();
        if total_addresses > 0 {
            info!("✅ Device frontload complete!");
            info!("   📊 Populated {} missing addresses in {:.2}s", total_addresses, elapsed.as_secs_f64());
        } else {
            info!("✅ Device frontload complete - all addresses already cached!");
            info!("   ⚡ No device calls needed, completed in {:.3}s", elapsed.as_secs_f64());
        }
        info!("   💾 Using database cache for fast startup");
        info!("   🏷️  Device: {}", features.label.as_deref().unwrap_or("Unnamed KeepKey"));
        
        // DEBUG: Test if we can read the addresses we just wrote
        info!("🔍 DEBUG: Testing address reading immediately after frontload...");
        match self.cache.debug_load_addresses(&device_id).await {
            Ok(addresses) => {
                info!("🔍 DEBUG: Successfully read {} addresses: {:?}", addresses.len(), addresses);
            }
            Err(e) => {
                error!("🔍 DEBUG: Failed to read addresses: {}", e);
            }
        }
        
        Ok(())
    }
    
    /// Ensure all default paths from JSON are loaded into database
    async fn ensure_all_default_paths_loaded(&self) -> Result<()> {
        info!("📂 Ensuring all default paths from JSON are loaded into database...");
        
        // Load and parse the JSON file
        let json_content = include_str!("../../default-paths.json");
        let json_data: serde_json::Value = serde_json::from_str(json_content)
            .map_err(|e| anyhow::anyhow!("Failed to parse default-paths.json: {}", e))?;
        
        let paths_array = json_data["paths"].as_array()
            .ok_or_else(|| anyhow::anyhow!("No 'paths' array found in default-paths.json"))?;
        
        info!("📥 Found {} default paths in JSON", paths_array.len());
        
        // Get existing paths from database
        let existing_paths = self.cache.get_paths().await?;
        info!("📊 Database currently has {} paths", existing_paths.len());
        
        // Create a set of existing path notes for quick lookup
        let existing_notes: std::collections::HashSet<String> = existing_paths
            .iter()
            .map(|p| p.note.clone())
            .collect();
        
        let mut loaded_count = 0;
        let mut skipped_count = 0;
        
        // Convert JSON paths to our Path struct and insert missing ones
        for (index, path_json) in paths_array.iter().enumerate() {
            let note = path_json.get("note")
                .and_then(|n| n.as_str())
                .unwrap_or("Unknown");
                
            if existing_notes.contains(note) {
                debug!("⏭️ Path already exists: {}", note);
                skipped_count += 1;
                continue;
            }
            
            match self.parse_and_insert_path(path_json).await {
                Ok(id) => {
                    loaded_count += 1;
                    info!("✅ Loaded missing path {}: {} (DB ID: {})", 
                        index + 1, note, id);
                }
                Err(e) => {
                    warn!("❌ Failed to load path {}: {} - Error: {}", 
                        index + 1, note, e);
                    // Continue loading other paths instead of failing completely
                }
            }
        }
        
        info!("🎯 Path loading complete: {} new paths loaded, {} already existed", loaded_count, skipped_count);
        
        Ok(())
    }
    
    /// Parse a JSON path object and insert it into the database
    async fn parse_and_insert_path(&self, path_json: &serde_json::Value) -> Result<i64> {
        use super::device_cache::Path;
        
        // Extract required fields
        let note = path_json.get("note")
            .and_then(|n| n.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'note' field"))?
            .to_string();
        
        let script_type = path_json.get("script_type")
            .and_then(|s| s.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'script_type' field"))?
            .to_string();
        
        let path_type = path_json.get("type")
            .and_then(|t| t.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'type' field"))?
            .to_string();
        
        let curve = path_json.get("curve")
            .and_then(|c| c.as_str())
            .unwrap_or("secp256k1")
            .to_string();
        
        let show_display = path_json.get("showDisplay")
            .and_then(|s| s.as_bool())
            .unwrap_or(false);
        
        // Extract address_n_list
        let address_n_list = path_json.get("addressNList")
            .and_then(|a| a.as_array())
            .ok_or_else(|| anyhow::anyhow!("Missing 'addressNList' field"))?
            .iter()
            .map(|v| v.as_u64().ok_or_else(|| anyhow::anyhow!("Invalid addressNList value"))
                     .map(|n| n as u32))
            .collect::<Result<Vec<u32>, _>>()?;
        
        // Extract address_n_list_master  
        let address_n_list_master = path_json.get("addressNListMaster")
            .and_then(|a| a.as_array())
            .ok_or_else(|| anyhow::anyhow!("Missing 'addressNListMaster' field"))?
            .iter()
            .map(|v| v.as_u64().ok_or_else(|| anyhow::anyhow!("Invalid addressNListMaster value"))
                     .map(|n| n as u32))
            .collect::<Result<Vec<u32>, _>>()?;
        
        // Extract networks
        let networks = path_json.get("networks")
            .and_then(|n| n.as_array())
            .ok_or_else(|| anyhow::anyhow!("Missing 'networks' field"))?
            .iter()
            .map(|v| v.as_str().ok_or_else(|| anyhow::anyhow!("Invalid networks value"))
                     .map(|s| s.to_string()))
            .collect::<Result<Vec<String>, _>>()?;
        
        // Extract available_script_types
        let available_script_types = path_json.get("available_script_types")
            .and_then(|a| a.as_array())
            .map(|arr| arr.iter()
                 .map(|v| v.as_str().unwrap_or("").to_string())
                 .collect::<Vec<String>>());
        
        // Extract optional fields
        let blockchain = path_json.get("blockchain")
            .and_then(|b| b.as_str())
            .map(|s| s.to_string());
        
        let symbol = path_json.get("symbol")
            .and_then(|s| s.as_str())
            .map(|s| s.to_string());
        
        let symbol_swap_kit = path_json.get("symbol_swap_kit")
            .and_then(|s| s.as_str())
            .map(|s| s.to_string());
        
        // Create Path struct (global paths, no device_id)
        let path = Path {
            id: 0, // Will be assigned by database
            note,
            blockchain,
            symbol,
            symbol_swap_kit,
            networks,
            script_type,
            available_script_types,
            path_type,
            address_n_list,
            address_n_list_master,
            curve,
            show_display,
        };
        
        // Insert into database
        let id = self.cache.add_path(&path).await?;
        Ok(id)
    }
    
    /// Populate only missing addresses based on database paths
    async fn populate_missing_addresses(&self, device_id: &str) -> Result<usize> {
        let mut count = 0;
        
        // Get paths from database
        let paths = self.cache.get_paths().await?;
        debug!("Found {} paths in database", paths.len());
        
        for path in paths {
            // Check which networks this path supports
            for network in &path.networks {
                // Determine coin name and script type from network and path
                let (coin_name, script_type) = match self.get_coin_info_from_network_and_path(network, &path.script_type, &path.address_n_list) {
                    Ok(info) => info,
                    Err(e) => {
                        warn!("❌ Failed to get coin info for network {}: {}", network, e);
                        continue; // Skip unsupported networks
                    }
                };
                
                // **NEW: For UTXO networks, we need to cache the xpub for balance queries**
                let is_utxo_network = network.starts_with("bip122:");
                if is_utxo_network {
                    // Use addressNList for XPUBs (always account-level: m/purpose'/coin'/account')
                    let account_path = &path.address_n_list; 
                    let xpub_script_type = format!("{}_xpub", script_type);
                    
                    // Check if xpub is already cached
                    if self.cache.get_cached_address(&coin_name, &xpub_script_type, account_path).is_none() {
                        // Xpub not cached - get it from device
                        match self.get_and_cache_xpub(device_id, &coin_name, &script_type, account_path).await {
                            Ok(xpub) => {
                                count += 1;
                                info!("✅ Cached missing {} {} xpub: {} at path {:?} for network {}", 
                                      coin_name, script_type, xpub, account_path, network);
                            },
                            Err(e) => {
                                warn!("❌ Failed to cache {} {} xpub at {:?} for network {}: {}", 
                                      coin_name, script_type, account_path, network, e);
                                // Continue with other paths instead of stopping
                            },
                        }
                    } else {
                        debug!("Xpub already cached for {} {} at path {:?}", coin_name, script_type, account_path);
                    }
                    
                    // Generate individual address paths from account path
                    let address_paths = self.generate_address_paths(&path.address_n_list);
                    
                    for address_path in address_paths {
                        // Check if this address is already cached
                        if self.cache.get_cached_address(&coin_name, &script_type, &address_path).is_none() {
                            // Address not cached - get it from device
                            match self.get_and_cache_address_from_path(device_id, &coin_name, &script_type, &address_path, network).await {
                                Ok(_) => {
                                    count += 1;
                                    info!("✅ Cached missing {} {} address at path {:?} for network {}", coin_name, script_type, address_path, network);
                                },
                                Err(e) => {
                                    warn!("❌ Failed to cache {} {} address at {:?} for network {}: {}", coin_name, script_type, address_path, network, e);
                                    // Continue with other addresses instead of stopping
                                },
                            }
                        } else {
                            debug!("Address already cached for {} {} at path {:?}", coin_name, script_type, address_path);
                        }
                    }
                } else {
                    // Account-based networks (Ethereum, Cosmos, etc.) - use addressNListMaster
                    let address_path = &path.address_n_list_master;
                    
                    // Check if this address is already cached
                    if self.cache.get_cached_address(&coin_name, &script_type, address_path).is_none() {
                        // Address not cached - get it from device
                        match self.get_and_cache_address_from_path(device_id, &coin_name, &script_type, address_path, network).await {
                            Ok(_) => {
                                count += 1;
                                info!("✅ Cached missing {} {} address at path {:?} for network {}", coin_name, script_type, address_path, network);
                            },
                            Err(e) => {
                                warn!("❌ Failed to cache {} {} address at {:?} for network {}: {}", coin_name, script_type, address_path, network, e);
                                // Continue with other addresses instead of stopping
                            },
                        }
                    } else {
                        debug!("Address already cached for {} {} at path {:?}", coin_name, script_type, address_path);
                    }
                }
            }
        }
        
        info!("📍 Populated {} missing addresses and xpubs from database paths", count);
        Ok(count)
    }
    
    /// Generate individual address paths from account path (first 5 addresses: 0-4)
    fn generate_address_paths(&self, account_path: &[u32]) -> Vec<Vec<u32>> {
        let mut paths = Vec::new();
        
        // Generate first 5 receiving addresses (0/0 through 0/4)
        for i in 0..5 {
            let mut full_path = account_path.to_vec();
            
            // Handle different path structures:
            // - 3 elements: m/purpose'/coin'/account' -> add /change/address_index
            // - 5 elements: already m/purpose'/coin'/account'/change/address_index -> use as is for first, modify index for others
            match full_path.len() {
                3 => {
                    // Account-level path, add change and address index
                    full_path.push(0); // change = 0 (receiving)
                    full_path.push(i); // address_index
                }
                5 => {
                    // Full address path, just change the last index
                    full_path[4] = i; // change address_index
                }
                _ => {
                    // Unexpected path length, skip
                    warn!("Unexpected path length {} for account path {:?}", full_path.len(), account_path);
                    continue;
                }
            }
            paths.push(full_path);
        }
        
        paths
    }
    
    /// Get coin info from network string and script type - uses path context to distinguish coins sharing same network
    fn get_coin_info_from_network_and_path(&self, network: &str, script_type: &str, address_n_list: &[u32]) -> Result<(String, String)> {
        // Bitcoin-based networks (UTXO)
        if network.starts_with("bip122:") {
            // Use path analysis to distinguish different Bitcoin-based coins
            if address_n_list.len() >= 2 {
                let coin_type = address_n_list[1];
                match coin_type {
                    2147483648 => return Ok(("Bitcoin".to_string(), script_type.to_string())), // 0' (Bitcoin)
                    2147483649 => return Ok(("Testnet".to_string(), script_type.to_string())), // 1' (Bitcoin Testnet)
                    2147483650 => return Ok(("Litecoin".to_string(), script_type.to_string())), // 2' (Litecoin)
                    2147483651 => return Ok(("Dogecoin".to_string(), script_type.to_string())), // 3' (Dogecoin)
                    2147483653 => return Ok(("Dash".to_string(), script_type.to_string())), // 5' (Dash)
                    2147483668 => return Ok(("DigiByte".to_string(), script_type.to_string())), // 20' (DigiByte)
                    2147483781 => return Ok(("Zcash".to_string(), script_type.to_string())), // 133' (Zcash)
                    2147483793 => return Ok(("BitcoinCash".to_string(), script_type.to_string())), // 145' (Bitcoin Cash)
                    // 2147483804 => return Ok(("Bgold".to_string(), script_type.to_string())), // 156' (Bitcoin Gold) - COMMENTED OUT as requested
                    2147483789 => return Ok(("Komodo".to_string(), script_type.to_string())), // 141' (Komodo)
                    _ => {
                        warn!("Unknown coin type {} for bip122 network {}, defaulting to Bitcoin", coin_type, network);
                        return Ok(("Bitcoin".to_string(), script_type.to_string())); // Default fallback
                    }
                }
            } else {
                warn!("Insufficient path elements ({}) for bip122 network {}", address_n_list.len(), network);
            }
            return Ok(("Bitcoin".to_string(), script_type.to_string()));
        }
        
        // Ethereum and EVM networks
        if network.starts_with("eip155:") {
            return Ok(("Ethereum".to_string(), "ethereum".to_string()));
        }
        
        // Cosmos-based networks
        if network.starts_with("cosmos:") {
            if network.contains("cosmoshub") {
                return Ok(("Cosmos".to_string(), "cosmos".to_string()));
            } else if network.contains("osmosis") {
                return Ok(("Osmosis".to_string(), "cosmos".to_string()));
            }
            return Ok(("Cosmos".to_string(), "cosmos".to_string()));
        }
        
        // THORChain
        if network.starts_with("thorchain:") {
            return Ok(("THORChain".to_string(), "cosmos".to_string()));
        }
        
        // MayaChain  
        if network.starts_with("mayachain:") {
            return Ok(("MayaChain".to_string(), "cosmos".to_string()));
        }
        
        // Ripple
        if network.starts_with("ripple:") {
            return Ok(("Ripple".to_string(), "ripple".to_string()));
        }
        
        Err(anyhow::anyhow!("Unsupported network: {} (script_type: {}, path: {:?})", network, script_type, address_n_list))
    }
    
    /// Get coin info from network string and script type (legacy method for compatibility)
    fn get_coin_info_from_network(&self, network: &str, script_type: &str) -> Result<(String, String)> {
        // Use empty path as fallback for legacy calls
        self.get_coin_info_from_network_and_path(network, script_type, &[])
    }
    
    /// Get and cache address from device for any supported network
    async fn get_and_cache_address_from_path(
        &self,
        device_id: &str,
        coin_name: &str,
        script_type: &str,
        path: &[u32],
        network: &str,
    ) -> Result<()> {
        if network.starts_with("eip155:") {
            // Ethereum network
            self.get_and_cache_ethereum_address(device_id, path).await
        } else if network.starts_with("bip122:") {
            // Bitcoin-like networks (Bitcoin, Bitcoin Cash, Litecoin)
            self.get_and_cache_bitcoin_address(device_id, coin_name, script_type, path).await
        } else if network.starts_with("cosmos:") || network.starts_with("thorchain:") || network.starts_with("mayachain:") {
            // Cosmos-based networks - we need specific handling for these
            self.get_and_cache_cosmos_address(device_id, coin_name, network, path).await
        } else if network.starts_with("ripple:") {
            // Ripple network - use Ripple address derivation
            self.get_and_cache_ripple_address(device_id, path).await
        } else {
            Err(anyhow::anyhow!("Unsupported network type: {}", network))
        }
    }
    
    /// Load device features
    async fn frontload_features(&self) -> Result<(routes::Features, String)> {
        info!("📱 Loading device features...");
        
        let mut transport_opt_guard = self.transport_arc.lock().await;
        let transport = transport_opt_guard.as_mut().ok_or_else(|| anyhow::anyhow!("Transport not available in frontload_features"))?;
        
        // Send GetFeatures message
        let get_features_msg = messages::GetFeatures {};
        let response = transport.with_standard_handler().handle(get_features_msg.into())?;
        
        match response {
            Message::Features(features_msg) => {
                // Extract device ID
                let device_id = features_msg.device_id
                    .as_ref()
                    .map(|id| hex::encode(id))
                    .unwrap_or_else(|| "unknown".to_string());
                
                info!("✅ Got device features - ID: {}, Label: {}", 
                    device_id, 
                    features_msg.label.as_deref().unwrap_or("Unnamed"));
                
                // Convert to our Features struct
                let features = routes::Features {
                    vendor: features_msg.vendor.or(Some("keepkey.com".to_string())),
                    major_version: features_msg.major_version.map(|v| v as u32),
                    minor_version: features_msg.minor_version.map(|v| v as u32),
                    patch_version: features_msg.patch_version.map(|v| v as u32),
                    bootloader_mode: features_msg.bootloader_mode,
                    device_id: Some(device_id.clone()),
                    pin_protection: features_msg.pin_protection,
                    passphrase_protection: features_msg.passphrase_protection,
                    language: features_msg.language,
                    label: features_msg.label,
                    initialized: features_msg.initialized,
                    revision: features_msg.revision.map(hex::encode),
                    bootloader_hash: features_msg.bootloader_hash.map(hex::encode),
                    imported: features_msg.imported,
                    pin_cached: features_msg.pin_cached,
                    passphrase_cached: features_msg.passphrase_cached,
                    firmware_hash: features_msg.firmware_hash.map(hex::encode),
                    firmware_variant: features_msg.firmware_variant,
                    no_backup: features_msg.no_backup,
                    wipe_code_protection: features_msg.wipe_code_protection,
                    model: features_msg.model.or(Some("Unknown".to_string())),
                    auto_lock_delay_ms: features_msg.auto_lock_delay_ms,
                    policies: if features_msg.policies.is_empty() {
                        None
                    } else {
                        Some(features_msg.policies.into_iter().map(|p| routes::Policy {
                            policy_name: p.policy_name.unwrap_or_default(),
                            enabled: p.enabled.unwrap_or(false),
                        }).collect())
                    },
                };
                
                Ok((features, device_id))
            }
            _ => {
                error!("Unexpected response to GetFeatures");
                Err(anyhow::anyhow!("Device did not return Features message"))
            }
        }
    }
    
    /// Get and cache a single Ethereum address
    async fn get_and_cache_ethereum_address(
        &self,
        device_id: &str,
        path: &[u32],
    ) -> Result<()> {
        let mut transport_opt_guard = self.transport_arc.lock().await;
        let transport = transport_opt_guard.as_mut().ok_or_else(|| anyhow::anyhow!("Transport not available in get_and_cache_ethereum_address"))?;
        
        // Create EthereumGetAddress message for proper hex format
        let ethereum_get_address_msg = messages::EthereumGetAddress {
            address_n: path.to_vec(),
            show_display: Some(false),
        };
        
        // Send message and get response
        let response = transport.with_standard_handler().handle(ethereum_get_address_msg.into())?;
        
        match response {
            Message::EthereumAddress(addr_msg) => {
                if !addr_msg.address.is_empty() {
                    // Convert bytes to hex string with 0x prefix
                    let address = format!("0x{}", hex::encode(&addr_msg.address));
                    
                    self.cache.save_address(
                        device_id,
                        "Ethereum",
                        "ethereum",
                        path,
                        &address,
                        None,
                    ).await?;
                    debug!("Cached Ethereum address: {}", address);
                }
                Ok(())
            }
            _ => Err(anyhow::anyhow!("Unexpected response to EthereumGetAddress")),
        }
    }
    
    /// Get and cache a single Bitcoin address
    async fn get_and_cache_bitcoin_address(
        &self,
        device_id: &str,
        coin_name: &str,
        script_type: &str,
        path: &[u32],
    ) -> Result<()> {
        let mut transport_opt_guard = self.transport_arc.lock().await;
        let transport = transport_opt_guard.as_mut().ok_or_else(|| anyhow::anyhow!("Transport not available in get_and_cache_bitcoin_address"))?;
        
        // Create GetAddress message
        let mut msg = messages::GetAddress::default();
        msg.address_n = path.to_vec();
        msg.coin_name = Some(coin_name.to_string());
        msg.show_display = Some(false);
        
        // Set script type
        match script_type {
            "p2pkh" => msg.script_type = Some(messages::InputScriptType::Spendaddress as i32),
            "p2wpkh" => msg.script_type = Some(messages::InputScriptType::Spendwitness as i32),
            "p2sh-p2wpkh" => msg.script_type = Some(messages::InputScriptType::Spendp2shwitness as i32),
            _ => return Err(anyhow::anyhow!("Unknown script type: {}", script_type)),
        }
        
        // Send message and get response
        let response = transport.with_standard_handler().handle(msg.into())?;
        
        match response {
            Message::Address(addr_msg) => {
                if !addr_msg.address.is_empty() {
                    self.cache.save_address(
                        device_id,
                        coin_name,
                        script_type,
                        path,
                        &addr_msg.address,
                        None, // We're not storing pubkeys for now
                    ).await?;
                    debug!("Cached {} {} address: {}", coin_name, script_type, addr_msg.address);
                }
                Ok(())
            }
            _ => Err(anyhow::anyhow!("Unexpected response to GetAddress")),
        }
    }
    
    /// Get and cache a single Cosmos-based address
    async fn get_and_cache_cosmos_address(
        &self,
        device_id: &str,
        coin_name: &str,
        network: &str,
        path: &[u32],
    ) -> Result<()> {
        let mut transport_opt_guard = self.transport_arc.lock().await;
        let transport = transport_opt_guard.as_mut().ok_or_else(|| anyhow::anyhow!("Transport not available in get_and_cache_cosmos_address"))?;
        
        // Create CosmosGetAddress message
        let cosmos_get_address_msg = messages::CosmosGetAddress {
            address_n: path.to_vec(),
            show_display: Some(false),
        };
        
        // Send message and get response
        let response = transport.with_standard_handler().handle(cosmos_get_address_msg.into())?;
        
        match response {
            Message::CosmosAddress(addr_msg) => {
                if let Some(address) = &addr_msg.address {
                    if !address.is_empty() {
                        self.cache.save_address(
                            device_id,
                            coin_name,
                            "cosmos",
                            path,
                            address,
                            None,
                        ).await?;
                        debug!("Cached {} cosmos address: {}", coin_name, address);
                    }
                }
                Ok(())
            }
            _ => Err(anyhow::anyhow!("Unexpected response to CosmosGetAddress")),
        }
    }
    
    /// Get and cache a single Ripple address
    async fn get_and_cache_ripple_address(
        &self,
        device_id: &str,
        path: &[u32],
    ) -> Result<()> {
        let mut transport_opt_guard = self.transport_arc.lock().await;
        let transport = transport_opt_guard.as_mut().ok_or_else(|| anyhow::anyhow!("Transport not available in get_and_cache_ripple_address"))?;
        
        // Create RippleGetAddress message
        let ripple_get_address_msg = messages::RippleGetAddress {
            address_n: path.to_vec(),
            show_display: Some(false),
        };
        
        // Send message and get response
        let response = transport.with_standard_handler().handle(ripple_get_address_msg.into())?;
        
        match response {
            Message::RippleAddress(addr_msg) => {
                if let Some(address) = &addr_msg.address {
                    if !address.is_empty() {
                        self.cache.save_address(
                            device_id,
                            "Ripple",
                            "ripple",
                            path,
                            address,
                            None,
                        ).await?;
                        debug!("Cached Ripple address: {}", address);
                    }
                }
                Ok(())
            }
            _ => Err(anyhow::anyhow!("Unexpected response to RippleGetAddress")),
        }
    }
    
    /// Get and cache extended public key (xpub) for UTXO networks
    async fn get_and_cache_xpub(
        &self,
        device_id: &str,
        coin_name: &str,
        script_type: &str,
        path: &[u32],
    ) -> Result<String> {
        let mut transport_opt_guard = self.transport_arc.lock().await;
        let transport = transport_opt_guard.as_mut().ok_or_else(|| anyhow::anyhow!("Transport not available in get_and_cache_xpub"))?;
        
        // Create GetPublicKey message to get xpub
        let mut msg = messages::GetPublicKey::default();
        msg.address_n = path.to_vec();
        msg.coin_name = Some(coin_name.to_string());
        msg.show_display = Some(false);
        
        // Set script type for xpub format
        match script_type {
            "p2pkh" => msg.script_type = Some(messages::InputScriptType::Spendaddress as i32), // xpub
            "p2wpkh" => msg.script_type = Some(messages::InputScriptType::Spendwitness as i32), // zpub
            "p2sh-p2wpkh" => msg.script_type = Some(messages::InputScriptType::Spendp2shwitness as i32), // ypub
            _ => return Err(anyhow::anyhow!("Unknown script type for xpub: {}", script_type)),
        }
        
        // Send message and get response
        let response = transport.with_standard_handler().handle(msg.into())?;
        
        match response {
            Message::PublicKey(pubkey_msg) => {
                if let Some(xpub) = &pubkey_msg.xpub {
                    if !xpub.is_empty() {
                        // Save xpub to cache (we'll use the address field to store the xpub)
                        self.cache.save_address(
                            device_id,
                            coin_name,
                            &format!("{}_xpub", script_type), // Mark as xpub variant
                            path,
                            xpub,
                            None,
                        ).await?;
                        
                        info!("✅ Cached {} {} xpub: {}", coin_name, script_type, xpub);
                        Ok(xpub.clone())
                    } else {
                        Err(anyhow::anyhow!("Empty xpub returned from device"))
                    }
                } else {
                    Err(anyhow::anyhow!("No xpub returned from device"))
                }
            }
            _ => Err(anyhow::anyhow!("Unexpected response to GetPublicKey")),
        }
    }
    
    /// Frontload balances from Pioneer API during startup
    async fn frontload_balances(&self, device_id: &str) -> Result<()> {
        let tag = "frontload_balances";
        info!("{}: Starting balance frontload for device {}", tag, device_id);
        
        // Check if balances need refresh - RESPECT THE CACHE!
        let needs_refresh = match self.cache.balances_need_refresh(device_id).await {
            Ok(needs) => {
                info!("{}: Balances need refresh: {}", tag, needs);
                needs // ✅ FIXED: Actually use the cache result instead of always true
            }
            Err(e) => {
                warn!("{}: Error checking refresh status, forcing refresh: {}", tag, e);
                true
            }
        };
        
        if needs_refresh {
            info!("{}: Refreshing balances from Pioneer API...", tag);
            
            // Import the refresh function from v2_endpoints
            if let Err(e) = self.refresh_balances_from_pioneer(device_id).await {
                return Err(anyhow::anyhow!("Failed to refresh balances from Pioneer: {}", e));
            }
        } else {
            info!("{}: ⚡ Balances are fresh (< 1h old) - skipping Pioneer API call for instant startup!", tag);
        }
        
        Ok(())
    }
    
    /// Refresh balances from Pioneer API (similar to v2_endpoints but adapted for frontload)
    async fn refresh_balances_from_pioneer(&self, device_id: &str) -> Result<()> {
        let tag = "refresh_balances_from_pioneer";
        
        // Get Pioneer server URL from config
        let pioneer_url = self.cache.get_pioneer_server_url().await?;
        info!("{}: Using Pioneer server URL: {}", tag, pioneer_url);
        
        // Get all pubkeys for building asset query
        let paths = self.cache.get_paths().await?;
        let mut asset_queries = Vec::new();
        
        for path in paths {
            for network in &path.networks {
                // Get CAIP for this network
                let caip = match network_to_caip(network) {
                    Ok(caip) => caip,
                    Err(e) => {
                        debug!("{}: Skipping unsupported network {}: {}", tag, network, e);
                        continue;
                    }
                };
                
                // Check if this is a UTXO network (needs xpub) or account network (needs address)
                let is_utxo_network = network.starts_with("bip122:");
                
                if is_utxo_network {
                    // UTXO networks: Use xpub for the account path (not individual addresses)
                    // For UTXO, we want the account-level path (usually m/purpose'/coin'/account')
                    // Use addressNList which is always the account-level path
                    let account_path = &path.address_n_list; 
                    
                    // Use path-aware coin mapping to get correct coin name
                    let (coin_name, script_type) = match self.get_coin_info_from_network_and_path(network, &path.script_type, account_path) {
                        Ok(info) => info,
                        Err(e) => {
                            debug!("{}: Skipping unsupported UTXO network {}: {}", tag, network, e);
                            continue;
                        }
                    };
                    
                    // Check if we have cached xpub, if not generate it
                    let xpub_script_type = format!("{}_xpub", script_type);
                    let xpub = if let Some(cached_xpub) = self.cache.get_cached_address(&coin_name, &xpub_script_type, account_path) {
                        cached_xpub.address
                    } else {
                        // Generate and cache xpub
                        match self.get_and_cache_xpub(device_id, &coin_name, &script_type, account_path).await {
                            Ok(xpub) => xpub,
                            Err(e) => {
                                warn!("{}: Failed to get xpub for {} {}: {}", tag, coin_name, script_type, e);
                                continue;
                            }
                        }
                    };
                    
                    info!("{}: Adding UTXO asset query: caip={}, xpub={}", tag, caip, xpub);
                    asset_queries.push(serde_json::json!({
                        "caip": caip,
                        "pubkey": xpub
                    }));
                } else {
                    // Account-based networks: Use individual address from addressNListMaster
                    let (coin_name, script_type) = match self.get_coin_info_from_network_and_path(network, &path.script_type, &path.address_n_list_master) {
                        Ok(info) => info,
                        Err(e) => {
                            debug!("{}: Skipping unsupported account network {}: {}", tag, network, e);
                            continue;
                        }
                    };
                    
                    if let Some(cached_addr) = self.cache.get_cached_address(&coin_name, &script_type, &path.address_n_list_master) {
                        let address = cached_addr.address.clone();
                        info!("{}: Adding account asset query: caip={}, address={}", tag, caip, address);
                        asset_queries.push(serde_json::json!({
                            "caip": caip,
                            "pubkey": address
                        }));
                    } else {
                        warn!("{}: No cached address found for {} {} at path {:?}", tag, coin_name, script_type, path.address_n_list_master);
                    }
                }
            }
        }
        
        if asset_queries.is_empty() {
            error!("{}: No asset queries to process - no cached addresses found!", tag);
            return Err(anyhow::anyhow!("No cached addresses available for balance lookup"));
        }
        
        info!("{}: Fetching balances for {} assets from {}", tag, asset_queries.len(), pioneer_url);
        
        // Build the full URL
        let portfolio_url = format!("{}/api/v1/portfolio", pioneer_url);
        info!("{}: Making POST request to: {}", tag, portfolio_url);
        
        // Log the request payload
        let payload = serde_json::to_string_pretty(&asset_queries)?;
        info!("{}: Request payload:\n{}", tag, payload);
        
        // Make HTTP request to Pioneer API
        let client = reqwest::Client::new();
        let response = client
            .post(&portfolio_url)
            .header("Content-Type", "application/json")
            .header("User-Agent", "kkcli/1.0")
            .json(&asset_queries)
            .send()
            .await?;
        
        let status = response.status();
        let headers = response.headers().clone();
        info!("{}: Response status: {}", tag, status);
        info!("{}: Response headers: {:?}", tag, headers);
        
        if !status.is_success() {
            // Try to get response body for debugging
            let body = response.text().await.unwrap_or_else(|_| "Could not read response body".to_string());
            error!("{}: Pioneer API error response body: {}", tag, body);
            return Err(anyhow::anyhow!("Pioneer API returned status: {} {}", status, body));
        }
        
        let response_text = response.text().await?;
        info!("{}: Raw response body:\n{}", tag, response_text);
        
        // Parse the response
        let balance_responses: Vec<serde_json::Value> = serde_json::from_str(&response_text)?;
        info!("{}: Parsed {} balance responses", tag, balance_responses.len());
        
        // Pretty print the full Pioneer API response
        info!("{}: 🎯 PIONEER API RESPONSE (Pretty Print):", tag);
        for (i, balance_data) in balance_responses.iter().enumerate() {
            info!("{}: Balance #{}: {}", tag, i + 1, serde_json::to_string_pretty(&balance_data)?);
        }
        
        // Convert to CachedBalance objects
        let mut cached_balances = Vec::new();
        for balance_data in balance_responses {
            info!("{}: Processing balance data: {}", tag, serde_json::to_string_pretty(&balance_data)?);
            
            if let (Some(caip), Some(pubkey), Some(balance), Some(price_usd), Some(value_usd)) = (
                balance_data.get("caip").and_then(|v| v.as_str()),
                balance_data.get("pubkey").and_then(|v| v.as_str()),
                balance_data.get("balance").and_then(|v| v.as_str()),
                balance_data.get("priceUsd").and_then(|v| v.as_str()),
                balance_data.get("valueUsd").and_then(|v| v.as_str()),
            ) {
                let network_id = caip_to_network_id(caip);
                
                let cached_balance = CachedBalance {
                    id: 0, // Will be set by database
                    device_id: device_id.to_string(),
                    caip: caip.to_string(),
                    pubkey: pubkey.to_string(),
                    balance: balance.to_string(),
                    price_usd: price_usd.to_string(),
                    value_usd: value_usd.to_string(),
                    symbol: extract_symbol_from_caip(caip),
                    network_id: Some(network_id),
                    last_updated: chrono::Utc::now().timestamp(),
                };
                
                info!("{}: ✅ Created cached balance: caip={}, pubkey={}, balance={}, value_usd=${}", tag, caip, pubkey, balance, value_usd);
                cached_balances.push(cached_balance);
            } else {
                warn!("{}: ❌ Skipping balance data with missing fields: {}", tag, serde_json::to_string(&balance_data)?);
            }
        }
        
        if cached_balances.is_empty() {
            warn!("{}: No valid balances received from Pioneer API", tag);
            return Ok(());
        }
        
        // Pretty print what we're about to save to database
        info!("{}: 💾 SAVING TO DATABASE (Pretty Print):", tag);
        for (i, cached_balance) in cached_balances.iter().enumerate() {
            info!("{}: Cached Balance #{}: {}", tag, i + 1, serde_json::to_string_pretty(&serde_json::json!({
                "caip": cached_balance.caip,
                "pubkey": cached_balance.pubkey,
                "balance": cached_balance.balance,
                "price_usd": cached_balance.price_usd,
                "value_usd": cached_balance.value_usd,
                "symbol": cached_balance.symbol,
                "network_id": cached_balance.network_id,
                "last_updated": cached_balance.last_updated
            }))?);
        }
        
        // Save to cache
        self.cache.save_balances(device_id, &cached_balances).await?;
        
        // Clean up old balances
        self.cache.clear_old_balances(device_id).await?;
        
        info!("{}: Successfully cached {} balances", tag, cached_balances.len());
        Ok(())
    }
}

/// Convert network identifier to CAIP format
fn network_to_caip(network: &str) -> Result<String> {
    match network {
        // Bitcoin networks
        n if n.starts_with("bip122:000000000019d6689c085ae165831e93") => Ok("bip122:000000000019d6689c085ae165831e93/slip44:0".to_string()), // Bitcoin
        n if n.starts_with("bip122:000000000000000000651ef99cb9fcbe") => Ok("bip122:000000000000000000651ef99cb9fcbe/slip44:145".to_string()), // Bitcoin Cash
        n if n.starts_with("bip122:12a765e31ffd4059bada1e25190f6e98") => Ok("bip122:12a765e31ffd4059bada1e25190f6e98/slip44:2".to_string()), // Litecoin
        
        // Ethereum networks  
        n if n.starts_with("eip155:1") => Ok("eip155:1/slip44:60".to_string()), // Ethereum mainnet
        n if n.starts_with("eip155:10") => Ok("eip155:10/slip44:60".to_string()), // Optimism
        n if n.starts_with("eip155:137") => Ok("eip155:137/slip44:60".to_string()), // Polygon
        n if n.starts_with("eip155:42161") => Ok("eip155:42161/slip44:60".to_string()), // Arbitrum
        n if n.starts_with("eip155:8453") => Ok("eip155:8453/slip44:60".to_string()), // Base
        n if n.starts_with("eip155:56") => Ok("eip155:56/slip44:60".to_string()), // BSC
        n if n.starts_with("eip155:43114") => Ok("eip155:43114/slip44:60".to_string()), // Avalanche
        n if n.starts_with("eip155:250") => Ok("eip155:250/slip44:60".to_string()), // Fantom
        
        // Handle generic eip155 networks
        n if n.starts_with("eip155:") => {
            let chain_id = n.strip_prefix("eip155:").unwrap_or("1");
            if chain_id == "*" {
                Ok("eip155:1/slip44:60".to_string()) // Default to Ethereum mainnet
            } else {
                Ok(format!("eip155:{}/slip44:60", chain_id))
            }
        }
        
        // Cosmos networks
        n if n.starts_with("cosmos:cosmoshub-4") => Ok("cosmos:cosmoshub-4/slip44:118".to_string()), // Cosmos Hub
        n if n.starts_with("cosmos:osmosis-1") => Ok("cosmos:osmosis-1/slip44:118".to_string()), // Osmosis
        n if n.starts_with("cosmos:") => Ok("cosmos:cosmoshub-4/slip44:118".to_string()), // Default Cosmos
        
        // THORChain
        n if n.starts_with("thorchain:") => Ok("thorchain:thorchain-mainnet-v1/slip44:931".to_string()),
        
        // Mayachain  
        n if n.starts_with("mayachain:") => Ok("mayachain:mayachain-mainnet-v1/slip44:931".to_string()),
        
        // Ripple
        n if n.starts_with("ripple:4109c6f2045fc7eff4cde8f9905d19c2") => Ok("ripple:4109c6f2045fc7eff4cde8f9905d19c2/slip44:144".to_string()),
        n if n.starts_with("ripple:") => Ok("ripple:4109c6f2045fc7eff4cde8f9905d19c2/slip44:144".to_string()), // Default ripple
        
        // Additional UTXO networks - Updated with correct hashes
        n if n.starts_with("bip122:000007d91d1254d60e2dd1ae58038307") => Ok("bip122:000007d91d1254d60e2dd1ae58038307/slip44:5".to_string()), // Dash
        n if n.starts_with("bip122:00000000001a91e3dace36e2be3bf030") => Ok("bip122:00000000001a91e3dace36e2be3bf030/slip44:3".to_string()), // Dogecoin
        n if n.starts_with("bip122:7497ea1b465eb39f1c8f507bc877078f") => Ok("bip122:7497ea1b465eb39f1c8f507bc877078f/slip44:20".to_string()), // DigiByte
        n if n.starts_with("bip122:0000000000196a45") => Ok("bip122:0000000000196a45/slip44:133".to_string()), // Zcash
        n if n.starts_with("bip122:027e3758c3a65b12aa1046462b486d0a") => Ok("bip122:027e3758c3a65b12aa1046462b486d0a/slip44:141".to_string()), // Komodo
        
        _ => Err(anyhow::anyhow!("Unsupported network for CAIP conversion: {}", network)),
    }
}

/// Convert CAIP to network identifier
fn caip_to_network_id(caip: &str) -> String {
    if caip.starts_with("bip122:000000000019d6689c085ae165831e93") {
        "bip122:000000000019d6689c085ae165831e93".to_string() // Bitcoin
    } else if caip.starts_with("bip122:000000000000000000651ef99cb9fcbe") {
        "bip122:000000000000000000651ef99cb9fcbe".to_string() // Bitcoin Cash
    } else if caip.starts_with("bip122:12a765e31ffd4059bada1e25190f6e98") {
        "bip122:12a765e31ffd4059bada1e25190f6e98".to_string() // Litecoin
    } else if caip.starts_with("eip155:1/") {
        "eip155:1".to_string() // Ethereum mainnet
    } else if caip.starts_with("eip155:10/") {
        "eip155:10".to_string() // Optimism
    } else if caip.starts_with("eip155:137/") {
        "eip155:137".to_string() // Polygon
    } else if caip.starts_with("eip155:42161/") {
        "eip155:42161".to_string() // Arbitrum
    } else if caip.starts_with("eip155:8453/") {
        "eip155:8453".to_string() // Base
    } else if caip.starts_with("eip155:56/") {
        "eip155:56".to_string() // BSC
    } else if caip.starts_with("eip155:43114/") {
        "eip155:43114".to_string() // Avalanche
    } else if caip.starts_with("eip155:250/") {
        "eip155:250".to_string() // Fantom
    } else if caip.starts_with("eip155:") {
        let chain_id = caip.split('/').next().unwrap_or("eip155:1");
        chain_id.to_string()
    } else if caip.starts_with("cosmos:cosmoshub-4") {
        "cosmos:cosmoshub-4".to_string()
    } else if caip.starts_with("cosmos:osmosis-1") {
        "cosmos:osmosis-1".to_string()
    } else if caip.starts_with("cosmos:") {
        "cosmos:cosmoshub-4".to_string()
    } else if caip.starts_with("thorchain:") {
        "thorchain:thorchain-mainnet-v1".to_string()
    } else if caip.starts_with("mayachain:") {
        "mayachain:mayachain-mainnet-v1".to_string()
    } else if caip.starts_with("ripple:4109c6f2045fc7eff4cde8f9905d19c2") {
        "ripple:4109c6f2045fc7eff4cde8f9905d19c2".to_string() // Ripple
    } else if caip.starts_with("ripple:") {
        "ripple:4109c6f2045fc7eff4cde8f9905d19c2".to_string() // Default ripple
    } else if caip.starts_with("bip122:000007d91d1254d60e2dd1ae58038307") {
        "bip122:000007d91d1254d60e2dd1ae58038307".to_string() // Dash
    } else if caip.starts_with("bip122:00000000001a91e3dace36e2be3bf030") {
        "bip122:00000000001a91e3dace36e2be3bf030".to_string() // Dogecoin
    } else if caip.starts_with("bip122:7497ea1b465eb39f1c8f507bc877078f") {
        "bip122:7497ea1b465eb39f1c8f507bc877078f".to_string() // DigiByte
    } else if caip.starts_with("bip122:0000000000196a45") {
        "bip122:0000000000196a45".to_string() // Zcash
    } else if caip.starts_with("bip122:027e3758c3a65b12aa1046462b486d0a") {
        "bip122:027e3758c3a65b12aa1046462b486d0a".to_string() // Komodo
    } else {
        caip.split('/').next().unwrap_or(caip).to_string()
    }
}

/// Extract symbol from CAIP
fn extract_symbol_from_caip(caip: &str) -> Option<String> {
    if caip.contains("bip122:000000000019d6689c085ae165831e93") {
        Some("BTC".to_string())
    } else if caip.contains("bip122:000000000000000000651ef99cb9fcbe") {
        Some("BCH".to_string()) // Bitcoin Cash
    } else if caip.contains("bip122:12a765e31ffd4059bada1e25190f6e98") {
        Some("LTC".to_string()) // Litecoin
    } else if caip.starts_with("eip155:1/") {
        Some("ETH".to_string()) // Ethereum mainnet
    } else if caip.starts_with("eip155:10/") {
        Some("OP".to_string()) // Optimism 
    } else if caip.starts_with("eip155:137/") {
        Some("MATIC".to_string()) // Polygon
    } else if caip.starts_with("eip155:42161/") {
        Some("ARB".to_string()) // Arbitrum
    } else if caip.starts_with("eip155:8453/") {
        Some("ETH".to_string()) // Base (uses ETH)
    } else if caip.starts_with("eip155:56/") {
        Some("BNB".to_string()) // BSC
    } else if caip.starts_with("eip155:43114/") {
        Some("AVAX".to_string()) // Avalanche
    } else if caip.starts_with("eip155:250/") {
        Some("FTM".to_string()) // Fantom
    } else if caip.starts_with("eip155:") {
        Some("ETH".to_string()) // Default to ETH for unknown EIP155 networks
    } else if caip.starts_with("cosmos:") {
        Some("ATOM".to_string())
    } else if caip.starts_with("thorchain:") {
        Some("RUNE".to_string()) // THORChain
    } else if caip.starts_with("mayachain:") {
        Some("CACAO".to_string()) // Mayachain
    } else if caip.starts_with("ripple:") {
        Some("XRP".to_string()) // Ripple
    } else if caip.contains("bip122:000007d91d1254d60e2dd1ae58038307") {
        Some("DASH".to_string()) // Dash
    } else if caip.contains("bip122:00000000001a91e3dace36e2be3bf030") {
        Some("DOGE".to_string()) // Dogecoin
    } else if caip.contains("bip122:7497ea1b465eb39f1c8f507bc877078f") {
        Some("DGB".to_string()) // DigiByte
    } else if caip.contains("bip122:0000000000196a45") {
        Some("ZEC".to_string()) // Zcash
    } else if caip.contains("bip122:027e3758c3a65b12aa1046462b486d0a") {
        Some("KMD".to_string()) // Komodo
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::messages::{Features, GetAddress, Address};
    use crate::server::cache::device_cache::DeviceCache;
    use tempfile::tempdir;
    use std::collections::HashMap;
    use rusqlite::Connection;
    
    // Mock transport for testing without physical device
    #[derive(Clone)]
    pub struct MockTransport {
        // Simulated device responses
        features: Features,
        addresses: HashMap<(String, String, Vec<u32>), (String, Option<String>)>,
    }
    
    impl MockTransport {
        pub fn new() -> Self {
            let mut features = Features::default();
            features.device_id = Some("mock_device_123".to_string());
            features.label = Some("Mock KeepKey".to_string());
            features.vendor = Some("KeepKey".to_string());
            features.major_version = Some(7);
            features.minor_version = Some(7);
            features.patch_version = Some(0);
            
            Self {
                features,
                addresses: HashMap::new(),
            }
        }
        
        pub fn add_mock_address(&mut self, coin: &str, script_type: &str, path: Vec<u32>, address: &str, pubkey: Option<&str>) {
            self.addresses.insert(
                (coin.to_string(), script_type.to_string(), path),
                (address.to_string(), pubkey.map(|s| s.to_string()))
            );
        }
        
        // Mock the get_address functionality
        pub fn mock_get_address(&self, coin: &str, script_type: &str, path: &[u32]) -> Option<(String, Option<String>)> {
            let key = (coin.to_string(), script_type.to_string(), path.to_vec());
            self.addresses.get(&key).cloned()
        }
        
        pub fn get_features(&self) -> &Features {
            &self.features
        }
    }
    
    // Helper function to create test cache
    async fn create_test_cache() -> DeviceCache {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("frontload_test.db");
        
        let conn = rusqlite::Connection::open(&db_path).unwrap();
        
        // Enable WAL mode for better concurrency
        conn.pragma_update(None, "journal_mode", "WAL").unwrap();
        conn.pragma_update(None, "foreign_keys", "ON").unwrap();
        
        // Execute database schema
        let schema = include_str!("schema.sql");
        conn.execute_batch(schema).unwrap();
        
        DeviceCache {
            db: std::sync::Arc::new(tokio::sync::Mutex::new(conn)),
            memory_cache: std::sync::Arc::new(std::sync::RwLock::new(crate::server::cache::device_cache::MemoryCache::default())),
        }
    }
    
    // Helper function to create test features
    fn create_test_features(device_id: &str, label: &str) -> crate::server::routes::Features {
        crate::server::routes::Features {
            vendor: Some("KeepKey".to_string()),
            major_version: Some(7),
            minor_version: Some(7),
            patch_version: Some(0),
            bootloader_mode: Some(false),
            device_id: Some(device_id.to_string()),
            pin_protection: Some(true),
            passphrase_protection: Some(false),
            language: Some("english".to_string()),
            label: Some(label.to_string()),
            initialized: Some(true),
            revision: None,
            firmware_hash: None,
            bootloader_hash: None,
            imported: Some(false),
            pin_cached: Some(false),
            passphrase_cached: Some(false),
            wipe_code_protection: Some(false),
            auto_lock_delay_ms: Some(0),
            policies: None,
            model: Some("KeepKey".to_string()),
            firmware_variant: Some("standard".to_string()),
            no_backup: Some(false),
        }
    }
    
    #[tokio::test]
    async fn test_frontload_cache_logic() {
        let cache = create_test_cache().await;
        let mut mock_transport = MockTransport::new();
        
        // Add some mock addresses
        mock_transport.add_mock_address("Bitcoin", "legacy", vec![44, 0, 0, 0, 0], "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa", Some("pub1"));
        mock_transport.add_mock_address("Bitcoin", "segwit", vec![44, 0, 0, 0, 1], "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4", Some("pub2"));
        mock_transport.add_mock_address("Ethereum", "legacy", vec![44, 60, 0, 0, 0], "0x742E4C4F4E7E3F2E3D2E1E0F0E0D0C0B0A090807", Some("pub3"));
        
        let device_id = "mock_device_123";
        let features = mock_transport.get_features();
        
        // Convert protobuf Features to routes::Features
        let routes_features = crate::server::routes::Features {
            vendor: features.vendor.clone(),
            major_version: features.major_version,
            minor_version: features.minor_version,
            patch_version: features.patch_version,
            bootloader_mode: Some(false),
            device_id: features.device_id.clone(),
            pin_protection: Some(true),
            passphrase_protection: Some(false),
            language: Some("english".to_string()),
            label: features.label.clone(),
            initialized: Some(true),
            revision: None,
            firmware_hash: None,
            bootloader_hash: None,
            imported: Some(false),
            pin_cached: Some(false),
            passphrase_cached: Some(false),
            wipe_code_protection: Some(false),
            auto_lock_delay_ms: Some(0),
            policies: None,
            model: Some("KeepKey".to_string()),
            firmware_variant: Some("standard".to_string()),
            no_backup: Some(false),
        };
        
        // Save device to cache
        cache.save_features(&routes_features, device_id).await.unwrap();
        
        // Verify device has no cached addresses initially
        assert!(!cache.has_cached_addresses(device_id).await.unwrap());
        
        // Simulate the frontload process by manually saving addresses
        for ((coin, script_type, path), (address, pubkey)) in &mock_transport.addresses {
            cache.save_address(device_id, coin, script_type, path, address, pubkey.as_deref()).await.unwrap();
        }
        
        // Verify device now has cached addresses
        assert!(cache.has_cached_addresses(device_id).await.unwrap());
        
        // Load device and verify addresses are in memory
        let loaded = cache.load_device(device_id).await.unwrap();
        assert!(loaded.is_some());
        
        // Test specific address retrieval
        let btc_legacy = cache.get_cached_address("Bitcoin", "legacy", &[44, 0, 0, 0, 0]);
        assert!(btc_legacy.is_some());
        assert_eq!(btc_legacy.unwrap().address, "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa");
        
        let eth_address = cache.get_cached_address("Ethereum", "legacy", &[44, 60, 0, 0, 0]);
        assert!(eth_address.is_some());
        assert_eq!(eth_address.unwrap().address, "0x742E4C4F4E7E3F2E3D2E1E0F0E0D0C0B0A090807");
    }
    
    #[tokio::test]
    async fn test_cache_hit_vs_miss_scenario() {
        let cache = create_test_cache().await;
        let mock_transport = MockTransport::new();
        let device_id = "cache_test_device";
        
        let features = create_test_features(device_id, "Cache Test Device");
        
        // Save device
        cache.save_features(&features, device_id).await.unwrap();
        
        // Test 1: Fresh device should have no cached addresses
        assert!(!cache.has_cached_addresses(device_id).await.unwrap());
        
        // Test 2: Add some addresses to cache
        cache.save_address(device_id, "Bitcoin", "legacy", &[44, 0, 0, 0, 0], "cached_address_1", None).await.unwrap();
        cache.save_address(device_id, "Bitcoin", "legacy", &[44, 0, 0, 0, 1], "cached_address_2", None).await.unwrap();
        
        // Test 3: Device should now have cached addresses
        assert!(cache.has_cached_addresses(device_id).await.unwrap());
        
        // Test 4: Load device and verify addresses are in memory
        let loaded = cache.load_device(device_id).await.unwrap();
        assert!(loaded.is_some());
        
        // Test 5: Verify specific addresses are available in memory cache
        let addr1 = cache.get_cached_address("Bitcoin", "legacy", &[44, 0, 0, 0, 0]);
        let addr2 = cache.get_cached_address("Bitcoin", "legacy", &[44, 0, 0, 0, 1]);
        
        assert!(addr1.is_some());
        assert!(addr2.is_some());
        assert_eq!(addr1.unwrap().address, "cached_address_1");
        assert_eq!(addr2.unwrap().address, "cached_address_2");
        
        // Test 6: Missing address should return None
        let missing = cache.get_cached_address("Bitcoin", "legacy", &[44, 0, 0, 0, 2]);
        assert!(missing.is_none());
    }
    
    #[tokio::test]
    async fn test_address_path_serialization() {
        let cache = create_test_cache().await;
        let device_id = "path_test_device";
        
        let features = create_test_features(device_id, "Path Test Device");
        
        cache.save_features(&features, device_id).await.unwrap();
        
        // Test various derivation path patterns
        let test_paths = vec![
            vec![44, 0, 0, 0, 0],                    // Standard Bitcoin path
            vec![44, 60, 0, 0, 0],                   // Ethereum path
            vec![2147483692, 2147483648, 2147483648, 0, 0], // Hardened path
            vec![44, 0, 0],                          // Short path
            vec![44, 0, 0, 0, 0, 1, 2, 3, 4, 5],     // Long path
        ];
        
        for (i, path) in test_paths.iter().enumerate() {
            let address = format!("test_address_{}", i);
            cache.save_address(device_id, "Bitcoin", "legacy", path, &address, None).await.unwrap();
        }
        
        // Load device to get addresses into memory
        cache.load_device(device_id).await.unwrap();
        
        // Verify all paths can be retrieved correctly
        for (i, path) in test_paths.iter().enumerate() {
            let expected_address = format!("test_address_{}", i);
            let retrieved = cache.get_cached_address("Bitcoin", "legacy", path);
            assert!(retrieved.is_some(), "Failed to retrieve address for path {:?}", path);
            assert_eq!(retrieved.unwrap().address, expected_address);
        }
    }
    
    #[tokio::test]
    async fn test_multiple_script_types_and_coins() {
        let cache = create_test_cache().await;
        let mock_transport = MockTransport::new();
        let device_id = "multi_test_device";
        
        let features = create_test_features(device_id, "Multi-Script Test Device");
        
        // Save device
        cache.save_features(&features, device_id).await.unwrap();
        
        // Test different script types for different coins
        let test_cases = vec![
            ("Bitcoin", "p2pkh", vec![44, 0, 0, 0, 0], "1BTC_P2PKH_Address"),
            ("Bitcoin", "p2wpkh", vec![84, 0, 0, 0, 0], "bc1_BTC_P2WPKH_Address"),  
            ("Bitcoin", "p2sh-p2wpkh", vec![49, 0, 0, 0, 0], "3BTC_P2SH_P2WPKH_Address"),
            ("Litecoin", "p2pkh", vec![44, 2, 0, 0, 0], "L_LTC_P2PKH_Address"),
            ("Ethereum", "ethereum", vec![44, 60, 0, 0, 0], "0x_ETH_Address"),
        ];
        
        for (coin, script_type, path, address) in test_cases {
            cache.save_address(device_id, coin, script_type, &path, address, None).await.unwrap();
        }
        
        // Load device and verify all addresses are available
        let loaded = cache.load_device(device_id).await.unwrap();
        assert!(loaded.is_some());
        
        // Test each address is available
        assert!(cache.get_cached_address("Bitcoin", "p2pkh", &[44, 0, 0, 0, 0]).is_some());
        assert!(cache.get_cached_address("Bitcoin", "p2wpkh", &[84, 0, 0, 0, 0]).is_some());
        assert!(cache.get_cached_address("Bitcoin", "p2sh-p2wpkh", &[49, 0, 0, 0, 0]).is_some());
        assert!(cache.get_cached_address("Litecoin", "p2pkh", &[44, 2, 0, 0, 0]).is_some());
        assert!(cache.get_cached_address("Ethereum", "ethereum", &[44, 60, 0, 0, 0]).is_some());
    }
    

} 