import React, { ReactNode } from 'react';
import {
  Box,
  Button,
  VStack,
  HStack,
  Text,
} from '@chakra-ui/react';

// Simple dialog overlay that works with current Chakra setup
interface DialogOverlayProps {
  isOpen: boolean;
  onClose: () => void;
  children: ReactNode;
}

const DialogOverlay: React.FC<DialogOverlayProps> = ({ isOpen, onClose, children }) => {
  if (!isOpen) return null;

  return (
    <Box
      position="fixed"
      top="0"
      left="0"
      right="0"
      bottom="0"
      bg="rgba(0, 0, 0, 0.8)"
      backdropFilter="blur(10px)"
      zIndex="overlay"
      display="flex"
      alignItems="center"
      justifyContent="center"
      onClick={onClose}
    >
      <Box onClick={(e) => e.stopPropagation()}>
        {children}
      </Box>
    </Box>
  );
};

interface DialogProps {
  isOpen: boolean;
  onClose: () => void;
  title?: string;
  children: ReactNode;
  footer?: ReactNode;
  size?: 'xs' | 'sm' | 'md' | 'lg' | 'xl' | 'full';
}

export const Dialog: React.FC<DialogProps> = ({
  isOpen,
  onClose,
  title,
  children,
  footer,
  size = 'md',
}) => {
  const getSizeWidth = () => {
    switch (size) {
      case 'xs': return '300px';
      case 'sm': return '400px';
      case 'md': return '500px';
      case 'lg': return '600px';
      case 'xl': return '800px';
      case 'full': return '90vw';
      default: return '500px';
    }
  };

  return (
    <DialogOverlay isOpen={isOpen} onClose={onClose}>
      <Box
        bg="#111111"
        border="1px solid #222222"
        borderRadius="2xl"
        boxShadow="0 20px 60px rgba(0, 0, 0, 0.8)"
        width={getSizeWidth()}
        maxWidth="90vw"
        maxHeight="90vh"
        overflow="hidden"
      >
        {title && (
          <HStack
            justify="space-between"
            align="center"
            p={4}
            borderBottom="1px solid #222222"
          >
            <Text
              color="white"
              fontSize="lg"
              fontWeight="bold"
            >
              {title}
            </Text>
            <Button
              size="sm"
              variant="ghost"
              color="gray.400"
              onClick={onClose}
              _hover={{ color: 'white' }}
            >
              ✕
            </Button>
          </HStack>
        )}
        <Box color="white" p={6} overflowY="auto">
          {children}
        </Box>
        {footer && (
          <Box borderTop="1px solid #222222" p={4}>
            {footer}
          </Box>
        )}
      </Box>
    </DialogOverlay>
  );
};

export default Dialog; 