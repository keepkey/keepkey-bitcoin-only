import React from 'react';
import { 
  VStack, 
  Text, 
  Box, 
  HStack,
  Icon,
  Badge,
  Flex,
  Spacer
} from '@chakra-ui/react';
import { FaBitcoin, FaInfoCircle, FaCheckCircle } from 'react-icons/fa';
import { useSettings, BitcoinAddressType } from '../contexts/SettingsContext';
import { useTranslation } from 'react-i18next';

export const BitcoinSettings: React.FC = () => {
  const { t } = useTranslation(['settings']);
  const { bitcoinAddressType, setBitcoinAddressType } = useSettings();

  const addressTypeInfo = {
    'p2wpkh': {
      label: 'Native SegWit (Bech32)',
      description: 'Lowest fees, best efficiency. Addresses start with bc1',
      example: 'bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh',
      badge: 'Recommended',
      badgeColor: 'green'
    },
    'p2sh-p2wpkh': {
      label: 'SegWit (P2SH-wrapped)',
      description: 'Good compatibility, moderate fees. Addresses start with 3',
      example: '3J98t1WpEZ73CNmQviecrnyiWrnqRhWNLy',
      badge: 'Compatible',
      badgeColor: 'blue'
    },
    'p2pkh': {
      label: 'Legacy',
      description: 'Maximum compatibility, highest fees. Addresses start with 1',
      example: '1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa',
      badge: 'Legacy',
      badgeColor: 'gray'
    }
  };

  return (
    <VStack gap={6} align="stretch">
      <Box>
        <HStack mb={4}>
          <Icon as={FaBitcoin} color="orange.500" boxSize={5} />
          <Text fontSize="lg" fontWeight="bold" color="white">
            Bitcoin Address Type
          </Text>
        </HStack>
        
        <Text fontSize="sm" color="gray.400" mb={4}>
          Choose your preferred Bitcoin address type. This affects both change addresses and receive addresses.
        </Text>

        <VStack gap={3} align="stretch">
          {(Object.keys(addressTypeInfo) as BitcoinAddressType[]).map((type) => {
            const info = addressTypeInfo[type];
            const isSelected = bitcoinAddressType === type;
            
            return (
              <Box
                key={type}
                p={4}
                borderWidth={2}
                borderRadius="md"
                borderColor={isSelected ? 'blue.500' : 'gray.700'}
                bg={isSelected ? 'gray.800' : 'gray.900'}
                cursor="pointer"
                onClick={() => setBitcoinAddressType(type)}
                transition="all 0.2s"
                _hover={{ borderColor: 'blue.400', bg: 'gray.800' }}
                position="relative"
              >
                <Flex align="center" mb={2}>
                  <HStack gap={3}>
                    <Box
                      width={5}
                      height={5}
                      borderRadius="full"
                      borderWidth={2}
                      borderColor={isSelected ? 'blue.500' : 'gray.600'}
                      bg={isSelected ? 'blue.500' : 'transparent'}
                      display="flex"
                      alignItems="center"
                      justifyContent="center"
                    >
                      {isSelected && (
                        <Icon as={FaCheckCircle} color="white" boxSize={3} />
                      )}
                    </Box>
                    <Text fontWeight="semibold" color="white">{info.label}</Text>
                    {info.badge && (
                      <Badge colorScheme={info.badgeColor} fontSize="xs">
                        {info.badge}
                      </Badge>
                    )}
                  </HStack>
                  <Spacer />
                </Flex>
                
                <Text fontSize="sm" color="gray.400" ml={8} mb={2}>
                  {info.description}
                </Text>
                
                <Box ml={8}>
                  <Text fontSize="xs" color="gray.500" fontFamily="mono">
                    Example: {info.example}
                  </Text>
                </Box>
              </Box>
            );
          })}
        </VStack>

        <HStack mt={4} p={3} bg="blue.900" borderRadius="md" borderWidth={1} borderColor="blue.700">
          <Icon as={FaInfoCircle} color="blue.400" />
          <Text fontSize="sm" color="blue.300">
            Native SegWit offers the lowest transaction fees and is widely supported. Use Legacy only if required for compatibility with older services.
          </Text>
        </HStack>
      </Box>
    </VStack>
  );
};