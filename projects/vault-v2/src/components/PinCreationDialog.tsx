import React from 'react';
import { Box } from '@chakra-ui/react';
import { 
  DialogRoot,
  DialogContent,
  DialogCloseTrigger
} from './ui/dialog';
import { DevicePin } from './WalletCreationWizard/DevicePin';
import { PinCreationSession } from '../types/pin';
import { FaTimes } from 'react-icons/fa';

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
    <DialogRoot
      open={isOpen}
      onOpenChange={(e) => {
        if (!e.open) {
          onClose();
        }
      }}
      size="full"
      placement="center"
      motionPreset="slideInBottom"
    >
      <DialogContent
        bg="transparent"
        boxShadow="none"
        display="flex"
        alignItems="center"
        justifyContent="center"
        minH="100vh"
        _backdrop={{
          bg: "rgba(0, 0, 0, 0.8)"
        }}
      >
        <DialogCloseTrigger
          color="gray.400"
          _hover={{ color: "white", bg: "gray.700" }}
          top={4}
          right={4}
          zIndex={1}
        >
          <FaTimes size={24} />
        </DialogCloseTrigger>
        
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
      </DialogContent>
    </DialogRoot>
  );
};