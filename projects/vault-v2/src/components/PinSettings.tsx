import React, { useState, useEffect } from 'react';
import {
  Box,
  Text,
  Flex,
  Spinner,
  Switch,
} from '@chakra-ui/react';
import { invoke } from '@tauri-apps/api/core';
import { useEnablePinDialog } from '../contexts/DialogContext';

interface PinSettingsProps {
  deviceId: string;
  isPinEnabled?: boolean;
  onPinToggle?: (enabled: boolean) => void;
}

export const PinSettings: React.FC<PinSettingsProps> = ({
  deviceId,
  isPinEnabled: initialEnabled = false,
  onPinToggle,
}) => {
  const [isEnabled, setIsEnabled] = useState(initialEnabled);
  const [isUpdating, setIsUpdating] = useState(false);
  const [statusMessage, setStatusMessage] = useState<string | null>(null);
  const enablePinDialog = useEnablePinDialog();

  // Debug log the initial prop value
  console.log(`[PinSettings] Component mounted/updated - deviceId: ${deviceId}, initialEnabled prop: ${initialEnabled}`);

  // Sync local state with device features state
  useEffect(() => {
    console.log(`[PinSettings] Syncing state - initialEnabled changed to: ${initialEnabled}`);
    setIsEnabled(initialEnabled);
  }, [initialEnabled]);

  // Log PIN protection status changes for debugging
  useEffect(() => {
    console.log(`[PinSettings] Device ${deviceId} - PIN protection state: ${isEnabled ? 'enabled' : 'disabled'}`);
  }, [isEnabled, deviceId]);

  const handleTogglePin = async () => {
    if (isUpdating) return;
    
    const newState = !isEnabled;
    
    try {
      if (newState) {
        // Enabling PIN protection - show the enable PIN dialog
        console.log('[PinSettings] Opening enable PIN dialog for device:', deviceId);
        
        enablePinDialog.show({
          deviceId,
          onSuccess: () => {
            console.log('[PinSettings] PIN enabled successfully');
            setIsEnabled(true);
            setStatusMessage('PIN protection enabled successfully!');
            
            if (onPinToggle) {
              onPinToggle(true);
            }
            
            // Clear success message after a delay
            setTimeout(() => setStatusMessage(null), 3000);
          },
          onError: (error) => {
            console.error('[PinSettings] Failed to enable PIN:', error);
            setStatusMessage(`Error: ${error}`);
            setTimeout(() => setStatusMessage(null), 5000);
            // Keep toggle in disabled state on error
            setIsEnabled(false);
          },
          onDialogClose: () => {
            console.log('[PinSettings] Enable PIN dialog closed');
            // User cancelled - keep toggle in disabled state
            setIsEnabled(false);
          }
        });
      } else {
        // Disabling PIN protection
        setIsUpdating(true);
        setStatusMessage('Enter your current PIN on the KeepKey to disable PIN protection...');
        
        await invoke('disable_pin_protection', {
          deviceId,
        });
        
        setIsEnabled(false);
        setStatusMessage('PIN protection disabled successfully!');
        
        if (onPinToggle) {
          onPinToggle(false);
        }
        
        // Clear success message after a delay
        setTimeout(() => setStatusMessage(null), 3000);
      }
      
    } catch (err) {
      console.error('Failed to update PIN protection:', err);
      
      setStatusMessage(`Error: ${err instanceof Error ? err.message : 'Failed to update PIN protection'}`);
      setTimeout(() => setStatusMessage(null), 5000);
      
      // Reset toggle to previous state on error
      setIsEnabled(!newState);
    } finally {
      if (!newState) {
        setIsUpdating(false);
      }
    }
  };

  const handleChangePin = async () => {
    if (isUpdating || !isEnabled) return;
    
    setIsUpdating(true);
    setStatusMessage('Follow the instructions on your KeepKey to change your PIN...');
    
    try {
      await invoke('change_pin', {
        deviceId,
      });
      
      setStatusMessage('PIN changed successfully!');
      setTimeout(() => setStatusMessage(null), 3000);
      
    } catch (err) {
      console.error('Failed to change PIN:', err);
      
      setStatusMessage(`Error: ${err instanceof Error ? err.message : 'Failed to change PIN'}`);
      setTimeout(() => setStatusMessage(null), 5000);
    } finally {
      setIsUpdating(false);
    }
  };

  return (
    <Box p={4} borderWidth={1} borderRadius="md">
      <Flex direction="column" gap={4}>
        <Text fontSize="lg" fontWeight="bold">
          PIN Protection
        </Text>
        
        <Flex align="center" justify="space-between">
          <Flex direction="column">
            <Text fontWeight="medium">
              Enable PIN
            </Text>
            <Text fontSize="sm" color="gray.600">
              Protect your device with a PIN code that you'll enter on each use.
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
              onCheckedChange={handleTogglePin}
              disabled={isUpdating}
              colorPalette="blue"
              size="lg"
            >
              <Switch.HiddenInput />
              <Switch.Control />
            </Switch.Root>
          </Flex>
        </Flex>

        {/* Change PIN button - only show when PIN is enabled */}
        {isEnabled && !isUpdating && (
          <Box>
            <Text 
              fontSize="sm" 
              color="blue.500" 
              cursor="pointer"
              _hover={{ textDecoration: 'underline' }}
              onClick={handleChangePin}
            >
              Change PIN
            </Text>
          </Box>
        )}

        {(isUpdating || statusMessage) && (
          <Box p={3} 
               bg={statusMessage?.startsWith('Error') ? "red.50" : "blue.50"} 
               borderRadius="md" 
               borderLeft="4px" 
               borderLeftColor={statusMessage?.startsWith('Error') ? "red.400" : "blue.400"}>
            <Flex align="center" gap={2}>
              {isUpdating && <Spinner size="sm" color="blue.500" />}
              <Text fontWeight="bold" 
                    fontSize="sm" 
                    color={statusMessage?.startsWith('Error') ? "red.600" : "blue.600"}>
                {statusMessage || 'Updating PIN protection settings...'}
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