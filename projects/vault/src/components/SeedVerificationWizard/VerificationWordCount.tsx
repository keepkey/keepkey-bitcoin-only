import React from 'react';
import { VStack, Text, Button, HStack, Icon } from '@chakra-ui/react';
import { FaKey } from 'react-icons/fa';

interface VerificationWordCountProps {
  onSelectWordCount: (wordCount: number) => void;
  isLoading: boolean;
}

const VerificationWordCount: React.FC<VerificationWordCountProps> = ({
  onSelectWordCount,
  isLoading
}) => {
  const wordCountOptions = [
    { count: 12, description: "Most common for KeepKey devices" },
    { count: 18, description: "Enhanced security option" },
    { count: 24, description: "Maximum security option" }
  ];

  return (
    <VStack gap={6} align="stretch">
      <VStack gap={3}>
        <Icon as={FaKey} boxSize={8} color="blue.500" />
        <Text textAlign="center" color="gray.500">
          Select the number of words in your recovery phrase that you want to verify.
        </Text>
      </VStack>

      <VStack gap={3}>
        {wordCountOptions.map(({ count, description }) => (
          <Button
            key={count}
            size="lg"
            variant="outline"
            width="full"
            height="auto"
            p={4}
            onClick={() => onSelectWordCount(count)}
            loading={isLoading}
            _hover={{ bg: 'gray.50', borderColor: 'blue.300' }}
          >
            <VStack gap={1}>
              <Text fontSize="xl" fontWeight="bold">
                {count} Words
              </Text>
              <Text fontSize="sm" color="gray.500">
                {description}
              </Text>
            </VStack>
          </Button>
        ))}
      </VStack>

      <Text fontSize="xs" textAlign="center" color="gray.400">
        Your recovery phrase length was determined when you first initialized your device.
      </Text>
    </VStack>
  );
};

export default VerificationWordCount; 