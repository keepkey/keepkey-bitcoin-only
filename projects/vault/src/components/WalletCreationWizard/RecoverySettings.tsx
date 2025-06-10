import { useState } from "react";
import {
  Box,
  Heading,
  VStack,
  HStack,
  Text,
  Button,
  Stack,
} from "@chakra-ui/react";

interface RecoverySettingsProps {
  onComplete: (settings: RecoverySettings) => void;
  onBack?: () => void;
  isLoading?: boolean;
  error?: string | null;
}

export interface RecoverySettings {
  wordCount: 12 | 18 | 24;
  usePassphrase: boolean;
}

export function RecoverySettings({ 
  onComplete, 
  onBack, 
  isLoading = false, 
  error 
}: RecoverySettingsProps) {
  const [wordCount, setWordCount] = useState<12 | 18 | 24>(12);
  const [usePassphrase, setUsePassphrase] = useState(false);

  const handleSubmit = () => {
    const settings: RecoverySettings = {
      wordCount,
      usePassphrase,
    };
    onComplete(settings);
  };

  const handleWordCountChange = (count: 12 | 18 | 24) => {
    setWordCount(count);
  };

  const handlePassphraseToggle = () => {
    setUsePassphrase(!usePassphrase);
  };

  return (
    <div style={{ 
      position: 'fixed', 
      top: 0, 
      left: 0, 
      right: 0, 
      bottom: 0, 
      backgroundColor: 'rgba(0, 0, 0, 0.8)', 
      display: 'flex', 
      alignItems: 'center', 
      justifyContent: 'center', 
      zIndex: 9999 
    }}>
      <Box 
        maxW="md" 
        bg="gray.800" 
        color="white" 
        p={8} 
        borderRadius="lg"
        boxShadow="xl"
      >
        <Heading size="lg" textAlign="center" mb={6}>
          Recover Your Wallet
        </Heading>
        
        <VStack gap={6}>
          {error && (
            <Box bg="red.900" color="white" p={4} borderRadius="md" w="100%">
              <Text>{error}</Text>
            </Box>
          )}

          <Box w="100%">
            <Text fontSize="lg" fontWeight="semibold" mb={4}>
              Recovery Sentence Length
            </Text>
            <Stack gap={3}>
              {[12, 18, 24].map((count) => (
                <Box
                  key={count}
                  p={3}
                  borderRadius="md"
                  cursor="pointer"
                  bg={wordCount === count ? "blue.600" : "gray.700"}
                  borderWidth={wordCount === count ? "2px" : "1px"}
                  borderColor={wordCount === count ? "blue.400" : "gray.600"}
                  onClick={() => handleWordCountChange(count as 12 | 18 | 24)}
                  transition="all 0.2s"
                  _hover={{
                    bg: wordCount === count ? "blue.700" : "gray.600"
                  }}
                >
                  <Text fontSize="md">{count} words</Text>
                </Box>
              ))}
            </Stack>
          </Box>

          <Box w="100%">
            <HStack justify="space-between" align="start">
              <VStack align="start" gap={1} flex={1}>
                <Text fontSize="lg" fontWeight="semibold">
                  Recover with passphrase?
                </Text>
                <Text fontSize="sm" color="gray.400">
                  If you are unsure if your recovery sentence is protected by a passphrase, leave this off.
                </Text>
              </VStack>
              <Box
                w="50px"
                h="25px"
                bg={usePassphrase ? "blue.500" : "gray.600"}
                borderRadius="full"
                cursor="pointer"
                onClick={handlePassphraseToggle}
                position="relative"
                transition="all 0.2s"
              >
                <Box
                  w="21px"
                  h="21px"
                  bg="white"
                  borderRadius="full"
                  position="absolute"
                  top="2px"
                  left={usePassphrase ? "27px" : "2px"}
                  transition="all 0.2s"
                />
              </Box>
            </HStack>
          </Box>

          <VStack gap={3} w="100%">
            <Button
              onClick={handleSubmit}
              colorScheme="blue"
              size="lg"
              w="100%"
              disabled={isLoading}
            >
              {isLoading ? "Starting Recovery..." : "Recover Wallet"}
            </Button>
            
            {onBack && (
              <Button
                onClick={onBack}
                variant="outline"
                size="lg"
                w="100%"
                disabled={isLoading}
                borderColor="gray.500"
                color="gray.300"
                _hover={{
                  borderColor: "gray.400",
                  color: "white"
                }}
              >
                Back
              </Button>
            )}
          </VStack>
        </VStack>
      </Box>
    </div>
  );
} 