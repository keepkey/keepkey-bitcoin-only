import { VStack, Text, Icon, Box, Button } from "@chakra-ui/react";
import { FaCheckCircle } from "react-icons/fa";
import { useEffect } from "react";

interface Step5CompleteProps {
  wizardData: any;
  flowType: 'create' | 'recover' | null;
  onNext: () => void;
}

export function Step5Complete({ wizardData, flowType, onNext }: Step5CompleteProps) {
  // Auto-complete after 3 seconds
  useEffect(() => {
    const timer = setTimeout(() => {
      onNext();
    }, 3000);

    return () => clearTimeout(timer);
  }, [onNext]);

  const isRecovery = flowType === 'recover';

  return (
    <VStack gap={6} textAlign="center" w="100%">
      <Box position="relative">
        <Icon 
          as={FaCheckCircle} 
          boxSize={20} 
          color="green.500"
          animation="pulse 2s ease-in-out infinite"
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
      >
        Start Using KeepKey
      </Button>
    </VStack>
  );
}