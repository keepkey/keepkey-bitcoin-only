import React, { createContext, useContext, useState, useEffect, ReactNode } from 'react';

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
    try {
      const [dashboardRes, networksRes, balancesRes] = await Promise.all([
        fetch('http://localhost:1646/api/v2/portfolio/summary'),
        fetch('http://localhost:1646/api/v2/networks'),
        fetch('http://localhost:1646/api/v2/balances')
      ]);

      if (!dashboardRes.ok || !networksRes.ok || !balancesRes.ok) {
        throw new Error('Failed to fetch data');
      }

      const [dashboard, networks, balances] = await Promise.all([
        dashboardRes.json(),
        networksRes.json(),
        balancesRes.json()
      ]);

      return {
        total_value_usd: dashboard.total_value_usd,
        assets: balances.map((balance: any) => ({
          symbol: balance.symbol,
          name: balance.symbol, // We could enhance this with full names
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
      console.error('AssetContext API Error:', error);
      return {
        total_value_usd: '0.00',
        assets: [],
        networks: []
      };
    }
  },

  async sendAsset(asset: Asset, toAddress: string, amount: string): Promise<boolean> {
    try {
      const response = await fetch('http://localhost:1646/api/v2/send', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          caip: asset.caip,
          to: toAddress,
          amount: amount
        })
      });
      return response.ok;
    } catch (error) {
      console.error('Send transaction error:', error);
      return false;
    }
  },

  async getReceiveAddress(asset: Asset): Promise<string | null> {
    try {
      const response = await fetch(`http://localhost:1646/api/v2/address/${asset.caip}`);
      if (!response.ok) throw new Error('Failed to get address');
      const data = await response.json();
      return data.address;
    } catch (error) {
      console.error('Get address error:', error);
      return null;
    }
  }
};

// Context Type
interface AssetContextType {
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
const AssetContext = createContext<AssetContextType | undefined>(undefined);

// Provider Props
interface AssetProviderProps {
  children: ReactNode;
}

// Provider Component
export const AssetProvider: React.FC<AssetProviderProps> = ({ children }) => {
  const [portfolio, setPortfolio] = useState<Portfolio | null>(null);
  const [selectedAsset, setSelectedAsset] = useState<Asset | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const refreshPortfolio = async () => {
    console.log('🔄 [AssetContext] Refreshing portfolio data');
    setLoading(true);
    setError(null);

    try {
      const portfolioData = await assetApiService.getPortfolio();
      setPortfolio(portfolioData);
      console.log('✅ [AssetContext] Portfolio updated:', portfolioData);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to fetch portfolio';
      setError(errorMessage);
      console.error('❌ [AssetContext] Error refreshing portfolio:', err);
    } finally {
      setLoading(false);
    }
  };

  const selectAsset = (asset: Asset | null) => {
    setSelectedAsset(asset);
    console.log('🎯 [AssetContext] Asset selected:', asset?.symbol);
  };

  const sendAsset = async (toAddress: string, amount: string): Promise<boolean> => {
    if (!selectedAsset) {
      console.error('❌ [AssetContext] No asset selected for sending');
      return false;
    }

    console.log(`📤 [AssetContext] Sending ${amount} ${selectedAsset.symbol} to ${toAddress}`);
    const success = await assetApiService.sendAsset(selectedAsset, toAddress, amount);
    
    if (success) {
      console.log('✅ [AssetContext] Send transaction successful');
      // Refresh portfolio after successful send
      await refreshPortfolio();
    } else {
      console.error('❌ [AssetContext] Send transaction failed');
    }
    
    return success;
  };

  const getReceiveAddress = async (): Promise<string | null> => {
    if (!selectedAsset) {
      console.error('❌ [AssetContext] No asset selected for receive address');
      return null;
    }

    console.log(`📥 [AssetContext] Getting receive address for ${selectedAsset.symbol}`);
    const address = await assetApiService.getReceiveAddress(selectedAsset);
    
    if (address) {
      console.log('✅ [AssetContext] Receive address obtained:', address);
    } else {
      console.error('❌ [AssetContext] Failed to get receive address');
    }
    
    return address;
  };

  // Initial load
  useEffect(() => {
    refreshPortfolio();
  }, []);

  const contextValue: AssetContextType = {
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
    <AssetContext.Provider value={contextValue}>
      {children}
    </AssetContext.Provider>
  );
};

// Hook to use the context
export const useAssets = (): AssetContextType => {
  const context = useContext(AssetContext);
  if (context === undefined) {
    throw new Error('useAssets must be used within an AssetProvider');
  }
  return context;
}; 