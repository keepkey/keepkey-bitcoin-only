import React from 'react';
import { 
  Box, 
  VStack, 
  Text, 
  Button, 
  HStack,
  Icon
} from '@chakra-ui/react';
import { FaUsb } from 'react-icons/fa';
import { useDialog } from '../contexts/DialogContext';

export interface NoDeviceDialogProps {
  onRetry?: () => void;
}

export function NoDeviceDialog({ onRetry }: NoDeviceDialogProps) {
  const { hide } = useDialog();
  
  const handleRetry = () => {
    if (onRetry) {
      onRetry();
    }
    hide('no-device-found');
  };

  return (
    <Box
      w="100%"
      maxW="500px"
      bg="gray.850"
      borderRadius="xl"
      boxShadow="2xl"
      borderWidth="1px"
      borderColor="orange.500"
      overflow="hidden"
    >
      <Box p={5} borderBottomWidth="1px" borderColor="gray.700" bg="gray.900">
        <HStack justifyContent="center">
          <Icon as={FaUsb} color="orange.500" boxSize={6} />
          <Text fontSize="lg" fontWeight="bold" color="orange.500">
            No KeepKey Detected
          </Text>
        </HStack>
      </Box>

      <Box p={6} bg="gray.800">
        <VStack align="center" gap={4}>
          <Text fontSize="md" color="gray.200" textAlign="center">
            Please connect your KeepKey device to continue
          </Text>
          
          <Box 
            borderWidth="1px" 
            borderColor="gray.700" 
            borderRadius="md" 
            p={4} 
            w="full" 
            bg="gray.750"
          >
            <VStack align="start" gap={2}>
              <Text fontSize="sm" color="gray.300" fontWeight="semibold">
                Troubleshooting tips:
              </Text>
              <Text fontSize="xs" color="gray.400">
                • Make sure your KeepKey is plugged in via USB
              </Text>
              <Text fontSize="xs" color="gray.400">
                • Try a different USB port or cable
              </Text>
              <Text fontSize="xs" color="gray.400">
                • Unplug and reconnect your device
              </Text>
              <Text fontSize="xs" color="gray.400">
                • Ensure no other apps are using the device
              </Text>
            </VStack>
          </Box>

          <Box 
            borderWidth="1px" 
            borderColor="orange.600" 
            borderRadius="md" 
            p={4} 
            w="full" 
            bg="orange.900"
            bgGradient="linear(to-br, orange.900, gray.800)"
          >
            <VStack align="start" gap={2}>
              <Text fontSize="sm" color="orange.300" fontWeight="bold">
                Device still not detected?
              </Text>
              <Text fontSize="xs" color="orange.200">
                Try putting your KeepKey into updater mode:
              </Text>
              <VStack align="start" gap={1} pl={2}>
                <Text fontSize="xs" color="gray.300">
                  1. Disconnect your KeepKey
                </Text>
                <Text fontSize="xs" color="gray.300">
                  2. Hold down the button on your KeepKey
                </Text>
                <Text fontSize="xs" color="gray.300">
                  3. While holding the button, reconnect the USB cable
                </Text>
                <Text fontSize="xs" color="gray.300">
                  4. Release the button when you see the KeepKey logo
                </Text>
              </VStack>
              <Text fontSize="xs" color="gray.400" fontStyle="italic" mt={1}>
                This will allow the device to be detected and updated if needed.
              </Text>
            </VStack>
          </Box>

          <HStack gap={3} w="full" justify="center">
            <Button
              variant="outline"
              colorScheme="gray"
              size="sm"
              onClick={() => hide('no-device-found')}
            >
              Close
            </Button>
            <Button
              colorScheme="orange"
              size="sm"
              onClick={handleRetry}
            >
              Retry Connection
            </Button>
          </HStack>
        </VStack>
      </Box>
    </Box>
  );
}