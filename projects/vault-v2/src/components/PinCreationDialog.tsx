import React from 'react';
import {
  Box,
  Modal,
  ModalOverlay,
  ModalContent,
  ModalCloseButton,
} from '@chakra-ui/react';
import { DevicePin } from './WalletCreationWizard/DevicePin';
import { PinCreationSession } from '../types/pin';

interface PinCreationDialogProps {
  isOpen: boolean;
  deviceId: string;
  deviceLabel?: string;
  onClose: () => void;
  onComplete?: () => void;
}

export const PinCreationDialog: React.FC<PinCreationDialogProps> = ({
  isOpen,
  deviceId,
  deviceLabel,
  onClose,
  onComplete,
}) => {
  const handlePinComplete = (session: PinCreationSession) => {
    console.log('[PinCreationDialog] PIN creation completed:', session);
    
    // Notify parent of completion
    if (onComplete) {
      onComplete();
    }
    
    // Close the dialog
    onClose();
  };

  const handleCancel = () => {
    console.log('[PinCreationDialog] PIN creation cancelled');
    onClose();
  };

  return (
    <Modal
      isOpen={isOpen}
      onClose={onClose}
      size="full"
      closeOnOverlayClick={false}
      closeOnEsc={false}
    >
      <ModalOverlay bg="rgba(0, 0, 0, 0.8)" />
      <ModalContent
        bg="transparent"
        boxShadow="none"
        display="flex"
        alignItems="center"
        justifyContent="center"
        minH="100vh"
      >
        <ModalCloseButton
          color="gray.400"
          _hover={{ color: "white", bg: "gray.700" }}
          size="lg"
          top={4}
          right={4}
          zIndex={1}
        />
        
        <Box
          display="flex"
          alignItems="center"
          justifyContent="center"
          w="100%"
          p={8}
        >
          <DevicePin
            deviceId={deviceId}
            deviceLabel={deviceLabel}
            mode="create"
            onComplete={handlePinComplete}
            onBack={handleCancel}
          />
        </Box>
      </ModalContent>
    </Modal>
  );
};