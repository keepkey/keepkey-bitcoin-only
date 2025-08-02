import {
  Button,
  Text,
  VStack,
  Box,
  HStack,
  SimpleGrid,
  Input,
  Icon,
  Heading,
  Image,
  Flex,
} from "@chakra-ui/react";
import { useState, useCallback, useRef, useEffect } from "react";
import { FaCircle, FaChevronDown, FaChevronRight, FaInfoCircle } from "react-icons/fa";
import cipherImage from "../../assets/onboarding/cipher.png";
import { PinService } from "../../services/pinService";
import { PinCreationSession, PinStep, PinPosition, PIN_MATRIX_LAYOUT } from "../../types/pin";

interface DevicePinHorizontalProps {
  deviceId: string;
  deviceLabel?: string;
  mode: 'create' | 'confirm';
  onComplete: (session: PinCreationSession) => void;
  onBack?: () => void;
  isLoading?: boolean;
  error?: string | null;
}

export function DevicePinHorizontal({ 
  deviceId, 
  deviceLabel, 
  mode, 
  onComplete, 
  onBack, 
  isLoading = false, 
  error 
}: DevicePinHorizontalProps) {
  const [positions, setPositions] = useState<PinPosition[]>([]);
  const [showMoreInfo, setShowMoreInfo] = useState(false);
  const [session, setSession] = useState<PinCreationSession | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [stepError, setStepError] = useState<string | null>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  // Dynamic title and description based on session state
  const getTitle = () => {
    if (!session) return 'Initializing PIN Setup...';
    switch (session.current_step) {
      case PinStep.AwaitingFirst:
        return 'Create Your PIN';
      case PinStep.AwaitingSecond:
        return 'Confirm Your PIN';
      default:
        return 'PIN Setup';
    }
  };

  const getDescription = () => {
    if (!session) return 'Starting PIN creation session with your KeepKey device...';
    switch (session.current_step) {
      case PinStep.AwaitingFirst:
        return 'Use PIN layout shown on your device to find the location to press on the PIN pad.';
      case PinStep.AwaitingSecond:
        return 'Re-enter your PIN to confirm it matches.';
      default:
        return 'Processing PIN setup...';
    }
  };

  // Initialize PIN creation session
  useEffect(() => {
    const initializeSession = async () => {
      if (!session && mode === 'create') {
        try {
          const newSession = await PinService.startPinCreation(deviceId, deviceLabel);
          setSession(newSession);
          setStepError(null);
        } catch (error) {
          console.error("PIN creation initialization error:", error);
          setStepError(`Failed to start PIN creation: ${error}`);
        }
      }
    };

    initializeSession();
  }, [deviceId, mode, session, deviceLabel]);

  useEffect(() => {
    if (inputRef.current) {
      inputRef.current.focus();
    }
  }, []);

  const handlePinPress = useCallback((position: PinPosition) => {
    if (positions.length < 9 && !isLoading && !isSubmitting) {
      setPositions(prev => [...prev, position]);
    }
  }, [positions.length, isLoading, isSubmitting]);

  const handleBackspace = useCallback(() => {
    if (!isLoading && !isSubmitting) {
      setPositions(prev => prev.slice(0, -1));
    }
  }, [isLoading, isSubmitting]);

  const handleSubmit = useCallback(async () => {
    if (positions.length > 0 && !isLoading && !isSubmitting && session) {
      setIsSubmitting(true);
      setStepError(null);
      
      try {
        const updatedSession = await PinService.submitPin(
          session.id, 
          deviceId, 
          positions, 
          session.current_step === PinStep.AwaitingFirst
        );
        
        if (updatedSession.current_step === PinStep.Complete && updatedSession.success) {
          onComplete(updatedSession);
        } else if (updatedSession.current_step === PinStep.AwaitingSecond) {
          setSession(updatedSession);
          setPositions([]);
        }
      } catch (error) {
        console.error("PIN submission error:", error);
        setStepError(`Failed to submit PIN: ${error}`);
      } finally {
        setIsSubmitting(false);
      }
    }
  }, [positions, isLoading, isSubmitting, session, deviceId, onComplete]);

  const handleKeyDown = useCallback((e: React.KeyboardEvent<HTMLInputElement>) => {
    if (isLoading || isSubmitting) return;
    
    if (e.key === 'Backspace') {
      handleBackspace();
    } else if (e.key === 'Enter') {
      handleSubmit();
    } else if (PIN_MATRIX_LAYOUT.includes(Number(e.key) as any)) {
      handlePinPress(Number(e.key) as PinPosition);
    }
  }, [handleBackspace, handleSubmit, handlePinPress, isLoading, isSubmitting]);

  // Generate PIN dots for display
  const maxDotsToShow = Math.max(4, positions.length + (positions.length < 8 ? 1 : 0));
  const pinDots = Array.from({ length: Math.min(maxDotsToShow, 8) }, (_, i) => (
    <Box
      key={i}
      w="12px"
      h="12px"
      borderRadius="full"
      bg={i < positions.length ? "green.400" : i < 4 ? "gray.500" : "gray.600"}
      opacity={i < positions.length ? 1 : i < 4 ? 0.7 : 0.4}
      transition="all 0.2s"
      transform={i === 3 && positions.length === 4 ? "scale(1.1)" : "scale(1)"}
    />
  ));

  return (
    <Box w="100%">
      <Flex
        direction={{ base: "column", lg: "row" }}
        gap={{ base: 4, lg: 8 }}
        align={{ base: "center", lg: "flex-start" }}
        w="100%"
      >
        {/* Left side - PIN Entry */}
        <VStack gap={4} flex={1} w={{ base: "100%", lg: "auto" }}>
          <VStack gap={2}>
            <Heading fontSize={{ base: "xl", md: "2xl" }} fontWeight="bold" color="white" textAlign="center">
              {getTitle()}
            </Heading>
            <Text 
              color="gray.400" 
              textAlign="center"
              fontSize={{ base: "sm", md: "md" }}
              lineHeight="1.6"
            >
              {getDescription()}
            </Text>
          </VStack>

          {/* PIN Dots Display */}
          <Box
            p={4}
            bg="gray.700"
            borderRadius="lg"
            borderWidth="2px"
            borderColor={positions.length === 4 ? "green.500" : "gray.600"}
            transition="all 0.2s"
          >
            <HStack gap={2} justify="center">
              {pinDots}
            </HStack>
          </Box>
          
          {/* PIN Length Hints */}
          <Box minH="20px">
            {positions.length === 0 && (
              <Text fontSize="xs" color="blue.400" textAlign="center">
                💡 We recommend using 4 digits for optimal security
              </Text>
            )}
            {positions.length > 0 && positions.length < 4 && (
              <Text fontSize="xs" color="yellow.400" textAlign="center">
                {4 - positions.length} more digit{4 - positions.length !== 1 ? 's' : ''} recommended
              </Text>
            )}
            {positions.length === 4 && (
              <Text fontSize="xs" color="green.400" textAlign="center">
                ✅ Perfect! 4 digits provides great security
              </Text>
            )}
          </Box>
        </VStack>

        {/* Right side - PIN Pad */}
        <VStack gap={4} flex={1} w={{ base: "100%", lg: "auto" }}>
          {/* Scrambled PIN Grid */}
          <Box bg="gray.700" p={4} borderRadius="lg">
            <SimpleGrid
              columns={3}
              gap={2}
              w="200px"
              mx="auto"
            >
              {PIN_MATRIX_LAYOUT.map((position) => (
                <Button
                  key={position}
                  onClick={() => handlePinPress(position as PinPosition)}
                  size="md"
                  h="50px"
                  bg="gray.600"
                  borderColor="gray.500"
                  borderWidth="1px"
                  color="gray.300"
                  fontSize="lg"
                  fontWeight="bold"
                  _hover={{
                    bg: "gray.500",
                    borderColor: "green.500",
                    transform: "scale(1.05)",
                  }}
                  _active={{
                    bg: "gray.400",
                    transform: "scale(0.95)",
                  }}
                  transition="all 0.15s"
                  disabled={isLoading || isSubmitting || positions.length >= 9}
                >
                  <FaCircle size="10px" />
                </Button>
              ))}
            </SimpleGrid>
          </Box>

          {/* Action Buttons */}
          <HStack gap={3} w="100%">
            <Button
              onClick={handleBackspace}
              variant="outline"
              colorScheme="gray"
              size="lg"
              flex={1}
              isDisabled={positions.length === 0 || isLoading || isSubmitting}
            >
              Backspace
            </Button>
            <Button
              onClick={handleSubmit}
              colorScheme="green"
              size="lg"
              flex={1}
              isLoading={isSubmitting}
              isDisabled={positions.length === 0 || isLoading}
            >
              {session?.current_step === PinStep.AwaitingSecond ? 'Confirm PIN' : 'Set PIN'}
            </Button>
          </HStack>
        </VStack>
      </Flex>

      {/* Hidden input for keyboard support */}
      <Input
        ref={inputRef}
        type="text"
        value=""
        onChange={() => {}}
        onKeyDown={handleKeyDown}
        position="absolute"
        opacity={0}
        pointerEvents="none"
        aria-hidden="true"
      />

      {/* Error display */}
      {(stepError || error) && (
        <Box mt={4}>
          <Text color="red.400" fontSize="sm" textAlign="center">
            {stepError || error}
          </Text>
        </Box>
      )}
    </Box>
  );
}