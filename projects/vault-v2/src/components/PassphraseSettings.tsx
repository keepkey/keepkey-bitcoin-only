import React, { useState, useEffect } from 'react';
import {
  Box,
  Text,
  Flex,
  Spinner,
} from '@chakra-ui/react';
// Removed icon import to fix component errors
import { invoke } from '@tauri-apps/api/core';

interface PassphraseSettingsProps {
  deviceId: string;
  isPassphraseEnabled?: boolean;
  onPassphraseToggle?: (enabled: boolean) => void;
}

export const PassphraseSettings: React.FC<PassphraseSettingsProps> = ({
  deviceId,
  isPassphraseEnabled: initialEnabled = false,
  onPassphraseToggle,
}) => {
  const [isEnabled, setIsEnabled] = useState(initialEnabled);
  const [isUpdating, setIsUpdating] = useState(false);

  useEffect(() => {
    setIsEnabled(initialEnabled);
  }, [initialEnabled]);

  const handleTogglePassphrase = async () => {
    if (isUpdating) return;
    
    const newState = !isEnabled;
    setIsUpdating(true);
    
    try {
      await invoke('enable_passphrase_protection', {
        deviceId,
        enabled: newState,
      });
      
      setIsEnabled(newState);
      
      if (onPassphraseToggle) {
        onPassphraseToggle(newState);
      }

      console.log(`Passphrase protection has been ${newState ? 'enabled' : 'disabled'}`);
      
      // Restart the app to apply the new settings
      setTimeout(async () => {
        try {
          await invoke('restart_app');
        } catch (restartErr) {
          console.error('Failed to restart app:', restartErr);
          setIsUpdating(false);
        }
      }, 1000);
    } catch (err) {
      console.error('Failed to update passphrase protection:', err);
      
      console.error('Failed to update passphrase protection');
      setIsUpdating(false);
    }
  };

  return (
    <Box p={4} borderWidth={1} borderRadius="md">
      <Flex direction="column" gap={4}>
        <Text fontSize="lg" fontWeight="bold">
          Passphrase Protection
        </Text>
        
        <Flex align="center" justify="space-between">
          <Flex direction="column">
            <Text fontWeight="medium">
              BIP39 Passphrase
            </Text>
            <Text fontSize="sm" color="gray.600">
              Add an extra word to your recovery phrase for additional security
            </Text>
            {isEnabled && (
              <Text fontSize="xs" color="green.600" mt={1}>
                Currently enabled
              </Text>
            )}
          </Flex>
          
          <Flex align="center" gap={2}>
            {isUpdating && (
              <Spinner size="sm" color="blue.500" />
            )}
            <Box position="relative">
              <input
                type="checkbox"
                checked={isEnabled}
                onChange={handleTogglePassphrase}
                disabled={isUpdating}
                style={{ 
                  transform: 'scale(1.5)',
                  transition: 'all 0.2s ease-in-out',
                  opacity: isUpdating ? 0.6 : 1,
                  cursor: isUpdating ? 'not-allowed' : 'pointer'
                }}
              />
              {isUpdating && (
                <Box 
                  position="absolute" 
                  top="50%" 
                  left="50%" 
                  transform="translate(-50%, -50%)"
                  pointerEvents="none"
                >
                  <Text fontSize="xs" color="blue.500" fontWeight="bold">
                    {isEnabled ? 'Enabling...' : 'Disabling...'}
                  </Text>
                </Box>
              )}
            </Box>
          </Flex>
        </Flex>

        <Box borderTop="1px" borderColor="gray.200" my={4} />

        <Box p={3} bg="blue.50" borderRadius="md" borderLeft="4px" borderLeftColor="blue.400">
          <Text fontWeight="bold" fontSize="sm" mb={2}>
            About Passphrase Protection
          </Text>
          <Text fontSize="sm" mb={2}>
            When enabled, you'll be prompted for a passphrase to access your wallet. Different
            passphrases create completely different wallets, allowing for:
          </Text>
          <Box pl={4}>
            <Text fontSize="sm">• Hidden wallets for additional privacy</Text>
            <Text fontSize="sm">• Plausible deniability under duress</Text>
            <Text fontSize="sm">• Multiple wallet configurations from one seed</Text>
          </Box>
        </Box>

        {isUpdating && (
          <Box p={3} bg="blue.50" borderRadius="md" borderLeft="4px" borderLeftColor="blue.400">
            <Flex align="center" gap={2}>
              <Spinner size="sm" color="blue.500" />
              <Text fontWeight="bold" fontSize="sm">
                Updating passphrase protection settings...
              </Text>
            </Flex>
            <Text fontSize="sm" mt={2} color="blue.600">
              The application will restart automatically to apply changes.
            </Text>
          </Box>
        )}

        {isEnabled && !isUpdating && (
          <Box p={3} bg="orange.50" borderRadius="md" borderLeft="4px" borderLeftColor="orange.400">
            <Text fontWeight="bold" fontSize="sm" mb={2}>
              Important Security Notes
            </Text>
            <Text fontSize="sm" mb={1}>• Write down your passphrase separately from your recovery phrase</Text>
            <Text fontSize="sm" mb={1}>• Use a strong, memorable passphrase</Text>
            <Text fontSize="sm">• Your passphrase cannot be recovered if lost</Text>
          </Box>
        )}

        <Text fontSize="xs" color="gray.500" textAlign="center">
          Device: {deviceId.slice(-8)}
        </Text>
      </Flex>
    </Box>
  );
};