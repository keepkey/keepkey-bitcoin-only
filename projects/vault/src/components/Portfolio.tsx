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

// Types (we'll need to create these or import from the backend)
interface Dashboard {
  total_value_usd: string;
  network_count: number;
  balance_count: number;
}

interface Network {
  id: number;
  network_name: string;
  symbol: string;
  chain_id_caip2: string;
  is_evm: boolean;
}

interface Balance {
  network_id: string;
  symbol: string;
  balance: string;
  value_usd: number;
  caip: string;
}

// Simple API service
const apiService = {
  async syncDevice(): Promise<{ success: boolean; device_id?: string; balances_cached?: number; message?: string }> {
    try {
      const response = await fetch('http://localhost:1646/api/v2/sync-device', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' }
      });
      if (!response.ok) throw new Error('Failed to sync device');
      return await response.json();
    } catch (error) {
      console.error('Sync Error:', error);
      throw error;
    }
  },

  async getDashboard(): Promise<Dashboard> {
    try {
      const response = await fetch('http://localhost:1646/api/v2/portfolio/summary');
      if (!response.ok) throw new Error('Failed to fetch dashboard');
      return await response.json();
    } catch (error) {
      console.error('API Error:', error);
      // Return mock data for now
      return {
        total_value_usd: '0.00',
        network_count: 0,
        balance_count: 0
      };
    }
  },

  async getNetworks(): Promise<Network[]> {
    try {
      const response = await fetch('http://localhost:1646/api/v2/networks');
      if (!response.ok) throw new Error('Failed to fetch networks');
      return await response.json();
    } catch (error) {
      console.error('API Error:', error);
      return [];
    }
  },

  async getBalances(): Promise<Balance[]> {
    try {
      const response = await fetch('http://localhost:1646/api/v2/balances');
      if (!response.ok) throw new Error('Failed to fetch balances');
      return await response.json();
    } catch (error) {
      console.error('API Error:', error);
      return [];
    }
  }
};

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
  const [dashboard, setDashboard] = useState<Dashboard | null>(null);
  const [balances, setBalances] = useState<Balance[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [syncStatus, setSyncStatus] = useState<string | null>(null);

  // Format USD value
  const formatUsd = (value: number | string) => {
    const numValue = typeof value === 'string' ? parseFloat(value) : value;
    if (isNaN(numValue)) return '0.00';
    return numValue.toLocaleString('en-US', {
      minimumFractionDigits: 2,
      maximumFractionDigits: 2,
    });
  };

  // Calculate total USD value from balances (bypass stale cache)
  const calculateTotalUsd = () => {
    if (!balances || balances.length === 0) return 0;
    
    const total = balances.reduce((sum, balance) => {
      const usdValue = parseFloat(String(balance.value_usd || '0'));
      return sum + usdValue;
    }, 0);
    
    console.log(`💰 Calculated total USD from ${balances.length} balances: $${total.toFixed(2)}`);
    return total;
  };

  // Debug function to inspect USD values
  const debugUsdValues = () => {
    console.group('🔍 DEBUG: USD Values Analysis');
    
    console.log('📊 Dashboard object:', dashboard);
    console.log('📈 Balances array:', balances);
    
    if (dashboard) {
      console.log('💰 Dashboard total_value_usd:', dashboard.total_value_usd, typeof dashboard.total_value_usd);
    }
    
    if (balances && balances.length > 0) {
      console.log('💵 Individual balance USD values:');
      balances.forEach((balance, index) => {
        console.log(`  [${index}] ${balance.symbol}: balance="${balance.balance}", value_usd=${balance.value_usd} (${typeof balance.value_usd})`);
      });
      
      const totalFromBalances = calculateTotalUsd();
      console.log('🧮 Calculated total from balances:', totalFromBalances);
    }
    
    console.groupEnd();
  };

  // Load portfolio data from API
  const loadPortfolioData = async () => {
    console.log('📊 Portfolio: Loading data from API...');
    
    try {
      const [dashboardData, balancesData] = await Promise.all([
        apiService.getDashboard(),
        apiService.getBalances()
      ]);
      
      console.log('📊 Portfolio: Dashboard data:', JSON.stringify(dashboardData, null, 2));
      console.log('📊 Portfolio: Balances data:', JSON.stringify(balancesData, null, 2));
      
      setDashboard(dashboardData);
      setBalances(balancesData);
      setError(null);
      
      console.log('✅ Portfolio: Data refreshed successfully');
      
      // Debug USD values after setting data
      setTimeout(debugUsdValues, 100);
    } catch (err) {
      console.error('❌ Portfolio: Failed to load data:', err);
      setError(`Failed to load portfolio data: ${err instanceof Error ? err.message : 'Unknown error'}`);
    }
  };

  // Sync device and refresh data
  const syncDevice = async () => {
    console.log('🔄 Portfolio: Starting device sync...');
    try {
      const syncResult = await apiService.syncDevice();
      console.log('✅ Portfolio: Device sync result:', JSON.stringify(syncResult, null, 2));
      
      console.log('🔄 Portfolio: Refreshing portfolio data...');
      await loadPortfolioData();
      
      return { success: true };
    } catch (error) {
      console.error('❌ Portfolio: Sync failed:', error);
      setError(`Sync failed: ${error instanceof Error ? error.message : 'Unknown error'}`);
      return { success: false, error: error instanceof Error ? error.message : 'Unknown error' };
    }
  };

  // Initial load with automatic sync
  useEffect(() => {
    const initializePortfolio = async () => {
      setLoading(true);
      setError(null);
      
      try {
        console.log('🏁 Portfolio: Initializing with auto-sync...');
        
        // First try to sync device to ensure fresh data
        await syncDevice();
        
        // If sync fails, still try to load existing data
        const [dashboardData, balancesData] = await Promise.all([
          apiService.getDashboard(),
          apiService.getBalances()
        ]);
        
        setDashboard(dashboardData);
        setBalances(balancesData);
        
      } catch (err) {
        console.error('❌ Portfolio: Initialization failed:', err);
        setError(err instanceof Error ? err.message : 'Failed to initialize portfolio');
      } finally {
        setLoading(false);
      }
    };
    
    initializePortfolio();
  }, []);

  if (loading) {
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
          <Text color="gray.300" fontSize="lg">Loading Portfolio...</Text>
          {syncStatus && (
            <Text color="gray.400" fontSize="sm">{syncStatus}</Text>
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
            onClick={() => window.location.reload()}
          >
            Retry
          </Button>
        </VStack>
      </Box>
    );
  }

  // Find BTC balance (sum all BTC balances if multiple, or 0 if none)
  const btcTotal = balances
    .filter(b => b.symbol === 'BTC')
    .reduce((sum, b) => sum + parseFloat(b.balance), 0);

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
            ${formatUsd(calculateTotalUsd())}
          </Text>
          <Text fontSize="md" color="gray.400" mt={1}>
            Total USD Value
          </Text>
        </Box>
        
        {/* BTC Balance */}
        <Box textAlign="center">
          <Text fontSize="2xl" fontWeight="semibold" color="orange.300">
            {btcTotal.toFixed(8)} BTC
          </Text>
          <Text fontSize="md" color="gray.400" mt={1}>
            Bitcoin Balance
          </Text>
        </Box>
        
        {/* Sync Status */}
        {syncStatus && (
          <Box textAlign="center">
            <Text fontSize="sm" color="blue.300">
              {syncStatus}
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
        
        {/* Debug Button - Remove this in production */}
        <Button 
          size="sm" 
          colorScheme="gray" 
          variant="outline"
          onClick={debugUsdValues}
          mt={2}
        >
          🔍 Debug USD Values
        </Button>
      </VStack>
    </Flex>
  );
};