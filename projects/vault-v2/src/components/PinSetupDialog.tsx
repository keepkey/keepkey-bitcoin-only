import React, { useState, useEffect, useCallback, useRef } from 'react';
import { Box, Text, VStack, HStack, Icon, Button, SimpleGrid, Input } from '@chakra-ui/react';
import { 
  DialogRoot,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogBody,
  DialogFooter,
  DialogCloseTrigger
} from './ui/dialog';
import { FaCircle } from 'react-icons/fa';
import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';

interface PinSetupDialogProps {
  isOpen: boolean;
  deviceId: string;
  onClose: () => void;
  onSuccess?: () => void;
  onError?: (error: string) => void;
}

const PIN_MATRIX_LAYOUT = [7, 8, 9, 4, 5, 6, 1, 2, 3] as const;

export const PinSetupDialog: React.FC<PinSetupDialogProps> = ({
  isOpen,
  deviceId,
  onClose,
  onSuccess,
  onError,
}) => {
  const [sessionId, setSessionId] = useState<string | null>(null);
  const [step, setStep] = useState<'initializing' | 'new_pin' | 'confirm_pin' | 'success' | 'error'>('initializing');
  const [positions, setPositions] = useState<number[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [isProcessing, setIsProcessing] = useState(false);
  const [isInitializing, setIsInitializing] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);
  const unlistenRef = useRef<UnlistenFn | null>(null);

  // Reset state when dialog closes
  useEffect(() => {
    if (!isOpen) {
      setSessionId(null);
      setStep('initializing');
      setPositions([]);
      setError(null);
      setIsProcessing(false);
      setIsInitializing(false);
      return;
    }

    const setupListeners = async () => {
      // Listen for PIN matrix requests
      const unlisten = await listen('pin_matrix_request', async (event) => {
        console.log('[PinSetupDialog] Received pin_matrix_request event:', event.payload);
        const payload = event.payload as any;
        
        if (payload.deviceId === deviceId) {
          // Store the session ID if we don't have it yet
          if (!sessionId && payload.sessionId) {
            setSessionId(payload.sessionId);
          }
          
          // Only process if this is our session or we don't have a session yet
          if (!sessionId || payload.sessionId === sessionId) {
            setStep(payload.step === 'confirm_pin' ? 'confirm_pin' : 'new_pin');
            setPositions([]); // Clear previous PIN entry
            setIsProcessing(false);
            setIsInitializing(false); // Reset initialization flag
          }
        }
      });

      // Listen for PIN setup completion
      const unlistenComplete = await listen('pin_setup_complete', (event) => {
        console.log('[PinSetupDialog] Received pin_setup_complete event:', event.payload);
        const payload = event.payload as any;
        
        if (payload.deviceId === deviceId && (!sessionId || payload.sessionId === sessionId)) {
          if (payload.success) {
            setStep('success');
            setTimeout(() => {
              if (onSuccess) onSuccess();
              onClose();
            }, 2000);
          } else {
            setError(payload.error || 'PIN setup failed');
            setStep('error');
            if (onError) onError(payload.error || 'PIN setup failed');
          }
          setIsProcessing(false);
        }
      });

      unlistenRef.current = async () => {
        await unlisten();
        await unlistenComplete();
      };
    };

    setupListeners();

    return () => {
      if (unlistenRef.current) {
        unlistenRef.current();
        unlistenRef.current = null;
      }
    };
  }, [isOpen, deviceId, sessionId, onSuccess, onError, onClose]);

  // Start PIN setup when dialog opens
  useEffect(() => {
    if (isOpen && !sessionId && step === 'initializing' && !isInitializing) {
      startPinSetup();
    }
  }, [isOpen, sessionId, step, isInitializing]);

  const startPinSetup = async () => {
    if (isInitializing) return; // Prevent double calls
    
    try {
      console.log('[PinSetupDialog] Starting PIN setup for device:', deviceId);
      setIsInitializing(true);
      setIsProcessing(true);
      
      const newSessionId = await invoke<string>('start_pin_setup', { deviceId });
      console.log('[PinSetupDialog] PIN setup session started:', newSessionId);
      setSessionId(newSessionId);
      
      // The backend will emit a pin_matrix_request event when ready
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      console.error('[PinSetupDialog] Failed to start PIN setup:', errorMessage);
      setError(errorMessage);
      setStep('error');
      setIsProcessing(false);
      setIsInitializing(false);
      
      if (onError) onError(errorMessage);
    }
  };

  const handlePinPress = useCallback((position: number) => {
    if (positions.length < 9 && !isProcessing) {
      setPositions(prev => [...prev, position]);
    }
  }, [positions.length, isProcessing]);

  const handleBackspace = useCallback(() => {
    if (!isProcessing) {
      setPositions(prev => prev.slice(0, -1));
    }
  }, [isProcessing]);

  const handleSubmit = useCallback(async () => {
    if (positions.length === 0 || !sessionId || isProcessing) return;
    
    setIsProcessing(true);
    
    try {
      // Convert positions to string (e.g., [1, 2, 3] -> "123")
      const pin = positions.join('');
      const currentStep = step === 'confirm_pin' ? 'confirm' : 'new';
      
      console.log(`[PinSetupDialog] Sending PIN for step: ${currentStep}`);
      const result = await invoke<string>('send_pin_setup_response', {
        deviceId,
        sessionId,
        pin,
        step: currentStep,
      });
      
      console.log('[PinSetupDialog] PIN response result:', result);
      
      // The backend will emit events for next steps
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      console.error('[PinSetupDialog] Failed to send PIN:', errorMessage);
      setError(errorMessage);
      setStep('error');
      setIsProcessing(false);
      
      if (onError) onError(errorMessage);
    }
  }, [positions, sessionId, isProcessing, step, deviceId, onError]);

  const handleKeyPress = useCallback((e: React.KeyboardEvent) => {
    e.preventDefault();
    
    if (isProcessing) return;

    if (e.key === 'Backspace') {
      handleBackspace();
    } else if (e.key === 'Enter') {
      handleSubmit();
    } else if (PIN_MATRIX_LAYOUT.includes(Number(e.key) as any)) {
      handlePinPress(Number(e.key));
    }
  }, [handleBackspace, handleSubmit, handlePinPress, isProcessing]);

  const getTitle = () => {
    switch (step) {
      case 'initializing':
        return 'Initializing PIN Setup...';
      case 'new_pin':
        return 'Create Your PIN';
      case 'confirm_pin':
        return 'Confirm Your PIN';
      case 'success':
        return 'PIN Enabled Successfully!';
      case 'error':
        return 'PIN Setup Failed';
      default:
        return 'PIN Setup';
    }
  };

  const getDescription = () => {
    switch (step) {
      case 'initializing':
        return 'Connecting to your device...';
      case 'new_pin':
        return 'Look at your KeepKey device for the number layout. Enter your new PIN using the positions shown below.';
      case 'confirm_pin':
        return 'Re-enter your PIN to confirm it matches.';
      case 'success':
        return 'Your device is now protected with a PIN.';
      case 'error':
        return error || 'An error occurred during PIN setup.';
      default:
        return '';
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
      size="lg"
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
        {!isProcessing && step !== 'success' && (
          <DialogCloseTrigger
            color="gray.400"
            _hover={{ color: "white", bg: "gray.700" }}
          />
        )}
        
        <DialogHeader>
          <DialogTitle color="white" fontSize="2xl" fontWeight="bold">
            {getTitle()}
          </DialogTitle>
        </DialogHeader>

        <DialogBody>
          <VStack spacing={6} align="stretch">
            <Text color="gray.300" textAlign="center">
              {getDescription()}
            </Text>

            {(step === 'new_pin' || step === 'confirm_pin') && (
              <>
                {/* PIN dots display */}
                <Box
                  p={4}
                  bg="gray.700"
                  borderRadius="lg"
                  borderWidth="2px"
                  borderColor={positions.length >= 4 ? "green.500" : "gray.600"}
                >
                  <HStack gap={2} justify="center">
                    {Array.from({ length: Math.max(4, positions.length) }, (_, i) => (
                      <Box
                        key={i}
                        w="12px"
                        h="12px"
                        borderRadius="full"
                        bg={i < positions.length ? "green.400" : "gray.500"}
                        opacity={i < positions.length ? 1 : 0.5}
                      />
                    ))}
                  </HStack>
                </Box>

                {positions.length < 4 && (
                  <Text fontSize="xs" color="yellow.400" textAlign="center">
                    Minimum 4 digits recommended
                  </Text>
                )}

                {/* PIN Matrix */}
                <SimpleGrid columns={3} gap={2} maxW="200px" mx="auto">
                  {PIN_MATRIX_LAYOUT.map((position) => (
                    <Button
                      key={position}
                      onClick={() => handlePinPress(position)}
                      size="lg"
                      h="60px"
                      bg="gray.700"
                      borderColor="gray.600"
                      borderWidth="1px"
                      color="gray.300"
                      _hover={{
                        bg: "gray.600",
                        borderColor: "green.500",
                      }}
                      disabled={isProcessing || positions.length >= 9}
                    >
                      <Icon as={FaCircle} boxSize={3} />
                    </Button>
                  ))}
                </SimpleGrid>

                {/* Hidden input for keyboard support */}
                <Input
                  ref={inputRef}
                  type="password"
                  value={positions.join('')}
                  onChange={() => {}}
                  onKeyDown={handleKeyPress}
                  style={{
                    position: 'absolute',
                    left: '-9999px',
                    opacity: 0,
                  }}
                />

                <Text fontSize="xs" color="gray.500" textAlign="center">
                  Look at your KeepKey device to see which number corresponds to each position
                </Text>
              </>
            )}

            {step === 'success' && (
              <VStack spacing={4} py={8}>
                <Text fontSize="4xl">✅</Text>
                <Text color="green.400" fontSize="lg" fontWeight="medium">
                  PIN Protection Enabled!
                </Text>
              </VStack>
            )}

            {step === 'error' && (
              <VStack spacing={4} py={8}>
                <Text fontSize="4xl">❌</Text>
                <Text color="red.400" fontSize="lg" fontWeight="medium">
                  {error || 'PIN setup failed'}
                </Text>
              </VStack>
            )}
          </VStack>
        </DialogBody>

        <DialogFooter>
          <HStack spacing={3}>
            {(step === 'new_pin' || step === 'confirm_pin') && (
              <>
                <Button
                  variant="outline"
                  onClick={handleBackspace}
                  disabled={isProcessing || positions.length === 0}
                  borderColor="gray.600"
                  color="gray.300"
                  _hover={{ bg: 'gray.700' }}
                >
                  Clear
                </Button>
                <Button
                  colorScheme="green"
                  onClick={handleSubmit}
                  disabled={isProcessing || positions.length === 0}
                  loading={isProcessing}
                >
                  {step === 'new_pin' ? 'Set PIN' : 'Confirm PIN'}
                </Button>
              </>
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