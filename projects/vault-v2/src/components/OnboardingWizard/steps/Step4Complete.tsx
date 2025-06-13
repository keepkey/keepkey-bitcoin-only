import { Box, Button, Card, HStack, Text, VStack, Icon } from "@chakra-ui/react";
import { FaCheckCircle, FaRocket } from "react-icons/fa";

interface StepProps {
  onNext: () => void;
  onPrevious: () => void;
}

export function Step4Complete({ onNext, onPrevious }: StepProps) {
  return (
    <VStack align="center" justify="center" minH="400px" gap={6}>
      <Icon asChild color="green.500" boxSize={20}>
        <FaCheckCircle />
      </Icon>
      
      <Card.Root width="full" maxWidth="md" bg="gray.900" borderColor="gray.700">
        <Card.Body>
          <VStack gap={4} align="center" textAlign="center">
            <Text fontSize="2xl" fontWeight="bold" color="green.400">
              Setup Complete!
            </Text>
            <Text color="gray.400">
              You're all set to start using KeepKey Desktop.
            </Text>
            <Text color="gray.400">
              Your preferences have been saved and you can now connect your KeepKey device.
            </Text>
            
            <Box w="full" mt={4} p={4} bg="gray.800" borderRadius="md">
              <HStack gap={3} justify="center">
                <Icon asChild color="yellow.400">
                  <FaRocket />
                </Icon>
                <Text color="white" fontWeight="medium">Next Steps</Text>
              </HStack>
              <Text color="gray.400" fontSize="sm" mt={2}>
                Connect your KeepKey device via USB to get started with managing your crypto assets securely.
              </Text>
            </Box>
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
        <Button 
          colorScheme="green" 
          onClick={onNext}
          size="lg"
          px={8}
        >
          Get Started
        </Button>
      </HStack>
    </VStack>
  );
} 