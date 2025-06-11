import { Box, VStack, Text, Spinner, Flex } from '@chakra-ui/react';
import { useState, useEffect } from 'react';
import { Portfolio } from '../Portfolio';

export const VaultView = () => {
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    // Simulate loading for now
    const timer = setTimeout(() => {
      setLoading(false);
    }, 1000);

    return () => clearTimeout(timer);
  }, []);

  if (loading) {
    return (
      <Box height="100%" bg="rgba(0, 0, 0, 0.4)" display="flex" alignItems="center" justifyContent="center">
        <VStack gap={4}>
          <Spinner size="xl" color="blue.400" />
          <Text color="white" fontSize="lg">Loading Vault...</Text>
        </VStack>
      </Box>
    );
  }

  return (
    <Box height="100%" bg="rgba(0, 0, 0, 0.4)">
      <Portfolio />
    </Box>
  );
}; 