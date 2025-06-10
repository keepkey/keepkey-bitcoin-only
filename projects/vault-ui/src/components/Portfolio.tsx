import React, { useState, useEffect } from 'react';
import {
  Box,
  Flex,
  Text,
  HStack,
  Button,
  VStack,
  Image,
} from '@chakra-ui/react';
import { Skeleton, SkeletonCircle } from "./ui/skeleton";
import { DonutChart, DonutChartItem, ChartLegend } from './chart';
import { apiService, Dashboard, Network, Balance } from '../services/api';
import SettingsDialog from './SettingsDialog';
import CountUp from 'react-countup';

// Theme colors
const theme = {
  bg: '#000000',
  cardBg: '#111111',
  gold: '#FFD700',
  goldHover: '#FFE135',
  border: '#222222',
};

// Custom scrollbar styles
const scrollbarStyles = {
  '&::-webkit-scrollbar': {
    width: '4px',
  },
  '&::-webkit-scrollbar-track': {
    width: '6px',
    background: 'transparent',
  },
  '&::-webkit-scrollbar-thumb': {
    background: '#4A5568',
    borderRadius: '24px',
  },
};

interface PortfolioProps {
  onSettingsClick?: () => void;
  onAddNetworkClick?: () => void;
}

const NetworkSkeleton = () => (
  <HStack gap="4" p={5} bg={theme.cardBg} borderRadius="2xl" boxShadow="lg">
    <SkeletonCircle size="12" />
    <Box flex="1">
      <Skeleton height="5" width="120px" mb={2} />
      <Skeleton height="4" width="80px" />
    </Box>
    <Box textAlign="right">
      <Skeleton height="5" width="70px" mb={2} />
      <Skeleton height="4" width="40px" />
    </Box>
  </HStack>
);

// Network colors for consistent theming
const getNetworkColor = (network: Network): string => {
  if (network.symbol === 'BTC' || network.network_name.toLowerCase().includes('bitcoin')) return '#f7931a';
  if (network.symbol === 'ETH' || network.is_evm) return '#627eea';
  if (network.symbol === 'LTC' || network.network_name.toLowerCase().includes('litecoin')) return '#bfbbbb';
  if (network.symbol === 'DOGE' || network.network_name.toLowerCase().includes('dogecoin')) return '#c2a633';
  if (network.network_name.toLowerCase().includes('cosmos')) return '#2e3148';
  if (network.network_name.toLowerCase().includes('maya')) return '#00d4ff';
  return theme.gold;
};

// Get coin icon URL from kkcli server
const getCoinIconUrl = (symbol: string): string => {
  return `http://localhost:1646/icons/${symbol.toLowerCase()}.png`;
};

// Component for displaying coin icon with fallback
const CoinIcon: React.FC<{ symbol: string; size: string; networkColor: string }> = ({ 
  symbol, 
  size, 
  networkColor 
}) => {
  const [imageError, setImageError] = useState(false);
  const iconUrl = getCoinIconUrl(symbol);

  if (imageError) {
    // Fallback to colored circle with first letter
    return (
      <Box
        w={size}
        h={size}
        borderRadius="full"
        bg={networkColor}
        display="flex"
        alignItems="center"
        justifyContent="center"
        fontSize={size === "48px" ? "lg" : "sm"}
        fontWeight="bold"
        color="white"
      >
        {symbol.charAt(0)}
      </Box>
    );
  }

  return (
    <Image
      src={iconUrl}
      alt={`${symbol} icon`}
      w={size}
      h={size}
      borderRadius="full"
      bg={networkColor}
      p="2px"
      onError={() => setImageError(true)}
    />
  );
};

const Portfolio: React.FC<PortfolioProps> = ({ 
  onSettingsClick = () => {}, 
  onAddNetworkClick = () => {} 
}) => {
  const [dashboard, setDashboard] = useState<Dashboard | null>(null);
  const [networks, setNetworks] = useState<Network[]>([]);
  const [balances, setBalances] = useState<Balance[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [activeSliceIndex, setActiveSliceIndex] = useState<number>(0);
  const [lastSync, setLastSync] = useState<number>(Date.now());
  const [offlineMode, setOfflineMode] = useState<boolean>(false);
  const [isSettingsOpen, setIsSettingsOpen] = useState(false);
  const [expandedNetworks, setExpandedNetworks] = useState<Set<number>>(new Set());

  // Format balance for display
  const formatBalance = (balance: string) => {
    try {
      const numericBalance = parseFloat(balance);
      const safeBalance = isNaN(numericBalance) ? '0' : balance;
      const [integer, decimal] = safeBalance.split('.');
      const largePart = decimal?.slice(0, 4) || '0000';
      const smallPart = decimal?.slice(4, 6) || '00';
      return { integer, largePart, smallPart };
    } catch (error) {
      console.error('Error in formatBalance:', error);
      return { integer: '0', largePart: '0000', smallPart: '00' };
    }
  };

  // Format USD value
  const formatUsd = (value: number | null | undefined) => {
    if (value === null || value === undefined || isNaN(value)) return '0.00';
    const numValue = typeof value === 'string' ? parseFloat(value) : value;
    if (isNaN(numValue)) return '0.00';
    return numValue.toLocaleString('en-US', {
      minimumFractionDigits: 2,
      maximumFractionDigits: 2,
    });
  };

  // Group balances by network
  const groupBalancesByNetwork = (networks: Network[], balances: Balance[]) => {
    const networkBalances = new Map<number, { network: Network; balances: Balance[]; totalValue: number }>();
    
    networks.forEach(network => {
      const networkBals = balances.filter(balance => {
        // Match by network name or CAIP identifier
        return balance.network_id === network.network_name || 
               balance.caip.includes(network.chain_id_caip2) ||
               (network.is_evm && balance.caip.startsWith('eip155:'));
      });
      
      const totalValue = networkBals.reduce((sum, bal) => sum + parseFloat(bal.value_usd.toString()), 0);
      
      networkBalances.set(network.id, {
        network,
        balances: networkBals,
        totalValue
      });
    });
    
    return Array.from(networkBalances.values()).sort((a, b) => b.totalValue - a.totalValue);
  };

  // Real chart data from balances
  const chartData: DonutChartItem[] = balances.length > 0 ? 
    balances.map(balance => ({
      name: balance.symbol,
      value: Math.max(parseFloat(balance.value_usd.toString()), 0.01), // Minimum value for visualization
      color: balance.symbol === 'BTC' ? '#f7931a' : 
             balance.symbol === 'ETH' ? '#627eea' : '#8247e5'
    })) : [
      // Default placeholder chart when no balances
      { name: 'BTC', value: 1, color: '#f7931a' },
      { name: 'ETH', value: 1, color: '#627eea' },
      { name: 'Others', value: 1, color: '#8247e5' }
    ];

  // Fetch dashboard data
  const fetchDashboard = async () => {
    console.log('📊 [Portfolio] Fetching dashboard data');
    setLoading(true);
    setError(null);
    
    try {
      const [dashboardData, networksData, balancesData] = await Promise.all([
        apiService.getDashboard(),
        apiService.getNetworks(),
        offlineMode ? Promise.resolve([]) : apiService.getBalances()
      ]);
      console.log('📊 [Portfolio] Dashboard data received:', dashboardData);
      console.log('📊 [Portfolio] Networks data received:', networksData);
      console.log('📊 [Portfolio] Balances data received:', balancesData);
      setDashboard(dashboardData);
      setNetworks(networksData);
      setBalances(balancesData);
    } catch (err) {
      console.error('❌ [Portfolio] Error fetching dashboard:', err);
      setError(err instanceof Error ? err.message : 'Failed to fetch dashboard data');
    } finally {
      setLoading(false);
    }
  };

  // Handle chart hover
  const handleHover = (index: number | null) => {
    if (index !== null) {
      setActiveSliceIndex(index);
    }
  };

  // Format value for chart display
  const formatValueForChart = (value: number) => {
    return formatUsd(value);
  };

  // Toggle network expansion
  const toggleNetworkExpansion = (networkId: number) => {
    const newExpanded = new Set(expandedNetworks);
    if (newExpanded.has(networkId)) {
      newExpanded.delete(networkId);
    } else {
      newExpanded.add(networkId);
    }
    setExpandedNetworks(newExpanded);
  };

  // Handle settings click
  const handleSettingsClick = () => {
    setIsSettingsOpen(true);
  };

  // Handle offline mode change
  const handleOfflineModeChange = (enabled: boolean) => {
    setOfflineMode(enabled);
    if (enabled) {
      setBalances([]); // Clear balances in offline mode
    } else {
      fetchDashboard(); // Refetch data when going back online
    }
  };

  // Initial load
  useEffect(() => {
    console.log('📊 [Portfolio] Component mounted');
    fetchDashboard();
    
    return () => console.log('📊 [Portfolio] Component unmounting');
  }, [offlineMode]);

  // Set up interval to sync market data every 30 seconds (only in online mode)
  useEffect(() => {
    if (offlineMode) return;
    
    const intervalId = setInterval(async () => {
      try {
        console.log("📊 [Portfolio] Syncing market data");
        await apiService.syncMarket();
        setLastSync(Date.now());
        fetchDashboard();
      } catch (error) {
        console.error("❌ [Portfolio] Error in syncMarket:", error);
      }
    }, 30000);

    return () => clearInterval(intervalId);
  }, [offlineMode]);

  if (error) {
    return (
      <Box height="100vh" bg={theme.bg} p={6}>
        <VStack justify="center" align="center" height="100%">
          <Text color="red.400" fontSize="lg">Error loading portfolio</Text>
          <Text color="gray.400" fontSize="sm">{error}</Text>
          <Button 
            colorScheme="teal" 
            onClick={fetchDashboard}
            size="sm"
          >
            Retry
          </Button>
        </VStack>
      </Box>
    );
  }

  const networkData = groupBalancesByNetwork(networks, balances);

  return (
    <Box height="100vh" bg={theme.bg}>
      {/* Header */}
      <Box 
        borderBottom="1px" 
        borderColor={theme.border}
        p={4}
        bg={theme.cardBg}
        backdropFilter="blur(10px)"
        position="relative"
        _after={{
          content: '""',
          position: "absolute",
          bottom: "-1px",
          left: "0",
          right: "0",
          height: "1px",
          background: `linear-gradient(90deg, transparent 0%, ${theme.gold}40 50%, transparent 100%)`,
        }}
      >
        <HStack justify="space-between" align="center">
          <HStack gap={3}>
            <Box w="24px" h="24px" bg={theme.gold} borderRadius="md" />
            <Text fontSize="lg" fontWeight="bold" color={theme.gold}>
              KeepKey Vault V2 {offlineMode && <Text as="span" fontSize="sm" color="orange.300">(Offline Mode)</Text>}
            </Text>
          </HStack>
          <Button
            size="sm"
            variant="ghost"
            color={theme.gold}
            _hover={{ color: theme.goldHover, bg: 'rgba(255, 215, 0, 0.1)' }}
            onClick={handleSettingsClick}
          >
            <HStack gap={2} align="center">
              <Text>Settings</Text>
              <Box 
                w="2px" 
                h="2px" 
                borderRadius="full" 
                bg={theme.gold} 
              />
            </HStack>
          </Button>
        </HStack>
      </Box>

      {/* Main Content */}
      <Box 
        height="calc(100% - 60px)" 
        overflowY="auto" 
        overflowX="hidden"
        p={6}
        css={scrollbarStyles}
      >
        <VStack gap={8} align="stretch">
          {/* Portfolio Overview with Chart - Hidden in offline mode */}
          {!offlineMode && (
            <Box 
              p={8} 
              borderRadius="2xl" 
              boxShadow={!loading && dashboard && chartData.length > 0 
                ? `0 4px 20px ${chartData[0].color}20, inset 0 0 20px ${chartData[0].color}10`
                : 'lg'
              }
              border="1px solid"
              borderColor={!loading && dashboard && chartData.length > 0 
                ? `${chartData[0].color}40`
                : theme.border
              }
              position="relative"
              overflow="hidden"
              bg={!loading && dashboard && chartData.length > 0 ? `${chartData[0].color}15` : theme.cardBg}
              _before={{
                content: '""',
                position: "absolute",
                top: 0,
                left: 0,
                right: 0,
                bottom: 0,
                background: !loading && dashboard && chartData.length > 0
                  ? `linear-gradient(135deg, ${chartData[0].color}40 0%, ${chartData[0].color}20 100%)`
                  : 'none',
                opacity: 0.6,
                zIndex: 0,
              }}
              _after={{
                content: '""',
                position: "absolute",
                top: "-50%",
                left: "-50%",
                right: "-50%",
                bottom: "-50%",
                background: "radial-gradient(circle, transparent 30%, rgba(0,0,0,0.8) 100%)",
                zIndex: 1,
              }}
            >
              <Flex 
                justify="center" 
                align="center" 
                position="relative" 
                zIndex={2}
                direction="column"
                gap={6}
                width="100%"
              >
                {loading ? (
                  <Flex direction="column" align="center" justify="center" py={6} width="100%">
                    <SkeletonCircle size="180px" />
                    <Skeleton height="4" width="140px" mt={4} />
                  </Flex>
                ) : dashboard ? (
                  <>
                    <Box textAlign="center" mb={4}>
                      <Text fontSize="3xl" fontWeight="bold" color="white">
                        ${formatUsd(parseFloat(dashboard.total_value_usd))}
                      </Text>
                      <Text fontSize="sm" color="gray.400">
                        Total Portfolio Value
                      </Text>
                      {parseFloat(dashboard.total_value_usd) === 0 && (
                        <Text fontSize="xs" color="gray.500" mt={2}>
                          Connect your wallet to view balances
                        </Text>
                      )}
                    </Box>
                    <Box 
                      width="100%"
                      maxWidth="210px"  
                      height="210px" 
                      mx="auto"
                      display="flex"
                      justifyContent="center"
                      alignItems="center"
                    >
                      <DonutChart 
                        data={chartData} 
                        formatValue={(value) => formatValueForChart(value)}
                        height={210}
                        width={210}
                        activeIndex={activeSliceIndex}
                        onHoverSlice={handleHover}
                      />
                    </Box>
                    <Box 
                      width="100%"
                      maxWidth="400px"
                      pt={2}
                      mt={1}
                      mx="auto"
                      height="40px"
                      display="flex"
                      alignItems="center"
                      justifyContent="center"
                      borderTop="1px solid"
                      borderColor="whiteAlpha.100"
                    >
                      <ChartLegend 
                        data={balances.length > 0 ? balances.map(balance => ({
                          name: balance.symbol,
                          value: parseFloat(balance.value_usd.toString()),
                          color: balance.symbol === 'BTC' ? '#f7931a' : 
                                 balance.symbol === 'ETH' ? '#627eea' : '#8247e5'
                        })) : chartData} 
                        total={parseFloat(dashboard?.total_value_usd || '0')}
                        formatValue={(value) => formatUsd(value)}
                        activeIndex={activeSliceIndex}
                        onHoverItem={handleHover}
                      />
                    </Box>
                  </>
                ) : (
                  <Text color={theme.gold}>No portfolio data available</Text>
                )}
              </Flex>
            </Box>
          )}

          {/* Network List */}
          <Box>
            <HStack justify="space-between" mb={5}>
              <HStack gap={2}>
                <Text fontSize="md" color="gray.400">
                  {offlineMode ? "Your Networks" : "Your Assets by Network"}
                </Text>
                <Text fontSize="xs" color="gray.600">
                  ({networks.length})
                </Text>
                {!offlineMode && networkData.length > 0 && networkData.every(n => n.totalValue === 0) && (
                  <Text fontSize="xs" color="orange.400">
                    • All networks have zero balance
                  </Text>
                )}
              </HStack>
              <Button
                size="xs"
                variant="ghost"
                color={theme.gold}
                _hover={{ color: theme.goldHover }}
                onClick={onAddNetworkClick}
              >
                Add Network
              </Button>
            </HStack>
            
            <VStack gap={4}>
              {loading || networks.length === 0 ? (
                <>
                  <NetworkSkeleton />
                  <NetworkSkeleton />
                  <NetworkSkeleton />
                </>
              ) : offlineMode ? (
                // Offline mode: Show networks with addresses only
                networks.map((network) => {
                  const networkColor = getNetworkColor(network);
                  const isExpanded = expandedNetworks.has(network.id);
                  
                  return (
                    <Box 
                      key={network.id}
                      w="100%"
                      bg={theme.cardBg}
                      borderRadius="2xl"
                      boxShadow="lg"
                      border="1px solid"
                      borderColor={theme.border}
                      transition="all 0.2s ease-in-out"
                      _hover={{
                        borderColor: networkColor,
                        boxShadow: `0 4px 12px ${networkColor}20`,
                        transform: 'translateY(-2px)',
                      }}
                    >
                      <Box 
                        p={5}
                        cursor="pointer"
                        onClick={() => toggleNetworkExpansion(network.id)}
                      >
                        <HStack justify="space-between" align="center">
                          <HStack gap={4}>
                            <CoinIcon 
                              symbol={network.symbol} 
                              size="48px" 
                              networkColor={networkColor} 
                            />
                            <Box>
                              <Text fontSize="md" fontWeight="bold" color="white">
                                {network.display_name}
                              </Text>
                              <Text fontSize="sm" color="gray.400">
                                {network.network_name} • {network.symbol}
                              </Text>
                            </Box>
                          </HStack>
                          
                          <VStack align="flex-end" gap={1}>
                            <Text fontSize="sm" color="gray.400">
                              Offline Mode
                            </Text>
                            <Text fontSize="lg" color={theme.gold}>
                              {isExpanded ? '▼' : '▶'}
                            </Text>
                          </VStack>
                        </HStack>
                      </Box>
                      
                      {isExpanded && (
                        <Box px={5} pb={5} borderTop="1px solid" borderColor={theme.border}>
                          <Text fontSize="sm" color="gray.400" mb={3}>
                            Addresses on this network will be shown here when available
                          </Text>
                        </Box>
                      )}
                    </Box>
                  );
                })
              ) : (
                // Online mode: Show networks with balances
                networkData.map(({ network, balances: networkBalances, totalValue }) => {
                  const networkColor = getNetworkColor(network);
                  const isExpanded = expandedNetworks.has(network.id);
                  const totalPortfolioValue = parseFloat(dashboard?.total_value_usd || '0');
                  const percentage = totalPortfolioValue > 0 ? (totalValue / totalPortfolioValue) * 100 : 0;

                  return (
                    <Box 
                      key={network.id}
                      w="100%"
                      bg={theme.cardBg}
                      borderRadius="2xl"
                      boxShadow="lg"
                      border="1px solid"
                      borderColor={theme.border}
                      transition="all 0.2s ease-in-out"
                      opacity={totalValue === 0 ? 0.6 : 1}
                      _hover={{
                        borderColor: networkColor,
                        boxShadow: `0 4px 12px ${networkColor}20`,
                        transform: 'translateY(-2px)',
                        opacity: 1,
                      }}
                    >
                      <Box 
                        p={5}
                        cursor="pointer"
                        onClick={() => toggleNetworkExpansion(network.id)}
                      >
                        <HStack justify="space-between" align="center">
                          <HStack gap={4}>
                            <CoinIcon 
                              symbol={network.symbol} 
                              size="48px" 
                              networkColor={networkColor} 
                            />
                            <Box>
                              <Text fontSize="md" fontWeight="bold" color="white">
                                {network.display_name}
                              </Text>
                              <Text fontSize="sm" color="gray.400">
                                {percentage.toFixed(1)}% of portfolio • {networkBalances.length} assets
                              </Text>
                            </Box>
                          </HStack>
                          
                          <VStack align="flex-end" gap={1}>
                            <Text fontSize="lg" fontWeight="bold" color="white">
                              ${formatUsd(totalValue)}
                            </Text>
                            <Text fontSize="lg" color={theme.gold}>
                              {isExpanded ? '▼' : '▶'}
                            </Text>
                          </VStack>
                        </HStack>
                      </Box>
                      
                                             {isExpanded && (
                         <Box px={5} pb={5} borderTop="1px solid" borderColor={theme.border}>
                           <Text fontSize="sm" color="gray.400" mb={3}>
                             Assets on {network.display_name}:
                           </Text>
                           <VStack gap={2} align="stretch">
                             {networkBalances.map((balance) => {
                               const { integer, largePart } = formatBalance(balance.balance);
                               return (
                                 <HStack 
                                   key={balance.caip}
                                   justify="space-between" 
                                   p={3}
                                   bg="rgba(255, 255, 255, 0.05)"
                                   borderRadius="md"
                                 >
                                   <HStack gap={3}>
                                     <CoinIcon 
                                       symbol={balance.symbol} 
                                       size="32px" 
                                       networkColor={networkColor} 
                                     />
                                     <VStack align="flex-start" gap={0}>
                                       <Text fontSize="sm" fontWeight="medium" color="white">
                                         {balance.symbol}
                                       </Text>
                                       <Text fontSize="xs" color="gray.500" fontFamily="mono">
                                         {balance.pubkey.slice(0, 10)}...{balance.pubkey.slice(-6)}
                                       </Text>
                                     </VStack>
                                   </HStack>
                                   <VStack align="flex-end" gap={0}>
                                     <Text fontSize="sm" fontWeight="medium" color="white">
                                       ${formatUsd(balance.value_usd)}
                                     </Text>
                                     <Text fontSize="xs" color="gray.400">
                                       {integer}.{largePart} {balance.symbol}
                                     </Text>
                                   </VStack>
                                 </HStack>
                               );
                             })}
                           </VStack>
                         </Box>
                       )}
                    </Box>
                  );
                })
              )}
            </VStack>
          </Box>
        </VStack>
      </Box>

      {/* Settings Dialog */}
      <SettingsDialog
        isOpen={isSettingsOpen}
        onClose={() => setIsSettingsOpen(false)}
        offlineMode={offlineMode}
        onOfflineModeChange={handleOfflineModeChange}
      />
    </Box>
  );
};

export default Portfolio; 