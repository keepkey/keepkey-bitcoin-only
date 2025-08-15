import React, { useState, useEffect } from 'react';
import {
  Box,
  Text,
  Flex,
  Spinner,
  useDisclosure,
} from '@chakra-ui/react';
// Removed icon import to fix component errors
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { usePassphraseDialog } from '../contexts/DialogContext';

interface PassphraseSettingsProps {
  deviceId: string;
  isPassphraseEnabled?: boolean;
  onPassphraseToggle?: (enabled: boolean) => void;
}

interface PassphraseRequestPayload {
  requestId: string;
  deviceId: string;
}

export const PassphraseSettings: React.FC<PassphraseSettingsProps> = ({
  deviceId,
  isPassphraseEnabled: initialEnabled = false,
  onPassphraseToggle,
}) => {
  const [isEnabled, setIsEnabled] = useState(initialEnabled);
  const [isUpdating, setIsUpdating] = useState(false);
  const passphraseDialog = usePassphraseDialog();
  const [pendingEnable, setPendingEnable] = useState(false);
  const [statusMessage, setStatusMessage] = useState<string | null>(null);

  useEffect(() => {
    setIsEnabled(initialEnabled);
  }, [initialEnabled]);

  // Listen for passphrase requests from the device during enable/disable flow
  useEffect(() => {
    const unlisten = listen<PassphraseRequestPayload>('passphrase_request', (event) => {
      console.log('Passphrase request received during settings update:', event.payload);
      
      // If we're in the middle of enabling passphrase protection and device requests passphrase
      if (pendingEnable && event.payload.deviceId === deviceId) {
        // Open the passphrase entry modal
        passphraseDialog.show({
          deviceId: deviceId,
          onSubmit: () => {
            console.log('Passphrase submitted successfully');
            
            // Complete the passphrase enable flow
            if (pendingEnable) {
              setIsEnabled(true);
              setPendingEnable(false);
              
              if (onPassphraseToggle) {
                onPassphraseToggle(true);
              }
              
              setStatusMessage('Passphrase protection enabled - restarting app...');
              
              // Refresh device features and restart app after passphrase is enabled
              setTimeout(async () => {
                try {
                  await invoke('refresh_device_features', { deviceId });
                  
                  // Restart the app since device now requires passphrase for all operations
                  console.log('Restarting app after enabling passphrase protection');
                  setTimeout(() => {
                    window.location.reload();
                  }, 1000);
                } catch (err) {
                  console.error('Failed to refresh device features:', err);
                  // Still restart even if refresh fails
                  setTimeout(() => {
                    window.location.reload();
                  }, 1000);
                }
              }, 1000);
            }
            
            setIsUpdating(false);
          },
          onDialogClose: () => {
            console.log('Passphrase dialog closed');
            setPendingEnable(false);
            setIsUpdating(false);
          }
        });
      }
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, [pendingEnable, deviceId, passphraseDialog]);

  const handleTogglePassphrase = async () => {
    if (isUpdating) return;
    
    const newState = !isEnabled;
    setIsUpdating(true);
    
    if (newState) {
      // When enabling, we expect the device to request a passphrase
      setPendingEnable(true);
      
      // Show info about the flow
      setStatusMessage('You will be prompted to set a passphrase on your device');
    }
    
    try {
      // Send the enable/disable command to the device
      // This will trigger a PassphraseRequest from the device if enabling
      await invoke('enable_passphrase_protection', {
        deviceId,
        enabled: newState,
      });
      
      // If disabling, we're done
      if (!newState) {
        setIsEnabled(false);
        setPendingEnable(false);
        
        if (onPassphraseToggle) {
          onPassphraseToggle(false);
        }
        
        setStatusMessage('Passphrase protection has been disabled');
        setTimeout(() => setStatusMessage(null), 3000);
        
        setIsUpdating(false);
      }
      // If enabling, wait for the passphrase modal to complete
    } catch (err) {
      console.error('Failed to update passphrase protection:', err);
      setPendingEnable(false);
      
      setStatusMessage(`Error: ${err instanceof Error ? err.message : 'Failed to update passphrase protection'}`);
      setTimeout(() => setStatusMessage(null), 5000);
      
      setIsUpdating(false);
    }
  };


  return (
    <>
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
                      {pendingEnable ? 'Enabling...' : 'Disabling...'}
                    </Text>
                  </Box>
                )}
              </Box>
            </Flex>
          </Flex>

        {/*<Box p={3} borderRadius="md" borderLeft="4px" borderLeftColor="blue.400">*/}
        {/*  <Text fontWeight="bold" fontSize="sm" mb={2}>*/}
        {/*    About Passphrase Protection*/}
        {/*  </Text>*/}
        {/*  <Text fontSize="sm" mb={2}>*/}
        {/*    When enabled, you'll be prompted for a passphrase to access your wallet. Different*/}
        {/*    passphrases create completely different wallets, allowing for:*/}
        {/*  </Text>*/}
        {/*  <Box pl={4}>*/}
        {/*    <Text fontSize="sm">• Hidden wallets for additional privacy</Text>*/}
        {/*    <Text fontSize="sm">• Plausible deniability under duress</Text>*/}
        {/*    <Text fontSize="sm">• Multiple wallet configurations from one seed</Text>*/}
        {/*  </Box>*/}
        {/*</Box>*/}

        {(isUpdating || statusMessage) && (
          <Box p={3} bg={statusMessage?.startsWith('Error') ? "red.50" : "blue.50"} 
               borderRadius="md" 
               borderLeft="4px" 
               borderLeftColor={statusMessage?.startsWith('Error') ? "red.400" : "blue.400"}>
            <Flex align="center" gap={2}>
              {isUpdating && <Spinner size="sm" color="blue.500" />}
              <Text fontWeight="bold" fontSize="sm" color={statusMessage?.startsWith('Error') ? "red.600" : "blue.600"}>
                {statusMessage || 'Updating passphrase protection settings...'}
              </Text>
            </Flex>
            {isUpdating && !statusMessage && (
              <Text fontSize="sm" mt={2} color="blue.600">
                Please follow the instructions on your device.
              </Text>
            )}
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
    </>
  );
};