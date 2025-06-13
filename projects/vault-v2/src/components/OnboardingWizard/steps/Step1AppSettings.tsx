import { Box, Button, Card, HStack, Text, VStack, Icon } from "@chakra-ui/react";
import { FaCog, FaPaintBrush, FaBell } from "react-icons/fa";

interface StepProps {
  onNext: () => void;
  onPrevious: () => void;
}

export function Step1AppSettings({ onNext, onPrevious }: StepProps) {
  return (
    <VStack align="center" justify="center" minH="400px" gap={6}>
      <Card.Root width="full" maxWidth="md" bg="gray.900" borderColor="gray.700">
        <Card.Header bg="gray.850">
          <HStack justify="center" gap={3}>
            <Icon asChild color="green.500">
              <FaCog />
            </Icon>
            <Text fontSize="xl" fontWeight="bold" color="white">
              App Settings
            </Text>
          </HStack>
        </Card.Header>
        <Card.Body>
          <VStack gap={4} align="start">
            <Text color="gray.400">
              Configure your application preferences here.
            </Text>
            
            <Box w="full" p={4} bg="gray.800" borderRadius="md">
              <HStack gap={3} mb={2}>
                <Icon asChild color="blue.400">
                  <FaPaintBrush />
                </Icon>
                <Text color="white" fontWeight="medium">Theme</Text>
              </HStack>
              <Text color="gray.400" fontSize="sm">
                Dark theme is currently active
              </Text>
            </Box>
            
            <Box w="full" p={4} bg="gray.800" borderRadius="md">
              <HStack gap={3} mb={2}>
                <Icon asChild color="yellow.400">
                  <FaBell />
                </Icon>
                <Text color="white" fontWeight="medium">Notifications</Text>
              </HStack>
              <Text color="gray.400" fontSize="sm">
                Get notified about important updates
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
        <Button colorScheme="green" onClick={onNext}>
          Next
        </Button>
      </HStack>
    </VStack>
  );
} 