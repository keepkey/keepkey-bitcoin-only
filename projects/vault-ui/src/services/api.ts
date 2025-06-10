import axios from 'axios';

// Use port 1646 directly to connect to kkcli server
const KKCLI_SERVER_URL = 'http://localhost:1646';

// Create axios instance with default configuration
const api = axios.create({
  baseURL: KKCLI_SERVER_URL,
  timeout: 30000,
  headers: {
    'Content-Type': 'application/json',
  },
});

// Add request interceptor for logging
api.interceptors.request.use(
  (config) => {
    console.log(`🌐 API Request: ${config.method?.toUpperCase()} ${config.url}`);
    return config;
  },
  (error) => {
    console.error('❌ API Request Error:', error);
    return Promise.reject(error);
  }
);

// Add response interceptor for logging and error handling
api.interceptors.response.use(
  (response) => {
    console.log(`✅ API Response: ${response.status} ${response.config.url}`);
    return response;
  },
  (error) => {
    console.error('❌ API Response Error:', error.response?.status, error.response?.statusText);
    return Promise.reject(error);
  }
);

// Network interface - matching v2 API response
export interface Network {
  id: number;
  chain_id_caip2: string;
  display_name: string;
  network_name: string;
  symbol: string;
  is_evm: boolean;
  is_testnet: boolean;
  enabled: boolean;
}

export interface NetworkPercentage {
  networkId: string;
  percentage: number;
}

// Balance interface - matching V2 API response
export interface Balance {
  caip: string;
  pubkey: string;
  balance: string;
  price_usd: number;
  value_usd: number;
  symbol: string;
  network_id: string;
}

// Dashboard interface - matching V2 portfolio/summary API response
export interface Dashboard {
  id: string;
  device_id: string;
  total_value_usd: string;
  network_count: number;
  asset_count: number;
  last_updated: number;
}

// API service methods
export const apiService = {
  // Get portfolio summary from V2 API
  async getDashboard(): Promise<Dashboard> {
    try {
      const response = await api.get('/v2/portfolio/summary');
      return response.data;
    } catch (error) {
      console.error('Failed to fetch dashboard:', error);
      throw error;
    }
  },

  // Get all balances from V2 API
  async getBalances(): Promise<Balance[]> {
    try {
      const response = await api.get('/v2/balances');
      return response.data;
    } catch (error) {
      console.error('Failed to fetch balances:', error);
      throw error;
    }
  },

  // Get all networks from V2 API
  async getNetworks(): Promise<Network[]> {
    try {
      const response = await api.get('/v2/networks');
      return response.data;
    } catch (error) {
      console.error('Failed to fetch networks:', error);
      throw error;
    }
  },

  // Get market data sync
  async syncMarket(): Promise<void> {
    try {
      await api.post('/api/v1/market/sync');
    } catch (error) {
      console.error('Failed to sync market data:', error);
      // Don't throw error for sync failures
    }
  },

  // Get network details
  async getNetwork(networkId: string): Promise<Network> {
    try {
      const response = await api.get(`/api/v1/networks/${networkId}`);
      return response.data;
    } catch (error) {
      console.error(`Failed to fetch network ${networkId}:`, error);
      throw error;
    }
  },

  // Health check
  async healthCheck(): Promise<boolean> {
    try {
      const response = await api.get('/api/v1/health');
      return response.status === 200;
    } catch (error) {
      console.error('Health check failed:', error);
      return false;
    }
  },
};

export default api; 