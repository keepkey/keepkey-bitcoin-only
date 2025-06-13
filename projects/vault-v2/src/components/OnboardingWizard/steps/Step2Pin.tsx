import { Box, Button, Card, HStack, Text, VStack, Icon } from "@chakra-ui/react";
import { FaShieldAlt, FaLock, FaKey } from "react-icons/fa";

interface StepProps {
  onNext: () => void;
  onPrevious: () => void;
}

export function Step2Pin({ onNext, onPrevious }: StepProps) {
  return (
    <VStack align="center" justify="center" minH="400px" gap={6}>
      <Card.Root width="full" maxWidth="md" bg="gray.900" borderColor="gray.700">
        <Card.Header bg="gray.850">
          <HStack justify="center" gap={3}>
            <Icon asChild color="green.500">
              <FaShieldAlt />
            </Icon>
            <Text fontSize="xl" fontWeight="bold" color="white">
              Security Setup
            </Text>
          </HStack>
        </Card.Header>
        <Card.Body>
          <VStack gap={4} align="start">
            <Text color="gray.400">
              Learn about PIN protection and security best practices for your KeepKey device.
            </Text>
            
            <Box w="full" p={4} bg="gray.800" borderRadius="md">
              <HStack gap={3} mb={2}>
                <Icon asChild color="green.400">
                  <FaLock />
                </Icon>
                <Text color="white" fontWeight="medium">PIN Protection</Text>
              </HStack>
              <Text color="gray.400" fontSize="sm">
                Your PIN protects your device from unauthorized access. Choose a PIN that's easy for you to remember but hard for others to guess.
              </Text>
            </Box>
            
            <Box w="full" p={4} bg="gray.800" borderRadius="md">
              <HStack gap={3} mb={2}>
                <Icon asChild color="orange.400">
                  <FaKey />
                </Icon>
                <Text color="white" fontWeight="medium">Recovery Phrase</Text>
              </HStack>
              <Text color="gray.400" fontSize="sm">
                Your recovery phrase is the master backup of your wallet. Keep it safe and never share it with anyone.
              </Text>
            </Box>
            
            <Text color="yellow.400" fontSize="sm" fontStyle="italic">
              Note: You'll set up your PIN directly on your KeepKey device when you connect it.
            </Text>
          </VStack>
        </Card.Body>
      </Card.Root>

      <HStack gap={4}>
        <Button 
          variant="outline" 
          onClick={onPrevious}
          borderColor="gray.600"
          color="gray.300"
          _hover={{ bg: "gray.700" }}
        >
          Previous
        </Button>
        <Button colorScheme="green" onClick={onNext}>
          Next
        </Button>
      </HStack>
    </VStack>
  );
} 