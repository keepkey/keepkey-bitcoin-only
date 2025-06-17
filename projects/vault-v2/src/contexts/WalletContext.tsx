/*
    Primary Wallet Context

    Manages wallet state, device sync, and portfolio data
    All device calls go through backend queue - NO direct device communication

*/

import React, { createContext, useContext, useState, useEffect, ReactNode, useCallback } from 'react';

// Import organized types and services
import { Asset, Portfolio, QueueStatus } from '../types';
import { WalletDatabase, PortfolioAPI, DeviceQueueAPI } from '../lib';

const TAG = " | WalletContext | ";

// Context Type
interface WalletContextType {
  portfolio: Portfolio | null;
  selectedAsset: Asset | null;
  loading: boolean;
  error: string | null;
  isSync: boolean;
  refreshPortfolio: () => Promise<void>;
  selectAsset: (asset: Asset | null) => void;
  sendAsset: (toAddress: string, amount: string) => Promise<boolean>;
  getReceiveAddress: () => Promise<string | null>;
  requestXpubFromDevice: (deviceId: string, path: string) => Promise<string>;
  getQueueStatus: (deviceId: string) => Promise<QueueStatus>;
}

// Create Context
const WalletContext = createContext<WalletContextType | undefined>(undefined);

// Provider Props
interface WalletProviderProps {
  children: ReactNode;
}

// Provider Component
export const WalletProvider: React.FC<WalletProviderProps> = ({ children }) => {
  const [portfolio, setPortfolio] = useState<Portfolio | null>(null);
  const [selectedAsset, setSelectedAsset] = useState<Asset | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [isSync, setIsSync] = useState(false);

  const onStart = useCallback(async () => {
    const tag = TAG + " | onStart | ";
    setLoading(true);
    
    try {
      // Initialize database
      await WalletDatabase.init();
      await WalletDatabase.seedMockDevice();

      // Check if onboarded
      const isOnboarded = true;
      console.log(tag, 'isOnboarded:', isOnboarded);

      // Get all devices from db
      const devices = await WalletDatabase.getDevices();
      console.log(tag, 'devices from db:', devices);

      if (devices.length === 0) {
        console.log(tag, 'No devices found, not synced');
        setIsSync(false);
        return;
      }

      // For each device, check if we have all required xpubs
      let allXpubsPresent = true;
      const requiredPaths = WalletDatabase.getRequiredPaths();
      
      for (const device of devices) {
        console.log(tag, 'Checking xpubs for device:', device.device_id);
        
        const existingXpubs = await WalletDatabase.getXpubs(device.device_id);
        console.log(tag, 'Existing xpubs:', existingXpubs);

        for (const requiredPath of requiredPaths) {
          const existingXpub = existingXpubs.find(
            x => x.path === requiredPath.path && x.caip === requiredPath.caip
          );
          
          if (!existingXpub) {
            console.log(tag, `Missing xpub for ${device.device_id} path ${requiredPath.path}, adding to queue`);
            allXpubsPresent = false;
            
            // Add request to device queue - NON-BLOCKING
            try {
              const requestId = await DeviceQueueAPI.requestXpubFromDevice(device.device_id, requiredPath.path);
              console.log(tag, `Added xpub request to queue: ${requestId}`);
            } catch (error) {
              console.warn(tag, `Failed to add xpub request to queue:`, error);
            }
          }
        }
      }

      // Set sync status
      setIsSync(allXpubsPresent);
      console.log(tag, 'Sync status:', allXpubsPresent);
      
      // Get portfolio data regardless of sync status
      const portfolioData = await PortfolioAPI.getPortfolio();
      console.log(tag, 'portfolioData:', portfolioData);
      setPortfolio(portfolioData);
      setError(null);

    } catch (error) {
      console.error(tag, '❌ Failed to initialize:', error);
      setError('❌ Failed to initialize: ' + (error instanceof Error ? error.message : 'Unknown error'));
    } finally {
      setLoading(false);
    }
  }, []);

  const refreshPortfolio = useCallback(async () => {
    const tag = TAG + " | refreshPortfolio | ";
    setLoading(true);
    
    try {
      const portfolioData = await PortfolioAPI.getPortfolio();
      console.log(tag, 'portfolioData:', portfolioData);
      setPortfolio(portfolioData);
      setError(null);
    } catch (error) {
      console.error('❌ Failed to refresh portfolio:', error);
      setError('❌ Failed to refresh portfolio: ' + (error instanceof Error ? error.message : 'Unknown error'));
    } finally {
      setLoading(false);
    }
  }, []);

  const selectAsset = (asset: Asset | null) => {
    setSelectedAsset(asset);
    console.log('🎯 Asset selected:', asset?.symbol);
  };

  const sendAsset = async (toAddress: string, amount: string): Promise<boolean> => {
    if (!selectedAsset) {
      console.error('❌ No asset selected for sending');
      return false;
    }

    console.log(`📤 Sending ${amount} ${selectedAsset.symbol} to ${toAddress}`);
    const success = await PortfolioAPI.sendAsset(selectedAsset, toAddress, amount);
    
    if (success) {
      console.log('✅ Send transaction successful');
      await refreshPortfolio();
    } else {
      console.error('❌ Send transaction failed');
    }
    
    return success;
  };

  const getReceiveAddress = async (): Promise<string | null> => {
    if (!selectedAsset) {
      console.error('❌ No asset selected for receive address');
      return null;
    }

    console.log(`📥 Getting receive address for ${selectedAsset.symbol} using device queue`);
    try {
      // Get the first device from database
      const devices = await WalletDatabase.getDevices();
      if (devices.length === 0) {
        throw new Error('No devices available');
      }
      
      const device = devices[0];
      
      // For Bitcoin, use the first available path (we'll enhance this later)
      // TODO: Make this configurable or determine from asset type
      const receivePath = "m/84'/0'/0'/0/0"; // Native SegWit receive path
      
      // Request address from device queue with display confirmation
      // Following vault v1 pattern: show_display=true bypasses cache and shows on device for security
      const requestId = await DeviceQueueAPI.requestReceiveAddressFromDevice(
        device.device_id,
        receivePath,
        'Bitcoin',
        'p2wpkh',
        true // IMPORTANT: Show on device for security verification
      );
      
      console.log(`📝 Address request queued with ID: ${requestId}`);
      
      // Poll for completion (in a real app, you'd use websockets or events)
      for (let i = 0; i < 30; i++) { // 30 second timeout
        await new Promise(resolve => setTimeout(resolve, 1000));
        
        const status = await DeviceQueueAPI.getQueueStatus(device.device_id);
        
        if (status.last_response) {
          // Check if this is our address response
          if ('Address' in status.last_response && 
              status.last_response.Address.request_id === requestId &&
              status.last_response.Address.success) {
            
            const address = status.last_response.Address.address;
            console.log('✅ Receive address obtained from device:', address);
            return address;
          }
          
          // Check for error
          if ('Address' in status.last_response && 
              status.last_response.Address.request_id === requestId &&
              !status.last_response.Address.success) {
            
            throw new Error(status.last_response.Address.error || 'Device returned error');
          }
        }
      }
      
      throw new Error('Timeout waiting for device response');
      
    } catch (error) {
      console.error('❌ Failed to get receive address from device:', error);
      throw error;
    }
  };

  const requestXpubFromDevice = async (deviceId: string, path: string): Promise<string> => {
    return DeviceQueueAPI.requestXpubFromDevice(deviceId, path);
  };

  const getQueueStatus = async (deviceId: string): Promise<QueueStatus> => {
    return DeviceQueueAPI.getQueueStatus(deviceId);
  };

  // Queue monitoring - check for completed xpub requests every 5 seconds
  useEffect(() => {
    const monitorQueue = async () => {
      try {
        const devices = await WalletDatabase.getDevices();
        
        for (const device of devices) {
          const status = await getQueueStatus(device.device_id);
          
          if (status.last_response) {
            // Handle xpub responses
            if ('Xpub' in status.last_response && status.last_response.Xpub.success) {
              const { device_id, path, xpub } = status.last_response.Xpub;
              
              // Check if we already have this xpub in the database
              const existingXpubs = await WalletDatabase.getXpubs(device_id);
              const exists = existingXpubs.some(x => x.path === path && x.pubkey === xpub);
              
              if (!exists) {
                console.log('📥 Storing completed xpub request:', path, '->', xpub.substring(0, 20) + '...');
                await WalletDatabase.insertXpubFromQueue(device_id, path, xpub);
                
                // Clear balance cache to force refresh with new xpub
                await WalletDatabase.clearBalanceCache();
                
                // Refresh portfolio to include new xpub data
                await refreshPortfolio();
              }
            }
            
            // Handle address responses (for receive address generation)
            if ('Address' in status.last_response && status.last_response.Address.success) {
              console.log('📥 Address generated:', status.last_response.Address.address);
              // Address responses are handled by the getReceiveAddress method
            }
          }
        }
      } catch (error) {
        // Silently ignore errors in queue monitoring
        console.debug('Queue monitoring error:', error);
      }
    };

    // Monitor every 5 seconds
    const interval = setInterval(monitorQueue, 5000);
    
    return () => clearInterval(interval);
  }, [getQueueStatus, refreshPortfolio]);

  // Initial load
  useEffect(() => {
    onStart();
  }, [onStart]);

  const contextValue: WalletContextType = {
    portfolio,
    selectedAsset,
    loading,
    error,
    isSync,
    refreshPortfolio,
    selectAsset,
    sendAsset,
    getReceiveAddress,
    requestXpubFromDevice,
    getQueueStatus,
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
    throw new Error('useWallet must be used within a WalletProvider');
  }
  return context;
}; 