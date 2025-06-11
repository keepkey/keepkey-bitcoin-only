import React, { useState, useEffect } from 'react';
import {
  Box,
  Flex,
  Text,
  HStack,
  Button,
  VStack,
  Spinner,
  Grid,
  GridItem,
  Badge,
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

// Theme colors matching vault-ui
const theme = {
  bg: '#000000',
  cardBg: '#111111',
  gold: '#FFD700',
  goldHover: '#FFE135',
  border: '#222222',
};

export const Portfolio: React.FC = () => {
  const [dashboard, setDashboard] = useState<Dashboard | null>(null);
  const [networks, setNetworks] = useState<Network[]>([]);
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

  // Get network color for theming
  const getNetworkColor = (network: Network): string => {
    if (network.symbol === 'BTC' || network.network_name.toLowerCase().includes('bitcoin')) return '#f7931a';
    if (network.symbol === 'ETH' || network.is_evm) return '#627eea';
    if (network.symbol === 'LTC' || network.network_name.toLowerCase().includes('litecoin')) return '#bfbbbb';
    if (network.symbol === 'DOGE' || network.network_name.toLowerCase().includes('dogecoin')) return '#c2a633';
    return theme.gold;
  };

  // Fetch data
  const fetchData = async () => {
    console.log('📊 [Portfolio] Fetching data');
    setLoading(true);
    setError(null);
    
    try {
      const [dashboardData, networksData, balancesData] = await Promise.all([
        apiService.getDashboard(),
        apiService.getNetworks(),
        apiService.getBalances()
      ]);
      
      console.log('📊 [Portfolio] Data received:', { dashboardData, networksData, balancesData });
      setDashboard(dashboardData);
      setNetworks(networksData);
      setBalances(balancesData);
    } catch (err) {
      console.error('❌ [Portfolio] Error fetching data:', err);
      setError(err instanceof Error ? err.message : 'Failed to fetch data');
    } finally {
      setLoading(false);
    }
  };

  // Initial load
  useEffect(() => {
    fetchData();
  }, []);

  if (loading) {
    return (
      <Box height="100%" display="flex" alignItems="center" justifyContent="center">
        <VStack gap={4}>
          <Spinner size="xl" color="blue.400" />
          <Text color="white" fontSize="lg">Loading Portfolio...</Text>
        </VStack>
      </Box>
    );
  }

  if (error) {
    return (
      <Box height="100%" p={6}>
        <VStack justify="center" align="center" height="100%">
          <Text color="red.400" fontSize="lg">Error loading portfolio</Text>
          <Text color="gray.400" fontSize="sm">{error}</Text>
          <Button 
            colorScheme="blue" 
            onClick={fetchData}
            size="sm"
          >
            Retry
          </Button>
        </VStack>
      </Box>
    );
  }

  return (
    <Box height="100%" overflowY="auto" p={6}>
      <VStack gap={6} align="stretch">
        {/* Portfolio Overview */}
        <Box 
          p={6} 
          bg={theme.cardBg}
          borderRadius="xl" 
          border="1px solid" 
          borderColor={theme.border}
        >
          <VStack gap={4}>
            <Text fontSize="2xl" fontWeight="bold" color={theme.gold}>
              Portfolio Overview
            </Text>
            
            <Grid templateColumns="repeat(auto-fit, minmax(200px, 1fr))" gap={4} w="100%">
              <GridItem>
                <Box textAlign="center" p={4} bg="gray.800" borderRadius="md">
                  <Text fontSize="sm" color="gray.400">Total Value</Text>
                  <Text fontSize="2xl" fontWeight="bold" color="white">
                    ${dashboard ? formatUsd(dashboard.total_value_usd) : '0.00'}
                  </Text>
                </Box>
              </GridItem>
              
              <GridItem>
                <Box textAlign="center" p={4} bg="gray.800" borderRadius="md">
                  <Text fontSize="sm" color="gray.400">Networks</Text>
                  <Text fontSize="2xl" fontWeight="bold" color="white">
                    {networks.length}
                  </Text>
                </Box>
              </GridItem>
              
              <GridItem>
                <Box textAlign="center" p={4} bg="gray.800" borderRadius="md">
                  <Text fontSize="sm" color="gray.400">Assets</Text>
                  <Text fontSize="2xl" fontWeight="bold" color="white">
                    {balances.length}
                  </Text>
                </Box>
              </GridItem>
            </Grid>
          </VStack>
        </Box>

        {/* Networks List */}
        <Box>
          <HStack justify="space-between" mb={4}>
            <Text fontSize="lg" fontWeight="semibold" color="white">
              Your Networks
            </Text>
            <Badge colorScheme="blue" variant="subtle">
              {networks.length} networks
            </Badge>
          </HStack>
          
          <Grid templateColumns="repeat(auto-fill, minmax(280px, 1fr))" gap={4}>
            {networks.map((network) => (
              <GridItem key={network.id}>
                <Box
                  p={4}
                  bg={theme.cardBg}
                  borderRadius="md"
                  border="1px solid"
                  borderColor={theme.border}
                  _hover={{
                    borderColor: getNetworkColor(network),
                    transform: "translateY(-2px)",
                  }}
                  transition="all 0.2s"
                >
                  <VStack align="start" gap={2}>
                    <HStack justify="space-between" w="100%">
                      <Text fontWeight="semibold" color="white">
                        {network.network_name}
                      </Text>
                      <Badge 
                        colorScheme={network.is_evm ? "purple" : "gray"}
                        variant="subtle"
                      >
                        {network.symbol}
                      </Badge>
                    </HStack>
                    
                    <Text fontSize="sm" color="gray.400">
                      Chain ID: {network.chain_id_caip2}
                    </Text>
                    
                    {network.is_evm && (
                      <Badge colorScheme="purple" size="sm">
                        EVM Compatible
                      </Badge>
                    )}
                  </VStack>
                </Box>
              </GridItem>
            ))}
          </Grid>
          
          {networks.length === 0 && (
            <Box 
              p={8} 
              textAlign="center" 
              bg={theme.cardBg} 
              borderRadius="md" 
              border="1px dashed" 
              borderColor={theme.border}
            >
              <Text color="gray.400">No networks configured</Text>
              <Text fontSize="sm" color="gray.500" mt={2}>
                Connect your KeepKey device to get started
              </Text>
            </Box>
          )}
        </Box>

        {/* Balances List */}
        {balances.length > 0 && (
          <Box>
            <HStack justify="space-between" mb={4}>
              <Text fontSize="lg" fontWeight="semibold" color="white">
                Asset Balances
              </Text>
              <Badge colorScheme="green" variant="subtle">
                {balances.length} assets
              </Badge>
            </HStack>
            
            <VStack gap={2} align="stretch">
              {balances.map((balance, index) => (
                <Box
                  key={index}
                  p={4}
                  bg={theme.cardBg}
                  borderRadius="md"
                  border="1px solid"
                  borderColor={theme.border}
                >
                  <HStack justify="space-between">
                    <VStack align="start" gap={1}>
                      <Text fontWeight="semibold" color="white">
                        {balance.symbol}
                      </Text>
                      <Text fontSize="sm" color="gray.400">
                        {balance.network_id}
                      </Text>
                    </VStack>
                    
                    <VStack align="end" gap={1}>
                      <Text fontWeight="semibold" color="white">
                        {balance.balance}
                      </Text>
                      <Text fontSize="sm" color="gray.400">
                        ${formatUsd(balance.value_usd)}
                      </Text>
                    </VStack>
                  </HStack>
                </Box>
              ))}
            </VStack>
          </Box>
        )}
      </VStack>
    </Box>
  );
}; 