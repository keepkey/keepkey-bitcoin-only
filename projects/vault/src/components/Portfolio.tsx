import React, { useState, useEffect } from 'react';
import {
  Box,
  Flex,
  Text,
  HStack,
  Button,
  VStack,
  Spinner
} from '@chakra-ui/react';
import { FaPaperPlane, FaDownload } from 'react-icons/fa';
import { SiBitcoin } from 'react-icons/si';
import { useWallet } from '../contexts/WalletContext';
import { invoke } from '@tauri-apps/api/core';

// Theme colors - dark theme for crypto app
const theme = {
  bg: 'gray.900',
  cardBg: 'gray.800',
  gold: '#FFD700',
  goldHover: '#FFE135',
  border: 'gray.700',
};

interface PortfolioProps {
  onNavigate?: (action: 'send' | 'receive') => void;
}

export const Portfolio: React.FC<PortfolioProps> = ({ onNavigate }) => {
  const {
    portfolio,
    summary,
    isLoading,
    isSyncing,
    syncProgress,
    lastSync,
    refreshPortfolio,
    getTotalBalance,
    extractXpubsFromCache,
  } = useWallet();

  const [error, setError] = useState<string | null>(null);

  // Format USD value
  const formatUsd = (value: number | string) => {
    const numValue = typeof value === 'string' ? parseFloat(value) : value;
    if (isNaN(numValue)) return '0.00';
    return numValue.toLocaleString('en-US', {
      minimumFractionDigits: 2,
      maximumFractionDigits: 2,
    });
  };

  // Get totals from wallet context
  const totals = getTotalBalance();

  // Refresh portfolio data
  const handleRefresh = async () => {
    console.log('🔄 Portfolio: Refreshing data...');
    setError(null);
    
    try {
      await refreshPortfolio(true); // Force refresh
      console.log('✅ Portfolio: Data refreshed successfully');
    } catch (err) {
      console.error('❌ Portfolio: Failed to refresh:', err);
      setError(`Failed to refresh portfolio: ${err instanceof Error ? err.message : 'Unknown error'}`);
    }
  };

  // Manual xpub extraction for debugging
  const handleExtractXpubs = async () => {
    console.log('🔄 Portfolio: Manually extracting xpubs...');
    setError(null);
    
    try {
      // Get connected devices
      const devices = await invoke<any[]>('get_connected_devices');
      if (devices.length > 0) {
        const firstDevice = devices[0];
        if (firstDevice?.device?.unique_id) {
          console.log('📡 Extracting xpubs for device:', firstDevice.device.unique_id);
          await extractXpubsFromCache(firstDevice.device.unique_id);
          console.log('✅ Manual xpub extraction completed');
        } else {
          setError('No device ID found');
        }
      } else {
        setError('No connected devices found');
      }
    } catch (err) {
      console.error('❌ Portfolio: Failed to extract xpubs:', err);
      setError(`Failed to extract xpubs: ${err instanceof Error ? err.message : 'Unknown error'}`);
    }
  };

  // Debug function to inspect values
  const debugValues = () => {
    console.group('🔍 DEBUG: Portfolio Values Analysis');
    console.log('📊 Summary:', summary);
    console.log('📈 Portfolio cache:', portfolio);
    console.log('💰 Calculated totals:', totals);
    console.log('⏰ Last sync:', lastSync ? new Date(lastSync * 1000).toLocaleString() : 'Never');
    console.groupEnd();
  };

  // Auto-refresh on mount if data is stale
  useEffect(() => {
    const now = Math.floor(Date.now() / 1000);
    const shouldRefresh = !lastSync || (now - lastSync > 600); // 10 minutes

    if (shouldRefresh && !isLoading) {
      console.log('📊 Portfolio: Auto-refreshing stale data...');
      handleRefresh();
    }
  }, []);

  if (isLoading || isSyncing) {
    return (
      <Box height="100%" display="flex" alignItems="center" justifyContent="center" bg="transparent">
        <VStack 
          gap={4}
          bg="rgba(26, 32, 44, 0.9)" 
          p={8} 
          borderRadius="xl" 
          backdropFilter="blur(20px)"
          border="1px solid rgba(255, 255, 255, 0.1)"
        >
          <Spinner size="xl" color="blue.400" />
          <Text color="gray.300" fontSize="lg">
            {isSyncing ? 'Syncing Wallet...' : 'Loading Portfolio...'}
          </Text>
          {syncProgress && (
            <Text color="gray.400" fontSize="sm">
              {syncProgress.label}: {syncProgress.status}
            </Text>
          )}
        </VStack>
      </Box>
    );
  }

  if (error) {
    return (
      <Box height="100%" display="flex" alignItems="center" justifyContent="center" bg="transparent">
        <VStack 
          gap={4}
          bg="rgba(26, 32, 44, 0.9)" 
          p={8} 
          borderRadius="xl" 
          backdropFilter="blur(20px)"
          border="1px solid rgba(255, 255, 255, 0.1)"
        >
          <Text color="red.400" textAlign="center">{error}</Text>
          <Button 
            colorScheme="blue" 
            size="sm" 
            onClick={handleRefresh}
          >
            Retry
          </Button>
        </VStack>
      </Box>
    );
  }

  return (
    <Flex align="center" justify="center" h="100%" bg="transparent">
      <VStack 
        gap={6} 
        align="center" 
        bg="rgba(26, 32, 44, 0.9)" 
        p={10} 
        borderRadius="xl" 
        boxShadow="2xl" 
        minW="340px"
        backdropFilter="blur(20px)"
        border="1px solid rgba(255, 255, 255, 0.1)"
      >
        {/* Bitcoin Logo */}
        <Box color="orange.400" fontSize="4xl">
          <SiBitcoin />
        </Box>
        
        {/* USD Balance */}
        <Box textAlign="center">
          <Text fontSize="3xl" fontWeight="bold" color="white">
            ${formatUsd(totals.usd)}
          </Text>
          <Text fontSize="md" color="gray.400" mt={1}>
            Total USD Value
          </Text>
        </Box>
        
        {/* BTC Balance */}
        <Box textAlign="center">
          <Text fontSize="2xl" fontWeight="semibold" color="orange.300">
            {totals.btc.toFixed(8)} BTC
          </Text>
          <Text fontSize="md" color="gray.400" mt={1}>
            Bitcoin Balance
          </Text>
        </Box>
        
        {/* Last Sync Info */}
        {lastSync && (
          <Box textAlign="center">
            <Text fontSize="sm" color="gray.500">
              Last updated: {new Date(lastSync * 1000).toLocaleTimeString()}
            </Text>
          </Box>
        )}
        
        {/* Send/Receive Buttons */}
        <HStack gap={8} mt={4}>
          <Button 
            colorScheme="blue" 
            size="lg" 
            fontWeight="bold"
            onClick={() => onNavigate?.('send')}
            minW="120px"
          >
            <HStack gap={2}>
              <FaPaperPlane />
              <Text>Send</Text>
            </HStack>
          </Button>
          <Button 
            colorScheme="green" 
            size="lg" 
            fontWeight="bold"
            onClick={() => onNavigate?.('receive')}
            minW="120px"
          >
            <HStack gap={2}>
              <FaDownload />
              <Text>Receive</Text>
            </HStack>
          </Button>
        </HStack>
        
        {/* Refresh Button */}
        <Button 
          size="sm" 
          colorScheme="blue" 
          variant="outline"
          onClick={handleRefresh}
          loading={isLoading}
          mt={2}
        >
          🔄 Refresh
        </Button>

        {/* Debug Button - Remove this in production */}
        <Button 
          size="sm" 
          colorScheme="gray" 
          variant="outline"
          onClick={debugValues}
          mt={1}
        >
          🔍 Debug Values
        </Button>
      </VStack>
    </Flex>
  );
};