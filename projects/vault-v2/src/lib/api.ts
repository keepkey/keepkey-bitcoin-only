import { invoke } from '@tauri-apps/api/core';
import { Asset, Portfolio } from '../types/wallet';
import { QueueStatus } from '../types/queue';

const TAG = " | API | ";

export class PortfolioAPI {
  static async getPortfolio(): Promise<Portfolio> {
    try {
      // Mock portfolio data - this would come from Pioneer API in production
      const calculatedTotalUsd = 297.51;

      const balances: any = [
        {
          "symbol": "BTC",
          "name": "Bitcoin",
          "balance": "0.00285985",
          "value_usd": 297.51,
          "network_id": "bitcoin",
          "caip": "bip122:000000000019d6689c085ae165831e93/slip44:0",
          "priceUsd": "104030.00",
          "pubkey": "zpub6rm1EEJg4JasiTqacdouiUVncAc5ymhKReiPZfLTGnH2GSZquRn9reJhj6sfs73PoSJNXzpERKPVLYbwwUGHNF6jkMX5R58vWaLB9FVyJuX",
          "valueUsd": "297.51"
        }
      ];

      const networks: any = [
        {
          id: 1,
          network_name: "Bitcoin",
          symbol: "BTC",
          chain_id_caip2: "bip122:000000000019d6689c085ae165831e93",
          is_evm: false
        }
      ];

      return {
        total_value_usd: calculatedTotalUsd.toFixed(2),
        assets: balances.map((balance: any) => ({
          symbol: balance.symbol,
          name: balance.name || balance.symbol,
          balance: balance.balance,
          value_usd: balance.value_usd,
          network_id: balance.network_id,
          caip: balance.caip,
          price_usd: balance.value_usd > 0 ? balance.value_usd / parseFloat(balance.balance) : 0,
          change_24h: 0
        })),
        networks
      };
    } catch (error) {
      console.error('PortfolioAPI Error:', error);
      return {
        total_value_usd: '0.00',
        assets: [],
        networks: []
      };
    }
  }

  static async sendAsset(asset: Asset, toAddress: string, amount: string): Promise<boolean> {
    try {
      console.log(TAG, 'sendAsset params:', { asset, toAddress, amount });
      return true;
    } catch (error) {
      console.error('Send transaction error:', error);
      return false;
    }
  }

  static async getReceiveAddress(asset: Asset): Promise<string> {
    try {
      console.log(TAG, 'Getting receive address for asset:', asset);
      
      // Return a mock Bitcoin address for testing
      // In production, this would generate a real address from the device
      const mockAddresses = [
        'bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh',
        'bc1q34aq5drpuwy3wgl9lhup9892qp6svr8ldzyy7c',
        'bc1qa5wkgaew2dkv56kfvj49j0av5nml45x9ek9hz6'
      ];
      
      // Return a different address each time for testing
      const randomIndex = Math.floor(Math.random() * mockAddresses.length);
      return mockAddresses[randomIndex];
    } catch (error) {
      console.error('Get address error:', error);
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