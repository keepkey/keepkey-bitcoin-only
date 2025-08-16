import React, { useState, useEffect } from 'react';
import {
  Box,
  Text,
  Flex,
  Spinner,
  Switch,
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

  // Debug log the initial prop value
  console.log(`[PassphraseSettings] Component mounted/updated - deviceId: ${deviceId}, initialEnabled prop: ${initialEnabled}`);

  // Sync local state with device features state
  useEffect(() => {
    console.log(`[PassphraseSettings] Syncing state - initialEnabled changed to: ${initialEnabled}`);
    setIsEnabled(initialEnabled);
  }, [initialEnabled]);

  // Log passphrase protection status changes for debugging
  useEffect(() => {
    console.log(`[PassphraseSettings] Device ${deviceId} - passphrase protection state: ${isEnabled ? 'enabled' : 'disabled'}`);
  }, [isEnabled, deviceId]);

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
              
              // Notify parent component to refresh device state
              if (onPassphraseToggle) {
                onPassphraseToggle(true);
              }
              
              setStatusMessage('Passphrase protection enabled - restarting app...');
              
              // Restart backend and app after passphrase is enabled
              setTimeout(async () => {
                try {
                  // Use the backend restart command to properly reset device state
                  await invoke('restart_backend_startup');
                  
                  // Wait a moment for backend restart, then reload the app
                  console.log('Restarting app after enabling passphrase protection');
                  setTimeout(() => {
                    window.location.reload();
                  }, 2000);
                } catch (err) {
                  console.error('Failed to restart backend:', err);
                  // Still restart even if backend restart fails
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
      setStatusMessage('Press and hold the button on your KeepKey to change the device Passphrase.');
    }
    
    try {
      // Send the enable/disable command to the device
      // This will trigger a PassphraseRequest from the device if enabling
      await invoke('enable_passphrase_protection', {
        deviceId,
        enabled: newState,
      });
      
      // If disabling, restart the app after disabling
      if (!newState) {
        setIsEnabled(false);
        setPendingEnable(false);
        
        if (onPassphraseToggle) {
          onPassphraseToggle(false);
        }
        
        setStatusMessage('Passphrase protection disabled - restarting app...');
        
        // Restart backend and app after passphrase is disabled
        setTimeout(async () => {
          try {
            // Use the backend restart command to properly reset device state
            await invoke('restart_backend_startup');
            
            // Wait a moment for backend restart, then reload the app
            console.log('Restarting app after disabling passphrase protection');
            setTimeout(() => {
              window.location.reload();
            }, 2000);
          } catch (err) {
            console.error('Failed to restart backend:', err);
            // Still restart even if backend restart fails
            setTimeout(() => {
              window.location.reload();
            }, 1000);
          }
        }, 1000);
        
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
    <Box p={4} borderWidth={1} borderRadius="md">
      <Flex direction="column" gap={4}>
          <Text fontSize="lg" fontWeight="bold">
            Passphrase Protection
          </Text>
          
          <Flex align="center" justify="space-between">
            <Flex direction="column">
              <Text fontWeight="medium">
                Enable Passphrase
              </Text>
              <Text fontSize="sm" color="gray.600">
                Enable the ability to have multiple wallets, accessible by using a unique passphrase for each wallet.
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
              <Switch.Root
                checked={isEnabled}
                onCheckedChange={handleTogglePassphrase}
                disabled={isUpdating}
                colorPalette="blue"
                size="lg"
              >
                <Switch.HiddenInput />
                <Switch.Control />
              </Switch.Root>
            </Flex>
          </Flex>

          {(isUpdating || statusMessage) && (
            <Box p={3} bg={statusMessage?.startsWith('Error') ? "red.50" : "blue.50"} 
                 borderRadius="md" 
                 borderLeft="4px" 
                 borderLeftColor={statusMessage?.startsWith('Error') ? "red.400" : "blue.400"}>
              <Flex align="center" gap={2}>
                {isUpdating && <Spinner size="sm" color="blue.500" />}
                <Text fontWeight="bold" fontSize="sm" color={statusMessage?.startsWith('Error') ? "red.600" : "blue.600"}>
                  {statusMessage ? statusMessage : pendingEnable ? 'Press and hold the button on your KeepKey to change the device Passphrase.' : 'Updating passphrase protection settings...'}
                </Text>
              </Flex>
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