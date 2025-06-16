import React, { createContext, useContext, useEffect, useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

// ========== Data Types ==========

export interface RequiredPath {
  path: string;
  label: string;
  caip: string;
}

export interface WalletXpub {
  id: number;
  device_id: string;
  path: string;
  label: string;
  caip: string;
  pubkey: string;
  created_at: number;
}

export interface PortfolioCache {
  id: number;
  pubkey: string;
  caip: string;
  balance: string;
  balance_usd: string;
  price_usd: string;
  symbol?: string;
  last_updated: number;
}

export interface FeeRateCache {
  id: number;
  caip: string;
  fastest: number;
  fast: number;
  average: number;
  last_updated: number;
}

export interface WalletSummary {
  total_balance_btc: number;
  total_balance_usd: number;
  total_xpubs: number;
  devices: Array<{
    device_id: string;
    xpubs: Array<{
      path: string;
      label: string;
      pubkey: string;
    }>;
    balance_btc: number;
    balance_usd: number;
  }>;
  last_updated: number;
}

export interface SyncProgress {
  device_id: string;
  path: string;
  label: string;
  status: 'fetching' | 'completed' | 'error';
  xpub?: string;
  error?: string;
}

// ========== Context ==========

interface WalletContextType {
  // State
  xpubs: WalletXpub[];
  portfolio: PortfolioCache[];
  summary: WalletSummary | null;
  isLoading: boolean;
  isSyncing: boolean;
  syncProgress: SyncProgress | null;
  lastSync: number | null;

  // Actions
  syncDeviceXpubs: (deviceId: string) => Promise<void>;
  refreshPortfolio: (force?: boolean) => Promise<void>;
  clearPortfolioCache: () => Promise<void>;
  getWalletSummary: () => Promise<void>;
  extractXpubsFromCache: (deviceId: string) => Promise<void>;
  
  // Data getters
  getXpubsForDevice: (deviceId: string) => WalletXpub[];
  getBalanceForXpub: (pubkey: string) => PortfolioCache | null;
  getTotalBalance: () => { btc: number; usd: number };
}

const WalletContext = createContext<WalletContextType | null>(null);

// ========== Provider ==========

export const WalletProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const [xpubs, setXpubs] = useState<WalletXpub[]>([]);
  const [portfolio, setPortfolio] = useState<PortfolioCache[]>([]);
  const [summary, setSummary] = useState<WalletSummary | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [isSyncing, setIsSyncing] = useState(false);
  const [syncProgress, setSyncProgress] = useState<SyncProgress | null>(null);
  const [lastSync, setLastSync] = useState<number | null>(null);

  // ========== Load Initial Data ==========

  const loadWalletData = useCallback(async () => {
    console.log("🔄 Loading wallet data from database...");
    setIsLoading(true);
    
    try {
      // Load xpubs and portfolio cache from database
      const [xpubsData, portfolioData] = await Promise.all([
        invoke<WalletXpub[]>('get_wallet_xpubs'),
        invoke<PortfolioCache[]>('get_portfolio_cache')
      ]);

      console.log(`📚 Loaded ${xpubsData.length} xpubs and ${portfolioData.length} portfolio entries`);
      
      setXpubs(xpubsData);
      setPortfolio(portfolioData);
      
      if (portfolioData.length > 0) {
        setLastSync(portfolioData[0].last_updated);
      }
      
    } catch (error) {
      console.error("❌ Failed to load wallet data:", error);
    } finally {
      setIsLoading(false);
    }
  }, []);

  // ========== Sync Operations ==========

  const syncDeviceXpubs = useCallback(async (deviceId: string) => {
    console.log(`🔄 Starting xpub sync for device: ${deviceId}`);
    setIsSyncing(true);
    setSyncProgress({ device_id: deviceId, path: '', label: 'Initializing...', status: 'fetching' });

    try {
      const syncedXpubs = await invoke<WalletXpub[]>('sync_device_xpubs', { 
        deviceId 
      });
      
      console.log(`✅ Synced ${syncedXpubs.length} xpubs for device ${deviceId}`);
      
      // Reload all xpubs to get fresh data
      await loadWalletData();
      
    } catch (error) {
      console.error(`❌ Failed to sync xpubs for device ${deviceId}:`, error);
      setSyncProgress({ 
        device_id: deviceId, 
        path: '', 
        label: 'Error', 
        status: 'error', 
        error: String(error) 
      });
      throw error;
    } finally {
      setIsSyncing(false);
      // Clear progress after 3 seconds
      setTimeout(() => setSyncProgress(null), 3000);
    }
  }, [loadWalletData]);

  const refreshPortfolio = useCallback(async (force: boolean = false) => {
    console.log(`🔄 Refreshing portfolio (force: ${force})...`);
    setIsLoading(true);

    try {
      const portfolioData = await invoke<PortfolioCache[]>('refresh_portfolio', { 
        force 
      });
      
      console.log(`✅ Refreshed portfolio with ${portfolioData.length} entries`);
      
      setPortfolio(portfolioData);
      if (portfolioData.length > 0) {
        setLastSync(portfolioData[0].last_updated);
      }
      
      // Also refresh summary
      await getWalletSummary();
      
    } catch (error) {
      console.error("❌ Failed to refresh portfolio:", error);
      throw error;
    } finally {
      setIsLoading(false);
    }
  }, []);

  const clearPortfolioCache = useCallback(async () => {
    console.log("🧹 Clearing portfolio cache...");
    
    try {
      await invoke('clear_portfolio_cache');
      setPortfolio([]);
      setSummary(null);
      setLastSync(null);
      console.log("✅ Portfolio cache cleared");
    } catch (error) {
      console.error("❌ Failed to clear portfolio cache:", error);
      throw error;
    }
  }, []);

  const getWalletSummary = useCallback(async () => {
    try {
      const summaryData = await invoke<WalletSummary>('get_wallet_summary');
      setSummary(summaryData);
      console.log(`📊 Wallet summary: ${summaryData.total_balance_btc} BTC ($${summaryData.total_balance_usd.toFixed(2)})`);
    } catch (error) {
      console.error("❌ Failed to get wallet summary:", error);
      throw error;
    }
  }, []);

  const extractXpubsFromCache = useCallback(async (deviceId: string) => {
    console.log(`🔄 Extracting xpubs from cache for device: ${deviceId}`);
    setIsLoading(true);

    try {
      const xpubsData = await invoke<WalletXpub[]>('extract_xpubs_from_cache', { 
        deviceId 
      });
      
      console.log(`✅ Extracted ${xpubsData.length} xpubs from cache for device ${deviceId}`);
      
      setXpubs(xpubsData);
      if (xpubsData.length > 0) {
        setLastSync(xpubsData[0].created_at);
      }
      
    } catch (error) {
      console.error(`❌ Failed to extract xpubs from cache for device ${deviceId}:`, error);
      throw error;
    } finally {
      setIsLoading(false);
    }
  }, []);

  // ========== Data Getters ==========

  const getXpubsForDevice = useCallback((deviceId: string) => {
    return xpubs.filter(xpub => xpub.device_id === deviceId);
  }, [xpubs]);

  const getBalanceForXpub = useCallback((pubkey: string) => {
    return portfolio.find(p => p.pubkey === pubkey) || null;
  }, [portfolio]);

  const getTotalBalance = useCallback(() => {
    const btc = portfolio.reduce((total, p) => total + parseFloat(p.balance || '0'), 0);
    const usd = portfolio.reduce((total, p) => total + parseFloat(p.balance_usd || '0'), 0);
    return { btc, usd };
  }, [portfolio]);

  // ========== Event Listeners ==========

  useEffect(() => {
    // Listen for wallet sync progress events
    const unlistenProgress = listen<SyncProgress>('wallet-sync-progress', (event) => {
      console.log("📡 Wallet sync progress:", event.payload);
      setSyncProgress(event.payload);
    });

    // Listen for wallet sync completion events
    const unlistenCompleted = listen<{ device_id: string; xpubs: WalletXpub[] }>('wallet-sync-completed', (event) => {
      console.log("🎉 Wallet sync completed:", event.payload);
      setSyncProgress(null);
      // Reload data
      loadWalletData();
    });

    // Listen for portfolio refresh progress events
    const unlistenPortfolioProgress = listen<{ status: string; total: number; completed: number; current?: string }>('portfolio-refresh-progress', (event) => {
      console.log("📊 Portfolio refresh progress:", event.payload);
    });

    // Listen for portfolio refresh completion events
    const unlistenPortfolioCompleted = listen<{ status: string; entries: number; data: PortfolioCache[] }>('portfolio-refresh-completed', (event) => {
      console.log("🎉 Portfolio refresh completed:", event.payload);
      setPortfolio(event.payload.data);
      if (event.payload.data.length > 0) {
        setLastSync(event.payload.data[0].last_updated);
      }
    });

    // Listen for portfolio refresh errors
    const unlistenPortfolioError = listen<{ xpub: string; error: string }>('portfolio-refresh-error', (event) => {
      console.warn("⚠️ Portfolio refresh error:", event.payload);
    });

    // Listen for xpub extraction progress
    const unlistenXpubProgress = listen<{ device_id: string; path: string; label: string; status: string; xpub?: string }>('xpub-extraction-progress', (event) => {
      console.log("📡 Xpub extraction progress:", event.payload);
    });

    // Listen for xpub extraction completion
    const unlistenXpubCompleted = listen<{ device_id: string; extracted: number; total: number; xpubs: WalletXpub[] }>('xpub-extraction-completed', (event) => {
      console.log("🎉 Xpub extraction completed:", event.payload);
      setXpubs(event.payload.xpubs);
      if (event.payload.xpubs.length > 0) {
        setLastSync(event.payload.xpubs[0].created_at);
      }
    });

    // Cleanup listeners
    return () => {
      unlistenProgress.then(fn => fn());
      unlistenCompleted.then(fn => fn());
      unlistenPortfolioProgress.then(fn => fn());
      unlistenPortfolioCompleted.then(fn => fn());
      unlistenPortfolioError.then(fn => fn());
      unlistenXpubProgress.then(fn => fn());
      unlistenXpubCompleted.then(fn => fn());
    };
  }, [loadWalletData]);

  // ========== Background Monitoring ==========

  useEffect(() => {
    // Load initial data on mount
    loadWalletData();

    // If we have no xpubs but the app just started, try to auto-extract from first connected device
    const autoExtractOnStart = async () => {
      try {
        // Get connected devices
        const devices = await invoke<any[]>('get_connected_devices');
        if (devices.length > 0 && xpubs.length === 0) {
          const firstDevice = devices[0];
          if (firstDevice?.device?.unique_id) {
            console.log("🔄 Auto-extracting xpubs for first device on startup:", firstDevice.device.unique_id);
            await extractXpubsFromCache(firstDevice.device.unique_id);
          }
        }
      } catch (error) {
        console.error("❌ Auto-extraction on startup failed:", error);
      }
    };

    // Run auto-extraction after a delay to let the app settle
    setTimeout(autoExtractOnStart, 2000);

    // Set up background monitoring (every 5 minutes)
    const monitorInterval = setInterval(async () => {
      console.log("⏰ Background wallet data refresh...");
      try {
        // Only refresh if we have xpubs and cache is getting old (>5 minutes)
        if (xpubs.length > 0 && lastSync && Date.now() / 1000 - lastSync > 300) {
          await refreshPortfolio(false); // Non-forced refresh
        }
      } catch (error) {
        console.error("❌ Background refresh failed:", error);
      }
    }, 5 * 60 * 1000); // 5 minutes

    return () => clearInterval(monitorInterval);
  }, [loadWalletData, refreshPortfolio, xpubs.length, lastSync, extractXpubsFromCache]);

  // ========== Context Value ==========

  const contextValue: WalletContextType = {
    // State
    xpubs,
    portfolio,
    summary,
    isLoading,
    isSyncing,
    syncProgress,
    lastSync,
    
    // Actions
    syncDeviceXpubs,
    refreshPortfolio,
    clearPortfolioCache,
    getWalletSummary,
    extractXpubsFromCache,
    
    // Data getters
    getXpubsForDevice,
    getBalanceForXpub,
    getTotalBalance,
  };

  return (
    <WalletContext.Provider value={contextValue}>
      {children}
    </WalletContext.Provider>
  );
};

// ========== Hook ==========

export const useWallet = () => {
  const context = useContext(WalletContext);
  if (!context) {
    throw new Error('useWallet must be used within a WalletProvider');
  }
  return context;
};

export default WalletContext; 