import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  Box,
  Heading,
  VStack,
  HStack,
  Text,
  Button,
  SimpleGrid,
  Input,
  Icon,
} from "@chakra-ui/react";
import { FaCircle } from "react-icons/fa";
import { PinPosition, PIN_MATRIX_LAYOUT } from "../../types/pin";

interface RecoveryFlowProps {
  deviceId: string;
  wordCount: number;
  passphraseProtection: boolean;
  deviceLabel: string;
  onComplete: () => void;
  onError: (error: string) => void;
  onBack?: () => void;
}

interface RecoverySession {
  session_id: string;
  device_id: string;
  word_count: number;
  current_word: number;
  current_character: number;
  is_active: boolean;
}

interface RecoveryProgress {
  word_pos: number;
  character_pos: number;
  auto_completed: boolean;
  is_complete: boolean;
  error?: string;
}

type RecoveryState = 
  | 'initializing'
  | 'pin-first'
  | 'pin-confirm'
  | 'button-confirm'
  | 'phrase-entry'
  | 'complete'
  | 'error';

export function RecoveryFlow({ 
  deviceId, 
  wordCount, 
  passphraseProtection,
  deviceLabel,
  onComplete, 
  onError, 
  onBack 
}: RecoveryFlowProps) {
  const [state, setState] = useState<RecoveryState>('initializing');
  const [session, setSession] = useState<RecoverySession | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [isProcessing, setIsProcessing] = useState(false);
  
  // PIN entry state
  const [pinPositions, setPinPositions] = useState<PinPosition[]>([]);
  
  // Recovery phrase entry state
  const [currentWord, setCurrentWord] = useState(0);
  const [currentChar, setCurrentChar] = useState(0);
  const [characterInputs, setCharacterInputs] = useState(['', '', '', '']);
  const [isAutoCompleted, setIsAutoCompleted] = useState(false);

  console.log("🔄 RecoveryFlow - Current state:", state);

  // Initialize recovery
  useEffect(() => {
    const startRecovery = async () => {
      if (state !== 'initializing' || session) return;
      
      try {
        console.log("🔄 Starting recovery device with:", {
          deviceId,
          wordCount,
          passphraseProtection,
          label: deviceLabel
        });
        
        const recoverySession = await invoke<RecoverySession>('start_device_recovery', {
          deviceId,
          wordCount,
          passphraseProtection,
          label: deviceLabel
        });
        
        console.log("🔄 Recovery session started:", recoverySession);
        setSession(recoverySession);
        
        // The device should respond with PinMatrixRequest
        setState('pin-first');
      } catch (error) {
        console.error("❌ Failed to start recovery:", error);
        setError(`Failed to start recovery: ${error}`);
        setState('error');
        onError(`Failed to start recovery: ${error}`);
      }
    };
    
    startRecovery();
  }, [deviceId, wordCount, passphraseProtection, deviceLabel, state, session, onError]);

  // PIN handling functions
  const handlePinPress = useCallback((position: PinPosition) => {
    if (pinPositions.length < 8 && !isProcessing) {
      setPinPositions(prev => [...prev, position]);
    }
  }, [pinPositions.length, isProcessing]);

  const handlePinBackspace = useCallback(() => {
    if (!isProcessing) {
      setPinPositions(prev => prev.slice(0, -1));
    }
  }, [isProcessing]);

  const handlePinSubmit = useCallback(async () => {
    if (pinPositions.length === 0 || !session || isProcessing) return;
    
    console.log("🔄 Submitting recovery PIN for state:", state);
    setIsProcessing(true);
    
    try {
      // Send PIN to backend using recovery-specific command
      const result = await invoke<RecoveryProgress>('send_recovery_pin_response', {
        sessionId: session.session_id,
        positions: Array.from(pinPositions)
      });
      
      console.log("🔄 Recovery PIN response result:", result);
      
      // Clear PIN for next entry if needed
      setPinPositions([]);
      
      // Handle the response based on the error field which contains next state info
      if (result.error === 'pin_confirm') {
        console.log("🔄 Device requesting PIN confirmation");
        setState('pin-confirm');
      } else if (result.error === 'button_confirm') {
        console.log("🔄 Device requesting button confirmation");
        setState('button-confirm');
        // Automatically send button ack after a moment
        setTimeout(async () => {
          try {
            await invoke('send_button_ack', { deviceId });
            setState('phrase-entry');
            setCurrentWord(result.word_pos);
            setCurrentChar(result.character_pos);
          } catch (error) {
            console.error("Failed to send button ack:", error);
          }
        }, 500);
      } else if (result.error === 'phrase_entry') {
        console.log("🔄 Device ready for phrase entry");
        setState('phrase-entry');
        setCurrentWord(result.word_pos);
        setCurrentChar(result.character_pos);
      } else if (result.is_complete) {
        console.log("🔄 Recovery completed during PIN setup");
        setState('complete');
        onComplete();
      } else {
        // Default progression for PIN creation flow
        if (state === 'pin-first') {
          setState('pin-confirm');
        } else {
          setState('button-confirm');
        }
      }
    } catch (error) {
      console.error("❌ Recovery PIN submission failed:", error);
      setError(`PIN submission failed: ${error}`);
    } finally {
      setIsProcessing(false);
    }
  }, [pinPositions, session, isProcessing, state, deviceId, onComplete]);

  // Character entry handling
  const handleCharacterInput = async (index: number, value: string) => {
    if (isProcessing || state !== 'phrase-entry') return;
    
    const letter = value.replace(/[^a-zA-Z]/g, '').toLowerCase();
    if (letter && letter !== value.toLowerCase()) {
      return;
    }
    
    const newInputs = [...characterInputs];
    newInputs[index] = letter;
    setCharacterInputs(newInputs);
    
    if (letter && session) {
      setIsProcessing(true);
      try {
        const result = await invoke<RecoveryProgress>('send_recovery_character', {
          sessionId: session.session_id,
          character: letter,
          action: null,
        });
        
        setCurrentWord(result.word_pos);
        setCurrentChar(result.character_pos);
        setIsAutoCompleted(result.auto_completed);
        
        if (result.is_complete) {
          setState('complete');
          onComplete();
        }
      } catch (error) {
        console.error('Failed to send character:', error);
        onError(`Failed to send character: ${error}`);
      } finally {
        setIsProcessing(false);
      }
    }
  };

  const handleNextWord = async () => {
    if (isProcessing || !session) return;
    
    setIsProcessing(true);
    try {
      const result = await invoke<RecoveryProgress>('send_recovery_character', {
        sessionId: session.session_id,
        character: null,
        action: 'Space',
      });
      
      setCurrentWord(result.word_pos);
      setCurrentChar(result.character_pos);
      setCharacterInputs(['', '', '', '']);
      setIsAutoCompleted(false);
      
      if (result.is_complete) {
        setState('complete');
        onComplete();
      }
    } catch (error) {
      console.error('Failed to move to next word:', error);
      onError(`Failed to move to next word: ${error}`);
    } finally {
      setIsProcessing(false);
    }
  };

  const handleRecoveryComplete = async () => {
    if (isProcessing || !session) return;
    
    setIsProcessing(true);
    try {
      const result = await invoke<RecoveryProgress>('send_recovery_character', {
        sessionId: session.session_id,
        character: null,
        action: 'Done',
      });
      
      if (result.is_complete) {
        setState('complete');
        onComplete();
      }
    } catch (error) {
      console.error('Failed to complete recovery:', error);
      onError(`Failed to complete recovery: ${error}`);
    } finally {
      setIsProcessing(false);
    }
  };

  // Render PIN entry UI
  const renderPinEntry = () => {
    const isConfirm = state === 'pin-confirm';
    
    return (
      <VStack gap={6}>
        <Heading size="lg" textAlign="center">
          {isConfirm ? 'Confirm Recovery PIN' : 'Create Recovery PIN'}
        </Heading>
        
        <Text color="gray.400" textAlign="center">
          {isConfirm 
            ? 'Re-enter your PIN to confirm it matches.'
            : 'Create a PIN to secure your recovered wallet. Use the layout shown on your device.'}
        </Text>
        
        {/* PIN dots display */}
        <Box
          p={4}
          bg="gray.700"
          borderRadius="lg"
          borderWidth="2px"
          borderColor={pinPositions.length === 4 ? "blue.500" : "gray.600"}
        >
          <HStack gap={2} justify="center">
            {Array.from({ length: Math.max(4, pinPositions.length + 1) }, (_, i) => (
              <Box
                key={i}
                w="12px"
                h="12px"
                borderRadius="full"
                bg={i < pinPositions.length ? "blue.400" : "gray.500"}
                opacity={i < pinPositions.length ? 1 : 0.5}
              />
            ))}
          </HStack>
        </Box>
        
        {/* PIN matrix */}
        <SimpleGrid columns={3} gap={4} w="250px">
          {PIN_MATRIX_LAYOUT.map((position) => (
            <Button
              key={position}
              onClick={() => handlePinPress(position as PinPosition)}
              size="lg"
              h="60px"
              bg="gray.700"
              borderColor="gray.600"
              borderWidth="1px"
              _hover={{
                bg: "gray.600",
                borderColor: "blue.500",
              }}
              disabled={isProcessing || pinPositions.length >= 8}
            >
              <Icon as={FaCircle} boxSize={4} />
            </Button>
          ))}
        </SimpleGrid>
        
        {/* Action buttons */}
        <HStack gap={4} w="full">
          <Button
            onClick={handlePinBackspace}
            variant="outline"
            size="lg"
            flex={1}
            disabled={isProcessing || pinPositions.length === 0}
          >
            Clear
          </Button>
          
          <Button
            onClick={handlePinSubmit}
            colorScheme="blue"
            size="lg"
            flex={2}
            disabled={isProcessing || pinPositions.length === 0}
          >
            {isProcessing ? 'Processing...' : isConfirm ? 'Confirm PIN' : 'Set PIN'}
          </Button>
        </HStack>
      </VStack>
    );
  };

  // Render phrase entry UI
  const renderPhraseEntry = () => {
    const progressPercent = ((currentWord + 1) / wordCount) * 100;
    const canProceed = currentChar >= 3 || isAutoCompleted;
    
    return (
      <VStack gap={6}>
        <Heading size="lg" textAlign="center">
          Enter Your Recovery Sentence
        </Heading>
        
        <Text fontSize="sm" color="gray.400" textAlign="center">
          Using the scrambled keyboard legend on your KeepKey, enter the first 4 letters of each word.
        </Text>
        
        {/* Progress */}
        <Box w="100%">
          <HStack justify="space-between" mb={2}>
            <Text fontSize="sm" color="gray.300">
              Word {currentWord + 1} of {wordCount}
            </Text>
            <Text fontSize="sm" color="gray.300">
              {Math.round(progressPercent)}% Complete
            </Text>
          </HStack>
          <Box
            w="100%"
            h="6px"
            bg="gray.700"
            borderRadius="md"
            overflow="hidden"
          >
            <Box
              h="100%"
              bg="blue.500"
              borderRadius="md"
              transition="width 0.3s"
              style={{ width: `${progressPercent}%` }}
            />
          </Box>
        </Box>
        
        {/* Character inputs */}
        <HStack gap={4}>
          {characterInputs.map((value, index) => (
            <Input
              key={index}
              value={value}
              onChange={(e) => handleCharacterInput(index, e.target.value)}
              maxLength={1}
              w="60px"
              h="60px"
              textAlign="center"
              fontSize="2xl"
              fontWeight="bold"
              bg={index < currentChar ? "blue.600" : "gray.700"}
              borderColor={index === currentChar ? "blue.400" : "gray.600"}
              disabled={isProcessing || index !== currentChar}
            />
          ))}
        </HStack>
        
        {isAutoCompleted && (
          <Text fontSize="sm" color="green.400" textAlign="center">
            ✓ Word auto-completed by device
          </Text>
        )}
        
        {/* Action buttons */}
        <HStack gap={4} w="100%">
          <Button
            onClick={currentWord < wordCount - 1 ? handleNextWord : handleRecoveryComplete}
            colorScheme="blue"
            size="lg"
            flex={1}
            disabled={isProcessing || !canProceed}
          >
            {isProcessing ? "Processing..." : 
             currentWord < wordCount - 1 ? "Next Word" : "Complete Recovery"}
          </Button>
        </HStack>
      </VStack>
    );
  };

  // Main render
  return (
    <div style={{ 
      position: 'fixed', 
      top: 0, 
      left: 0, 
      right: 0, 
      bottom: 0, 
      backgroundColor: 'rgba(0, 0, 0, 0.8)', 
      display: 'flex', 
      alignItems: 'center', 
      justifyContent: 'center', 
      zIndex: 9999 
    }}>
      <Box
        maxW="lg"
        bg="gray.800"
        color="white"
        p={8}
        borderRadius="lg"
        boxShadow="xl"
        w="90%"
      >
        {state === 'initializing' && (
          <VStack gap={4}>
            <Heading size="lg">Starting Recovery...</Heading>
            <Text color="gray.400">Initializing device recovery process</Text>
          </VStack>
        )}
        
        {(state === 'pin-first' || state === 'pin-confirm') && renderPinEntry()}
        
        {state === 'button-confirm' && (
          <VStack gap={4}>
            <Heading size="lg">Confirm on Device</Heading>
            <Text color="gray.400">Please confirm the recovery on your KeepKey device</Text>
          </VStack>
        )}
        
        {state === 'phrase-entry' && renderPhraseEntry()}
        
        {state === 'complete' && (
          <VStack gap={4}>
            <Heading size="lg">Recovery Complete!</Heading>
            <Text color="gray.400">Your wallet has been successfully recovered</Text>
          </VStack>
        )}
        
        {state === 'error' && (
          <VStack gap={4}>
            <Heading size="lg" color="red.400">Recovery Failed</Heading>
            <Text color="gray.400">{error}</Text>
            {onBack && (
              <Button onClick={onBack} variant="outline" size="lg">
                Try Again
              </Button>
            )}
          </VStack>
        )}
        
        {onBack && state !== 'error' && state !== 'complete' && (
          <Box mt={4} textAlign="center">
            <Button
              onClick={onBack}
              variant="ghost"
              size="sm"
              color="gray.400"
            >
              Cancel Recovery
            </Button>
          </Box>
        )}
      </Box>
    </div>
  );
} 