import React, { useState, useEffect } from 'react';
import { Box, Text, VStack, HStack, Icon, Spinner, Button, Image } from '@chakra-ui/react';
import { 
  DialogRoot,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogBody,
  DialogFooter,
  DialogCloseTrigger
} from './ui/dialog';
import { FaShieldAlt, FaLock, FaEye, FaBolt } from 'react-icons/fa';
import { invoke } from '@tauri-apps/api/core';
import cipherImage from '../assets/onboarding/cipher.png';

interface EnablePinDialogProps {
  isOpen: boolean;
  deviceId: string;
  onClose: () => void;
  onSuccess?: () => void;
  onError?: (error: string) => void;
}

export const EnablePinDialog: React.FC<EnablePinDialogProps> = ({
  isOpen,
  deviceId,
  onClose,
  onSuccess,
  onError,
}) => {
  const [step, setStep] = useState<'instructions' | 'setting' | 'confirming' | 'success' | 'error'>('instructions');
  const [error, setError] = useState<string | null>(null);
  const [isProcessing, setIsProcessing] = useState(false);

  useEffect(() => {
    if (isOpen) {
      setStep('instructions');
      setError(null);
      setIsProcessing(false);
    }
  }, [isOpen]);

  const handleStartPinSetup = async () => {
    setIsProcessing(true);
    setStep('setting');
    
    try {
      // Call the enable_pin_protection command
      await invoke('enable_pin_protection', { deviceId });
      
      // Success - PIN was set
      setStep('success');
      setTimeout(() => {
        if (onSuccess) onSuccess();
        onClose();
      }, 2000);
      
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      console.error('Failed to enable PIN protection:', errorMessage);
      
      setError(errorMessage);
      setStep('error');
      
      if (onError) onError(errorMessage);
    } finally {
      setIsProcessing(false);
    }
  };

  const handleCancel = () => {
    if (!isProcessing) {
      onClose();
    }
  };

  return (
    <DialogRoot
      open={isOpen}
      onOpenChange={(e) => {
        if (!e.open && !isProcessing) {
          onClose();
        }
      }}
      size="xl"
      placement="center"
      motionPreset="slideInBottom"
    >
      <DialogContent
        bg="gray.800"
        borderColor="gray.700"
        _backdrop={{
          bg: "rgba(0, 0, 0, 0.8)"
        }}
      >
        {!isProcessing && (
          <DialogCloseTrigger
            color="gray.400"
            _hover={{ color: "white", bg: "gray.700" }}
          />
        )}
        
        <DialogHeader>
          <DialogTitle color="white" fontSize="2xl" fontWeight="bold">
            {step === 'instructions' && 'Enable PIN Protection'}
            {step === 'setting' && 'Setting Up PIN...'}
            {step === 'confirming' && 'Confirming PIN...'}
            {step === 'success' && 'PIN Protection Enabled!'}
            {step === 'error' && 'Setup Failed'}
          </DialogTitle>
        </DialogHeader>

        <DialogBody>
          <VStack spacing={6} align="stretch">
            {step === 'instructions' && (
              <>
                <Box 
                  p={4} 
                  bg="blue.900" 
                  borderRadius="md" 
                  borderWidth="1px" 
                  borderColor="blue.700"
                >
                  <HStack spacing={3} align="flex-start">
                    <Icon as={FaShieldAlt} color="blue.400" boxSize={5} mt={1} />
                    <VStack align="start" spacing={2} flex={1}>
                      <Text color="blue.100" fontWeight="semibold">
                        About to Enable PIN Protection
                      </Text>
                      <Text color="blue.200" fontSize="sm">
                        You'll set up a PIN directly on your KeepKey device. This PIN will be required 
                        every time you connect your device.
                      </Text>
                    </VStack>
                  </HStack>
                </Box>

                <VStack spacing={4} align="stretch">
                  <Text color="gray.300" fontWeight="semibold">
                    How KeepKey PIN Security Works:
                  </Text>
                  
                  <HStack spacing={3} align="flex-start">
                    <Icon as={FaLock} color="green.400" boxSize={5} mt={1} />
                    <VStack align="start" spacing={1} flex={1}>
                      <Text color="gray.200" fontSize="sm" fontWeight="medium">
                        Zero-Knowledge Security
                      </Text>
                      <Text color="gray.400" fontSize="xs">
                        Your device displays scrambled numbers. Only the device knows your actual PIN.
                      </Text>
                    </VStack>
                  </HStack>

                  <HStack spacing={3} align="flex-start">
                    <Icon as={FaEye} color="green.400" boxSize={5} mt={1} />
                    <VStack align="start" spacing={1} flex={1}>
                      <Text color="gray.200" fontSize="sm" fontWeight="medium">
                        Anti-Keylogging Protection
                      </Text>
                      <Text color="gray.400" fontSize="xs">
                        Even if malware monitors your computer, it can't decode your PIN.
                      </Text>
                    </VStack>
                  </HStack>

                  <HStack spacing={3} align="flex-start">
                    <Icon as={FaBolt} color="green.400" boxSize={5} mt={1} />
                    <VStack align="start" spacing={1} flex={1}>
                      <Text color="gray.200" fontSize="sm" fontWeight="medium">
                        Side-Channel Protection
                      </Text>
                      <Text color="gray.400" fontSize="xs">
                        Constant power consumption prevents electrical analysis attacks.
                      </Text>
                    </VStack>
                  </HStack>
                </VStack>

                {cipherImage && (
                  <Box borderRadius="md" overflow="hidden" borderWidth="1px" borderColor="gray.600">
                    <Image 
                      src={cipherImage} 
                      alt="KeepKey PIN Security"
                      maxW="100%"
                    />
                  </Box>
                )}

                <Box 
                  p={3} 
                  bg="yellow.900" 
                  borderRadius="md" 
                  borderWidth="1px" 
                  borderColor="yellow.700"
                >
                  <Text color="yellow.100" fontSize="sm">
                    <strong>Important:</strong> Choose a PIN you'll remember. If forgotten, you'll need 
                    to recover your wallet using your recovery phrase.
                  </Text>
                </Box>
              </>
            )}

            {(step === 'setting' || step === 'confirming') && (
              <VStack spacing={6} py={8}>
                <Spinner size="xl" color="blue.400" thickness="4px" />
                
                <VStack spacing={2}>
                  <Text color="gray.200" fontSize="lg" fontWeight="medium" textAlign="center">
                    {step === 'setting' 
                      ? 'Look at your KeepKey device'
                      : 'Confirm your PIN on the device'}
                  </Text>
                  <Text color="gray.400" fontSize="sm" textAlign="center">
                    {step === 'setting'
                      ? 'Enter your new PIN using the scrambled number pad shown on the device screen.'
                      : 'Re-enter your PIN to confirm it.'}
                  </Text>
                </VStack>

                <Box 
                  p={4} 
                  bg="gray.700" 
                  borderRadius="md" 
                  borderWidth="1px" 
                  borderColor="gray.600"
                  w="full"
                >
                  <VStack spacing={2}>
                    <Text color="gray.300" fontSize="sm" fontWeight="medium">
                      On Your KeepKey:
                    </Text>
                    <Text color="gray.400" fontSize="xs" textAlign="center">
                      {step === 'setting'
                        ? '1. Look at the scrambled number grid\n2. Click positions on this screen that match your desired PIN\n3. Press the button to confirm'
                        : '1. Re-enter the same PIN\n2. Press the button to confirm'}
                    </Text>
                  </VStack>
                </Box>
              </VStack>
            )}

            {step === 'success' && (
              <VStack spacing={4} py={8}>
                <Box 
                  p={3}
                  borderRadius="full"
                  bg="green.900"
                  color="green.400"
                >
                  <Icon as={FaShieldAlt} boxSize={12} />
                </Box>
                <Text color="green.400" fontSize="lg" fontWeight="medium">
                  PIN Protection Successfully Enabled!
                </Text>
                <Text color="gray.400" fontSize="sm" textAlign="center">
                  Your device is now protected. You'll need to enter this PIN each time you connect.
                </Text>
              </VStack>
            )}

            {step === 'error' && (
              <VStack spacing={4} py={8}>
                <Box 
                  p={3}
                  borderRadius="full"
                  bg="red.900"
                  color="red.400"
                >
                  <Text fontSize="2xl" fontWeight="bold">✕</Text>
                </Box>
                <Text color="red.400" fontSize="lg" fontWeight="medium">
                  Failed to Enable PIN
                </Text>
                {error && (
                  <Text color="gray.400" fontSize="sm" textAlign="center">
                    {error}
                  </Text>
                )}
              </VStack>
            )}
          </VStack>
        </DialogBody>

        <DialogFooter>
          <HStack spacing={3}>
            {step === 'instructions' && (
              <>
                <Button
                  variant="outline"
                  onClick={handleCancel}
                  borderColor="gray.600"
                  color="gray.300"
                  _hover={{ bg: 'gray.700' }}
                >
                  Cancel
                </Button>
                <Button
                  colorScheme="blue"
                  onClick={handleStartPinSetup}
                  loading={isProcessing}
                >
                  Start PIN Setup
                </Button>
              </>
            )}
            
            {(step === 'setting' || step === 'confirming') && (
              <Button
                variant="outline"
                onClick={handleCancel}
                borderColor="gray.600"
                color="gray.300"
                _hover={{ bg: 'gray.700' }}
                disabled={true}
              >
                Please complete on device...
              </Button>
            )}
            
            {step === 'error' && (
              <Button
                colorScheme="red"
                onClick={onClose}
              >
                Close
              </Button>
            )}
          </HStack>
        </DialogFooter>
      </DialogContent>
    </DialogRoot>
  );
};