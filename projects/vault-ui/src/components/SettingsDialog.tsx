import React, { useState, useEffect } from 'react';
import {
  Box,
  VStack,
  HStack,
  Text,
  Button,
} from '@chakra-ui/react';
import Dialog from './ui/dialog';

interface SettingsDialogProps {
  isOpen: boolean;
  onClose: () => void;
  offlineMode: boolean;
  onOfflineModeChange: (enabled: boolean) => void;
}

// Theme colors to match the main app
const theme = {
  gold: '#FFD700',
  goldHover: '#FFE135',
  cardBg: '#111111',
  border: '#222222',
};

const SettingsDialog: React.FC<SettingsDialogProps> = ({
  isOpen,
  onClose,
  offlineMode,
  onOfflineModeChange,
}) => {
  const [tempOfflineMode, setTempOfflineMode] = useState(offlineMode);

  // Update local state when prop changes
  useEffect(() => {
    setTempOfflineMode(offlineMode);
  }, [offlineMode]);

  const handleSave = () => {
    onOfflineModeChange(tempOfflineMode);
    onClose();
  };

  const handleCancel = () => {
    setTempOfflineMode(offlineMode); // Reset to original value
    onClose();
  };

  const footer = (
    <HStack justify="flex-end" gap={3}>
      <Button
        variant="ghost"
        color="gray.400"
        _hover={{ color: 'white' }}
        onClick={handleCancel}
      >
        Cancel
      </Button>
      <Button
        bg={theme.gold}
        color="black"
        _hover={{ bg: theme.goldHover }}
        onClick={handleSave}
      >
        Save Settings
      </Button>
    </HStack>
  );

  return (
    <Dialog
      isOpen={isOpen}
      onClose={handleCancel}
      title="Vault Settings"
      footer={footer}
      size="md"
    >
      <VStack align="stretch" gap={6}>
        {/* Offline Signer Mode Section */}
        <Box>
          <VStack align="stretch" gap={4}>
            <Box>
              <Text fontSize="lg" fontWeight="bold" color="white" mb={2}>
                Security Mode
              </Text>
              <Text fontSize="sm" color="gray.400">
                Configure how the vault handles online features and balance data
              </Text>
            </Box>

            <Box height="1px" bg={theme.border} />

            <HStack justify="space-between" align="center" py={2}>
              <VStack align="flex-start" gap={1} flex="1">
                <Text fontWeight="medium" color="white">
                  Offline Signer Mode
                </Text>
                <Text fontSize="sm" color="gray.400">
                  {tempOfflineMode 
                    ? "Only show addresses and sign transactions. Hide all balances and online data."
                    : "Show balances, prices, and portfolio data from online sources."
                  }
                </Text>
              </VStack>
              
              {/* Custom toggle switch */}
              <Button
                onClick={() => setTempOfflineMode(!tempOfflineMode)}
                bg={tempOfflineMode ? theme.gold : 'gray.600'}
                color={tempOfflineMode ? 'black' : 'white'}
                size="sm"
                px={4}
                _hover={{
                  bg: tempOfflineMode ? theme.goldHover : 'gray.500'
                }}
              >
                {tempOfflineMode ? 'ON' : 'OFF'}
              </Button>
            </HStack>

            {tempOfflineMode && (
              <Box
                p={4}
                bg="orange.900"
                border="1px solid"
                borderColor="orange.700"
                borderRadius="md"
              >
                <Text fontSize="sm" color="orange.200">
                  <strong>Offline Mode Active:</strong> Balance data, prices, and portfolio 
                  overview will be hidden. Only addresses and transaction signing will be available.
                </Text>
              </Box>
            )}
          </VStack>
        </Box>

        {/* Future settings sections can be added here */}
        {/*
        <Box>
          <Text fontSize="lg" fontWeight="bold" color="white" mb={4}>
            Display Preferences
          </Text>
          // Add more settings here
        </Box>
        */}
      </VStack>
    </Dialog>
  );
};

export default SettingsDialog; 