/*
    Primary Wallet Context

    Manages wallet state, device sync, and portfolio data
    All device calls go through backend queue - NO direct device communication

*/

import React, { createContext, useContext, useState, useEffect, useRef, ReactNode, useCallback } from 'react';
import { listen } from '@tauri-apps/api/event';

// Import organized types and services
import { Asset, Portfolio, QueueStatus } from '../types';
import { WalletDatabase, PortfolioAPI, DeviceQueueAPI, PioneerAPI } from '../lib';

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
  const initializingRef = useRef(false);

  // Track pending xpub requests to prevent duplicates
  const pendingXpubRequests = new Set<string>();
  
  // Store fetched xpubs in memory (not database for v2)
  const [fetchedXpubs, setFetchedXpubs] = useState<Array<{path: string, xpub: string, caip: string}>>([]);

  const refreshPortfolio = useCallback(async () => {
    const tag = TAG + " | refreshPortfolio | ";
    setLoading(true);
    
    try {
      console.log(tag, `Refreshing portfolio with ${fetchedXpubs.length} xpubs in memory`);
      
      if (fetchedXpubs.length === 0) {
        console.log(tag, 'No xpubs available yet, showing empty portfolio');
        setPortfolio(null);
        return;
      }
      
      // Convert in-memory xpubs to Pioneer API format and fetch portfolio
      const requests = fetchedXpubs.map(x => ({
        caip: x.caip,
        pubkey: x.xpub
      }));
      
      console.log(tag, 'Calling Pioneer API with in-memory xpubs:', requests);
      
      // Call Pioneer API directly with in-memory xpubs
      const portfolioData = await PioneerAPI.getPortfolio(requests);
      
      // Transform to portfolio format (simplified version of PortfolioAPI.transformToPortfolio)
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

      for (const item of portfolioData) {
        const balance = parseFloat(item.balance) || 0;
        const valueUsd = parseFloat(item.valueUsd) || 0;
        const priceUsd = parseFloat(item.priceUsd) || 0;
        
        if (symbolGroups.has(item.symbol)) {
          const existing = symbolGroups.get(item.symbol)!;
          existing.balance += balance;
          existing.valueUsd += valueUsd;
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
          change_24h: 0
        });
      }

      const portfolio: Portfolio = {
        total_value_usd: totalValueUsd.toFixed(2),
        assets,
        networks
      };
      
      setPortfolio(portfolio);
      setError(null);
      console.log(tag, 'Portfolio refreshed successfully:', portfolio);
      
    } catch (error) {
      console.error(tag, 'Failed to refresh portfolio:', error);
      setError('Failed to refresh portfolio: ' + (error instanceof Error ? error.message : 'Unknown error'));
    } finally {
      setLoading(false);
    }
  }, [fetchedXpubs]);

  // Requests all required xpubs from device queue (event-driven, no polling!)
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

      // Send all xpub requests at once (no polling needed - events will handle responses)
      for (const pathInfo of requiredPaths) {
        const requestKey = `${deviceId}:${pathInfo.path}`;
        
        // Skip if request is already pending
        if (pendingXpubRequests.has(requestKey)) {
          console.log(tag, `Skipping duplicate request for ${requestKey}`);
          continue;
        }

        // Skip if we already have this xpub in memory
        const alreadyHas = fetchedXpubs.some(x => x.path === pathInfo.path && x.caip === pathInfo.caip);
        if (alreadyHas) {
          console.log(tag, `Xpub already fetched for ${deviceId} ${pathInfo.path}`);
          continue;
        }

        // Mark request as pending
        pendingXpubRequests.add(requestKey);

        try {
          console.log(tag, `🔄 Requesting xpub for ${deviceId} ${pathInfo.path}`);
          
          // Send xpub request (response will come via event)
          const requestId = await DeviceQueueAPI.requestXpubFromDevice(deviceId, pathInfo.path);
          console.log(tag, `✅ Queued xpub request (requestId: ${requestId}) - waiting for event...`);
          
        } catch (err) {
          console.error(tag, `❌ Failed to queue xpub request for ${deviceId} ${pathInfo.path}:`, err);
          pendingXpubRequests.delete(requestKey);
        }
      }
      
    } catch (error) {
      console.error(tag, 'Error in getXpubsFromDeviceQueue:', error);
    }
  }

  async function onStart() {
    const tag = TAG + " | onStart | ";
    
    // Prevent multiple simultaneous initializations
    if (isInitializing || initializingRef.current) {
      console.log(tag, 'Already initializing, skipping duplicate onStart call');
      return;
    }
    
    initializingRef.current = true;
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
      // Skip the old portfolio API call - we'll use in-memory xpubs via refreshPortfolio
      console.log(tag, 'Skipping database portfolio call - using in-memory xpubs instead');
      
      // If we have xpubs in memory, refresh portfolio immediately
      if (fetchedXpubs.length > 0) {
        console.log(tag, `We have ${fetchedXpubs.length} xpubs in memory, refreshing portfolio`);
        await refreshPortfolio();
      } else {
        console.log(tag, 'No xpubs in memory yet, portfolio will refresh when xpubs arrive via events');
        setPortfolio(null);
      }
      
      setError(null);

    } catch (error) {
      console.error(tag, '❌ Failed to initialize:', error);
      setError('❌ Failed to initialize: ' + (error instanceof Error ? error.message : 'Unknown error'));
    } finally {
      setLoading(false);
      setIsInitializing(false);
      initializingRef.current = false;
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

  // Listen for device responses (event-driven xpub and address handling)
  useEffect(() => {
    let unlistenResponse: Promise<() => void>;
    
    (async () => {
      unlistenResponse = listen('device:response', async (event: any) => {
        const tag = TAG + " | device:response | ";
        const { device_id, request_id, response } = event.payload;
        
        console.log(tag, `Received response for ${request_id}:`, response);
        
        // Handle xpub responses
        if (response && 'Xpub' in response) {
          const xpubResponse = response.Xpub;
          const requestKey = `${device_id}:${xpubResponse.path}`;
          
          // Remove from pending requests
          pendingXpubRequests.delete(requestKey);
          
          if (xpubResponse.success) {
            // Find the path info to get the CAIP
            const requiredPaths = WalletDatabase.getRequiredPaths();
            const pathInfo = requiredPaths.find(p => p.path === xpubResponse.path);
            
            if (pathInfo) {
              const xpubData = {
                path: xpubResponse.path,
                xpub: xpubResponse.xpub,
                caip: pathInfo.caip
              };
              
              setFetchedXpubs(prev => {
                // Avoid duplicates
                const exists = prev.some(x => x.path === xpubData.path && x.caip === xpubData.caip);
                if (exists) {
                  console.log(tag, `Duplicate xpub for ${xpubData.path}, skipping`);
                  return prev;
                }

                console.log(tag, `✅ Added xpub for ${xpubData.path}: ${xpubData.xpub.substring(0, 20)}...`);
                const newXpubs = [...prev, xpubData];
                console.log(tag, `Now have ${newXpubs.length} xpubs in memory:`, newXpubs.map(x => x.path));

                // Let useEffect handle portfolio refresh when fetchedXpubs updates
                const requiredPaths = WalletDatabase.getRequiredPaths();
                const expectedXpubCount = requiredPaths.length;
                if (newXpubs.length === expectedXpubCount) {
                  console.log(tag, `All expected xpubs (${expectedXpubCount}) are present. Portfolio will refresh via useEffect.`);
                } else {
                  console.log(tag, `Waiting for all xpubs: have ${newXpubs.length}, need ${expectedXpubCount}`);
                }

                return newXpubs;
              });
            }
          } else {
            console.error(tag, `❌ Xpub request failed for ${xpubResponse.path}:`, xpubResponse.error);
          }
        }
        
        // Handle address responses
        if (response && 'Address' in response) {
          const addressResponse = response.Address;
          console.log(tag, `📥 Address response:`, addressResponse);
          
          if (addressResponse.success) {
            console.log(tag, `✅ Address generated: ${addressResponse.address}`);
            // The address response will be caught by the polling in getReceiveAddress()
            // No additional handling needed here as the polling loop will pick it up
          } else {
            console.error(tag, `❌ Address request failed:`, addressResponse.error);
          }
        }
      });
    })();

    return () => {
      unlistenResponse?.then(fn => fn());
    };
  }, [refreshPortfolio]);

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

  // Watch fetchedXpubs and refresh portfolio when all expected xpubs are present
  useEffect(() => {
    const tag = TAG + " | fetchedXpubs useEffect | ";
    
    if (fetchedXpubs.length === 0) {
      console.log(tag, 'No xpubs in memory yet');
      return;
    }
    
    const requiredPaths = WalletDatabase.getRequiredPaths();
    const expectedXpubCount = requiredPaths.length;
    
    console.log(tag, `Current xpubs: ${fetchedXpubs.length}, expected: ${expectedXpubCount}`);
    
    if (fetchedXpubs.length === expectedXpubCount) {
      console.log(tag, `All ${expectedXpubCount} xpubs present, refreshing portfolio with current state`);
      refreshPortfolio();
    }
  }, [fetchedXpubs, refreshPortfolio]);

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