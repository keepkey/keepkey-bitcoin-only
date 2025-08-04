import React, { useEffect } from 'react';
import { VStack, Text, Box, Icon } from '@chakra-ui/react';
import { FaCog } from 'react-icons/fa';
import { invoke } from '@tauri-apps/api/core';
import type { StepProps } from '../BootloaderUpdateWizard';

export const Step1UpdateInProgress: React.FC<StepProps> = ({ 
  deviceId,
  onNext, 
  onError,
  onSetProgress,
  clearError
}) => {
  useEffect(() => {
    clearError(); // Clear previous errors when entering this step
    const performUpdate = async () => {
      try {
        // TODO: Replace with actual Tauri command to start bootloader update
        // For now, we simulate the update process.

        // Simulate the update process
        await new Promise(resolve => setTimeout(resolve, 15000)); // Simulate update time

        onNext(); // Move to completion step
      } catch (err: any) {
        console.error('Bootloader update failed:', err);
        const errorMessage = err.message || 'An unknown error occurred during the update.';
        const errorAdvice = 'Please ensure your device remained connected. You may need to unplug and replug your device, then try the update again. If the problem persists, contact support.';
        onError(errorMessage, errorAdvice);
      }
    };

    performUpdate();
  }, [deviceId, onNext, onError, clearError]);

  return (
    <VStack align="center" justify="center" gap={6} h="100%">
      <VStack align="center" gap={4}>
        <Text fontSize="2xl" fontWeight="bold" color="orange.500">
          Follow directions on device
        </Text>
        <Text fontSize="md" color="gray.400" textAlign="center">
          Your KeepKey will guide you through the update process.
        </Text>
      </VStack>
      
      <Box 
        borderWidth="1px" 
        borderColor="blue.500" 
        borderRadius="md" 
        p={4} 
        w="full" 
        bg="blue.900"
      >
        <Text fontSize="sm" color="gray.200">
          <Icon as={FaCog} mr={2} color="yellow.400" />
          <Text as="span" fontWeight="bold">Note:</Text> On the KeepKey, it will ask you to verify backup. 
          We will do this after updating - hold the button to skip for now to continue.
        </Text>
      </Box>
      
      <Box 
        borderWidth="1px" 
        borderColor="gray.700" 
        borderRadius="md" 
        p={4} 
        w="full" 
        bg="gray.750"
      >
        <Text fontSize="xs" color="yellow.400" fontWeight="bold">
          <Icon as={FaCog} mr={2} />
          DO NOT DISCONNECT YOUR DEVICE.
        </Text>
        <Text fontSize="xs" color="gray.300" mt={1}>
          The update process is critical. Disconnecting your KeepKey now may render it unusable.
        </Text>
      </Box>
    </VStack>
  );
};
