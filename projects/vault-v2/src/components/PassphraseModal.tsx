import React, { useState, useEffect } from 'react';
import {
  Modal,
  ModalOverlay,
  ModalContent,
  ModalHeader,
  ModalBody,
  ModalFooter,
  Button,
  Input,
  VStack,
  Text,
  Alert,
  FormControl,
  FormLabel,
  InputGroup,
  InputRightElement,
  IconButton,
  Checkbox,
  Box,
} from '@chakra-ui/react';
import { FiEye as ViewIcon, FiEyeOff as ViewOffIcon } from 'react-icons/fi';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

interface PassphraseModalProps {
  isOpen: boolean;
  onClose: () => void;
  deviceId?: string;
}

interface PassphraseRequestPayload {
  requestId: string;
  deviceId: string;
}

export const PassphraseModal: React.FC<PassphraseModalProps> = ({
  isOpen,
  onClose,
  deviceId: propDeviceId,
}) => {
  const [passphrase, setPassphrase] = useState('');
  const [confirmPassphrase, setConfirmPassphrase] = useState('');
  const [showPassphrase, setShowPassphrase] = useState(false);
  const [showConfirm, setShowConfirm] = useState(false);
  const [understood, setUnderstood] = useState(false);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [deviceId, setDeviceId] = useState<string | null>(propDeviceId || null);
  const [requestId, setRequestId] = useState<string | null>(null);

  // Listen for passphrase request events from the device
  useEffect(() => {
    const unlisten = listen<PassphraseRequestPayload>('passphrase_request', (event) => {
      console.log('Received passphrase request:', event.payload);
      setDeviceId(event.payload.deviceId);
      setRequestId(event.payload.requestId);
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const handleSubmit = async () => {
    // Validation
    if (!passphrase) {
      setError('Please enter a passphrase');
      return;
    }

    if (passphrase !== confirmPassphrase) {
      setError('Passphrases do not match');
      return;
    }

    if (!understood) {
      setError('Please confirm that you understand the warning');
      return;
    }

    if (!deviceId) {
      setError('No device ID available');
      return;
    }

    setIsSubmitting(true);
    setError(null);

    try {
      // Send passphrase to device
      await invoke('send_passphrase', {
        passphrase,
        deviceId,
      });

      toast({
        title: 'Passphrase sent',
        description: 'Your passphrase has been sent to the device',
        status: 'success',
        duration: 3000,
        isClosable: true,
      });

      // Clear sensitive data
      setPassphrase('');
      setConfirmPassphrase('');
      setUnderstood(false);
      
      // Close modal
      onClose();
    } catch (err) {
      console.error('Failed to send passphrase:', err);
      setError(err instanceof Error ? err.message : 'Failed to send passphrase');
      
      toast({
        title: 'Error',
        description: 'Failed to send passphrase to device',
        status: 'error',
        duration: 5000,
        isClosable: true,
      });
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleCancel = () => {
    // Clear sensitive data
    setPassphrase('');
    setConfirmPassphrase('');
    setUnderstood(false);
    setError(null);
    onClose();
  };

  return (
    <Modal
      isOpen={isOpen}
      onClose={handleCancel}
      size="lg"
      closeOnOverlayClick={false}
      closeOnEsc={false}
    >
      <ModalOverlay />
      <ModalContent>
        <ModalHeader>Enter BIP39 Passphrase</ModalHeader>
        
        <ModalBody>
          <VStack spacing={4} align="stretch">
            <Alert status="warning">
              <Box>
                <Text fontWeight="bold">Warning:</Text>
                <Text fontSize="sm">
                  This passphrase cannot be recovered if lost! Each unique passphrase creates a
                  completely different wallet. If you forget your passphrase, you will lose access
                  to any funds in that wallet.
                </Text>
              </Box>
            </Alert>

            {error && (
              <Alert status="error">
                <Text>{error}</Text>
              </Alert>
            )}

            <FormControl isRequired>
              <FormLabel>Passphrase</FormLabel>
              <InputGroup>
                <Input
                  type={showPassphrase ? 'text' : 'password'}
                  value={passphrase}
                  onChange={(e) => setPassphrase(e.target.value)}
                  placeholder="Enter your passphrase"
                  autoComplete="off"
                  isDisabled={isSubmitting}
                />
                <InputRightElement>
                  <IconButton
                    aria-label={showPassphrase ? 'Hide passphrase' : 'Show passphrase'}
                    icon={showPassphrase ? <ViewOffIcon /> : <ViewIcon />}
                    size="sm"
                    variant="ghost"
                    onClick={() => setShowPassphrase(!showPassphrase)}
                    isDisabled={isSubmitting}
                  />
                </InputRightElement>
              </InputGroup>
            </FormControl>

            <FormControl isRequired>
              <FormLabel>Confirm Passphrase</FormLabel>
              <InputGroup>
                <Input
                  type={showConfirm ? 'text' : 'password'}
                  value={confirmPassphrase}
                  onChange={(e) => setConfirmPassphrase(e.target.value)}
                  placeholder="Confirm your passphrase"
                  autoComplete="off"
                  isDisabled={isSubmitting}
                />
                <InputRightElement>
                  <IconButton
                    aria-label={showConfirm ? 'Hide confirmation' : 'Show confirmation'}
                    icon={showConfirm ? <ViewOffIcon /> : <ViewIcon />}
                    size="sm"
                    variant="ghost"
                    onClick={() => setShowConfirm(!showConfirm)}
                    isDisabled={isSubmitting}
                  />
                </InputRightElement>
              </InputGroup>
            </FormControl>

            <Checkbox
              isChecked={understood}
              onChange={(e) => setUnderstood(e.target.checked)}
              isDisabled={isSubmitting}
            >
              <Text fontSize="sm">
                I understand that this passphrase creates a completely different wallet and cannot
                be recovered if lost
              </Text>
            </Checkbox>

            <Alert status="info">
              <VStack align="start" spacing={1}>
                <Text fontSize="sm" fontWeight="bold">
                  Tips for creating a strong passphrase:
                </Text>
                <Text fontSize="xs">• Use a memorable phrase or sentence</Text>
                <Text fontSize="xs">• Consider using spaces between words</Text>
                <Text fontSize="xs">• Store a backup in a secure location</Text>
                <Text fontSize="xs">• Test with small amounts first</Text>
              </VStack>
            </Alert>
          </VStack>
        </ModalBody>

        <ModalFooter>
          <Button variant="ghost" mr={3} onClick={handleCancel} isDisabled={isSubmitting}>
            Cancel
          </Button>
          <Button
            colorScheme="blue"
            onClick={handleSubmit}
            isLoading={isSubmitting}
            isDisabled={!passphrase || !confirmPassphrase || !understood || isSubmitting}
          >
            Continue
          </Button>
        </ModalFooter>
      </ModalContent>
    </Modal>
  );
};

export default PassphraseModal;