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
import { useTypedTranslation } from '../hooks/useTypedTranslation';

export interface NoDeviceDialogProps {
  onRetry?: () => void;
}

export function NoDeviceDialog({ onRetry }: NoDeviceDialogProps) {
  const { hide } = useDialog();
  const { t } = useTypedTranslation('dialogs');
  
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
            {t('noDevice.title')}
          </Text>
        </HStack>
      </Box>

      <Box p={6} bg="gray.800">
        <VStack align="center" gap={4}>
          <Text fontSize="md" color="gray.200" textAlign="center">
            {t('noDevice.subtitle')}
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
                {t('noDevice.troubleshooting.title')}
              </Text>
              <Text fontSize="xs" color="gray.400">
                • {t('noDevice.troubleshooting.tips.pluggedIn')}
              </Text>
              <Text fontSize="xs" color="gray.400">
                • {t('noDevice.troubleshooting.tips.differentPort')}
              </Text>
              <Text fontSize="xs" color="gray.400">
                • {t('noDevice.troubleshooting.tips.reconnect')}
              </Text>
              <Text fontSize="xs" color="gray.400">
                • {t('noDevice.troubleshooting.tips.noOtherApps')}
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
                {t('noDevice.updaterMode.title')}
              </Text>
              <Text fontSize="xs" color="orange.200">
                {t('noDevice.updaterMode.subtitle')}
              </Text>
              <VStack align="start" gap={1} pl={2}>
                <Text fontSize="xs" color="gray.300">
                  1. {t('noDevice.updaterMode.steps.1')}
                </Text>
                <Text fontSize="xs" color="gray.300">
                  2. {t('noDevice.updaterMode.steps.2')}
                </Text>
                <Text fontSize="xs" color="gray.300">
                  3. {t('noDevice.updaterMode.steps.3')}
                </Text>
                <Text fontSize="xs" color="gray.300">
                  4. {t('noDevice.updaterMode.steps.4')}
                </Text>
              </VStack>
              <Text fontSize="xs" color="gray.400" fontStyle="italic" mt={1}>
                {t('noDevice.updaterMode.note')}
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
              {t('noDevice.buttons.close')}
            </Button>
            <Button
              colorScheme="orange"
              size="sm"
              onClick={handleRetry}
            >
              {t('noDevice.buttons.retry')}
            </Button>
          </HStack>
        </VStack>
      </Box>
    </Box>
  );
}