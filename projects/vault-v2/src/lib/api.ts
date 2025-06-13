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

export class PortfolioAPI {
  static async getPortfolio(): Promise<Portfolio> {
    const tag = TAG + " | getPortfolio | ";
    
    try {
      console.log(tag, 'Getting portfolio from live Pioneer API data');

      // 1. Get all xpubs from database
      const db = await Database.load(DB_PATH);
      const xpubs = await db.select('SELECT * FROM xpubs ORDER BY created_at ASC') as XpubInfo[];
      
      if (xpubs.length === 0) {
        console.warn(tag, 'No xpubs found in database');
        return this.getEmptyPortfolio();
      }

      console.log(tag, `Found ${xpubs.length} xpubs in database`);

      // 2. Check cache for fresh data
      const cachedBalances = await db.select('SELECT * FROM balance_cache ORDER BY last_updated DESC') as BalanceCache[];
      const needsRefresh = cachedBalances.length === 0 || 
        PioneerAPI.isCacheExpired(cachedBalances[0]?.last_updated || 0);

      let portfolioData: PioneerPortfolioResponse[] = [];

      if (needsRefresh) {
        console.log(tag, 'Cache expired or empty, calling Pioneer API');
        
        // 3. Call Pioneer API with real xpubs
        const portfolioRequests: PioneerPortfolioRequest[] = xpubs.map(xpub => ({
          caip: xpub.caip,
          pubkey: xpub.pubkey
        }));

        try {
          portfolioData = await PioneerAPI.getPortfolio(portfolioRequests);
          
          // 4. Cache the results
          await this.cachePortfolioData(portfolioData);
          console.log(tag, 'Portfolio data cached successfully');
          
        } catch (apiError) {
          console.warn(tag, 'Pioneer API failed, using cached data if available:', apiError);
          portfolioData = await this.getPortfolioDataFromCache();
        }
      } else {
        console.log(tag, 'Using fresh cached data');
        portfolioData = await this.getPortfolioDataFromCache();
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
    try {
      console.log(TAG, 'sendAsset params:', { asset, toAddress, amount });
      // TODO: Implement real transaction sending through Pioneer/device
      return true;
    } catch (error) {
      console.error('Send transaction error:', error);
      return false;
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
  static async requestXpubFromDevice(deviceId: string, path: string): Promise<string> {
    try {
      const requestId = await invoke('add_to_device_queue', { deviceId, path });
      return requestId as string;
    } catch (error) {
      console.error('Failed to add to device queue:', error);
      throw error;
    }
  }

  static async getQueueStatus(deviceId: string): Promise<QueueStatus> {
    try {
      const status = await invoke('get_queue_status', { deviceId });
      return status as QueueStatus;
    } catch (error) {
      console.error('Failed to get queue status:', error);
      throw error;
    }
  }
} 