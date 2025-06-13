import Database from '@tauri-apps/plugin-sql';
import { DeviceInfo, XpubInfo, RequiredPath } from '../types/database';

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

    // Create indexes
    await db.execute(`CREATE INDEX IF NOT EXISTS idx_devices_device_id ON devices(device_id)`);
    await db.execute(`CREATE INDEX IF NOT EXISTS idx_xpubs_device_id ON xpubs(device_id)`);
    await db.execute(`CREATE INDEX IF NOT EXISTS idx_xpubs_lookup ON xpubs(device_id, path, caip)`);

    return db;
  }

  static async getDevices(): Promise<DeviceInfo[]> {
    const db = await Database.load(DB_PATH);
    const devices = await db.select('SELECT * FROM devices ORDER BY last_seen DESC');
    return devices as DeviceInfo[];
  }

  static async getXpubs(deviceId: string): Promise<XpubInfo[]> {
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

  static async seedMockDevice(): Promise<void> {
    const db = await Database.load(DB_PATH);
    const now = Math.floor(Date.now() / 1000);
    
    // Insert mock device if not exists
    await db.execute(
      'INSERT OR IGNORE INTO devices (device_id, label, features_json, last_seen, created_at) VALUES (?, ?, ?, ?, ?)',
      [
        'testingDevice123',
        'Test KeepKey',
        JSON.stringify({"label":"Test KeepKey","vendor":"KeepKey","model":"KeepKey","device_id":"testingDevice123"}),
        now,
        now
      ]
    );
  }

  static getRequiredPaths(): RequiredPath[] {
    return [
      { path: "m/44'/0'/0'", label: "Bitcoin Legacy", caip: "bip122:000000000019d6689c085ae165831e93/slip44:0" },
      { path: "m/49'/0'/0'", label: "Bitcoin Segwit", caip: "bip122:000000000019d6689c085ae165831e93/slip44:0" },
      { path: "m/84'/0'/0'", label: "Bitcoin Native Segwit", caip: "bip122:000000000019d6689c085ae165831e93/slip44:0" }
    ];
  }
} 