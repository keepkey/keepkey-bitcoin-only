import { Box, Text, HStack } from '@chakra-ui/react';
import { keyframes } from '@emotion/react';
import { SiBitcoin } from 'react-icons/si';
import { Portfolio } from '../Portfolio';
import { KeepKeyUILogo } from '../logo/keepkey-ui';

interface VaultViewProps {
  onNavigate?: (action: 'send' | 'receive') => void;
}

// Animation for the KeepKey logo
const pulseGlow = keyframes`
  0% { 
    opacity: 0.6;
    transform: scale(1);
    filter: drop-shadow(0 0 5px rgba(255, 255, 255, 0.3));
  }
  50% { 
    opacity: 0.9;
    transform: scale(1.05);
    filter: drop-shadow(0 0 10px rgba(255, 255, 255, 0.5));
  }
  100% { 
    opacity: 0.6;
    transform: scale(1);
    filter: drop-shadow(0 0 5px rgba(255, 255, 255, 0.3));
  }
`;

export const VaultView = ({ onNavigate }: VaultViewProps) => {
  return (
    <Box height="100%" position="relative">
      {/* Bitcoin CAIP Info - Top Right */}
      <Box
        position="absolute"
        top={3}
        right={3}
        zIndex={10}
        bg="rgba(0, 0, 0, 0.8)"
        borderRadius="md"
        px={2}
        py={1}
        backdropFilter="blur(10px)"
        border="1px solid rgba(255, 255, 255, 0.1)"
      >
        <HStack gap={1} align="center">
          <Box color="orange.400" fontSize="xs">
            <SiBitcoin />
          </Box>
          <Text fontSize="xs" color="gray.300" fontFamily="mono" fontWeight="bold">
            bip122:000000000019d6689c085ae165831e93
          </Text>
          <Text fontSize="xs" color="gray.400">
            (bitcoin only)
          </Text>
        </HStack>
      </Box>

      {/* KeepKey Logo Animation - Bottom Left */}
      <Box
        position="absolute"
        bottom={2}
        left={2}
        zIndex={10}
        width="50px"
        height="50px"
        animation={`${pulseGlow} 3s ease-in-out infinite`}
      >
        <KeepKeyUILogo />
      </Box>

      {/* Main Content - Portfolio with transparent background */}
      <Box height="100%" bg="transparent">
        <Portfolio onNavigate={onNavigate} />
      </Box>
    </Box>
  );
}; 