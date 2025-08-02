import { VStack, Text, Icon, Box } from "@chakra-ui/react";
import { FaBitcoin } from "react-icons/fa";
import { useEffect } from "react";

interface Step0WelcomeProps {
  onNext: () => void;
}

export function Step0Welcome({ onNext }: Step0WelcomeProps) {
  // Auto-advance after 3 seconds
  useEffect(() => {
    const timer = setTimeout(() => {
      onNext();
    }, 3000);

    return () => clearTimeout(timer);
  }, [onNext]);

  return (
    <VStack gap={6} textAlign="center" w="100%">
      <Box position="relative">
        <Icon 
          as={FaBitcoin} 
          boxSize={20} 
          color="orange.500"
          animation="pulse 2s ease-in-out infinite"
        />
      </Box>
      
      <VStack gap={3}>
        <Text fontSize="3xl" fontWeight="bold" color="white">
          Welcome to KeepKey
        </Text>
        <Text fontSize="xl" color="orange.500">
          Bitcoin-Only Edition
        </Text>
        <Text fontSize="md" color="gray.400" maxW="400px">
          The secure hardware wallet focused exclusively on Bitcoin
        </Text>
      </VStack>

      <Text fontSize="sm" color="gray.500">
        Setting up your device...
      </Text>
    </VStack>
  );
}