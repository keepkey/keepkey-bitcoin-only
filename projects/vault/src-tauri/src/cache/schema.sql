-- KeepKey CLI Cache Database Schema
-- Complete schema designed for multi-device support from day 1

-- Devices table - stores device features and metadata
CREATE TABLE IF NOT EXISTS devices (
    id              INTEGER PRIMARY KEY,
    device_id       TEXT NOT NULL UNIQUE,
    label           TEXT,
    vendor          TEXT,
    major_version   INTEGER,
    minor_version   INTEGER,
    patch_version   INTEGER,
    revision        TEXT,
    firmware_hash   TEXT,
    bootloader_hash TEXT,
    features_json   TEXT NOT NULL,
    last_seen       INTEGER NOT NULL,
    created_at      INTEGER NOT NULL
);

-- Networks table - supported blockchain networks
CREATE TABLE IF NOT EXISTS networks (
    id                INTEGER PRIMARY KEY,
    chain_id_caip2    TEXT NOT NULL UNIQUE,
    display_name      TEXT NOT NULL,
    network_name      TEXT NOT NULL,
    symbol            TEXT NOT NULL,
    is_evm            BOOLEAN NOT NULL DEFAULT 0,
    is_testnet        BOOLEAN NOT NULL DEFAULT 0,
    enabled           BOOLEAN NOT NULL DEFAULT 1
);

-- Paths table - derivation paths for each device/blockchain combination
CREATE TABLE IF NOT EXISTS paths (
    id                          INTEGER PRIMARY KEY,
    device_id                   TEXT, -- NULL means global path, device-specific when set
    note                        TEXT NOT NULL,
    blockchain                  TEXT,
    symbol                      TEXT,
    symbol_swap_kit             TEXT,
    networks                    TEXT NOT NULL, -- JSON array of network IDs
    script_type                 TEXT NOT NULL,
    available_script_types      TEXT, -- JSON array of available script types
    type                        TEXT NOT NULL, -- xpub, ypub, zpub, address
    address_n_list              TEXT NOT NULL, -- JSON array of derivation path
    address_n_list_master       TEXT NOT NULL, -- JSON array of master derivation path
    curve                       TEXT NOT NULL DEFAULT 'secp256k1',
    show_display                BOOLEAN NOT NULL DEFAULT 0,
    created_at                  INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
    -- Note: No foreign key constraint to allow global paths with NULL device_id
);

-- Cached addresses table - derived addresses for each device/path combination
CREATE TABLE IF NOT EXISTS cached_addresses (
    id               INTEGER PRIMARY KEY,
    device_id        TEXT NOT NULL,
    coin             TEXT NOT NULL,
    script_type      TEXT NOT NULL,
    derivation_path  TEXT NOT NULL, -- JSON array of the derivation path
    address          TEXT NOT NULL,
    pubkey           TEXT,
    created_at       INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    UNIQUE(device_id, coin, script_type, derivation_path),
    FOREIGN KEY (device_id) REFERENCES devices(device_id) ON DELETE CASCADE
);

-- Balances table - cached balance information from Pioneer API  
CREATE TABLE IF NOT EXISTS cached_balances (
    id              INTEGER PRIMARY KEY,
    device_id       TEXT NOT NULL,
    caip            TEXT NOT NULL, -- CAIP identifier (network + asset)
    pubkey          TEXT NOT NULL,
    balance         TEXT NOT NULL, -- String to preserve precision
    price_usd       TEXT NOT NULL DEFAULT '0.00',
    value_usd       TEXT NOT NULL DEFAULT '0.00',
    symbol          TEXT,
    network_id      TEXT,
    last_updated    INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    UNIQUE(device_id, caip, pubkey),
    FOREIGN KEY (device_id) REFERENCES devices(device_id) ON DELETE CASCADE
);

-- Portfolio summaries table - aggregated portfolio data
CREATE TABLE IF NOT EXISTS portfolio_summaries (
    id                INTEGER PRIMARY KEY,
    device_id         TEXT NOT NULL UNIQUE,
    total_value_usd   TEXT NOT NULL DEFAULT '0.00',
    network_count     INTEGER NOT NULL DEFAULT 0,
    asset_count       INTEGER NOT NULL DEFAULT 0,
    last_updated      INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    FOREIGN KEY (device_id) REFERENCES devices(device_id) ON DELETE CASCADE
);

-- Configuration table - app settings and configuration
CREATE TABLE IF NOT EXISTS config (
    key         TEXT PRIMARY KEY,
    value       TEXT NOT NULL,
    description TEXT,
    updated_at  INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_devices_device_id ON devices(device_id);
CREATE INDEX IF NOT EXISTS idx_networks_chain_id ON networks(chain_id_caip2);
CREATE INDEX IF NOT EXISTS idx_networks_enabled ON networks(enabled);
CREATE INDEX IF NOT EXISTS idx_paths_device_id ON paths(device_id);
CREATE INDEX IF NOT EXISTS idx_cached_addresses_device_id ON cached_addresses(device_id);
CREATE INDEX IF NOT EXISTS idx_cached_addresses_lookup ON cached_addresses(device_id, coin, script_type);
CREATE INDEX IF NOT EXISTS idx_cached_balances_device_id ON cached_balances(device_id);
CREATE INDEX IF NOT EXISTS idx_cached_balances_last_updated ON cached_balances(last_updated);
CREATE INDEX IF NOT EXISTS idx_portfolio_device_id ON portfolio_summaries(device_id);

-- Insert default configuration values
INSERT OR IGNORE INTO config (key, value, description) VALUES 
('pioneer_server_url', 'https://pioneers.dev', 'Pioneer API server URL for balance and price data'),
('cache_ttl_minutes', '10', 'How many minutes to cache balance data before refreshing'),
('auto_discover_assets', 'true', 'Whether to automatically discover new assets for connected devices'); 