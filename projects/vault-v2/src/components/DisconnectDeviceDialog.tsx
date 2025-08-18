import React, { useState, useEffect } from 'react';
import {
  DialogRoot,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogBody,
  DialogFooter,
} from "./ui/dialog";
import {
  Box,
  Text,
  Button,
  VStack,
  HStack,
  Icon,
  Spinner,
} from '@chakra-ui/react';
import { FaUsb, FaCheckCircle, FaExclamationTriangle } from 'react-icons/fa';
import { listen } from '@tauri-apps/api/event';

interface DisconnectDeviceDialogProps {
  isOpen: boolean;
  deviceId: string;
  title?: string;
  message?: string;
  onDeviceDisconnected: () => void;
  onCancel?: () => void;
}

export const DisconnectDeviceDialog: React.FC<DisconnectDeviceDialogProps> = ({
  isOpen,
  deviceId,
  title = "Disconnect Your KeepKey",
  message = "Please disconnect your KeepKey and reconnect it to complete the operation.",
  onDeviceDisconnected,
  onCancel,
}) => {
  const [deviceDisconnected, setDeviceDisconnected] = useState(false);
  const [deviceReconnected, setDeviceReconnected] = useState(false);
  const [waitingForReconnect, setWaitingForReconnect] = useState(false);

  useEffect(() => {
    if (!isOpen) {
      // Reset states when dialog closes
      setDeviceDisconnected(false);
      setDeviceReconnected(false);
      setWaitingForReconnect(false);
      return;
    }

    let unlistenDisconnect: (() => void) | null = null;
    let unlistenConnect: (() => void) | null = null;

    const setupListeners = async () => {
      // Listen for device disconnection
      unlistenDisconnect = await listen<string>('device:disconnected', (event) => {
        console.log('Device disconnected event:', event.payload);
        if (event.payload === deviceId) {
          setDeviceDisconnected(true);
          setWaitingForReconnect(true);
        }
      });

      // Listen for device reconnection
      unlistenConnect = await listen<any>('device:connected', (event) => {
        console.log('Device connected event:', event.payload);
        // Check if it's the same device or any KeepKey
        if (deviceDisconnected && event.payload?.is_keepkey) {
          setDeviceReconnected(true);
          setWaitingForReconnect(false);
          
          // Auto-close after successful reconnection
          setTimeout(() => {
            onDeviceDisconnected();
          }, 1500);
        }
      });
    };

    setupListeners();

    return () => {
      if (unlistenDisconnect) unlistenDisconnect();
      if (unlistenConnect) unlistenConnect();
    };
  }, [isOpen, deviceId, deviceDisconnected, onDeviceDisconnected]);

  const getStatusIcon = () => {
    if (deviceReconnected) {
      return <Icon as={FaCheckCircle} boxSize={16} color="green.500" />;
    } else if (waitingForReconnect) {
      return <Spinner size="xl" color="blue.500" />;
    } else {
      return <Icon as={FaUsb} boxSize={16} color="gray.400" />;
    }
  };

  const getStatusMessage = () => {
    if (deviceReconnected) {
      return "Device reconnected successfully!";
    } else if (waitingForReconnect) {
      return "Waiting for device to reconnect...";
    } else if (deviceDisconnected) {
      return "Device disconnected. Please reconnect it now.";
    } else {
      return message;
    }
  };

  const getStatusColor = () => {
    if (deviceReconnected) return "green.600";
    if (waitingForReconnect) return "blue.600";
    if (deviceDisconnected) return "orange.600";
    return "gray.600";
  };

  return (
    <DialogRoot 
      size="md" 
      placement="center" 
      open={isOpen}
      onOpenChange={(e) => {
        if (!e.open && onCancel && !deviceReconnected) {
          onCancel();
        }
      }}
    >
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{title}</DialogTitle>
        </DialogHeader>
        
        <DialogBody>
          <VStack gap={6} py={4}>
            {/* Icon */}
            <Box>{getStatusIcon()}</Box>
            
            {/* Status Message */}
            <Text 
              fontSize="md" 
              color={getStatusColor()}
              textAlign="center"
              fontWeight={deviceReconnected ? "bold" : "medium"}
            >
              {getStatusMessage()}
            </Text>

            {/* Additional instructions */}
            {!deviceDisconnected && (
              <VStack gap={2}>
                <HStack gap={2} align="center">
                  <Icon as={FaExclamationTriangle} color="orange.500" />
                  <Text fontSize="sm" color="gray.600">
                    Important: This will reset your device session
                  </Text>
                </HStack>
                <Text fontSize="sm" color="gray.500" textAlign="center">
                  After reconnecting, you may need to unlock your device again.
                </Text>
              </VStack>
            )}

            {/* Progress indicator for waiting state */}
            {waitingForReconnect && (
              <Text fontSize="sm" color="blue.600" textAlign="center">
                Please reconnect your KeepKey to continue...
              </Text>
            )}

            {/* Success message */}
            {deviceReconnected && (
              <Text fontSize="sm" color="green.600" textAlign="center">
                Your device has been reset. This dialog will close automatically.
              </Text>
            )}
          </VStack>
        </DialogBody>

        <DialogFooter>
          {!deviceReconnected && onCancel && (
            <Button
              variant="ghost"
              onClick={onCancel}
              disabled={waitingForReconnect}
            >
              Cancel
            </Button>
          )}
        </DialogFooter>
      </DialogContent>
    </DialogRoot>
  );
};