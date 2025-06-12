use anyhow::{anyhow, Result};
use rusqlite::{Connection, params, OptionalExtension};
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use std::path::PathBuf;
use tracing::{debug, error, info, warn};
use crate::server::routes;
use tokio;

#[derive(Clone)]
pub struct DeviceCache {
    db: Arc<tokio::sync::Mutex<Connection>>,
    memory_cache: Arc<RwLock<MemoryCache>>,
}

#[derive(Default)]
pub struct MemoryCache {
    features: Option<CachedFeatures>,
    addresses: HashMap<AddressKey, CachedAddress>,
    device_id: Option<String>,
}

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
struct AddressKey {
    coin: String,
    script_type: String,
    path: Vec<u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CachedFeatures {
    pub device_id: String,
    pub label: Option<String>,
    pub vendor: Option<String>,
    pub major_version: Option<u32>,
    pub minor_version: Option<u32>,
    pub patch_version: Option<u32>,
    pub revision: Option<String>,
    pub firmware_hash: Option<String>,
    pub bootloader_hash: Option<String>,
    pub features_json: String,
    pub last_seen: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CachedAddress {
    pub address: String,
    pub pubkey: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Network {
    #[serde(serialize_with = "crate::server::cache::device_cache::as_string")]
    pub id: i64,
    pub chain_id_caip2: String,
    pub display_name: String,
    pub network_name: String,
    pub symbol: String,
    pub is_evm: bool,
    pub is_testnet: bool,
    pub enabled: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Path {
    #[serde(serialize_with = "crate::server::cache::device_cache::as_string", default)]
    pub id: i64,
    pub note: String,
    pub blockchain: Option<String>,
    pub symbol: Option<String>,
    pub symbol_swap_kit: Option<String>,
    pub networks: Vec<String>,
    pub script_type: String,
    pub available_script_types: Option<Vec<String>>,
    #[serde(rename = "type")]
    pub path_type: String,
    #[serde(rename = "addressNList")]
    pub address_n_list: Vec<u32>,
    #[serde(rename = "addressNListMaster")]
    pub address_n_list_master: Vec<u32>,
    pub curve: String,
    #[serde(rename = "showDisplay")]
    pub show_display: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CachedBalance {
    #[serde(serialize_with = "crate::server::cache::device_cache::as_string")]
    pub id: i64,
    pub device_id: String,
    pub caip: String,
    pub pubkey: String,
    pub balance: String,
    pub price_usd: String,
    pub value_usd: String,
    pub symbol: Option<String>,
    pub network_id: Option<String>,
    pub last_updated: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PortfolioSummary {
    #[serde(serialize_with = "crate::server::cache::device_cache::as_string")]
    pub id: i64,
    pub device_id: String,
    pub total_value_usd: String,
    pub network_count: i64,
    pub asset_count: i64,
    pub last_updated: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConfigEntry {
    pub key: String,
    pub value: String,
    pub description: Option<String>,
}

fn as_string<S>(x: &i64, s: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    s.serialize_str(&x.to_string())
}

impl DeviceCache {
    /// Fetch all enabled networks from the cache DB (for v2 endpoints)
    pub async fn get_enabled_networks(&self) -> Result<Vec<Network>> {
        let db = self.db.lock().await;
        let mut stmt = db.prepare("SELECT id, chain_id_caip2, display_name, network_name, symbol, is_evm, is_testnet, enabled FROM networks WHERE enabled = 1")?;
        let rows = stmt.query_map([], |row| {
            Ok(Network {
                id: row.get(0)?,
                chain_id_caip2: row.get(1)?,
                display_name: row.get(2)?,
                network_name: row.get(3)?,
                symbol: row.get(4)?,
                is_evm: row.get(5)?,
                is_testnet: row.get(6)?,
                enabled: row.get(7)?,
            })
        })?;
        let mut networks = Vec::new();
        for net in rows {
            networks.push(net?);
        }
        Ok(networks)
    }
    
    /// Add a new network or update an existing one by chain_id_caip2
    pub async fn add_network(&self, network: &Network) -> Result<i64> {
        let db = self.db.lock().await;
        
        // Check if network already exists by chain_id_caip2
        let existing_id: Option<i64> = db.query_row(
            "SELECT id FROM networks WHERE chain_id_caip2 = ?1",
            params![network.chain_id_caip2],
            |row| row.get(0)
        ).optional()?;
        
        if let Some(id) = existing_id {
            // Update existing network
            db.execute(
                "UPDATE networks SET 
                display_name = ?1, 
                network_name = ?2, 
                symbol = ?3, 
                is_evm = ?4, 
                is_testnet = ?5, 
                enabled = ?6 
                WHERE id = ?7",
                params![
                    network.display_name,
                    network.network_name,
                    network.symbol,
                    network.is_evm,
                    network.is_testnet,
                    network.enabled,
                    id
                ],
            )?;
            debug!("Updated network {} ({})", network.display_name, network.chain_id_caip2);
            Ok(id)
        } else {
            // Insert new network
            db.execute(
                "INSERT INTO networks 
                (chain_id_caip2, display_name, network_name, symbol, is_evm, is_testnet, enabled) 
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    network.chain_id_caip2,
                    network.display_name,
                    network.network_name,
                    network.symbol,
                    network.is_evm,
                    network.is_testnet,
                    network.enabled
                ],
            )?;
            let id = db.last_insert_rowid();
            debug!("Added new network {} ({})", network.display_name, network.chain_id_caip2);
            Ok(id)
        }
    }

    /// Open or create the device cache database
    pub fn open() -> Result<Self> {
        let cache_dir = Self::get_cache_dir()?;
        info!("🔍 DEBUG: cache_dir resolved to: {}", cache_dir.display());
        std::fs::create_dir_all(&cache_dir)?;
        
        let db_path = cache_dir.join("device_cache.db");
        info!("🔍 DEBUG: db_path resolved to: {}", db_path.display());
        info!("Opening device cache at: {}", db_path.display());
        
        let mut conn = Connection::open(&db_path)?;
        
        // Set up database configuration
        conn.pragma_update(None, "journal_mode", "DELETE")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        
        // Execute database schema
        let schema = include_str!("schema.sql");
        conn.execute_batch(schema)?;
        
        Ok(Self {
            db: Arc::new(tokio::sync::Mutex::new(conn)),
            memory_cache: Arc::new(RwLock::new(MemoryCache::default())),
        })
    }
    
    /// Get the cache directory based on OS
    fn get_cache_dir() -> Result<PathBuf> {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| anyhow!("Could not determine home directory"))?;
        
        #[cfg(target_os = "windows")]
        let cache_dir = dirs::data_dir()
            .ok_or_else(|| anyhow!("Could not determine data directory"))?
            .join("KeepKey")
            .join("kkcli");
        
        #[cfg(not(target_os = "windows"))]
        let cache_dir = home_dir
            .join(".keepkey")
            .join("kkcli");
        
        Ok(cache_dir)
    }
    
    /// Check if a device exists in the cache
    pub async fn has_device(&self, device_id: &str) -> Result<bool> {
        let clean_device_id = device_id.trim();
        info!("🔍 Checking if device exists in cache: {}", clean_device_id);
        let db = self.db.lock().await;
        let exists: bool = db.query_row(
            "SELECT EXISTS(SELECT 1 FROM devices WHERE device_id = ?1)",
            params![clean_device_id],
            |row| row.get(0),
        )?;
        info!("📋 Device {} exists in cache: {}", clean_device_id, exists);
        Ok(exists)
    }
    
    /// Check if device has ALL required cached addresses from default paths
    pub async fn has_cached_addresses(&self, device_id: &str) -> Result<bool> {
        let clean_device_id = device_id.trim();
        
        // Get the current address count for logging
        let db = self.db.lock().await;
        let address_count: i64 = db.query_row(
            "SELECT COUNT(*) FROM cached_addresses WHERE device_id = ?1",
            params![clean_device_id],
            |row| row.get(0),
        )?;
        info!("📊 Device {} currently has {} cached addresses", clean_device_id, address_count);
        
        // Dump existing addresses for debugging
        {
            let mut stmt = db.prepare("SELECT coin, script_type, derivation_path, address, pubkey FROM cached_addresses WHERE device_id = ?1")?;
            let mut rows = stmt.query(params![clean_device_id])?;
            let mut found_any = false;
            while let Some(row) = rows.next()? {
                let coin: String = row.get(0)?;
                let script_type: String = row.get(1)?;
                let path_json: String = row.get(2)?;
                let address: String = row.get(3)?;
                let pubkey: Option<String> = row.get(4)?;
                debug!("📋 Existing: {} {} {} {} {:?}", coin, script_type, path_json, address, pubkey);
                found_any = true;
            }
            if !found_any {
                info!("📋 No existing addresses found for device {}", clean_device_id);
            }
        } // stmt and rows are dropped here
        
        drop(db); // Release the lock before doing more complex operations
        
        // Get all paths from database to check what should be cached
        let paths = self.get_paths().await?;
        if paths.is_empty() {
            warn!("📂 No paths found in database - device needs frontload to load default paths");
            return Ok(false);
        }
        
        info!("🔍 Checking if ALL {} paths have their required addresses cached", paths.len());
        
        let mut missing_addresses = 0;
        let mut total_required = 0;
        
        // Build in-memory set of cached (coin, script_type, derivation_path json) from DB
        let mut cached_set: HashSet<(String,String,String)> = HashSet::new();
        {
            let db = self.db.lock().await;
            let mut stmt = db.prepare("SELECT coin, script_type, derivation_path FROM cached_addresses WHERE device_id = ?1")?;
            let rows = stmt.query_map(params![clean_device_id], |row| {
                let coin: String = row.get(0)?;
                let script_type: String = row.get(1)?;
                let path_json: String = row.get(2)?;
                Ok((coin.clone(), script_type.clone(), path_json.clone()))
            })?;
            for row_result in rows {
                match row_result {
                    Ok((coin, script_type, path_json)) => {
                        cached_set.insert((coin, script_type, path_json));
                    }
                    Err(e) => {
                        error!("Failed to parse cached address row: {}", e);
                        // Continue processing other addresses instead of failing
                    }
                }
            }
        }
        
        // Check each path to see if its required addresses are cached
        for path in &paths {
            for network in &path.networks {
                // Use the same logic as frontload to determine what addresses should be cached
                let (coin_name, script_type) = match self.get_coin_info_from_network_and_path(network, &path.script_type, &path.address_n_list) {
                    Ok(info) => info,
                    Err(_) => continue, // Skip unsupported networks
                };
                
                let is_utxo_network = network.starts_with("bip122:");
                
                if is_utxo_network {
                    // UTXO networks need xpub cached (use addressNList for account-level path)
                    let account_path = &path.address_n_list;
                    let xpub_script_type = format!("{}_xpub", script_type);
                    total_required += 1;
                    
                    let account_path_json = serde_json::to_string(account_path)?;
                    if !cached_set.contains(&(coin_name.clone(), xpub_script_type.clone(), account_path_json.clone())) {
                        missing_addresses += 1;
                        debug!("❌ Missing xpub for {} {} at path {:?}", coin_name, script_type, account_path);
                    }
                } else {
                    // Account-based networks need master address cached (use addressNListMaster)
                    let address_path = &path.address_n_list_master;
                    total_required += 1;
                    
                    let addr_json = serde_json::to_string(address_path)?;
                    if !cached_set.contains(&(coin_name.clone(), script_type.clone(), addr_json.clone())) {
                        missing_addresses += 1;
                        debug!("❌ Missing address for {} {} at path {:?}", coin_name, script_type, address_path);
                    }
                }
            }
        }
        
        let has_all_addresses = missing_addresses == 0;
        
        if has_all_addresses {
            info!("✅ All {} required addresses are cached - device is ready!", total_required);
        } else {
            info!("📝 Missing {} out of {} required addresses - device needs frontload", missing_addresses, total_required);
        }
        
        Ok(has_all_addresses)
    }
    
    /// Helper method to get coin info from network and path (used by has_cached_addresses)
    fn get_coin_info_from_network_and_path(&self, network: &str, script_type: &str, address_n_list: &[u32]) -> Result<(String, String)> {
        // Bitcoin-based networks (UTXO)
        if network.starts_with("bip122:") {
            // Use path analysis to distinguish different Bitcoin-based coins
            if address_n_list.len() >= 2 {
                let coin_type = address_n_list[1];
                match coin_type {
                    2147483648 => return Ok(("Bitcoin".to_string(), script_type.to_string())), // 0' (Bitcoin)
                    2147483649 => return Ok(("Bitcoin".to_string(), script_type.to_string())), // 1' (Bitcoin Testnet - use Bitcoin for KeepKey)
                    2147483650 => return Ok(("Litecoin".to_string(), script_type.to_string())), // 2' (Litecoin)
                    2147483651 => return Ok(("Dogecoin".to_string(), script_type.to_string())), // 3' (Dogecoin)
                    2147483653 => return Ok(("Dash".to_string(), script_type.to_string())), // 5' (Dash)
                    2147483668 => return Ok(("DigiByte".to_string(), script_type.to_string())), // 20' (DigiByte)
                    2147483781 => return Ok(("Zcash".to_string(), script_type.to_string())), // 133' (Zcash)
                    2147483793 => return Ok(("BitcoinCash".to_string(), script_type.to_string())), // 145' (Bitcoin Cash)
                    2147483804 => return Ok(("BitcoinGold".to_string(), script_type.to_string())), // 156' (Bitcoin Gold)
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
    
    /// Load device data from database into memory cache
    pub async fn load_device(&self, device_id: &str) -> Result<Option<CachedFeatures>> {
        // Clean the device_id by trimming any whitespace/newlines
        let clean_device_id = device_id.trim();
        
        // Use the shared database connection for consistency with save operations
        let db = self.db.lock().await;
        
        // Load features and addresses using shared connection
        let features: Option<CachedFeatures>;
        let mut cached_addresses: Vec<(AddressKey, CachedAddress)> = Vec::new();
        
        {
            // Load features
            features = db.query_row(
                "SELECT device_id, label, vendor, major_version, minor_version, patch_version,
                        revision, firmware_hash, bootloader_hash, features_json, last_seen
                 FROM devices WHERE device_id = ?1",
                params![clean_device_id],
                |row| {
                    Ok(CachedFeatures {
                        device_id: row.get(0)?,
                        label: row.get(1)?,
                        vendor: row.get(2)?,
                        major_version: row.get(3)?,
                        minor_version: row.get(4)?,
                        patch_version: row.get(5)?,
                        revision: row.get(6)?,
                        firmware_hash: row.get(7)?,
                        bootloader_hash: row.get(8)?,
                        features_json: row.get(9)?,
                        last_seen: row.get(10)?,
                    })
                },
            ).optional()?;
            
            if features.is_some() {
                // Load addresses using shared connection
                let mut stmt = db.prepare(
                    "SELECT coin, script_type, derivation_path, address, pubkey
                     FROM cached_addresses WHERE device_id = ?1"
                )?;
                
                let addresses = stmt.query_map(params![clean_device_id], |row| {
                    let coin: String = row.get(0)?;
                    let script_type: String = row.get(1)?;
                    let path_json: String = row.get(2)?;
                    let address: String = row.get(3)?;
                    let pubkey: Option<String> = row.get(4)?;
                    
                    let path: Vec<u32> = serde_json::from_str(&path_json)
                        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                            2, rusqlite::types::Type::Text, Box::new(e)
                        ))?;
                    
                    Ok((
                        AddressKey { coin, script_type, path },
                        CachedAddress { address, pubkey }
                    ))
                })?;
                
                // Consume the iterator while database lock is held
                for addr_result in addresses {
                    match addr_result {
                        Ok((key, value)) => {
                            cached_addresses.push((key, value));
                        }
                        Err(e) => {
                            error!("Failed to parse cached address row: {}", e);
                            // Continue processing other addresses instead of failing
                        }
                    }
                }
                
                info!("📈 Database query result: Loaded {} cached addresses for device {}", cached_addresses.len(), clean_device_id);

            }
        } // Shared connection is dropped here
        
        // Now populate memory cache if we have features
        if let Some(ref features) = features {
            let mut cache = self.memory_cache.write().unwrap();
            cache.features = Some(features.clone());
            cache.device_id = Some(clean_device_id.to_string());
            
            let address_count = cached_addresses.len();
            for (key, value) in cached_addresses {
                cache.addresses.insert(key, value);
            }
            
            info!("Loaded {} cached addresses into memory for device {}", address_count, clean_device_id);
        }
        
        Ok(features)
    }
    
    /// Save device features to database
    pub async fn save_features(&self, features: &routes::Features, device_id: &str) -> Result<()> {
        let features_json = serde_json::to_string(features)?;
        let now = chrono::Utc::now().timestamp();
        
        let db = self.db.lock().await;
        db.execute(
            "INSERT INTO devices 
             (device_id, label, vendor, major_version, minor_version, patch_version,
              revision, firmware_hash, bootloader_hash, features_json, last_seen, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11,
                     COALESCE((SELECT created_at FROM devices WHERE device_id = ?1), ?11))
             ON CONFLICT(device_id) DO UPDATE SET
               label = excluded.label,
               vendor = excluded.vendor,
               major_version = excluded.major_version,
               minor_version = excluded.minor_version,
               patch_version = excluded.patch_version,
               revision = excluded.revision,
               firmware_hash = excluded.firmware_hash,
               bootloader_hash = excluded.bootloader_hash,
               features_json = excluded.features_json,
               last_seen = excluded.last_seen",
            params![
                device_id,
                features.label.as_deref(),
                features.vendor.as_deref(),
                features.major_version,
                features.minor_version,
                features.patch_version,
                features.revision.as_deref(),
                features.firmware_hash.as_deref(),
                features.bootloader_hash.as_deref(),
                features_json,
                now,
            ],
        )?;
        

        
        // Update memory cache
        let mut cache = self.memory_cache.write().unwrap();
        cache.features = Some(CachedFeatures {
            device_id: device_id.to_string(),
            label: features.label.clone(),
            vendor: features.vendor.clone(),
            major_version: features.major_version,
            minor_version: features.minor_version,
            patch_version: features.patch_version,
            revision: features.revision.clone(),
            firmware_hash: features.firmware_hash.clone(),
            bootloader_hash: features.bootloader_hash.clone(),
            features_json,
            last_seen: now,
        });
        cache.device_id = Some(device_id.to_string());
        
        info!("Saved features for device {}", device_id);
        Ok(())
    }
    
    /// Save an address to the cache database 
    /// 
    /// 🚨 CRITICAL WARNING: ON DELETE CASCADE DANGER 🚨
    /// 
    /// This function previously used "INSERT OR REPLACE" which caused CASCADE deletions!
    /// The cached_addresses table has: FOREIGN KEY (device_id) REFERENCES devices(device_id) ON DELETE CASCADE
    /// When "INSERT OR REPLACE" encounters a UNIQUE constraint conflict, it:
    /// 1. DELETES the existing row (triggering CASCADE deletion of ALL related data!)
    /// 2. INSERTS the new row
    /// 
    /// This was the root cause of the cache clearing bug documented in:
    /// /docs/debugging/resolved_cache_bug_summary_20250609.md
    /// 
    /// SOLUTION: Use "INSERT ... ON CONFLICT DO UPDATE" instead, which updates in-place
    /// without triggering CASCADE deletions.
    /// 
    /// ⚠️ NEVER USE "INSERT OR REPLACE" ON TABLES WITH CASCADE FOREIGN KEYS! ⚠️
    pub async fn save_address(
        &self,
        device_id: &str,
        coin: &str,
        script_type: &str,
        path: &[u32],
        address: &str,
        pubkey: Option<&str>,
    ) -> Result<()> {
        let path_json = serde_json::to_string(path)?;
        let now = chrono::Utc::now().timestamp();
        
        let db = self.db.lock().await;
        
        // 🚨 FIXED: Using INSERT ... ON CONFLICT DO UPDATE instead of INSERT OR REPLACE
        // to prevent CASCADE deletion of cached data!
        db.execute(
            "INSERT INTO cached_addresses 
             (device_id, coin, script_type, derivation_path, address, pubkey, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(device_id, coin, script_type, derivation_path) DO UPDATE SET
               address = excluded.address,
               pubkey = excluded.pubkey,
               created_at = excluded.created_at",
            params![device_id, coin, script_type, path_json, address, pubkey, now],
        )?;

        // FAIL FAST: Immediately query for the just-saved row
        let row_exists: bool = db.query_row(
            "SELECT EXISTS(SELECT 1 FROM cached_addresses WHERE device_id = ?1 AND coin = ?2 AND script_type = ?3 AND derivation_path = ?4 AND address = ?5)",
            params![device_id, coin, script_type, path_json, address],
            |row| row.get(0),
        )?;
        if !row_exists {
            error!("FAIL FAST: Failed to verify address save for {}/{} at path {:?} (address: {})", coin, script_type, path, address);
            return Err(anyhow::anyhow!("Failed to verify address save for {}/{} at path {:?} (address: {})", coin, script_type, path, address));
        } else {
            info!("✅ Verified address save for {}/{} at path {:?} (address: {})", coin, script_type, path, address);
        }

        // Update memory cache
        let mut cache = self.memory_cache.write().unwrap();
        let key = AddressKey {
            coin: coin.to_string(),
            script_type: script_type.to_string(),
            path: path.to_vec(),
        };
        cache.addresses.insert(key, CachedAddress {
            address: address.to_string(),
            pubkey: pubkey.map(|s| s.to_string()),
        });
        
        debug!("Cached address for {}/{} at path {:?}", coin, script_type, path);
        Ok(())
    }
    
    /// Get a cached address from memory
    pub fn get_cached_address(
        &self,
        coin: &str,
        script_type: &str,
        path: &[u32],
    ) -> Option<CachedAddress> {
        let cache = self.memory_cache.read().unwrap();
        let key = AddressKey {
            coin: coin.to_string(),
            script_type: script_type.to_string(),
            path: path.to_vec(),
        };
        cache.addresses.get(&key).cloned()
    }
    
    /// Get cached features from memory
    pub fn get_cached_features(&self) -> Option<CachedFeatures> {
        let cache = self.memory_cache.read().unwrap();
        cache.features.clone()
    }
    
    /// Get currently loaded device ID (with database fallback)
    /// 
    /// ✅ FIXED: Now uses database fallback when memory cache is empty
    /// This fixes the "No device found in cache" errors that occur during startup.
    pub fn get_device_id(&self) -> Option<String> {
        // First try memory cache (fast path)
        {
            let cache = self.memory_cache.read().unwrap();
            if let Some(device_id) = &cache.device_id {
                return Some(device_id.clone());
            }
        }
        
        // Memory cache empty - try database fallback (slower but reliable)
        match self.get_first_device_from_db() {
            Ok(Some(device_id)) => {
                info!("💾 Using database fallback for device ID: {}", device_id);
                
                // Populate memory cache for next time
                {
                    let mut cache = self.memory_cache.write().unwrap();
                    cache.device_id = Some(device_id.clone());
                }
                
                Some(device_id)
            },
            Ok(None) => {
                debug!("📭 No device found in database");
                None
            },
            Err(e) => {
                warn!("❌ Database fallback failed: {}", e);
                None
            }
        }
    }
    
    /// Get first device ID from database (fallback method)
    pub fn get_first_device_from_db(&self) -> Result<Option<String>> {
        let db = self.db.blocking_lock();
        
        let device_id: Option<String> = db.query_row(
            "SELECT device_id FROM devices LIMIT 1",
            [],
            |row| Ok(row.get::<_, String>(0)?),
        ).optional()?;
        
        Ok(device_id)
    }
    
    /// Clear all caches for a device
    pub async fn clear_device(&self, device_id: &str) -> Result<()> {
        let db = self.db.lock().await;
        db.execute("DELETE FROM devices WHERE device_id = ?1", params![device_id])?;
        
        // Clear memory cache if it's the current device
        let mut cache = self.memory_cache.write().unwrap();
        if cache.device_id.as_deref() == Some(device_id) {
            *cache = MemoryCache::default();
        }
        
        info!("Cleared all cached data for device {}", device_id);
        Ok(())
    }

    /// Get all paths from the database
    pub async fn get_paths(&self) -> Result<Vec<Path>> {
        let db = self.db.lock().await;
        
        let mut stmt = db.prepare(
            "SELECT id, device_id, note, blockchain, symbol, symbol_swap_kit, networks, 
             script_type, available_script_types, type, address_n_list, 
             address_n_list_master, curve, show_display FROM paths"
        )?;
        
        let rows = stmt.query_map([], |row| {
            let networks_json: String = row.get(6)?; // Updated index
            let networks: Vec<String> = serde_json::from_str(&networks_json)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(6, 
                    rusqlite::types::Type::Text, Box::new(e)))?;
                
            let available_script_types: Option<Vec<String>> = row.get::<_, Option<String>>(8)? // Updated index
                .map(|s| serde_json::from_str(&s)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(8, 
                        rusqlite::types::Type::Text, Box::new(e))))
                .transpose()?;
                
            let address_n_list_json: String = row.get(10)?; // Updated index
            let address_n_list: Vec<u32> = serde_json::from_str(&address_n_list_json)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(10, 
                    rusqlite::types::Type::Text, Box::new(e)))?;
                
            let address_n_list_master_json: String = row.get(11)?; // Updated index
            let address_n_list_master: Vec<u32> = serde_json::from_str(&address_n_list_master_json)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(11, 
                    rusqlite::types::Type::Text, Box::new(e)))?;
                
            Ok(Path {
                id: row.get(0)?,
                // Skip device_id (index 1) since Path struct doesn't include it
                note: row.get(2)?,
                blockchain: row.get(3)?,
                symbol: row.get(4)?,
                symbol_swap_kit: row.get(5)?,
                networks,
                script_type: row.get(7)?,
                available_script_types,
                path_type: row.get(9)?,
                address_n_list,
                address_n_list_master,
                curve: row.get(12)?,
                show_display: row.get(13)?,
            })
        })?;
        
        let mut paths = Vec::new();
        for path in rows {
            paths.push(path?);
        }
        
        Ok(paths)
    }
    
    /// Get a specific path by ID
    pub async fn get_path(&self, id: i64) -> Result<Option<Path>> {
        let db = self.db.lock().await;
        
        let result = db.query_row(
            "SELECT id, device_id, note, blockchain, symbol, symbol_swap_kit, networks, 
             script_type, available_script_types, type, address_n_list, 
             address_n_list_master, curve, show_display FROM paths WHERE id = ?1",
            params![id],
            |row| {
                let networks_json: String = row.get(6)?; // Updated index
                let networks: Vec<String> = serde_json::from_str(&networks_json)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(6, 
                        rusqlite::types::Type::Text, Box::new(e)))?;
                    
                let available_script_types: Option<Vec<String>> = row.get::<_, Option<String>>(8)? // Updated index
                    .map(|s| serde_json::from_str(&s)
                        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(8, 
                            rusqlite::types::Type::Text, Box::new(e))))
                    .transpose()?;
                    
                let address_n_list_json: String = row.get(10)?; // Updated index
                let address_n_list: Vec<u32> = serde_json::from_str(&address_n_list_json)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(10, 
                        rusqlite::types::Type::Text, Box::new(e)))?;
                    
                let address_n_list_master_json: String = row.get(11)?; // Updated index
                let address_n_list_master: Vec<u32> = serde_json::from_str(&address_n_list_master_json)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(11, 
                        rusqlite::types::Type::Text, Box::new(e)))?;
                    
                Ok(Path {
                    id: row.get(0)?,
                    // Skip device_id (index 1) since Path struct doesn't include it
                    note: row.get(2)?,
                    blockchain: row.get(3)?,
                    symbol: row.get(4)?,
                    symbol_swap_kit: row.get(5)?,
                    networks,
                    script_type: row.get(7)?,
                    available_script_types,
                    path_type: row.get(9)?,
                    address_n_list,
                    address_n_list_master,
                    curve: row.get(12)?,
                    show_display: row.get(13)?,
                })
            },
        ).optional()?;
        
        Ok(result)
    }
    
    /// Add a new path to the database
    pub async fn add_path(&self, path: &Path) -> Result<i64> {
        let networks_json = serde_json::to_string(&path.networks)?;
        let available_script_types_json = path.available_script_types.as_ref()
            .map(|types| serde_json::to_string(types))
            .transpose()?;
        let address_n_list_json = serde_json::to_string(&path.address_n_list)?;
        let address_n_list_master_json = serde_json::to_string(&path.address_n_list_master)?;
        
        let db = self.db.lock().await;
        
        db.execute(
            "INSERT INTO paths 
            (device_id, note, blockchain, symbol, symbol_swap_kit, networks, script_type, 
             available_script_types, type, address_n_list, address_n_list_master, 
             curve, show_display) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                None::<String>, // device_id = NULL for global paths
                path.note,
                path.blockchain,
                path.symbol,
                path.symbol_swap_kit,
                networks_json,
                path.script_type,
                available_script_types_json,
                path.path_type,
                address_n_list_json,
                address_n_list_master_json,
                path.curve,
                path.show_display
            ],
        )?;
        
        Ok(db.last_insert_rowid())
    }
    
    /// Update an existing path
    pub async fn update_path(&self, id: i64, path: &Path) -> Result<()> {
        let networks_json = serde_json::to_string(&path.networks)?;
        let available_script_types_json = path.available_script_types.as_ref()
            .map(|types| serde_json::to_string(types))
            .transpose()?;
        let address_n_list_json = serde_json::to_string(&path.address_n_list)?;
        let address_n_list_master_json = serde_json::to_string(&path.address_n_list_master)?;
        
        let db = self.db.lock().await;
        
        let rows_affected = db.execute(
            "UPDATE paths SET 
            note = ?1, 
            blockchain = ?2, 
            symbol = ?3, 
            symbol_swap_kit = ?4, 
            networks = ?5, 
            script_type = ?6, 
            available_script_types = ?7, 
            type = ?8, 
            address_n_list = ?9, 
            address_n_list_master = ?10, 
            curve = ?11, 
            show_display = ?12
            WHERE id = ?13",
            params![
                path.note,
                path.blockchain,
                path.symbol,
                path.symbol_swap_kit,
                networks_json,
                path.script_type,
                available_script_types_json,
                path.path_type,
                address_n_list_json,
                address_n_list_master_json,
                path.curve,
                path.show_display,
                id
            ],
        )?;
        
        if rows_affected == 0 {
            return Err(anyhow!("Path with ID {} not found", id));
        }
        
        Ok(())
    }
    
    /// Delete a path by ID
    pub async fn delete_path(&self, id: i64) -> Result<()> {
        let db = self.db.lock().await;
        
        let rows_affected = db.execute("DELETE FROM paths WHERE id = ?1", params![id])?;
        
        if rows_affected == 0 {
            return Err(anyhow!("Path with ID {} not found", id));
        }
        
        Ok(())
    }

    // === Configuration Methods ===

    /// Get a configuration value
    pub async fn get_config(&self, key: &str) -> Result<Option<String>> {
        let db = self.db.lock().await;
        let value: Option<String> = db.query_row(
            "SELECT value FROM config WHERE key = ?1",
            params![key],
            |row| row.get(0),
        ).optional()?;
        Ok(value)
    }

    /// Set a configuration value
    pub async fn set_config(&self, key: &str, value: &str, description: Option<&str>) -> Result<()> {
        let db = self.db.lock().await;
        let now = chrono::Utc::now().timestamp();
        
        db.execute(
            "INSERT OR REPLACE INTO config (key, value, description, updated_at) 
             VALUES (?1, ?2, ?3, ?4)",
            params![key, value, description, now],
        )?;
        
        debug!("Set config: {} = {}", key, value);
        Ok(())
    }

    /// Get Pioneer server URL from config
    pub async fn get_pioneer_server_url(&self) -> Result<String> {
        match self.get_config("pioneer_server_url").await? {
            Some(url) => Ok(url),
            None => {
                // Set default if not found
                let default_url = "https://pioneers.dev";
                self.set_config("pioneer_server_url", default_url, Some("Pioneer server URL for balance and market data")).await?;
                Ok(default_url.to_string())
            }
        }
    }

    // === Balance Methods ===

    /// Save balances to cache
    /// 
    /// 🚨 CRITICAL WARNING: ON DELETE CASCADE DANGER 🚨
    /// 
    /// This function previously used "INSERT OR REPLACE" which caused CASCADE deletions!
    /// The cached_balances table has: FOREIGN KEY (device_id) REFERENCES devices(device_id) ON DELETE CASCADE
    /// When "INSERT OR REPLACE" encounters a UNIQUE constraint conflict, it:
    /// 1. DELETES the existing row (triggering CASCADE deletion of ALL related data!)
    /// 2. INSERTS the new row
    /// 
    /// This was the root cause of the cache clearing bug documented in:
    /// /docs/debugging/resolved_cache_bug_summary_20250609.md
    /// 
    /// SOLUTION: Use "INSERT ... ON CONFLICT DO UPDATE" instead, which updates in-place
    /// without triggering CASCADE deletions.
    /// 
    /// ⚠️ NEVER USE "INSERT OR REPLACE" ON TABLES WITH CASCADE FOREIGN KEYS! ⚠️
    pub async fn save_balances(&self, device_id: &str, balances: &[CachedBalance]) -> Result<()> {
        let db = self.db.lock().await;
        let now = chrono::Utc::now().timestamp();
        
        for balance in balances {
            // 🚨 FIXED: Using INSERT ... ON CONFLICT DO UPDATE instead of INSERT OR REPLACE
            // to prevent CASCADE deletion of cached data!
            db.execute(
                "INSERT INTO cached_balances 
                 (device_id, caip, pubkey, balance, price_usd, value_usd, symbol, network_id, last_updated)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                 ON CONFLICT(device_id, caip, pubkey) DO UPDATE SET
                   balance = excluded.balance,
                   price_usd = excluded.price_usd,
                   value_usd = excluded.value_usd,
                   symbol = excluded.symbol,
                   network_id = excluded.network_id,
                   last_updated = excluded.last_updated",
                params![
                    device_id,
                    balance.caip,
                    balance.pubkey,
                    balance.balance,
                    balance.price_usd,
                    balance.value_usd,
                    balance.symbol,
                    balance.network_id,
                    now
                ],
            )?;
        }
        
        // 🚨 FIXED: Clear portfolio summary cache when balances update
        // This ensures USD values update immediately in the frontend
        db.execute(
            "DELETE FROM portfolio_summaries WHERE device_id = ?1",
            params![device_id],
        )?;
        
        info!("💾 Saved {} balances for device {} and cleared portfolio summary cache", balances.len(), device_id);
        Ok(())
    }

    /// Get cached balances for a device
    pub async fn get_cached_balances(&self, device_id: &str) -> Result<Vec<CachedBalance>> {
        let db = self.db.lock().await;
        
        let mut stmt = db.prepare(
            "SELECT id, device_id, caip, pubkey, balance, price_usd, value_usd, 
             symbol, network_id, last_updated
             FROM cached_balances WHERE device_id = ?1"
        )?;
        
        let rows = stmt.query_map(params![device_id], |row| {
            Ok(CachedBalance {
                id: row.get(0)?,
                device_id: row.get(1)?,
                caip: row.get(2)?,
                pubkey: row.get(3)?,
                balance: row.get(4)?,
                price_usd: row.get(5)?,
                value_usd: row.get(6)?,
                symbol: row.get(7)?,
                network_id: row.get(8)?,
                last_updated: row.get(9)?,
            })
        })?;
        
        let mut balances = Vec::new();
        for balance in rows {
            balances.push(balance?);
        }
        
        Ok(balances)
    }

    /// Check if balances need refresh (older than 1 hour)
    pub async fn balances_need_refresh(&self, device_id: &str) -> Result<bool> {
        let db = self.db.lock().await;
        let one_hour_ago = chrono::Utc::now().timestamp() - 3600; // 1 hour in seconds (was 600 = 10 min)
        
        let count: i64 = db.query_row(
            "SELECT COUNT(*) FROM cached_balances 
             WHERE device_id = ?1 AND last_updated > ?2",
            params![device_id, one_hour_ago],
            |row| row.get(0),
        )?;
        
        // If we have no recent balances, we need refresh
        let needs_refresh = count == 0;
        info!("💾 Cache check for device {}: {} fresh balances (< 1h old) → need refresh: {}", 
            device_id, count, needs_refresh);
        Ok(needs_refresh)
    }

    /// Clear old balances (older than 1 hour)
    pub async fn clear_old_balances(&self, device_id: &str) -> Result<()> {
        let db = self.db.lock().await;
        let one_hour_ago = chrono::Utc::now().timestamp() - 3600; // 1 hour in seconds
        
        let rows_affected = db.execute(
            "DELETE FROM cached_balances WHERE device_id = ?1 AND last_updated < ?2",
            params![device_id, one_hour_ago],
        )?;
        
        if rows_affected > 0 {
            debug!("Cleared {} old balances for device {}", rows_affected, device_id);
        }
        
        Ok(())
    }

    /// Save portfolio summary
    /// 
    /// 🚨 CRITICAL WARNING: ON DELETE CASCADE DANGER 🚨
    /// 
    /// This function previously used "INSERT OR REPLACE" which caused CASCADE deletions!
    /// The portfolio_summaries table has: FOREIGN KEY (device_id) REFERENCES devices(device_id) ON DELETE CASCADE
    /// When "INSERT OR REPLACE" encounters a UNIQUE constraint conflict, it:
    /// 1. DELETES the existing row (triggering CASCADE deletion of ALL related data!)
    /// 2. INSERTS the new row
    /// 
    /// This was the root cause of the cache clearing bug documented in:
    /// /docs/debugging/resolved_cache_bug_summary_20250609.md
    /// 
    /// SOLUTION: Use "INSERT ... ON CONFLICT DO UPDATE" instead, which updates in-place
    /// without triggering CASCADE deletions.
    /// 
    /// ⚠️ NEVER USE "INSERT OR REPLACE" ON TABLES WITH CASCADE FOREIGN KEYS! ⚠️
    pub async fn save_portfolio_summary(&self, device_id: &str, summary: &PortfolioSummary) -> Result<()> {
        let db = self.db.lock().await;
        let now = chrono::Utc::now().timestamp();
        
        // 🚨 FIXED: Using INSERT ... ON CONFLICT DO UPDATE instead of INSERT OR REPLACE
        // to prevent CASCADE deletion of cached data!
        db.execute(
            "INSERT INTO portfolio_summaries 
             (device_id, total_value_usd, network_count, asset_count, last_updated)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(device_id) DO UPDATE SET
               total_value_usd = excluded.total_value_usd,
               network_count = excluded.network_count,
               asset_count = excluded.asset_count,
               last_updated = excluded.last_updated",
            params![
                device_id,
                summary.total_value_usd,
                summary.network_count,
                summary.asset_count,
                now
            ],
        )?;
        
        debug!("Saved portfolio summary for device {}", device_id);
        Ok(())
    }

    /// Get portfolio summary
    pub async fn get_portfolio_summary(&self, device_id: &str) -> Result<Option<PortfolioSummary>> {
        let db = self.db.lock().await;
        
        let summary = db.query_row(
            "SELECT id, device_id, total_value_usd, network_count, asset_count, last_updated
             FROM portfolio_summaries WHERE device_id = ?1",
            params![device_id],
            |row| {
                Ok(PortfolioSummary {
                    id: row.get(0)?,
                    device_id: row.get(1)?,
                    total_value_usd: row.get(2)?,
                    network_count: row.get(3)?,
                    asset_count: row.get(4)?,
                    last_updated: row.get(5)?,
                })
            },
        ).optional()?;
        
        Ok(summary)
    }

    // === Debug Methods ===
    
    /// Debug method to test address loading with detailed logging
    pub async fn debug_load_addresses(&self, device_id: &str) -> Result<Vec<String>> {
        let clean_device_id = device_id.trim();
        info!("🔍 DEBUG: Starting address load for device {}", clean_device_id);
        
        let db = self.db.lock().await;
        info!("🔍 DEBUG: Acquired database lock");
        
        // First, let's see if we can query devices table
        let device_exists = db.query_row(
            "SELECT EXISTS(SELECT 1 FROM devices WHERE device_id = ?1)",
            params![clean_device_id],
            |row| row.get::<_, bool>(0),
        )?;
        info!("🔍 DEBUG: Device exists in devices table: {}", device_exists);
        
        // Now let's count addresses
        let address_count: i64 = db.query_row(
            "SELECT COUNT(*) FROM cached_addresses WHERE device_id = ?1",
            params![clean_device_id],
            |row| row.get(0),
        )?;
        info!("🔍 DEBUG: Address count from COUNT query: {}", address_count);
        
        if address_count == 0 {
            error!("FAIL FAST: No cached addresses found for device {}! DB is present but empty. Stopping.", clean_device_id);
            return Err(anyhow::anyhow!("No cached addresses found for device {}! DB is present but empty. Stopping.", clean_device_id));
        }
        
        // Now let's try to actually fetch them
        let mut stmt = db.prepare(
            "SELECT coin, script_type, derivation_path, address, pubkey
             FROM cached_addresses WHERE device_id = ?1"
        )?;
        info!("🔍 DEBUG: Prepared statement successfully");
        
        let rows = stmt.query_map(params![clean_device_id], |row| {
            let coin: String = row.get(0)?;
            let script_type: String = row.get(1)?;
            let path_json: String = row.get(2)?;
            let address: String = row.get(3)?;
            let pubkey: Option<String> = row.get(4)?;
            
            info!("🔍 DEBUG: Processing row - coin: {}, script_type: {}, path_json: {}", coin, script_type, path_json);
            
            // Try to parse the JSON path
            match serde_json::from_str::<Vec<u32>>(&path_json) {
                Ok(path) => {
                    info!("🔍 DEBUG: Successfully parsed path: {:?}", path);
                    Ok(format!("{}/{}/{}", coin, script_type, address))
                }
                Err(e) => {
                    error!("🔍 DEBUG: Failed to parse path JSON '{}': {}", path_json, e);
                    Err(rusqlite::Error::FromSqlConversionFailure(
                        2, rusqlite::types::Type::Text, Box::new(e)
                    ))
                }
            }
        })?;
        
        let mut addresses = Vec::new();
        for (i, row_result) in rows.enumerate() {
            info!("🔍 DEBUG: Processing row {}", i);
            match row_result {
                Ok(address_info) => {
                    info!("🔍 DEBUG: Successfully processed address: {}", address_info);
                    addresses.push(address_info);
                }
                Err(e) => {
                    error!("🔍 DEBUG: Failed to process row {}: {}", i, e);
                }
            }
        }
        
        info!("🔍 DEBUG: Final result - loaded {} addresses", addresses.len());
        Ok(addresses)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // Helper function to create test cache
    async fn create_test_cache() -> Result<DeviceCache> {
        let temp_dir = TempDir::new()?;
        create_test_cache_with_path(temp_dir.path()).await
    }

    async fn create_test_cache_with_path(db_path: &std::path::Path) -> Result<DeviceCache> {
        let db_file = db_path.join("test_device_cache.db");
        let conn = Connection::open(&db_file)?;
        
        // Enable WAL mode like in production
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        
        // Execute database schema
        let schema = include_str!("schema.sql");
        conn.execute_batch(schema)?;
        
        Ok(DeviceCache {
            db: Arc::new(tokio::sync::Mutex::new(conn)),
            memory_cache: Arc::new(RwLock::new(MemoryCache::default())),
        })
    }

    fn mock_routes_features() -> crate::server::routes::Features {
        crate::server::routes::Features {
            vendor: Some("KeepKey".to_string()),
            major_version: Some(1),
            minor_version: Some(0),
            patch_version: Some(0),
            bootloader_mode: Some(false),
            device_id: Some("test_device_123".to_string()),
            pin_protection: Some(false),
            passphrase_protection: Some(false),
            language: Some("english".to_string()),
            label: Some("Test KeepKey".to_string()),
            initialized: Some(true),
            revision: Some("test_revision".to_string()),
            firmware_hash: Some("test_firmware_hash".to_string()),
            bootloader_hash: Some("test_bootloader_hash".to_string()),
            imported: Some(false),
            pin_cached: Some(false),
            passphrase_cached: Some(false),
            wipe_code_protection: Some(false),
            auto_lock_delay_ms: Some(0),
            policies: None,
            model: Some("K1-14AM".to_string()),
            firmware_variant: Some("standard".to_string()),
            no_backup: Some(false),
        }
    }

    /// Test that reproduces the exact startup cache loading bug scenario
    #[tokio::test] 
    async fn test_startup_cache_loading_bug_reproduction() {
        let temp_dir = TempDir::new().unwrap();
        let device_id = "333433373337333430463437333633333146303033423030"; // Real device ID from logs
        
        // Simulate first startup: Device detected, features saved, addresses cached
        {
            let cache = create_test_cache_with_path(temp_dir.path()).await.unwrap();
            let features = mock_routes_features();
            
            // Save device features (this always happens on device detection)
            cache.save_features(&features, device_id).await.unwrap();
            
            // Simulate successful frontload - save addresses
            cache.save_address(device_id, "bitcoin", "p2pkh", &[44, 0, 0, 0, 0], "1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2", Some("pubkey1")).await.unwrap();
            cache.save_address(device_id, "bitcoin", "p2pkh", &[44, 0, 0, 0, 1], "1C5bSj1iEGUgSTbziymG7Cn18ENQuT36vv", Some("pubkey2")).await.unwrap();
            
            // Verify data exists in first session
            assert!(cache.has_device(device_id).await.unwrap());
            assert!(cache.has_cached_addresses(device_id).await.unwrap());
            
            // Verify memory cache is populated
            cache.load_device(device_id).await.unwrap();
            assert!(cache.get_cached_address("bitcoin", "p2pkh", &[44, 0, 0, 0, 0]).is_some());
        } // Simulate application shutdown
        
        // Simulate second startup: Should load existing cache but logs show it doesn't
        {
            let cache = create_test_cache_with_path(temp_dir.path()).await.unwrap();
            
            // This is what the logs show - device exists but no addresses detected
            println!("🧪 Testing cache persistence after restart...");
            assert!(cache.has_device(device_id).await.unwrap(), "Device should exist in cache");
            
            let has_addresses = cache.has_cached_addresses(device_id).await.unwrap();
            println!("🧪 Device has cached addresses: {}", has_addresses);
            
            if !has_addresses {
                println!("🚨 BUG REPRODUCED: Device exists but has 0 cached addresses!");
                println!("   This matches the logs showing 'Device has 0 cached addresses'");
                
                // Let's debug the database directly
                let db = cache.db.lock().await;
                let total_devices: i64 = db.query_row("SELECT COUNT(*) FROM devices", [], |row| row.get(0)).unwrap();
                let total_addresses: i64 = db.query_row("SELECT COUNT(*) FROM cached_addresses", [], |row| row.get(0)).unwrap();
                let device_addresses: i64 = db.query_row("SELECT COUNT(*) FROM cached_addresses WHERE device_id = ?1", [device_id], |row| row.get(0)).unwrap();
                
                println!("🧪 Database debug:");
                println!("   Total devices: {}", total_devices);
                println!("   Total addresses: {}", total_addresses);
                println!("   Addresses for this device: {}", device_addresses);
                
                if total_addresses > 0 && device_addresses == 0 {
                    println!("🚨 CRITICAL: Addresses exist in DB but not associated with device!");
                    
                    // Check for device_id mismatch
                    let mut stmt = db.prepare("SELECT DISTINCT device_id FROM cached_addresses LIMIT 3").unwrap();
                    let device_ids = stmt.query_map([], |row| {
                        Ok(row.get::<_, String>(0).unwrap())
                    }).unwrap();
                    
                    for (i, device_id_result) in device_ids.enumerate() {
                        if let Ok(db_device_id) = device_id_result {
                            println!("🧪 DB device_id #{}: '{}'", i+1, db_device_id);
                            println!("🧪 Query device_id:   '{}'", device_id);
                            println!("🧪 IDs match: {}", db_device_id == device_id);
                        }
                    }
                }
                
                // Try to load device and see what happens
                println!("🧪 Attempting to load device...");
                let loaded_features = cache.load_device(device_id).await.unwrap();
                assert!(loaded_features.is_some(), "Features should load");
                
                // Check if addresses are now in memory
                let addr_check = cache.get_cached_address("bitcoin", "p2pkh", &[44, 0, 0, 0, 0]);
                println!("🧪 Address in memory after load_device: {}", addr_check.is_some());
                
                if addr_check.is_none() {
                    println!("🚨 CONFIRMED BUG: load_device() is not loading addresses into memory!");
                }
            } else {
                // If addresses are detected, verify they can be loaded
                cache.load_device(device_id).await.unwrap();
                assert!(cache.get_cached_address("bitcoin", "p2pkh", &[44, 0, 0, 0, 0]).is_some(), 
                       "Address should be loadable from cache");
                println!("✅ Cache persistence working correctly");
            }
        }
    }
} 