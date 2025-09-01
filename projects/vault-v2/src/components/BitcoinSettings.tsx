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
  const { t } = useTranslation(['settings', 'common']);
  const { bitcoinAddressType, setBitcoinAddressType } = useSettings();

  const addressTypeInfo = {
    'p2wpkh': {
      label: t('bitcoin.addressType.nativeSegwit.title'),
      description: t('bitcoin.addressType.nativeSegwit.description'),
      example: t('bitcoin.addressType.nativeSegwit.example'),
      badge: t('recommended', { ns: 'common' }),
      badgeColor: 'green'
    },
    'p2sh-p2wpkh': {
      label: t('bitcoin.addressType.segwit.title'),
      description: t('bitcoin.addressType.segwit.description'),
      example: t('bitcoin.addressType.segwit.example'),
      badge: t('compatible', { ns: 'common' }),
      badgeColor: 'blue'
    },
    'p2pkh': {
      label: t('bitcoin.addressType.legacy.title'),
      description: t('bitcoin.addressType.legacy.description'),
      example: t('bitcoin.addressType.legacy.example'),
      badge: t('legacy', { ns: 'common' }),
      badgeColor: 'gray'
    }
  };

  return (
    <VStack gap={6} align="stretch">
      <Box>
        <HStack mb={4}>
          <Icon as={FaBitcoin} color="orange.500" boxSize={5} />
          <Text fontSize="lg" fontWeight="bold" color="white">
            {t('bitcoin.addressType.label')}
          </Text>
        </HStack>
        
        <Text fontSize="sm" color="gray.400" mb={4}>
          {t('bitcoin.addressType.description')}
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
            {t('bitcoin.addressType.recommendation')}
          </Text>
        </HStack>
      </Box>
    </VStack>
  );
};