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

export const Portfolio: React.FC = () => {
  const [dashboard, setDashboard] = useState<Dashboard | null>(null);
  const [balances, setBalances] = useState<Balance[]>([]);
  const [loading, setLoading] = useState(true);
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

  

  



  // Initial load
  useEffect(() => {
    // Only fetch dashboard and balances now
    const fetchSimpleData = async () => {
      setLoading(true);
      setError(null);
      try {
        const [dashboardData, balancesData] = await Promise.all([
          apiService.getDashboard(),
          apiService.getBalances()
        ]);
        setDashboard(dashboardData);
        setBalances(balancesData);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to fetch data');
      } finally {
        setLoading(false);
      }
    };
    fetchSimpleData();
  }, []);

  if (loading) {
    return (
      <Box height="100%" display="flex" alignItems="center" justifyContent="center" bg={theme.bg}>
        <VStack gap={4}>
          <Spinner size="xl" color="blue.400" />
          <Text color="gray.300" fontSize="lg">Loading Portfolio...</Text>
        </VStack>
      </Box>
    );
  }

  if (error) {
    return (
      <Box p={8} textAlign="center">
        <Text color="red.400">{error}</Text>
      </Box>
    );
  }

  // Find BTC balance (sum all BTC balances if multiple, or 0 if none)
  const btcTotal = balances
    .filter(b => b.symbol === 'BTC')
    .reduce((sum, b) => sum + parseFloat(b.balance), 0);

  return (
    <Flex align="center" justify="center" h="100%" bg={theme.bg}>
      <VStack gap={6} align="center" bg={theme.cardBg} p={10} borderRadius="xl" boxShadow="lg" minW="340px">
        {/* USD Balance */}
        <Box textAlign="center">
          <Text fontSize="3xl" fontWeight="bold" color="white">
            ${dashboard ? formatUsd(dashboard.total_value_usd) : '0.00'}
          </Text>
          <Text fontSize="md" color="gray.400" mt={1}>
            Total USD Value
          </Text>
        </Box>
        {/* BTC Balance */}
        <Box textAlign="center">
          <Text fontSize="2xl" fontWeight="semibold" color="orange.300">
            {btcTotal} BTC
          </Text>
          <Text fontSize="md" color="gray.400" mt={1}>
            Bitcoin Balance
          </Text>
        </Box>
        {/* Send/Receive Buttons */}
        <HStack gap={8} mt={4}>
          <Button colorScheme="blue" size="lg" fontWeight="bold">Send</Button>
          <Button colorScheme="green" size="lg" fontWeight="bold">Receive</Button>
        </HStack>
      </VStack>
    </Flex>
  );
};