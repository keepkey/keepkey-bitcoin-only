import React, { useState, useCallback, useEffect } from 'react';
import {
  DialogRoot,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogBody,
  DialogFooter,
  DialogActionTrigger,
  DialogCloseTrigger,
} from '@chakra-ui/react';
import {
  Box,
  Text,
  Button,
  HStack,
  VStack,
  Grid,
  Spinner,
  IconButton,
} from '@chakra-ui/react';
import { LuX, LuDelete } from 'react-icons/lu';
import { invoke } from '@tauri-apps/api/core';
import { useTypedTranslation } from '../hooks/useTypedTranslation';

interface DevicePinDialogProps {
  isOpen: boolean;
  deviceId: string;
  requestId: string;
  operationType?: string;
  onSubmit?: (pin: string) => void;
  onCancel?: () => void;
  onClose: () => void;
}

export const DevicePinDialog = ({ 
  isOpen, 
  deviceId, 
  requestId,
  operationType = 'operation',
  onSubmit, 
  onCancel,
  onClose 
}: DevicePinDialogProps) => {
  const { t } = useTypedTranslation('dialogs');
  const [pinPositions, setPinPositions] = useState<number[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);

  // Reset state when dialog opens
  useEffect(() => {
    if (isOpen) {
      setPinPositions([]);
      setError(null);
      setIsSubmitting(false);
    }
  }, [isOpen]);

  const handlePinButtonClick = useCallback((position: number) => {
    if (pinPositions.length >= 9) return;
    console.log('Button clicked: position', position);
    setPinPositions(prev => [...prev, position]);
  }, [pinPositions.length]);

  const handleBackspace = useCallback(() => {
    setPinPositions(prev => prev.slice(0, -1));
  }, []);

  const handleSubmitPin = async () => {
    if (pinPositions.length === 0) {
      setError('Please enter your PIN');
      return;
    }

    if (onSubmit) {
      // Convert positions to PIN string (1-9)
      const pinString = pinPositions.join('');
      console.log('🔐 Submitting PIN for request:', requestId);
      setIsSubmitting(true);
      try {
        await onSubmit(pinString);
      } catch (err) {
        console.error('PIN submission error:', err);
        setError('Failed to submit PIN');
        setIsSubmitting(false);
      }
    }
  };

  const handleCancel = () => {
    if (onCancel) {
      onCancel();
    }
    onClose();
  };

  const getOperationDescription = () => {
    switch (operationType) {
      case 'settings':
        return 'Enter your PIN to confirm settings change';
      case 'tx':
        return 'Enter your PIN to sign transaction';
      case 'export':
        return 'Enter your PIN to export data';
      default:
        return 'Enter your PIN to continue';
    }
  };

  return (
    <DialogRoot
      open={isOpen}
      onOpenChange={(e) => {
        if (!e.open && !isSubmitting) {
          handleCancel();
        }
      }}
      size="md"
      placement="center"
      motionPreset="slide-in-bottom"
    >
      <DialogContent>
        <DialogHeader>
          <DialogTitle>PIN Required</DialogTitle>
          <DialogCloseTrigger asChild>
            <IconButton
              variant="ghost"
              size="sm"
              disabled={isSubmitting}
            >
              <LuX />
            </IconButton>
          </DialogCloseTrigger>
        </DialogHeader>

        <DialogBody>
          <VStack spacing={6} py={4}>
            <Text fontSize="md" color="gray.600" textAlign="center">
              {getOperationDescription()}
            </Text>

            {/* PIN Display */}
            <HStack spacing={2}>
              {[...Array(9)].map((_, i) => (
                <Box
                  key={i}
                  w="10px"
                  h="10px"
                  borderRadius="full"
                  bg={i < pinPositions.length ? 'blue.500' : 'gray.300'}
                  transition="all 0.2s"
                />
              ))}
            </HStack>

            {/* Error Message */}
            {error && (
              <Box
                p={3}
                borderRadius="md"
                bg="red.50"
                borderWidth="1px"
                borderColor="red.200"
              >
                <Text color="red.700" fontSize="sm">{error}</Text>
              </Box>
            )}

            {/* PIN Matrix */}
            <Box
              p={4}
              borderRadius="lg"
              borderWidth="1px"
              borderColor="gray.200"
              bg="gray.50"
            >
              <Text fontSize="sm" color="gray.600" mb={3} textAlign="center">
                Look at your device screen for the numbers layout
              </Text>
              
              <Grid templateColumns="repeat(3, 1fr)" gap={2}>
                {[7, 8, 9, 4, 5, 6, 1, 2, 3].map((num, index) => (
                  <Button
                    key={num}
                    onClick={() => handlePinButtonClick(index + 1)}
                    isDisabled={isSubmitting || pinPositions.length >= 9}
                    size="lg"
                    h="60px"
                    w="60px"
                    fontSize="xl"
                    colorScheme="gray"
                    variant="outline"
                    _hover={{ bg: 'gray.100' }}
                  >
                    •
                  </Button>
                ))}
              </Grid>

              <HStack mt={3} spacing={2} justify="center">
                <Button
                  leftIcon={<LuDelete />}
                  onClick={handleBackspace}
                  isDisabled={isSubmitting || pinPositions.length === 0}
                  size="sm"
                  variant="outline"
                >
                  Backspace
                </Button>
              </HStack>
            </Box>
          </VStack>
        </DialogBody>

        <DialogFooter>
          <HStack spacing={3}>
            <DialogActionTrigger asChild>
              <Button 
                variant="outline" 
                onClick={handleCancel}
                isDisabled={isSubmitting}
              >
                Cancel
              </Button>
            </DialogActionTrigger>
            <Button
              colorScheme="blue"
              onClick={handleSubmitPin}
              isLoading={isSubmitting}
              isDisabled={pinPositions.length === 0}
            >
              Submit PIN
            </Button>
          </HStack>
        </DialogFooter>
      </DialogContent>
    </DialogRoot>
  );
};
