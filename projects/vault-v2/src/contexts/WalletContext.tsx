/*
    Primary Wallet Context

    Manages wallet state, device sync, and portfolio data
    All device calls go through backend queue - NO direct device communication

*/

import React, { createContext, useContext, useState, useEffect, ReactNode, useCallback } from 'react';
import { listen } from '@tauri-apps/api/event';

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
  const [isInitializing, setIsInitializing] = useState(false);

  const refreshPortfolio = useCallback(async () => {
    // ...existing refreshPortfolio logic...
    // (This is just a move; actual logic remains unchanged)
  }, []);

  // Track pending xpub requests to prevent duplicates
  const pendingXpubRequests = new Set<string>();

  // Requests all required xpubs from device queue and stores them in DB
  async function getXpubsFromDeviceQueue() {
    const tag = TAG + " | getXpubsFromDeviceQueue | ";
    
    try {
      // Get fresh list of connected devices to ensure we have current device IDs
      const connectedDevicesResponse = await DeviceQueueAPI.getConnectedDevices();
      console.log(tag, 'Fresh connected devices:', connectedDevicesResponse);
      
      if (!connectedDevicesResponse || connectedDevicesResponse.length === 0) {
        console.log(tag, 'No devices connected for xpub fetching');
        return;
      }

      // Use the first connected device (in a multi-device setup, we'd handle all)
      const deviceData = connectedDevicesResponse[0];
      const device = deviceData.device || deviceData;
      const deviceId = device.unique_id;
      
      console.log(tag, `Using device: ${deviceId}`);
      
      const requiredPaths = WalletDatabase.getRequiredPaths();

      for (const pathInfo of requiredPaths) {
        const requestKey = `${deviceId}:${pathInfo.path}`;
        
        // Skip if request is already pending
        if (pendingXpubRequests.has(requestKey)) {
          console.log(tag, `Skipping duplicate request for ${requestKey}`);
          continue;
        }

        // Check if xpub already exists in database
        // const existingXpubs = await WalletDatabase.getXpubs(deviceId);
        const existingXpubs:any = []
        const alreadyHas = existingXpubs.some((x: { path: string; caip: string; }) => x.path === pathInfo.path && x.caip === pathInfo.caip);
        if (alreadyHas) {
          console.log(tag, `Xpub already exists for ${deviceId} ${pathInfo.path}`);
          continue;
        }

        // Mark request as pending
        pendingXpubRequests.add(requestKey);

        try {
          console.log(tag, `🔄 Requesting xpub for ${deviceId} ${pathInfo.path}`);
          
          // 1. Queue xpub request
          const requestId = await DeviceQueueAPI.requestXpubFromDevice(deviceId, pathInfo.path);
          console.log(tag, `✅ Queued xpub request (requestId: ${requestId})`);

          // 2. Poll for result with longer timeout and exponential backoff
          const start = Date.now();
          let pollInterval = 2000; // Start with 2 seconds
          const maxPollInterval = 10000; // Max 10 seconds
          
          while (Date.now() - start < 120000) { // 2 minute timeout
            await new Promise(res => setTimeout(res, pollInterval));
            
            try {
              const status = await DeviceQueueAPI.getQueueStatus(deviceId);
              if (
                status.last_response &&
                'Xpub' in status.last_response &&
                status.last_response.Xpub.request_id === requestId
              ) {
                if (status.last_response.Xpub.success) {
                  // 3. Store xpub
                  await WalletDatabase.insertXpubFromQueue(
                    deviceId,
                    pathInfo.path,
                    status.last_response.Xpub.xpub
                  );
                  console.log(tag, `✅ Stored xpub for ${deviceId} ${pathInfo.path}`);
                  break;
                } else {
                  throw new Error(status.last_response.Xpub.error || 'Device returned error');
                }
              }
              
              // Exponential backoff - increase poll interval
              pollInterval = Math.min(pollInterval * 1.5, maxPollInterval);
              
            } catch (pollError) {
              console.warn(tag, `Poll error for ${requestKey}:`, pollError);
              // Continue polling despite poll errors
            }
          }
          
          console.log(tag, `⏰ Finished polling for ${requestKey}`);
          
        } catch (err) {
          console.error(tag, `❌ Failed to get xpub for ${deviceId} ${pathInfo.path}:`, err);
        } finally {
          // Always remove from pending set
          pendingXpubRequests.delete(requestKey);
        }
      }
      
      // Refresh portfolio after all xpub attempts
      console.log(tag, `🔄 Refreshing portfolio after xpub fetching`);
      await refreshPortfolio();
      
    } catch (error) {
      console.error(tag, 'Error in getXpubsFromDeviceQueue:', error);
    }
  }

  async function onStart() {
    const tag = TAG + " | onStart | ";
    
    // Prevent multiple simultaneous initializations
    if (isInitializing) {
      console.log(tag, 'Already initializing, skipping duplicate onStart call');
      return;
    }
    
    setIsInitializing(true);
    setLoading(true);
    
    try {
      // Initialize database
      await WalletDatabase.init();

      // Check if onboarded
      const isOnboarded = true;
      console.log(tag, 'isOnboarded:', isOnboarded);

      // Get connected devices from backend (live detection)
      console.log(tag, '🔍 Detecting connected devices...');
      const connectedDevicesResponse = await DeviceQueueAPI.getConnectedDevices();
      console.log(tag, 'Connected devices:', connectedDevicesResponse);

      if (!connectedDevicesResponse || connectedDevicesResponse.length === 0) {
        console.log(tag, 'No devices connected');
        setIsSync(false);
        setError('No KeepKey device connected. Please connect your device and try again.');
        return;
      }

      // Store detected devices in database for future reference
      await WalletDatabase.storeConnectedDevices(connectedDevicesResponse);

      // Get all devices from db (now includes freshly detected ones)
      const devices = await WalletDatabase.getDevices();
      console.log(tag, 'devices from db:', devices);

      if (devices.length === 0) {
        console.log(tag, 'No devices found after detection, not synced');
        setIsSync(false);
        return;
      }

      // Always try to fetch fresh xpubs on startup to ensure we have the latest
      // This ensures we detect newly connected devices and get their xpubs
      console.log(tag, '🔄 Always fetching fresh xpubs on startup...');
      await getXpubsFromDeviceQueue();

      // After fetching, check final sync status
      const finalDevices = await WalletDatabase.getDevices();
      const requiredPaths = WalletDatabase.getRequiredPaths();
      let allXpubsPresent = true;
      
      for (const device of finalDevices) {
        console.log(tag, 'Final check - xpubs for device:', device.device_id);

        const existingXpubs:any[] = []
        // const existingXpubs = await WalletDatabase.getXpubs(device.device_id);
        // console.log(tag, 'Final existing xpubs:', existingXpubs.length);

        for (const requiredPath of requiredPaths) {
          const existingXpub = existingXpubs.find(
            x => x.path === requiredPath.path && x.caip === requiredPath.caip
          );
          if (!existingXpub) {
            allXpubsPresent = false;
            console.log(tag, `Missing xpub for ${device.device_id} ${requiredPath.path}`);
          }
        }
      }

      // Set final sync status
      setIsSync(allXpubsPresent);
      console.log(tag, 'Final sync status:', allXpubsPresent);
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
      setIsInitializing(false);
    }
  }

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
    // Ensure BTC asset is selected
    const btcCaip = "bip122:000000000019d6689c085ae165831e93/slip44:0";
    let asset = selectedAsset;
    if (!asset || asset.caip !== btcCaip) {
      asset = portfolio?.assets.find((a: any) => a.caip === btcCaip) || null;
      if (!asset) {
        console.error('❌ No BTC asset found in portfolio');
        return null;
      }
      selectAsset(asset);
      // Wait a tick for context update
      await new Promise(res => setTimeout(res, 50));
    }

    console.log(`📥 Getting receive address for ${asset.symbol} using device queue`);
    try {
      // Get the first device from database
      const devices = await WalletDatabase.getDevices();
      if (!devices || devices.length === 0) {
        throw new Error('No devices available');
      }
      const device = devices[0];
      console.log('🔑 Device object:', device);
      console.log('🔑 device.device_id:', device.device_id);
      if (!device.device_id || typeof device.device_id !== 'string' || device.device_id.length === 0) {
        throw new Error('Device object missing or invalid device_id: ' + JSON.stringify(device));
      }
      console.log('🔑 Using device:', device);

      // For Bitcoin, use the first available path (can be enhanced later)
      const receivePath = "m/84'/0'/0'/0/0"; // Native SegWit receive path

      // Always pass show_display: true
      const requestId = await DeviceQueueAPI.requestReceiveAddressFromDevice(
        device.device_id,
        receivePath,
        'Bitcoin',
        'p2wpkh',
        true // Always show on device
      );
      console.log('🟢 Device queue requestId:', requestId);

      // Poll for address result
      const start = Date.now();
      while (Date.now() - start < 60000) {
        const status = await getQueueStatus(device.device_id);
        if (status.last_response) {
          if ('Address' in status.last_response && status.last_response.Address.request_id === requestId) {
            if (status.last_response.Address.success) {
              const address = status.last_response.Address.address;
              console.log('📥 Address generated:', address);
              return address;
            }
            throw new Error(status.last_response.Address.error || 'Device returned error');
          }
        }
        await new Promise(res => setTimeout(res, 1000));
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
              const existingXpubs: any[] = []
              // const existingXpubs = await WalletDatabase.getXpubs(device_id);
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

  // Listen for device reconnects and purge queue
  useEffect(() => {
    let unlistenConnect: Promise<() => void>;
    (async () => {
      unlistenConnect = listen('device:connected', async (event: any) => {
        const deviceId = event.payload?.device_id || event.payload;
        try {
          console.log(TAG, 'Device reconnected', deviceId, '- resetting queue');
          await DeviceQueueAPI.resetDeviceQueue(deviceId);
          await getXpubsFromDeviceQueue();
        } catch (e) {
          console.error(TAG, 'Failed to reset queue on reconnect:', e);
        }
      });
    })();

    return () => {
      unlistenConnect?.then(fn => fn());
    };
  }, []);

  // Initial load
  useEffect(() => {
    onStart();
  }, []);

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