import { VStack, Text, Icon, Box, Button } from "@chakra-ui/react";
import { keyframes } from "@emotion/react";
import { FaCheckCircle } from "react-icons/fa";
import { useEffect } from "react";

interface Step5CompleteProps {
  wizardData: any;
  flowType: 'create' | 'recover' | null;
  onNext: () => void;
}

const confettiFall = keyframes`
  0% { transform: translateY(-100vh) rotate(0deg); opacity: 1; }
  100% { transform: translateY(100vh) rotate(720deg); opacity: 0; }
`;

const pulse = keyframes`
  0% { transform: scale(1); }
  50% { transform: scale(1.1); }
  100% { transform: scale(1); }
`;

export function Step5Complete({ wizardData, flowType, onNext }: Step5CompleteProps) {
  // Auto-complete after 5 seconds
  useEffect(() => {
    const timer = setTimeout(() => {
      onNext();
    }, 5000);

    return () => clearTimeout(timer);
  }, [onNext]);

  const isRecovery = flowType === 'recover';

  // Generate confetti pieces
  const confettiColors = ['#10b981', '#f59e0b', '#ef4444', '#3b82f6', '#8b5cf6', '#ec4899'];
  const confettiPieces = Array.from({ length: 50 }, (_, i) => ({
    id: i,
    color: confettiColors[i % confettiColors.length],
    left: `${Math.random() * 100}%`,
    delay: `${Math.random() * 3}s`,
    duration: `${3 + Math.random() * 2}s`
  }));

  return (
    <Box position="relative" w="100%" overflow="hidden">
      {/* Confetti animation */}
      {confettiPieces.map(piece => (
        <Box
          key={piece.id}
          position="absolute"
          w="10px"
          h="10px"
          bg={piece.color}
          left={piece.left}
          top="-10px"
          animation={`${confettiFall} ${piece.duration} linear ${piece.delay} infinite`}
          borderRadius="2px"
          transform="rotate(45deg)"
        />
      ))}
      
      <VStack gap={6} textAlign="center" w="100%" position="relative" zIndex={1}>
        <Box position="relative">
          <Icon 
            as={FaCheckCircle} 
            boxSize={20} 
            color="green.500"
            animation={`${pulse} 2s ease-in-out infinite`}
          />
        </Box>
      
      <VStack gap={3}>
        <Text fontSize="3xl" fontWeight="bold" color="white">
          🎉 {isRecovery ? 'Wallet Recovered!' : 'Wallet Created!'}
        </Text>
        <Text fontSize="lg" color="gray.300">
          Your KeepKey {wizardData.deviceLabel && `"${wizardData.deviceLabel}"`} is ready
        </Text>
        <Text fontSize="md" color="gray.400" maxW="400px">
          {isRecovery 
            ? 'Your wallet has been successfully restored from your recovery phrase'
            : 'Your new Bitcoin wallet is now secure and ready to use'
          }
        </Text>
      </VStack>

        <Button
          colorScheme="green"
          size="lg"
          onClick={onNext}
          _hover={{ transform: "scale(1.05)" }}
        >
          Start Using KeepKey
        </Button>
      </VStack>
    </Box>
  );
}