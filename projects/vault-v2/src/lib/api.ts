import { invoke } from '@tauri-apps/api/core';
import axios from 'axios';
import Database from '@tauri-apps/plugin-sql';
import { Asset, Portfolio } from '../types/wallet';
import { QueueStatus } from '../types/queue';
import { XpubInfo } from '../types/database';

const TAG = " | API | ";
const PIONEER_BASE_URL = 'https://pioneers.dev';
const CACHE_TTL_MINUTES = 10;
const DB_PATH = 'sqlite:vault.db';

// Pioneer API Types
export interface PioneerPortfolioRequest {
  caip: string;
  pubkey: string;
}

export interface PioneerPortfolioResponse {
  caip: string;
  pubkey: string;
  balance: string;
  valueUsd: string;
  priceUsd: string;
  symbol: string;
}

export interface PioneerFeeRateResponse {
  fastest: number;
  fast: number;
  average: number;
}

export interface PioneerAddressResponse {
  address: string;
  addressIndex: number;
}

// Database Cache Types
export interface BalanceCache {
  id: number;
  pubkey: string;
  caip: string;
  balance: string;
  balance_usd: string;
  price_usd: string;
  symbol: string | null;
  last_updated: number;
}

export class PioneerAPI {
  static async getPortfolio(requests: PioneerPortfolioRequest[]): Promise<PioneerPortfolioResponse[]> {
    try {
      console.log('🌐 Calling Pioneer portfolio API with', requests.length, 'xpubs');
      console.log('📤 Requests:', requests);
      
      const response = await axios.post(
        `${PIONEER_BASE_URL}/api/v1/portfolio`,
        requests,
        { 
          headers: { 
            'Content-Type': 'application/json',
            'accept': 'application/json' 
          },
          timeout: 30000
        }
      );

      console.log('✅ Pioneer portfolio API successful:', response.data.length, 'entries');
      console.log('📥 Raw response:', response.data);
      return response.data;
    } catch (error) {
      console.error('❌ Pioneer portfolio API failed:', error);
      throw error;
    }
  }

  static async getFeeRates(caip: string): Promise<PioneerFeeRateResponse> {
    try {
      console.log('💰 Getting fee rates for', caip);
      
      const encodedCaip = encodeURIComponent(caip);
      const response = await axios.get(
        `${PIONEER_BASE_URL}/api/v1/GetFeeRate/${encodedCaip}`,
        { 
          headers: { 'accept': 'application/json' },
          timeout: 10000
        }
      );

      console.log('✅ Fee rates retrieved:', response.data);
      return response.data;
    } catch (error) {
      console.error('❌ Fee rates API failed:', error);
      throw error;
    }
  }

  static async getReceiveAddress(coin: string, xpub: string): Promise<PioneerAddressResponse> {
    try {
      console.log('🏠 Getting receive address for', coin, 'xpub');
      
      const response = await axios.get(
        `${PIONEER_BASE_URL}/api/v1/getNewAddress/${coin}/${xpub}`,
        { 
          headers: { 'accept': 'application/json' },
          timeout: 10000
        }
      );

      console.log('✅ Receive address retrieved:', response.data);
      return response.data;
    } catch (error) {
      console.error('❌ Receive address API failed:', error);
      throw error;
    }
  }

  static isCacheExpired(lastUpdated: number): boolean {
    const now = Math.floor(Date.now() / 1000);
    const cacheAge = now - lastUpdated;
    const maxAge = CACHE_TTL_MINUTES * 60;
    return cacheAge > maxAge;
  }
}

// Global flag to enable/disable SQL caching (default: ON for xpub storage)
export const SQL_CACHE_ENABLED = true;

export class PortfolioAPI {
  static async getPortfolio(): Promise<Portfolio> {
    const tag = TAG + " | getPortfolio | ";
    
    try {
      console.log(tag, 'Getting portfolio from live Pioneer API data');

      if (!SQL_CACHE_ENABLED) {
        // SQL caching is globally disabled; skip all DB reads/writes
        try {
          // Instead, get xpubs and portfolio data directly from API
          // (simulate empty xpubs if DB is not used)
          const xpubs: XpubInfo[] = [];
          const requests = xpubs.map(x => ({ caip: x.caip, pubkey: x.pubkey }));
          const portfolioData: PioneerPortfolioResponse[] = await PioneerAPI.getPortfolio(requests);
          return this.transformToPortfolio(portfolioData);
        } catch (error) {
          console.error(tag, 'Portfolio API Error (cache disabled):', error);
          return this.getEmptyPortfolio();
        }
      }

      // 1. Get all xpubs from database
      const db = await Database.load(DB_PATH);
      const xpubs = await db.select('SELECT * FROM xpubs ORDER BY created_at ASC') as XpubInfo[];
      
      if (xpubs.length === 0) {
        console.warn(tag, 'No xpubs found in database');
        return this.getEmptyPortfolio();
      }

      console.log(tag, `Found ${xpubs.length} xpubs in database`);

      // 2. Check cache for fresh data
      let portfolioData: PioneerPortfolioResponse[] = [];
      let cacheFresh = false;
      if (SQL_CACHE_ENABLED) {
        try {
          const db = await Database.load(DB_PATH);
          const cached = await db.select('SELECT * FROM balance_cache') as BalanceCache[];
          if (cached.length > 0) {
            const latest = Math.max(...cached.map(c => c.last_updated));
            const now = Math.floor(Date.now() / 1000);
            cacheFresh = now - latest < 300; // 5 min
            if (cacheFresh) {
              portfolioData = cached.map(item => ({
                caip: item.caip,
                pubkey: item.pubkey,
                balance: item.balance,
                valueUsd: item.balance_usd,
                priceUsd: item.price_usd,
                symbol: item.symbol || 'BTC'
              }));
              console.log(tag, 'Using fresh cached data');
            }
          }
        } catch (cacheErr) {
          console.warn(tag, 'Cache read failed:', cacheErr);
        }
      }
      if (!cacheFresh) {
        try {
          const requests = xpubs.map(x => ({ caip: x.caip, pubkey: x.pubkey }));
            portfolioData = await PioneerAPI.getPortfolio(requests);
          if (SQL_CACHE_ENABLED) {
            await this.cachePortfolioData(portfolioData);
            console.log(tag, 'Portfolio data cached successfully');
          }
        } catch (apiError) {
          console.warn(tag, 'Pioneer API failed, using cached data if available:', apiError);
          if (SQL_CACHE_ENABLED) {
            portfolioData = await this.getPortfolioDataFromCache();
          }
        }
      }

      // 5. Transform to Portfolio format
      return this.transformToPortfolio(portfolioData);

    } catch (error) {
      console.error(tag, 'Portfolio API Error:', error);
      return this.getEmptyPortfolio();
    }
  }

  private static async cachePortfolioData(data: PioneerPortfolioResponse[]): Promise<void> {
    const db = await Database.load(DB_PATH);
    const now = Math.floor(Date.now() / 1000);

    // Clear old cache
    await db.execute('DELETE FROM balance_cache');

    // Insert new data
    for (const item of data) {
      // Derive symbol from CAIP if not provided
      let symbol = item.symbol;
      if (!symbol) {
        // Extract symbol from CAIP - e.g., "bip122:000000000019d6689c085ae165831e93/slip44:0" -> "BTC"
        if (item.caip.includes('bip122:000000000019d6689c085ae165831e93')) {
          symbol = 'BTC';
        } else {
          symbol = 'UNKNOWN';
        }
      }

      console.log('💾 Caching:', {
        pubkey: item.pubkey.substring(0, 20) + '...',
        symbol,
        balance: item.balance,
        valueUsd: item.valueUsd
      });

      await db.execute(
        `INSERT INTO balance_cache (pubkey, caip, balance, balance_usd, price_usd, symbol, last_updated) 
         VALUES (?, ?, ?, ?, ?, ?, ?)`,
        [item.pubkey, item.caip, item.balance, item.valueUsd, item.priceUsd, symbol, now]
      );
    }
  }

  private static async getPortfolioDataFromCache(): Promise<PioneerPortfolioResponse[]> {
    const db = await Database.load(DB_PATH);
    const cached = await db.select('SELECT * FROM balance_cache') as BalanceCache[];
    
    console.log('📊 Loading from cache:', cached.length, 'entries');
    
    return cached.map(item => ({
      caip: item.caip,
      pubkey: item.pubkey,
      balance: item.balance,
      valueUsd: item.balance_usd,
      priceUsd: item.price_usd,
      symbol: item.symbol || 'BTC' // Fallback to BTC if symbol is null
    }));
  }

  private static transformToPortfolio(data: PioneerPortfolioResponse[]): Portfolio {
    const tag = TAG + " | transformToPortfolio | ";
    
    let totalValueUsd = 0;
    const assets: Asset[] = [];
    const networks = [
      {
        id: 1,
        network_name: "Bitcoin",
        symbol: "BTC", 
        chain_id_caip2: "bip122:000000000019d6689c085ae165831e93",
        is_evm: false
      }
    ];

    // Group by symbol and sum balances
    const symbolGroups = new Map<string, {
      balance: number,
      valueUsd: number,
      priceUsd: number,
      caip: string
    }>();

    for (const item of data) {
      const balance = parseFloat(item.balance) || 0;
      const valueUsd = parseFloat(item.valueUsd) || 0;
      const priceUsd = parseFloat(item.priceUsd) || 0;
      
      if (symbolGroups.has(item.symbol)) {
        const existing = symbolGroups.get(item.symbol)!;
        existing.balance += balance;
        existing.valueUsd += valueUsd;
        // Keep the same price (should be same for same symbol)
      } else {
        symbolGroups.set(item.symbol, {
          balance,
          valueUsd,
          priceUsd,
          caip: item.caip
        });
      }
      
      totalValueUsd += valueUsd;
    }

    // Convert to assets
    for (const [symbol, group] of symbolGroups) {
      assets.push({
        symbol,
        name: symbol === 'BTC' ? 'Bitcoin' : symbol,
        balance: group.balance.toString(),
        value_usd: group.valueUsd,
        network_id: 'bitcoin',
        caip: group.caip,
        price_usd: group.priceUsd,
        change_24h: 0 // TODO: Add 24h change if Pioneer provides it
      });
    }

    console.log(tag, `Portfolio: ${assets.length} assets, total $${totalValueUsd.toFixed(2)}`);

    return {
      total_value_usd: totalValueUsd.toFixed(2),
      assets,
      networks
    };
  }

  private static getEmptyPortfolio(): Portfolio {
    return {
      total_value_usd: '0.00',
      assets: [],
      networks: [{
        id: 1,
        network_name: "Bitcoin",
        symbol: "BTC",
        chain_id_caip2: "bip122:000000000019d6689c085ae165831e93",
        is_evm: false
      }]
    };
  }

  static async sendAsset(asset: Asset, toAddress: string, amount: string): Promise<boolean> {
    const tag = TAG + " | sendAsset | ";
    
    try {
      console.log(tag, 'Send transaction request:', { 
        asset: asset.symbol, 
        caip: asset.caip,
        toAddress, 
        amount,
        availableBalance: asset.balance
      });

      // Validate inputs
      if (!toAddress || !amount) {
        throw new Error('Missing required parameters: address and amount');
      }

      const sendAmount = parseFloat(amount);
      const availBalance = parseFloat(asset.balance);

      if (sendAmount <= 0) {
        throw new Error('Amount must be greater than 0');
      }

      if (sendAmount > availBalance) {
        throw new Error(`Insufficient balance. Available: ${asset.balance} ${asset.symbol}, Requested: ${amount} ${asset.symbol}`);
      }

      // TODO: Implement actual transaction building and signing
      console.warn(tag, '⚠️ TRANSACTION NOT IMPLEMENTED - This would:');
      console.warn(tag, '  1. Call Pioneer API to build transaction');
      console.warn(tag, '  2. Get UTXOs for the xpub');
      console.warn(tag, '  3. Build PSBT with proper fees');
      console.warn(tag, '  4. Send to device queue for signing');
      console.warn(tag, '  5. Broadcast signed transaction');
      console.warn(tag, '  6. Return transaction ID');

      // For now, return false to indicate the feature is not implemented
      // This prevents the UI from showing "success" when nothing actually happened
      throw new Error('❌ Transaction sending not yet implemented. This is a placeholder.');

    } catch (error) {
      console.error(tag, 'Send transaction error:', error);
      throw error;
    }
  }

  static async getReceiveAddress(asset: Asset): Promise<string> {
    const tag = TAG + " | getReceiveAddress | ";
    
    try {
      console.log(tag, 'Getting receive address for asset:', asset.symbol);
      
      // Get the xpub for this asset
      const db = await Database.load(DB_PATH);
      const xpubs = await db.select(
        'SELECT * FROM xpubs WHERE caip = ? ORDER BY created_at ASC LIMIT 1',
        [asset.caip]
      ) as XpubInfo[];

      if (xpubs.length === 0) {
        throw new Error(`No xpub found for asset ${asset.symbol}`);
      }

      const xpub = xpubs[0];
      console.log(tag, `Using xpub for ${asset.symbol}:`, xpub.pubkey.substring(0, 20) + '...');

      // Call Pioneer API for fresh address
      const addressResponse = await PioneerAPI.getReceiveAddress('BTC', xpub.pubkey);
      
      console.log(tag, 'Generated fresh receive address:', addressResponse.address);
      return addressResponse.address;
      
    } catch (error) {
      console.error(tag, 'Failed to get receive address:', error);
      throw error;
    }
  }
}

export class DeviceQueueAPI {
  static async getConnectedDevices(): Promise<any[]> {
    try {
      const devices = await invoke('get_connected_devices');
      return devices as any[];
    } catch (error) {
      console.error('Failed to get connected devices:', error);
      throw error;
    }
  }

  static async requestXpubFromDevice(deviceId: string, path: string): Promise<string> {
    try {
      const requestId = `xpub_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
      const request = {
        device_id: deviceId,
        request_id: requestId,
        request: {
          GetXpub: { path }
        }
      };
      
      console.log('🔄 Requesting xpub from device:', { deviceId, path, requestId });
      await invoke('add_to_device_queue', { request });
      return requestId;
    } catch (error) {
      console.error('Failed to add xpub request to device queue:', error);
      throw error;
    }
  }

  static async requestReceiveAddressFromDevice(
    deviceId: string, 
    path: string, 
    coinName: string, 
    scriptType?: string,
    showDisplay?: boolean
  ): Promise<string> {
    try {
      const requestId = `addr_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
      return await this.requestReceiveAddressFromDeviceWithId(
        deviceId, path, coinName, scriptType, showDisplay, requestId
      );
    } catch (error) {
      console.error('Failed to add address request to device queue:', error);
      throw error;
    }
  }

  static async requestReceiveAddressFromDeviceWithId(
    deviceId: string, 
    path: string, 
    coinName: string, 
    scriptType: string | undefined,
    showDisplay: boolean | undefined,
    requestId: string
  ): Promise<string> {
    try {
      const request = {
        device_id: deviceId,
        request_id: requestId,
        request: {
          GetAddress: { 
            path, 
            coin_name: coinName, 
            script_type: scriptType, 
            show_display: showDisplay // Will be undefined if not provided, which becomes None in Rust
          }
        }
      };
      
      console.log('🔄 Requesting address from device:', { deviceId, path, requestId });
      await invoke('add_to_device_queue', { request });
      return requestId;
    } catch (error) {
      console.error('Failed to add address request to device queue:', error);
      throw error;
    }
  }

  static async resetDeviceQueue(deviceId: string): Promise<void> {
    await invoke('reset_device_queue', { deviceId });
  }

  static async getQueueStatus(deviceId?: string): Promise<QueueStatus> {
    try {
      const status = await invoke('get_queue_status', { device_id: deviceId });
      return status as QueueStatus;
    } catch (error) {
      console.error('Failed to get queue status:', error);
      throw error;
    }
  }
} 