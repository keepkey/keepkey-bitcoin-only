import Database from '@tauri-apps/plugin-sql';
import { DeviceInfo, XpubInfo, RequiredPath } from '../types/database';
import { SQL_CACHE_ENABLED } from './api';

const DB_PATH = 'sqlite:vault.db';

export class WalletDatabase {
  
  static async init(): Promise<Database> {
    const db = await Database.load(DB_PATH);
    
    // Create tables
    await db.execute(`CREATE TABLE IF NOT EXISTS devices (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      device_id TEXT NOT NULL UNIQUE,
      label TEXT,
      features_json TEXT NOT NULL,
      last_seen INTEGER NOT NULL,
      created_at INTEGER NOT NULL
    )`);

    await db.execute(`CREATE TABLE IF NOT EXISTS xpubs (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      device_id TEXT NOT NULL,
      path TEXT NOT NULL,
      label TEXT NOT NULL,
      caip TEXT NOT NULL,
      pubkey TEXT NOT NULL,
      created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
      UNIQUE(device_id, path, caip),
      FOREIGN KEY (device_id) REFERENCES devices(device_id) ON DELETE CASCADE
    )`);

      // Create balance cache table for Pioneer API data
  await db.execute(`CREATE TABLE IF NOT EXISTS balance_cache (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pubkey TEXT NOT NULL,
    caip TEXT NOT NULL,
    balance TEXT NOT NULL,
    balance_usd TEXT NOT NULL,
    price_usd TEXT NOT NULL,
    symbol TEXT,
    last_updated INTEGER NOT NULL,
    UNIQUE(pubkey, caip)
  )`);

  // Create fee rate cache table
  await db.execute(`CREATE TABLE IF NOT EXISTS fee_rate_cache (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    caip TEXT NOT NULL UNIQUE,
    fastest INTEGER NOT NULL,
    fast INTEGER NOT NULL,
    average INTEGER NOT NULL,
    last_updated INTEGER NOT NULL
  )`);

  // Create indexes
  await db.execute(`CREATE INDEX IF NOT EXISTS idx_devices_device_id ON devices(device_id)`);
  await db.execute(`CREATE INDEX IF NOT EXISTS idx_xpubs_device_id ON xpubs(device_id)`);
  await db.execute(`CREATE INDEX IF NOT EXISTS idx_xpubs_lookup ON xpubs(device_id, path, caip)`);
  await db.execute(`CREATE INDEX IF NOT EXISTS idx_balance_cache_updated ON balance_cache(last_updated)`);
  await db.execute(`CREATE INDEX IF NOT EXISTS idx_fee_cache_updated ON fee_rate_cache(last_updated)`);

  return db;
  }

  static async getDevices(): Promise<DeviceInfo[]> {
    // If SQL caching is disabled, never return persisted devices
    // (Live device enumeration should be implemented here in the future)
    if (!SQL_CACHE_ENABLED) {
      return [];
    }
    const db = await Database.load(DB_PATH);
    const devices = await db.select('SELECT * FROM devices ORDER BY last_seen DESC');
    return devices as DeviceInfo[];
  }

  static async storeConnectedDevices(connectedDevices: any[]): Promise<void> {
    const db = await Database.load(DB_PATH);
    const now = Math.floor(Date.now() / 1000);
    
    for (const deviceData of connectedDevices) {
      const device = deviceData.device || deviceData; // Handle nested structure
      if (device && device.unique_id) {
        // Store device info
        await db.execute(
          'INSERT OR REPLACE INTO devices (device_id, label, features_json, last_seen, created_at) VALUES (?, ?, ?, ?, ?)',
          [
            device.unique_id,
            device.name || 'KeepKey',
            JSON.stringify(device),
            now,
            now
          ]
        );
        console.log(`✅ Stored device: ${device.unique_id}`);
      }
    }
  }

  static async getXpubs(deviceId: string): Promise<XpubInfo[]> {
    // If SQL caching is disabled, never return persisted xpubs
    if (!SQL_CACHE_ENABLED) {
      return [];
    }
    const db = await Database.load(DB_PATH);
    const xpubs = await db.select('SELECT * FROM xpubs WHERE device_id = ? ORDER BY created_at ASC', [deviceId]);
    return xpubs as XpubInfo[];
  }

  static async insertOrUpdateXpub(xpub: Omit<XpubInfo, 'id' | 'created_at'>): Promise<void> {
    const db = await Database.load(DB_PATH);
    const now = Math.floor(Date.now() / 1000);
    
    await db.execute(
      'INSERT OR REPLACE INTO xpubs (device_id, path, label, caip, pubkey, created_at) VALUES (?, ?, ?, ?, ?, ?)',
      [xpub.device_id, xpub.path, xpub.label, xpub.caip, xpub.pubkey, now]
    );
  }

  static getRequiredPaths(): RequiredPath[] {
    return [
      { path: "m/44'/0'/0'", label: "Bitcoin Legacy", caip: "bip122:000000000019d6689c085ae165831e93/slip44:0" },
      { path: "m/49'/0'/0'", label: "Bitcoin Segwit", caip: "bip122:000000000019d6689c085ae165831e93/slip44:0" },
      { path: "m/84'/0'/0'", label: "Bitcoin Native Segwit", caip: "bip122:000000000019d6689c085ae165831e93/slip44:0" }
    ];
  }

  static async insertXpubFromQueue(deviceId: string, path: string, xpub: string): Promise<void> {
    const db = await Database.load(DB_PATH);
    const now = Math.floor(Date.now() / 1000);
    
    // Find the matching required path to get the label and caip
    const requiredPaths = this.getRequiredPaths();
    const pathInfo = requiredPaths.find(p => p.path === path);
    
    if (!pathInfo) {
      throw new Error(`Unknown path: ${path}`);
    }

    await db.execute(
      'INSERT OR REPLACE INTO xpubs (device_id, path, label, caip, pubkey, created_at) VALUES (?, ?, ?, ?, ?, ?)',
      [deviceId, path, pathInfo.label, pathInfo.caip, xpub, now]
    );

    console.log(`✅ Stored xpub for ${deviceId} path ${path}: ${xpub.substring(0, 20)}...`);
  }

  static async getAllXpubs(): Promise<XpubInfo[]> {
    const db = await Database.load(DB_PATH);
    const xpubs = await db.select('SELECT * FROM xpubs ORDER BY created_at ASC');
    return xpubs as XpubInfo[];
  }

  static async clearBalanceCache(): Promise<void> {
    const db = await Database.load(DB_PATH);
    // Drop and recreate table to handle schema changes
    await db.execute('DROP TABLE IF EXISTS balance_cache');
    await db.execute(`CREATE TABLE balance_cache (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      pubkey TEXT NOT NULL,
      caip TEXT NOT NULL,
      balance TEXT NOT NULL,
      balance_usd TEXT NOT NULL,
      price_usd TEXT NOT NULL,
      symbol TEXT,
      last_updated INTEGER NOT NULL,
      UNIQUE(pubkey, caip)
    )`);
    await db.execute(`CREATE INDEX IF NOT EXISTS idx_balance_cache_updated ON balance_cache(last_updated)`);
    console.log('🧹 Balance cache cleared and recreated');
  }
} 