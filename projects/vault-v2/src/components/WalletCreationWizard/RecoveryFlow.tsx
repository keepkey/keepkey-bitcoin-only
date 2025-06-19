import { useState, useEffect, useCallback, useRef } from "react";
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
import { FaCircle, FaBackspace, FaCheckCircle, FaExclamationTriangle } from "react-icons/fa";
import { PinPosition, PIN_MATRIX_LAYOUT } from "../../types/pin";
import confetti from 'canvas-confetti';

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
  | 'character-success'
  | 'character-failure'
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
  const [lastCharacterResult, setLastCharacterResult] = useState<'success' | 'failure' | null>(null);
  const [feedbackMessage, setFeedbackMessage] = useState<string>('');
  const [isRecoveryLocked, setIsRecoveryLocked] = useState(false);

  // Input refs for automatic focus management
  const inputRefs = [
    useRef<HTMLInputElement>(null),
    useRef<HTMLInputElement>(null),
    useRef<HTMLInputElement>(null),
    useRef<HTMLInputElement>(null),
  ];

  console.log("🔄 RecoveryFlow - Current state:", state);

  // Confetti animation for success
  const triggerConfetti = () => {
    const duration = 3000;
    const animationEnd = Date.now() + duration;
    const defaults = { startVelocity: 30, spread: 360, ticks: 60, zIndex: 0 };

    function randomInRange(min: number, max: number) {
      return Math.random() * (max - min) + min;
    }

    const interval = setInterval(function() {
      const timeLeft = animationEnd - Date.now();

      if (timeLeft <= 0) {
        return clearInterval(interval);
      }

      const particleCount = 50 * (timeLeft / duration);
      confetti(Object.assign({}, defaults, { 
        particleCount,
        origin: { x: randomInRange(0.1, 0.3), y: Math.random() - 0.2 },
        colors: ['#4FD1C7', '#38B2AC', '#319795', '#2C7A7B']
      }));
      confetti(Object.assign({}, defaults, { 
        particleCount,
        origin: { x: randomInRange(0.7, 0.9), y: Math.random() - 0.2 },
        colors: ['#68D391', '#48BB78', '#38A169', '#2F855A']
      }));
    }, 250);
  };

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
        setIsRecoveryLocked(false); // Unlock on critical error
        onError(`Failed to start recovery: ${error}`);
      }
    };
    
    startRecovery();
  }, [deviceId, wordCount, passphraseProtection, deviceLabel, state, session, onError]);

  // Focus management for phrase entry
  useEffect(() => {
    if (state === 'phrase-entry' && currentChar >= 0 && currentChar < inputRefs.length) {
      // Focus the current character input
      const currentInputRef = inputRefs[currentChar];
      if (currentInputRef.current) {
        // Small delay to ensure the input is enabled before focusing
        setTimeout(() => {
          currentInputRef.current?.focus();
        }, 100);
      }
    }
  }, [currentChar, state]);

  // Clear character inputs when moving to next word or starting phrase entry
  useEffect(() => {
    if (state === 'phrase-entry') {
      // Reset character inputs when starting a new word or entering phrase entry state
      setCharacterInputs(['', '', '', '']);
    }
  }, [currentWord, state]);

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
            setIsRecoveryLocked(true); // Lock the UI during recovery phrase entry
          } catch (error) {
            console.error("Failed to send button ack:", error);
          }
        }, 500);
      } else if (result.error === 'phrase_entry') {
        console.log("🔄 Device ready for phrase entry");
        setState('phrase-entry');
        setCurrentWord(result.word_pos);
        setCurrentChar(result.character_pos);
        setIsRecoveryLocked(true); // Lock the UI during recovery phrase entry
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
      setIsRecoveryLocked(false); // Unlock on PIN error so user can retry
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
        
        console.log('🔄 Character result:', result);
        
        // Check if this was successful or failed
        if (result.error && result.error.includes('Failure')) {
          // Check if this is a final failure (recovery completely failed)
          if (currentWord >= wordCount - 1 && currentChar >= 3) {
            // This is a complete recovery failure
            setLastCharacterResult('failure');
            setFeedbackMessage('Recovery failed - the seed phrase was incorrect');
            setState('character-failure');
            setIsRecoveryLocked(false); // Unlock so user can try again
            
            // Don't auto-return to input - let user decide what to do
          } else {
            // This is just an individual character failure
            setLastCharacterResult('failure');
            setFeedbackMessage('Incorrect character - please try again');
            setState('character-failure');
            
            // Show failure feedback for 1.5 seconds then return to input
            setTimeout(() => {
              setState('phrase-entry');
              setLastCharacterResult(null);
              setFeedbackMessage('');
              // Keep recovery locked during individual character failure
            }, 1500);
          }
        } else {
          // Character was accepted - update state and continue
          setCurrentWord(result.word_pos);
          setCurrentChar(result.character_pos);
          setIsAutoCompleted(result.auto_completed);
          
          if (result.is_complete) {
            // ONLY show big success animation when ENTIRE recovery is complete
            triggerConfetti();
            setLastCharacterResult('success');
            setFeedbackMessage('Recovery completed successfully!');
            setState('character-success');
            setIsRecoveryLocked(false); // Unlock UI on completion
            
            setTimeout(() => {
              setState('complete');
              onComplete();
            }, 3000); // Let them enjoy the confetti longer
          } else {
            // Just continue normally for individual characters - no animation spam
            setState('phrase-entry');
          }
        }
      } catch (error) {
        console.error('Failed to send character:', error);
        setLastCharacterResult('failure');
        setFeedbackMessage(`Failed to send character: ${error}`);
        setState('character-failure');
        setIsRecoveryLocked(false); // Unlock on communication error so user can exit/retry
        
        // Don't auto-return on communication errors - let user decide
      } finally {
        setIsProcessing(false);
      }
    }
  };

  // Handle delete/backspace - communicates with device to go back
  const handleDelete = async () => {
    if (isProcessing || !session || state !== 'phrase-entry') return;
    
    // Only allow delete if we're not at the very beginning
    if (currentWord === 0 && currentChar === 0) return;
    
    setIsProcessing(true);
    
    try {
      const result = await invoke<RecoveryProgress>('send_recovery_character', {
        sessionId: session.session_id,
        character: null,
        action: 'Delete',
      });
      
      console.log('🔄 Delete result:', result);
      
      // Delete was successful - just update state and continue
      setCurrentWord(result.word_pos);
      setCurrentChar(result.character_pos);
      setIsAutoCompleted(result.auto_completed);
      
      // Update local input state to reflect the new position
      const newInputs = [...characterInputs];
      if (result.character_pos < newInputs.length) {
        newInputs[result.character_pos] = '';
        // Clear any inputs after the current position
        for (let i = result.character_pos + 1; i < newInputs.length; i++) {
          newInputs[i] = '';
        }
      }
      setCharacterInputs(newInputs);
      
      if (result.is_complete) {
        // ONLY show big success animation when ENTIRE recovery is complete
        triggerConfetti();
        setLastCharacterResult('success');
        setFeedbackMessage('Recovery completed successfully!');
        setState('character-success');
        setIsRecoveryLocked(false); // Unlock UI on completion
        
        setTimeout(() => {
          setState('complete');
          onComplete();
        }, 3000);
      } else {
        // Just continue normally for delete operations - no animation spam
        setState('phrase-entry');
      }
    } catch (error) {
      console.error('Failed to delete character:', error);
      setLastCharacterResult('failure');
      setFeedbackMessage(`Failed to delete character: ${error}`);
      setState('character-failure');
      setIsRecoveryLocked(false); // Unlock on delete error so user can exit/retry
      
      // Don't auto-return on delete errors - let user decide
    } finally {
      setIsProcessing(false);
    }
  };

  // Keyboard navigation for character inputs
  const handleKeyDown = (event: React.KeyboardEvent, index: number) => {
    if (isProcessing || state !== 'phrase-entry') return;

    switch (event.key) {
      case 'Backspace':
        event.preventDefault();
        handleDelete();
        break;
      case 'ArrowLeft':
        event.preventDefault();
        if (index > 0) {
          inputRefs[index - 1]?.current?.focus();
        }
        break;
      case 'ArrowRight':
        event.preventDefault();
        if (index < inputRefs.length - 1) {
          inputRefs[index + 1]?.current?.focus();
        }
        break;
      case 'Enter':
      case ' ':
        event.preventDefault();
        if (currentWord < wordCount - 1) {
          handleNextWord();
        } else {
          handleRecoveryComplete();
        }
        break;
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
      setLastCharacterResult('failure');
      setFeedbackMessage(`Failed to move to next word: ${error}`);
      setState('character-failure');
      setIsRecoveryLocked(false); // Unlock on error so user can exit/retry
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
      setLastCharacterResult('failure');
      setFeedbackMessage(`Failed to complete recovery: ${error}`);
      setState('character-failure');
      setIsRecoveryLocked(false); // Unlock on error so user can exit/retry
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
              ref={inputRefs[index]}
              value={value}
              onChange={(e) => handleCharacterInput(index, e.target.value)}
              onKeyDown={(e) => handleKeyDown(e, index)}
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
            onClick={handleDelete}
            variant="outline"
            size="lg"
            flex={1}
            disabled={isProcessing || (currentWord === 0 && currentChar === 0)}
          >
            <HStack gap={2}>
              <Icon as={FaBackspace} />
              <Text>Delete</Text>
            </HStack>
          </Button>
          
          <Button
            onClick={currentWord < wordCount - 1 ? handleNextWord : handleRecoveryComplete}
            colorScheme="blue"
            size="lg"
            flex={2}
            disabled={isProcessing || !canProceed}
          >
            {isProcessing ? "Processing..." : 
             currentWord < wordCount - 1 ? "Next Word" : "Complete Recovery"}
          </Button>
        </HStack>
      </VStack>
    );
  };



  // Render character success feedback
  const renderCharacterSuccess = () => (
    <VStack gap={6}>
      <Box
        transform="scale(1.2)"
        transition="transform 0.3s ease-in-out"
        style={{
          animation: 'bounce 0.8s ease-in-out',
          filter: 'drop-shadow(0 0 20px rgba(72, 187, 120, 0.5))'
        }}
      >
        <Icon as={FaCheckCircle} boxSize={16} color="green.400" />
      </Box>
      <Heading size="lg" textAlign="center" color="green.400">
        Success!
      </Heading>
      <Text color="gray.300" textAlign="center">
        {feedbackMessage}
      </Text>
    </VStack>
  );

  // Render character failure feedback
  const renderCharacterFailure = () => {
    const isCompleteFailure = !isRecoveryLocked; // Complete failures unlock the UI
    
    return (
      <VStack gap={6}>
        <Box
          style={{
            animation: 'shake 0.5s ease-in-out',
            filter: 'drop-shadow(0 0 20px rgba(245, 101, 101, 0.5))'
          }}
        >
          <Icon as={FaExclamationTriangle} boxSize={16} color="red.400" />
        </Box>
        <Heading size="lg" textAlign="center" color="red.400">
          {isCompleteFailure ? 'Recovery Failed' : 'Try Again'}
        </Heading>
        <Text color="gray.300" textAlign="center">
          {feedbackMessage}
        </Text>
        
        {isCompleteFailure && (
          <VStack gap={3} w="100%">
            <Button
              onClick={() => {
                // Reset recovery state and restart
                setState('initializing');
                setCurrentWord(0);
                setCurrentChar(0);
                setCharacterInputs(['', '', '', '']);
                setIsAutoCompleted(false);
                setLastCharacterResult(null);
                setFeedbackMessage('');
                setIsRecoveryLocked(false);
              }}
              colorScheme="blue"
              size="lg"
              w="100%"
            >
              Try Recovery Again
            </Button>
            
            {onBack && (
              <Button
                onClick={onBack}
                variant="outline"
                size="lg"
                w="100%"
              >
                Cancel Recovery
              </Button>
            )}
          </VStack>
        )}
      </VStack>
    );
  };

  // Prevent escape key and other interruptions when recovery is locked
  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if (isRecoveryLocked && event.key === 'Escape') {
        event.preventDefault();
        event.stopPropagation();
      }
    };

    if (isRecoveryLocked) {
      document.addEventListener('keydown', handleKeyDown, true);
      return () => document.removeEventListener('keydown', handleKeyDown, true);
    }
  }, [isRecoveryLocked]);

  // Main render
  return (
    <>
      <style>
        {`
          @keyframes bounce {
            0%, 20%, 53%, 80%, 100% { transform: translate3d(0,0,0) scale(1.2); }
            40%, 43% { transform: translate3d(0, -15px, 0) scale(1.3); }
            70% { transform: translate3d(0, -7px, 0) scale(1.25); }
            90% { transform: translate3d(0, -2px, 0) scale(1.22); }
          }
          @keyframes shake {
            0%, 100% { transform: translateX(0); }
            10%, 30%, 50%, 70%, 90% { transform: translateX(-8px); }
            20%, 40%, 60%, 80% { transform: translateX(8px); }
          }
        `}
      </style>
      <div 
        style={{ 
          position: 'fixed', 
          top: 0, 
          left: 0, 
          right: 0, 
          bottom: 0, 
          backgroundColor: isRecoveryLocked ? 'rgba(0, 0, 0, 0.95)' : 'rgba(0, 0, 0, 0.8)', 
          display: 'flex', 
          alignItems: 'center', 
          justifyContent: 'center', 
          zIndex: 9999 
        }}
        onClick={(e) => {
          // Prevent clicking outside to close when recovery is locked
          if (isRecoveryLocked) {
            e.preventDefault();
            e.stopPropagation();
          }
        }}
      >
        <Box
          maxW="lg"
          bg="gray.800"
          color="white"
          p={8}
          borderRadius="lg"
          boxShadow="xl"
          w="90%"
          onClick={(e) => e.stopPropagation()} // Prevent event bubbling
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
        
        {state === 'character-success' && renderCharacterSuccess()}
        
        {state === 'character-failure' && renderCharacterFailure()}
        
        {state === 'complete' && (
          <VStack gap={4}>
            <Icon as={FaCheckCircle} boxSize={16} color="green.400" />
            <Heading size="lg" color="green.400">Recovery Complete!</Heading>
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
        
        {onBack && state !== 'error' && state !== 'complete' && !isRecoveryLocked && (
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
        
        {isRecoveryLocked && (
          <Box mt={4} textAlign="center">
            <Text fontSize="xs" color="yellow.400" textAlign="center">
              ⚠️ Recovery in progress - please do not disconnect your device
            </Text>
          </Box>
        )}
      </Box>
    </div>
    </>
  );
} 