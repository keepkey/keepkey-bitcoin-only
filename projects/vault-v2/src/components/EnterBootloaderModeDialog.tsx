import { 
  DialogRoot, 
  DialogContent, 
  DialogHeader, 
  DialogTitle, 
  DialogCloseTrigger,
  DialogBody
} from '@chakra-ui/react'
import { VStack, Text, Box, Button, HStack } from '@chakra-ui/react'
import { FaTimes, FaExclamationTriangle } from 'react-icons/fa'
import type { BootloaderCheck } from '../types/device'

// Import the SVG image for the hold-and-connect instructions
import holdAndConnectSvg from '../assets/svg/hold-and-connect.svg'

interface EnterBootloaderModeDialogProps {
  isOpen: boolean
  bootloaderCheck: BootloaderCheck
  deviceId: string  
  onClose: () => void
}

export const EnterBootloaderModeDialog = ({ 
  isOpen, 
  bootloaderCheck, 
  deviceId,
  onClose 
}: EnterBootloaderModeDialogProps) => {
  return (
    <DialogRoot open={isOpen} onOpenChange={({ open }) => !open && onClose()}>
      <DialogContent 
        maxW="md" 
        bg="gray.900" 
        color="white" 
        borderColor="orange.600" 
        borderWidth="2px"
        borderRadius="xl"
      >
        <DialogHeader borderBottomWidth="1px" borderColor="gray.700" pb={4}>
          <HStack gap={2}>
            <FaExclamationTriangle color="orange" />
            <DialogTitle color="white">Enter Bootloader Mode Required</DialogTitle>
          </HStack>
          <DialogCloseTrigger color="gray.400" _hover={{ color: "white" }}>
            <FaTimes />
          </DialogCloseTrigger>
        </DialogHeader>
        
        <DialogBody py={6}>
          <VStack align="stretch" gap={4}>
            <VStack gap={2}>
              <Text fontWeight="bold" textAlign="center">
                Bootloader Update Available
              </Text>
              <Text fontSize="sm" color="gray.300" textAlign="center">
                Current: v{bootloaderCheck.currentVersion} → Latest: v{bootloaderCheck.latestVersion}
              </Text>
            </VStack>
            
            <Text color="gray.300">
              Your KeepKey needs to be in <Text as="span" fontWeight="bold" color="orange.300">Bootloader Mode</Text> to update the bootloader.
            </Text>
            
            <Box display="flex" justifyContent="center" py={4}>
              <img 
                src={holdAndConnectSvg} 
                alt="Hold button while connecting device" 
                style={{ maxWidth: '240px', height: 'auto' }} 
              />
            </Box>
            
            <VStack align="stretch" gap={2} bg="gray.800" p={4} borderRadius="md">
              <Text fontWeight="semibold" color="orange.300">How to enter Bootloader Mode:</Text>
              <Text fontSize="sm">1. Unplug your KeepKey</Text>
              <Text fontSize="sm">2. Hold down the button on your device</Text>
              <Text fontSize="sm">3. While holding the button, plug in your KeepKey</Text>
              <Text fontSize="sm">4. Continue holding until you see "BOOTLOADER MODE" on the screen</Text>
              <Text fontSize="sm">5. Release the button</Text>
            </VStack>
            
            <Box bg="blue.900" p={3} borderRadius="md" borderLeft="4px solid" borderLeftColor="blue.400">
              <Text fontSize="sm" color="blue.200">
                <Text as="span" fontWeight="bold">💡 Tip:</Text> Once your device shows "BOOTLOADER MODE" on screen, 
                the bootloader update dialog will automatically appear.
              </Text>
            </Box>
            
            <Text fontSize="xs" color="gray.500" textAlign="center">
              Device ID: {deviceId}
            </Text>
          </VStack>
        </DialogBody>
        
        <Box borderTopWidth="1px" borderColor="gray.700" pt={4} mt={4}>
          <Button 
            colorScheme="orange" 
            onClick={onClose} 
            width="full"
            variant="outline"
          >
            I'll Enter Bootloader Mode
          </Button>
        </Box>
      </DialogContent>
    </DialogRoot>
  )
} 