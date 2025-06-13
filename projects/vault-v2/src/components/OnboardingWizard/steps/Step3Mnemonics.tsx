import { Box, Button, Card, HStack, Text, VStack, Icon, List } from "@chakra-ui/react";
import { FaBook, FaExclamationTriangle, FaCheckCircle } from "react-icons/fa";

interface StepProps {
  onNext: () => void;
  onPrevious: () => void;
}

export function Step3Mnemonics({ onNext, onPrevious }: StepProps) {
  return (
    <VStack align="center" justify="center" minH="350px" gap={6}>
      <Card.Root width="full" maxWidth="4xl" bg="gray.900" borderColor="gray.700">
        <Card.Header bg="gray.850">
          <HStack justify="center" gap={3}>
            <Icon asChild color="green.500">
              <FaBook />
            </Icon>
            <Text fontSize="xl" fontWeight="bold" color="white">
              Recovery Phrases
            </Text>
          </HStack>
        </Card.Header>
        <Card.Body>
          <Box width="full">
            <Text color="gray.400" mb={4}>
              Understanding recovery phrases and backup best practices.
            </Text>
            
            <HStack gap={6} alignItems="flex-start">
              <Box flex="1" p={4} bg="red.900" borderRadius="md" borderColor="red.700" borderWidth="1px">
                <HStack gap={3} mb={2}>
                  <Icon asChild color="red.400">
                    <FaExclamationTriangle />
                  </Icon>
                  <Text color="white" fontWeight="medium">Important Security Notice</Text>
                </HStack>
                <Text color="red.200" fontSize="sm">
                  Never share your recovery phrase with anyone. KeepKey staff will never ask for it. Anyone with your recovery phrase can steal your funds.
                </Text>
              </Box>
              
              <Box flex="1" p={4} bg="gray.800" borderRadius="md">
                <Text color="white" fontWeight="medium" mb={2}>What is a Recovery Phrase?</Text>
                <List.Root>
                  <List.Item color="gray.400" fontSize="sm">
                    <Icon asChild color="green.400" mr={2}>
                      <FaCheckCircle />
                    </Icon>
                    A 12, 18, or 24-word backup of your entire wallet
                  </List.Item>
                  <List.Item color="gray.400" fontSize="sm" mt={2}>
                    <Icon asChild color="green.400" mr={2}>
                      <FaCheckCircle />
                    </Icon>
                    The only way to recover your funds if your device is lost or damaged
                  </List.Item>
                  <List.Item color="gray.400" fontSize="sm" mt={2}>
                    <Icon asChild color="green.400" mr={2}>
                      <FaCheckCircle />
                    </Icon>
                    Should be written down and stored in a secure location
                  </List.Item>
                </List.Root>
              </Box>
            </HStack>
            
            <Text color="gray.400" fontSize="sm" mt={4}>
              Your KeepKey will generate a recovery phrase when you initialize it. Make sure you have pen and paper ready.
            </Text>
          </Box>
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
          I Understand
        </Button>
      </HStack>
    </VStack>
  );
} 