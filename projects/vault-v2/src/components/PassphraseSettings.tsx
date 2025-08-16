import React, { useState, useEffect } from 'react';
import {
  Box,
  Text,
  Flex,
  Spinner,
  Switch,
} from '@chakra-ui/react';
import { invoke } from '@tauri-apps/api/core';
import { relaunch } from '@tauri-apps/plugin-process';

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

  // Note: Removed passphrase_request listener since enabling passphrase protection
  // now directly succeeds with ApplySettings and doesn't require passphrase entry
  // during the enable flow. The app will restart and prompt for passphrase on next use.

  const handleTogglePassphrase = async () => {
    if (isUpdating) return;
    
    const newState = !isEnabled;
    setIsUpdating(true);
    
    if (newState) {
      // Show info about the flow
      setStatusMessage('Press and hold the button on your KeepKey to confirm enabling passphrase protection.');
    }
    
    try {
      // Send the enable/disable command to the device
      await invoke('enable_passphrase_protection', {
        deviceId,
        enabled: newState,
      });
      
      // Handle both enable and disable with restart
      if (newState) {
        // If enabling, update state and restart immediately
        setIsEnabled(true);
        
        if (onPassphraseToggle) {
          onPassphraseToggle(true);
        }
        
        setStatusMessage('Passphrase protection enabled successfully! Restarting app...');
        
        // Restart entire app after a short delay to show success message
        setTimeout(async () => {
          try {
            console.log('Restarting entire app after enabling passphrase protection');
            await relaunch();
          } catch (err) {
            console.error('Failed to restart app:', err);
            // Fallback to window reload if relaunch fails
            window.location.reload();
          }
        }, 2000);
        
        setIsUpdating(false);
      } else {
        // If disabling, tell user to unplug device and restart the app
        setIsEnabled(false);
        
        if (onPassphraseToggle) {
          onPassphraseToggle(false);
        }
        
        setStatusMessage('Passphrase protection disabled - Please unplug your KeepKey and reconnect it. Restarting app...');
        
        // Give user time to see the unplug message, then restart entire app
        setTimeout(async () => {
          try {
            console.log('Restarting entire app after disabling passphrase protection');
            await relaunch();
          } catch (err) {
            console.error('Failed to restart app:', err);
            // Fallback to window reload if relaunch fails
            window.location.reload();
          }
        }, 3000); // Increased to 3 seconds to give user time to see the unplug message
        
        setIsUpdating(false);
      }
    } catch (err) {
      console.error('Failed to update passphrase protection:', err);
      
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
            <Box p={3} 
                 bg={statusMessage?.startsWith('Error') ? "red.50" : 
                     statusMessage?.includes('unplug') ? "orange.50" : "blue.50"} 
                 borderRadius="md" 
                 borderLeft="4px" 
                 borderLeftColor={statusMessage?.startsWith('Error') ? "red.400" : 
                                  statusMessage?.includes('unplug') ? "orange.400" : "blue.400"}>
              <Flex align="center" gap={2}>
                {isUpdating && !statusMessage?.includes('unplug') && <Spinner size="sm" color="blue.500" />}
                <Text fontWeight="bold" 
                      fontSize={statusMessage?.includes('unplug') ? "md" : "sm"} 
                      color={statusMessage?.startsWith('Error') ? "red.600" : 
                             statusMessage?.includes('unplug') ? "orange.600" : "blue.600"}>
                  {statusMessage || 'Updating passphrase protection settings...'}
                </Text>
              </Flex>
            </Box>
          )}

          <Text fontSize="xs" color="gray.500" textAlign="center">
            Device: {deviceId.slice(-8)}
          </Text>
      </Flex>
    </Box>
  );
};