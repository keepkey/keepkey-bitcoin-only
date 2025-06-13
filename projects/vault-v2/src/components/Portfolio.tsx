import React from 'react';
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

interface PortfolioProps {
  onNavigate?: (action: 'send' | 'receive') => void;
}

export const Portfolio: React.FC<PortfolioProps> = ({ onNavigate }) => {
  const { portfolio, loading, error, refreshPortfolio } = useWallet();

  // Format USD value
  const formatUsd = (value: number | string) => {
    const numValue = typeof value === 'string' ? parseFloat(value) : value;
    if (isNaN(numValue)) return '0.00';
    return numValue.toLocaleString('en-US', {
      minimumFractionDigits: 2,
      maximumFractionDigits: 2,
    });
  };

  // Debug function to inspect USD values
  const debugUsdValues = () => {
    console.group('🔍 DEBUG: USD Values Analysis');
    
    console.log('📊 Portfolio object:', portfolio);
    
    if (portfolio) {
      console.log('💰 Portfolio total_value_usd:', portfolio.total_value_usd, typeof portfolio.total_value_usd);
      
      if (portfolio.assets && portfolio.assets.length > 0) {
        console.log('💵 Individual asset USD values:');
        portfolio.assets.forEach((asset, index) => {
          console.log(`  [${index}] ${asset.symbol}: balance="${asset.balance}", value_usd=${asset.value_usd} (${typeof asset.value_usd})`);
        });
      }
    }
    
    console.groupEnd();
  };

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
        </VStack>
      </Box>
    );
  }

  if (!portfolio) {
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
          <Text color="gray.400" textAlign="center">No portfolio data available</Text>
          <Button 
            colorScheme="blue" 
            size="sm" 
            onClick={refreshPortfolio}
          >
            Load Portfolio
          </Button>
        </VStack>
      </Box>
    );
  }

  // Find BTC balance (sum all BTC balances if multiple, or 0 if none)
  const btcTotal = portfolio.assets
    .filter(asset => asset.symbol === 'BTC')
    .reduce((sum, asset) => sum + parseFloat(asset.balance), 0);

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
        <Box color="orange.400" fontSize="4xl" onClick={debugUsdValues}>
          <SiBitcoin />
        </Box>
        
        {/* USD Balance */}
        <Box textAlign="center">
          <Text fontSize="3xl" fontWeight="bold" color="white">
            ${formatUsd(portfolio.total_value_usd)}
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

      </VStack>
    </Flex>
  );
};