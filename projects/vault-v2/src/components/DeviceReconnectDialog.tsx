import React, { useEffect, useState } from 'react';
import {
  DialogRoot,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogBody,
  DialogCloseTrigger,
  VStack,
  Text,
  Spinner,
  Icon,
  Box,
  IconButton,
} from '@chakra-ui/react';
import { FaUsb } from 'react-icons/fa';
import { listen, UnlistenFn } from '@tauri-apps/api/event';

interface DeviceReconnectDialogProps {
  isOpen: boolean;
  onClose: () => void;
  deviceId: string;
  reason?: 'passphrase_enabled' | 'passphrase_disabled' | 'device_reset';
  onReconnected?: () => void;
}

export const DeviceReconnectDialog: React.FC<DeviceReconnectDialogProps> = ({
  isOpen,
  onClose,
  deviceId,
  reason = 'device_reset',
  onReconnected,
}) => {
  const [isReconnecting, setIsReconnecting] = useState(false);
  const bgColor = 'gray.800';
  const textColor = 'white';
  const subtextColor = 'gray.400';
  const iconColor = 'blue.400';

  useEffect(() => {
    if (!isOpen) return;

    let unlisten: UnlistenFn | undefined;

    const setupListener = async () => {
      // Listen for device reconnection
      unlisten = await listen<{ device_id: string }>('device:connected', (event) => {
        if (event.payload.device_id === deviceId) {
          console.log('[DeviceReconnectDialog] Device reconnected:', deviceId);
          setIsReconnecting(true);
          
          // Wait a moment for the device to fully initialize
          setTimeout(() => {
            setIsReconnecting(false);
            if (onReconnected) {
              onReconnected();
            }
            onClose();
          }, 1500);
        }
      });
    };

    setupListener();

    // Cleanup
    return () => {
      if (unlisten) {
        unlisten();
      }
    };
  }, [isOpen, deviceId, onClose, onReconnected]);

  const getReasonText = () => {
    switch (reason) {
      case 'passphrase_enabled':
        return 'Passphrase protection has been enabled';
      case 'passphrase_disabled':
        return 'Passphrase protection has been disabled';
      case 'device_reset':
      default:
        return 'Device settings have been changed';
    }
  };

  const getInstructionText = () => {
    if (isReconnecting) {
      return 'Device detected! Initializing...';
    }
    return 'Please unplug your KeepKey and reconnect it to apply the changes.';
  };

  return (
    <DialogRoot
      open={isOpen}
      onOpenChange={() => {}}
      size="md"
      placement="center"
      motionPreset="slide-in-bottom"
    >
      <DialogContent bg={bgColor}>
        <DialogHeader>
          <DialogTitle color={textColor}>Device Reconnection Required</DialogTitle>
          {!isReconnecting && (
            <DialogCloseTrigger asChild>
              <IconButton
                onClick={onClose}
                variant="ghost"
                size="sm"
              />
            </DialogCloseTrigger>
          )}
        </DialogHeader>
        <DialogBody pb={6}>
          <VStack gap={6} align="center">
            <Box position="relative">
              <Icon
                as={FaUsb}
                boxSize={16}
                color={iconColor}
                className={isReconnecting ? 'pulse-animation' : ''}
              />
              {isReconnecting && (
                <Box position="absolute" top="50%" left="50%" transform="translate(-50%, -50%)">
                  <Spinner size="xl" color={iconColor} />
                </Box>
              )}
            </Box>

            <VStack gap={3} align="center">
              <Text fontWeight="bold" fontSize="lg" color={textColor}>
                {getReasonText()}
              </Text>
              <Text
                textAlign="center"
                color={subtextColor}
                fontSize="md"
              >
                {getInstructionText()}
              </Text>
              
              {!isReconnecting && (
                <VStack gap={2} mt={4}>
                  <Text fontSize="sm" color={subtextColor}>
                    Waiting for device reconnection...
                  </Text>
                  <Spinner size="md" color={iconColor} />
                </VStack>
              )}
            </VStack>
          </VStack>
        </DialogBody>
      </DialogContent>
      
      <style>{`
        @keyframes pulse {
          0% {
            transform: scale(1);
            opacity: 1;
          }
          50% {
            transform: scale(1.1);
            opacity: 0.8;
          }
          100% {
            transform: scale(1);
            opacity: 1;
          }
        }
        
        .pulse-animation {
          animation: pulse 1.5s ease-in-out infinite;
        }
      `}</style>
    </DialogRoot>
  );
};