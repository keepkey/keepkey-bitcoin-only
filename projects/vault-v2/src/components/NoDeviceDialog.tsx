import { 
  DialogRoot, 
  DialogContent, 
  DialogHeader, 
  DialogTitle,
  DialogBody
} from '@chakra-ui/react'
import { VStack, Text, Box, HStack, Button } from '@chakra-ui/react'
import { FaUsb, FaInfoCircle } from 'react-icons/fa'
import { Logo } from './logo/logo'

interface NoDeviceDialogProps {
  isOpen: boolean
  onClose: () => void
}

export const NoDeviceDialog = ({ 
  isOpen, 
  onClose 
}: NoDeviceDialogProps) => {
  return (
    <DialogRoot 
      open={isOpen} 
      onOpenChange={onClose}
      modal={true}
    >
      <DialogContent 
        maxW="lg" 
        w="500px"
        bg="gray.900" 
        color="white" 
        borderColor="blue.500" 
        borderWidth="2px"
        borderRadius="md"
        boxShadow="0 25px 50px -12px rgba(0, 0, 0, 0.8)"
      >
        <DialogHeader borderBottomWidth="1px" borderColor="gray.700" pb={4}>
          <HStack gap={3} justify="center">
            <Logo width="40px" />
            <DialogTitle fontSize="xl" fontWeight="bold" color="blue.400">
              No KeepKey Detected
            </DialogTitle>
          </HStack>
        </DialogHeader>
        
        <DialogBody py={6}>
          <VStack gap={6} textAlign="center">
            {/* Large USB icon */}
            <Box>
              <FaUsb size="64px" color="#4299E1" />
            </Box>
            
            {/* Main message */}
            <VStack gap={3}>
              <Text fontSize="lg" fontWeight="semibold" color="white">
                Please connect your KeepKey device
              </Text>
              
              <Text fontSize="md" color="gray.300" lineHeight="1.6">
                Make sure your KeepKey hardware wallet is connected to your computer via USB.
              </Text>
            </VStack>
            
            {/* Instructions */}
            <Box 
              bg="blue.900" 
              borderRadius="md" 
              p={4} 
              borderLeft="4px solid" 
              borderColor="blue.400"
              w="full"
            >
              <HStack gap={2} align="flex-start" mb={2}>
                <FaInfoCircle color="#4299E1" size="16px" style={{ marginTop: '2px' }} />
                <Text fontSize="sm" fontWeight="semibold" color="blue.200">
                  Connection Steps:
                </Text>
              </HStack>
              
              <VStack align="flex-start" gap={1} ml={6}>
                <Text fontSize="sm" color="gray.300">
                  1. Connect your KeepKey to a USB port
                </Text>
                <Text fontSize="sm" color="gray.300">
                  2. Make sure the USB cable is secure
                </Text>
                <Text fontSize="sm" color="gray.300">
                  3. Wait for the device to be detected
                </Text>
              </VStack>
            </Box>
            
            {/* Action buttons */}
            <HStack gap={3} w="full" justify="center">
              <Button
                variant="outline"
                colorScheme="blue"
                onClick={onClose}
                size="md"
                px={6}
              >
                OK
              </Button>
            </HStack>
          </VStack>
        </DialogBody>
      </DialogContent>
    </DialogRoot>
  )
} 