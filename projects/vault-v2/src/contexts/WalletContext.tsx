/*
    Primary Wallet controller

    DB cache

    Queue to device lookup





 */

import React, { createContext, useContext, useState, useEffect, ReactNode, useCallback } from 'react';
import Database from '@tauri-apps/plugin-sql';
import paths from '../consts/default-paths.json'

const TAG = " | WalletContext | "

const DB_PATH = 'sqlite:vault.db';

// Types
export interface Asset {
  symbol: string;
  name: string;
  balance: string;
  value_usd: number;
  network_id: string;
  caip: string;
  price_usd?: number;
  change_24h?: number;
}

export interface Network {
  id: number;
  network_name: string;
  symbol: string;
  chain_id_caip2: string;
  is_evm: boolean;
}

export interface Portfolio {
  total_value_usd: string;
  assets: Asset[];
  networks: Network[];
}

// API Service
const assetApiService = {
  async getPortfolio(): Promise<Portfolio> {
    let tag = TAG + " | getPortfolio | "
    try {

      return {
        total_value_usd: calculatedTotalUsd.toFixed(2), // Use calculated value instead of dashboard
        assets: balances.map((balance: any) => ({
          symbol: balance.symbol,
          name: balance.name || balance.symbol, // We could enhance this with full names
          balance: balance.balance,
          value_usd: balance.value_usd,
          network_id: balance.network_id,
          caip: balance.caip,
          price_usd: balance.value_usd > 0 ? balance.value_usd / parseFloat(balance.balance) : 0,
          change_24h: 0 // TODO: Add 24h change data
        })),
        networks
      };
    } catch (error) {
      console.error('WalletContext API Error:', error);
      return {
        total_value_usd: '0.00',
        assets: [],
        networks: []
      };
    }
  },

  async sendAsset(asset: Asset, toAddress: string, amount: string): Promise<boolean> {
    let tag = " | sendAsset | "
    try {
      console.log(tag,'params: ',{asset, toAddress, amount})
      return true
    } catch (error) {
      console.error('Send transaction error:', error);
      return false;
    }
  },

  async getReceiveAddress(asset: Asset): Promise<string> {
    let tag = " | getReceiveAddress | "
    try {
      console.log(tag, 'Getting receive address for asset:', asset);
      
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
};

// Context Type
interface WalletContextType {
  portfolio: Portfolio | null;
  selectedAsset: Asset | null;
  loading: boolean;
  error: string | null;
  refreshPortfolio: () => Promise<void>;
  selectAsset: (asset: Asset | null) => void;
  sendAsset: (toAddress: string, amount: string) => Promise<boolean>;
  getReceiveAddress: () => Promise<string | null>;
}

// Create Context
const WalletContext = createContext<WalletContextType | undefined>(undefined);

// Provider Props
interface WalletProviderProps {
  children: ReactNode;
}


 async function initFooBarTable() {
  const db = await Database.load(DB_PATH);
  await db.execute(`CREATE TABLE IF NOT EXISTS foo_bar (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    foo TEXT NOT NULL,
    bar TEXT NOT NULL
  )`);
  return db;
}

 async function insertHelloWorld(db: Database) {
  await db.execute(
      'INSERT INTO foo_bar (foo, bar) VALUES ($1, $2)',
      ['hello', 'world']
  );
}

 async function getAllFooBar(db: Database) {
  return await db.select('SELECT * FROM foo_bar');
}

// Provider Component
export const WalletProvider: React.FC<WalletProviderProps> = ({ children }) => {
  const [portfolio, setPortfolio] = useState<Portfolio | null>(null);
  const [selectedAsset, setSelectedAsset] = useState<Asset | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [inSync, setInSync] = useState(false);

  const onStart = useCallback(async () => {
    let tag = TAG+" | onStart | "
    setLoading(true);
    try {
      //connect to DB
      const db = await initFooBarTable();
      await insertHelloWorld(db);
      const rows = await getAllFooBar(db);
      console.log(tag,rows);

      //isOnbarded
      let isOnboarded = true
      console.log(tag,'isOnboarded: ',isOnboarded)

      //get all devices from db
      let devices: any = [
        {
          deviceId: 'testingDevice123'
        }
      ]

      //get all xpubs from db for deviceId
      let allPubkeys = db.getPubkeys({deviceId: 'testingDevice123'});

      //verify we have for defaults
      console.log('paths: ',paths);
      for(let i=0; i<paths.paths.length; i++) {
        let path = paths[i];

      }

      //any missing, send to backend queue for device query

      //if complete
      setInSync(true);


      //get xpubs from db
      let xpubs : any = [
        {
          "device":"testingDevice123",
          "path":path,
          "label":" defualt x path",
          "caip": "bip122:000000000019d6689c085ae165831e93/slip44:0",
          "pubkey": "xpub6BxKtd6aAuz23XqtWXeSqxShJZn8yqiUmaTdvsPWS3riKkNRcXEPmn1CXmKM1M43mrWfN5QwjdLRghZLrgwMLCeRZqZNuYhVNXr6Pp7aDsH"
        },
        {
          "device":"testingDevice123",
          "path":path,
          "label":" defualt x path",
          "caip": "bip122:000000000019d6689c085ae165831e93/slip44:0",
          "pubkey": "ypub6WamSeXgTYgy7W25fVorMLDHFx5SPkuYaE7ToWCiyCUK2jdWpufQ8VqkDg83YjBtJFHDoekhf9ESdPDbL9aCPXC5NnmzXUiq3J6oycFShfS"
        },
        {
          "device":"testingDevice123",
          "path":path,
          "label":" defualt x path",
          "caip": "bip122:000000000019d6689c085ae165831e93/slip44:0",
          "pubkey": "zpub6rm1EEJg4JasiTqacdouiUVncAc5ymhKReiPZfLTGnH2GSZquRn9reJhj6sfs73PoSJNXzpERKPVLYbwwUGHNF6jkMX5R58vWaLB9FVyJuX"
        }
      ]
      console.log(tag, xpubs);

      //get balances for all

      let calculatedTotalUsd = 297.51

      let balances: any = [
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
      ]

      let networks: any = [
        {
          id: 1,
          network_name: "Bitcoin",
          symbol: "BTC",
          chain_id_caip2: "bip122:000000000019d6689c085ae165831e93",
          is_evm: false
        }
      ]


      // Get portfolio data and set it immediately
      const portfolioData = await assetApiService.getPortfolio();
      console.log(tag, 'portfolioData: ', portfolioData);
      setPortfolio(portfolioData);
      setError(null);

    } catch (error) {
      console.error(tag,'❌ [WalletContext] Failed to initialize:', error);
      setError('❌ [WalletContext] Failed to initialize: ' + (error instanceof Error ? error.message : 'Unknown error'))
    } finally {
      setLoading(false);
    }
  }, []);

  const refreshPortfolio = useCallback(async () => {
    let tag = TAG + " | refreshPortfolio | "
    setLoading(true);
    try {
      let portfolioData = await assetApiService.getPortfolio();
      console.log(tag,'portfolioData: ',portfolioData);

      setPortfolio(portfolioData);
      setError(null);
    } catch (error) {
      console.error('❌ [WalletContext] Failed to refresh portfolio:', error);
      setError('❌ [WalletContext] Failed to refresh portfolio: ' + (error instanceof Error ? error.message : 'Unknown error'))
    } finally {
      setLoading(false);
    }
  }, []);

  const selectAsset = (asset: Asset | null) => {
    setSelectedAsset(asset);
    console.log('🎯 [WalletContext] Asset selected:', asset?.symbol);
  };

  const sendAsset = async (toAddress: string, amount: string): Promise<boolean> => {
    if (!selectedAsset) {
      console.error('❌ [WalletContext] No asset selected for sending');
      return false;
    }

    console.log(`📤 [WalletContext] Sending ${amount} ${selectedAsset.symbol} to ${toAddress}`);
    const success = await assetApiService.sendAsset(selectedAsset, toAddress, amount);
    
    if (success) {
      console.log('✅ [WalletContext] Send transaction successful');
      // Refresh portfolio after successful send
      await refreshPortfolio();
    } else {
      console.error('❌ [WalletContext] Send transaction failed');
    }
    
    return success;
  };

  const getReceiveAddress = async (): Promise<string | null> => {
    if (!selectedAsset) {
      console.error('❌ [WalletContext] No asset selected for receive address');
      return null;
    }

    console.log(`📥 [WalletContext] Getting receive address for ${selectedAsset.symbol}`);
    const address = await assetApiService.getReceiveAddress(selectedAsset);
    
    if (address) {
      console.log('✅ [WalletContext] Receive address obtained:', address);
    } else {
      console.error('❌ [WalletContext] Failed to get receive address');
    }
    
    return address;
  };

  // Initial load
  useEffect(() => {
    onStart()
    refreshPortfolio();
  }, [onStart, refreshPortfolio]);

  const contextValue: WalletContextType = {
    portfolio,
    selectedAsset,
    loading,
    error,
    refreshPortfolio,
    selectAsset,
    sendAsset,
    getReceiveAddress,
  };

  return (
    <WalletContext.Provider value={contextValue}>
      {children}
    </WalletContext.Provider>
  );
};

// Hook to use the context
export const useWallet = (): WalletContextType => {
  const context = useContext(WalletContext);
  if (context === undefined) {
    throw new Error('useWallet must be used within an WalletProvider');
  }
  return context;
}; 