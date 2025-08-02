import { VStack, Text, Button, Box, Icon, HStack } from "@chakra-ui/react";
import { FaPlus, FaKey } from "react-icons/fa";

interface Step1CreateOrRecoverProps {
  onFlowTypeSelect: (type: 'create' | 'recover') => void;
}

export function Step1CreateOrRecover({ onFlowTypeSelect }: Step1CreateOrRecoverProps) {
  return (
    <VStack gap={8} w="100%">
      <VStack gap={2}>
        <Text fontSize="2xl" fontWeight="bold" color="white">
          Choose Setup Method
        </Text>
        <Text fontSize="md" color="gray.400">
          Create a new wallet or restore an existing one
        </Text>
      </VStack>

      <HStack gap={6} w="100%">
        {/* Create New Wallet */}
        <Box
          flex={1}
          p={6}
          borderRadius="lg"
          borderWidth="2px"
          borderColor="transparent"
          bg="gray.700"
          cursor="pointer"
          transition="all 0.2s"
          _hover={{
            borderColor: "orange.500",
            bg: "gray.650",
            transform: "translateY(-2px)",
          }}
          onClick={() => onFlowTypeSelect('create')}
        >
          <VStack gap={4}>
            <Box
              p={4}
              borderRadius="full"
              bg="orange.500"
              color="white"
            >
              <Icon as={FaPlus} boxSize={8} />
            </Box>
            <VStack gap={2}>
              <Text fontSize="xl" fontWeight="bold" color="white">
                Create New Wallet
              </Text>
              <Text fontSize="sm" color="gray.400" textAlign="center">
                Generate a new recovery phrase and set up a fresh wallet
              </Text>
            </VStack>
            <Button
              colorScheme="orange"
              size="lg"
              w="100%"
              onClick={(e) => {
                e.stopPropagation();
                onFlowTypeSelect('create');
              }}
            >
              Create New
            </Button>
          </VStack>
        </Box>

        {/* Recover Existing Wallet */}
        <Box
          flex={1}
          p={6}
          borderRadius="lg"
          borderWidth="2px"
          borderColor="transparent"
          bg="gray.700"
          cursor="pointer"
          transition="all 0.2s"
          _hover={{
            borderColor: "blue.500",
            bg: "gray.650",
            transform: "translateY(-2px)",
          }}
          onClick={() => onFlowTypeSelect('recover')}
        >
          <VStack gap={4}>
            <Box
              p={4}
              borderRadius="full"
              bg="blue.500"
              color="white"
            >
              <Icon as={FaKey} boxSize={8} />
            </Box>
            <VStack gap={2}>
              <Text fontSize="xl" fontWeight="bold" color="white">
                Recover Wallet
              </Text>
              <Text fontSize="sm" color="gray.400" textAlign="center">
                Restore your wallet using an existing recovery phrase
              </Text>
            </VStack>
            <Button
              colorScheme="blue"
              size="lg"
              w="100%"
              onClick={(e) => {
                e.stopPropagation();
                onFlowTypeSelect('recover');
              }}
            >
              Recover
            </Button>
          </VStack>
        </Box>
      </HStack>
    </VStack>
  );
}