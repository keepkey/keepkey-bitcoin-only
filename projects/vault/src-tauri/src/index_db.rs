use anyhow::Result;
use rusqlite::{Connection, OpenFlags, params};
use dirs;
use chrono::Utc;
use serde::{Serialize, Deserialize};
use serde_json;

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceRecord {
    pub device_id: String,
    pub vendor: Option<String>,
    pub model: Option<String>,
    pub label: Option<String>,
    pub firmware_variant: Option<String>,
    pub firmware_version: Option<String>,
    pub bootloader_mode: bool,
    pub initialized: bool,
    pub pin_protection: bool,
    pub passphrase_protection: bool,
    pub first_seen: i64,
    pub last_seen: i64,
    pub features: Option<String>,
    pub is_connected: bool,
}

pub struct IndexDb {
    conn: Connection,
}

impl IndexDb {
    /// Check if database file exists (for first-time detection)
    pub fn database_exists() -> bool {
        if let Some(home_dir) = dirs::home_dir() {
            let db_path = home_dir.join(".keepkey/index.db");
            db_path.exists()
        } else {
            false
        }
    }
    
    pub fn open() -> Result<Self> {
        // Get the path to ~/.keepkey/index.db
        let data_dir = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?
            .join(".keepkey");
        
        // Create directory if it doesn't exist
        std::fs::create_dir_all(&data_dir)?;
        
        let db_path = data_dir.join("index.db");
        
        // Only log on first open
        static FIRST_OPEN: std::sync::Once = std::sync::Once::new();
        FIRST_OPEN.call_once(|| {
            log::info!("Opening database at: {:?}", db_path);
        });
        
        // Open connection with SQLite flags
        let conn = Connection::open_with_flags(
            db_path,
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE,
        )?;
        
        // Enable WAL mode for better performance
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        
        // Create tables if they don't exist
        conn.execute_batch(SCHEMA)?;
        
        Ok(Self { conn })
    }
    
    // Check if user has completed onboarding
    pub fn is_onboarded(&self) -> Result<bool> {
        log::debug!("Checking onboarding status in database");
        let mut stmt = self.conn.prepare("SELECT val FROM meta WHERE key = 'onboarding_completed'")?;
        let result: Option<String> = stmt.query_row([], |row| row.get(0)).ok();
        
        let is_onboarded = result.map(|v| v == "true").unwrap_or(false);
        log::debug!("Database onboarding status: {}", is_onboarded);
        Ok(is_onboarded)
    }
    
    // Mark onboarding as completed
    pub fn set_onboarding_completed(&self) -> Result<()> {
        log::info!("Marking onboarding as completed in database");
        let timestamp = Utc::now().timestamp();
        self.conn.execute(
            "INSERT OR REPLACE INTO meta (key, val) VALUES ('onboarding_completed', 'true')",
            [],
        )?;
        self.conn.execute(
            "INSERT OR REPLACE INTO meta (key, val) VALUES ('onboarding_timestamp', ?1)",
            params![timestamp.to_string()],
        )?;
        log::info!("Onboarding marked as completed at timestamp: {}", timestamp);
        Ok(())
    }
    
    // Get onboarding timestamp
    pub fn get_onboarding_timestamp(&self) -> Result<Option<i64>> {
        let mut stmt = self.conn.prepare("SELECT val FROM meta WHERE key = 'onboarding_timestamp'")?;
        let result: Option<String> = stmt.query_row([], |row| row.get(0)).ok();
        
        Ok(result.and_then(|v| v.parse::<i64>().ok()))
    }
    
    // Set user preferences
    pub fn set_preference(&self, key: &str, value: &str) -> Result<()> {
        log::debug!("Setting preference: {} = {}", key, value);
        let pref_key = format!("pref_{}", key);
        self.conn.execute(
            "INSERT OR REPLACE INTO meta (key, val) VALUES (?1, ?2)",
            params![pref_key, value],
        )?;
        Ok(())
    }
    
    // Get user preference
    pub fn get_preference(&self, key: &str) -> Result<Option<String>> {
        let pref_key = format!("pref_{}", key);
        let mut stmt = self.conn.prepare("SELECT val FROM meta WHERE key = ?1")?;
        let result: Option<String> = stmt.query_row(params![pref_key], |row| row.get(0)).ok();
        Ok(result)
    }
    
    // Check if this is a first-time install
    pub fn is_first_time_install(&self) -> Result<bool> {
        log::debug!("Checking if this is a first-time install");
        
        // Check if the meta table even exists
        let stmt = self.conn.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='meta'").ok();
        
        if let Some(mut stmt) = stmt {
            let exists: Option<String> = stmt.query_row([], |row| row.get(0)).ok();
            if exists.is_none() {
                // Table doesn't exist yet, so it's definitely a first install
                log::debug!("Meta table doesn't exist - first time install");
                return Ok(true);
            }
        }
        
        // Check if first_install_timestamp exists - this is only set during initial DB creation
        let mut stmt = self.conn.prepare("SELECT val FROM meta WHERE key = 'first_install_timestamp'")?;
        let first_install_timestamp: Option<String> = stmt.query_row([], |row| row.get(0)).ok();
        
        if first_install_timestamp.is_none() {
            log::debug!("No first_install_timestamp found - first time install");
            return Ok(true);
        }
        
        // Check if onboarding has been completed
        let onboarding_completed = self.is_onboarded()?;
        let is_first_time = !onboarding_completed;
        
        log::debug!("First install timestamp exists: {:?}, onboarding completed: {}, returning first_time: {}", 
                   first_install_timestamp, onboarding_completed, is_first_time);
        
        Ok(is_first_time)
    }
    
    // Device tracking methods
    
    /// Record a device connection
    pub fn device_connected(&self, device_id: &str, features: Option<&crate::features::DeviceFeatures>) -> Result<()> {
        log::info!("Recording device connection: {}", device_id);
        let now = Utc::now().timestamp();
        
        // First, upsert the device record
        if let Some(features) = features {
            let features_json = serde_json::to_string(features)?;
            
            self.conn.execute(
                "INSERT INTO devices (device_id, vendor, model, label, firmware_variant, firmware_version, 
                                     bootloader_mode, initialized, pin_protection, passphrase_protection, 
                                     first_seen, last_seen, features)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?11, ?12)
                 ON CONFLICT(device_id) DO UPDATE SET
                    vendor = ?2, model = ?3, label = ?4, firmware_variant = ?5, firmware_version = ?6,
                    bootloader_mode = ?7, initialized = ?8, pin_protection = ?9, passphrase_protection = ?10,
                    last_seen = ?11, features = ?12",
                params![
                    device_id,
                    features.vendor,
                    features.model,
                    features.label,
                    features.firmware_variant,
                    features.version,
                    features.bootloader_mode,
                    features.initialized,
                    features.pin_protection,
                    features.passphrase_protection,
                    now,
                    features_json
                ],
            )?;
        } else {
            // No features yet, just record the device
            self.conn.execute(
                "INSERT INTO devices (device_id, first_seen, last_seen)
                 VALUES (?1, ?2, ?2)
                 ON CONFLICT(device_id) DO UPDATE SET last_seen = ?2",
                params![device_id, now],
            )?;
        }
        
        // Record the connection
        self.conn.execute(
            "INSERT INTO device_connections (device_id, connected_at) VALUES (?1, ?2)",
            params![device_id, now],
        )?;
        
        log::info!("Device {} connected and recorded", device_id);
        Ok(())
    }
    
    /// Record a device disconnection
    pub fn device_disconnected(&self, device_id: &str) -> Result<()> {
        log::info!("Recording device disconnection: {}", device_id);
        let now = Utc::now().timestamp();
        
        // Update the most recent connection without a disconnection time
        self.conn.execute(
            "UPDATE device_connections 
             SET disconnected_at = ?1 
             WHERE device_id = ?2 AND disconnected_at IS NULL",
            params![now, device_id],
        )?;
        
        log::info!("Device {} disconnected and recorded", device_id);
        Ok(())
    }
    
    /// Get all devices with their connection status
    pub fn get_all_devices(&self) -> Result<Vec<DeviceRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT d.*, 
                    CASE WHEN EXISTS (
                        SELECT 1 FROM device_connections 
                        WHERE device_id = d.device_id 
                        AND disconnected_at IS NULL
                    ) THEN 1 ELSE 0 END as is_connected
             FROM devices d
             ORDER BY d.last_seen DESC"
        )?;
        
        let devices = stmt.query_map([], |row| {
            Ok(DeviceRecord {
                device_id: row.get(0)?,
                vendor: row.get(1)?,
                model: row.get(2)?,
                label: row.get(3)?,
                firmware_variant: row.get(4)?,
                firmware_version: row.get(5)?,
                bootloader_mode: row.get(6)?,
                initialized: row.get(7)?,
                pin_protection: row.get(8)?,
                passphrase_protection: row.get(9)?,
                first_seen: row.get(10)?,
                last_seen: row.get(11)?,
                features: row.get(12)?,
                is_connected: row.get(13)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
        
        Ok(devices)
    }
    
    /// Get only connected devices
    pub fn get_connected_devices(&self) -> Result<Vec<DeviceRecord>> {
        let all_devices = self.get_all_devices()?;
        Ok(all_devices.into_iter().filter(|d| d.is_connected).collect())
    }
    
    /// Get only disconnected devices
    pub fn get_disconnected_devices(&self) -> Result<Vec<DeviceRecord>> {
        let all_devices = self.get_all_devices()?;
        Ok(all_devices.into_iter().filter(|d| !d.is_connected).collect())
    }
}

// Embedded schema
const SCHEMA: &str = r#"
-- KeepKey Desktop v5 Database Schema
PRAGMA journal_mode=WAL;
PRAGMA foreign_keys = ON;

-- Accounts table for wallet information
CREATE TABLE IF NOT EXISTS accounts (
    id           INTEGER PRIMARY KEY,
    wallet_fp    TEXT NOT NULL,      -- 4-byte fingerprint (hex)
    kind         TEXT NOT NULL,      -- 'keepkey' | 'digital'
    xpub         TEXT NOT NULL,
    label        TEXT,
    added_ts     INTEGER NOT NULL    -- epoch seconds
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_accounts_fp_xpub ON accounts(wallet_fp, xpub);

-- Addresses table for derived addresses
CREATE TABLE IF NOT EXISTS addresses (
    id           INTEGER PRIMARY KEY,
    account_id   INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    address      TEXT NOT NULL UNIQUE,
    deriv_path   TEXT NOT NULL,      -- "m/84'/0'/0'/0/15"
    first_seen   INTEGER             -- block height
);

CREATE INDEX IF NOT EXISTS idx_addresses_account ON addresses(account_id);

-- Transactions table
CREATE TABLE IF NOT EXISTS txs (
    txid         TEXT PRIMARY KEY,
    account_id   INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    block_height INTEGER,
    direction    INTEGER NOT NULL,   -- +sats (recv) / -sats (send)
    amount       INTEGER NOT NULL,   -- satoshis (always positive)
    fee          INTEGER,            -- satoshis
    timestamp    INTEGER             -- tx time per node
);

CREATE INDEX IF NOT EXISTS idx_txs_account_block ON txs(account_id, block_height);

-- Devices table for tracking all connected devices
CREATE TABLE IF NOT EXISTS devices (
    device_id    TEXT PRIMARY KEY,   -- Unique device ID
    vendor       TEXT,
    model        TEXT,
    label        TEXT,
    firmware_variant TEXT,
    firmware_version TEXT,
    bootloader_mode BOOLEAN,
    initialized  BOOLEAN,
    pin_protection BOOLEAN,
    passphrase_protection BOOLEAN,
    first_seen   INTEGER NOT NULL,   -- epoch seconds
    last_seen    INTEGER NOT NULL,   -- epoch seconds
    features     TEXT                -- JSON blob of full features
);

-- Device connections table for tracking connection history
CREATE TABLE IF NOT EXISTS device_connections (
    id           INTEGER PRIMARY KEY,
    device_id    TEXT NOT NULL REFERENCES devices(device_id),
    connected_at INTEGER NOT NULL,   -- epoch seconds
    disconnected_at INTEGER,         -- epoch seconds, NULL if still connected
    session_data TEXT                -- JSON blob of session-specific data
);

CREATE INDEX IF NOT EXISTS idx_device_connections_device ON device_connections(device_id);
CREATE INDEX IF NOT EXISTS idx_device_connections_time ON device_connections(connected_at, disconnected_at);

-- Meta table for key-value storage (including onboarding state)
CREATE TABLE IF NOT EXISTS meta (
    key TEXT PRIMARY KEY,
    val TEXT
);

-- Default onboarding data
-- This will be inserted only if the table is empty (first time install)
INSERT OR IGNORE INTO meta (key, val) VALUES 
    ('db_version', '1'),
    ('onboarding_completed', 'false'),
    ('first_install_timestamp', CAST(strftime('%s', 'now') AS TEXT));

-- User preferences with defaults
INSERT OR IGNORE INTO meta (key, val) VALUES 
    ('pref_language', 'en'),
    ('pref_theme', 'system'),
    ('pref_currency', 'USD'),
    ('pref_units', 'metric'),
    ('pref_analytics_enabled', 'false');
"#; 