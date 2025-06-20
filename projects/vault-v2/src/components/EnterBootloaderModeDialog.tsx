import { 
  DialogRoot, 
  DialogContent, 
  DialogHeader, 
  DialogTitle,
  DialogBody
} from '@chakra-ui/react'
import { VStack, Text, Box, HStack } from '@chakra-ui/react'
import { FaExclamationTriangle } from 'react-icons/fa'
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
    <DialogRoot 
      open={isOpen} 
      onOpenChange={() => {}} // Prevent closing by clicking outside
      modal={true}
      closeOnInteractOutside={false}
      closeOnEscape={false}
    >
      <DialogContent 
        maxW="2xl" 
        w="600px"
        h="auto"
        bg="gray.900" 
        color="white" 
        borderColor="yellow.500" 
        borderWidth="3px"
        borderRadius="md"
        boxShadow="0 25px 50px -12px rgba(0, 0, 0, 0.9)"
      >
        <DialogHeader borderBottomWidth="1px" borderColor="gray.700" pb={3}>
          <HStack gap={2}>
            <FaExclamationTriangle color="#F59E0B" size={20} />
            <DialogTitle color="white" fontSize="lg">Bootloader Update Required</DialogTitle>
          </HStack>
          {/* Removed DialogCloseTrigger - this dialog cannot be dismissed */}
        </DialogHeader>
        
        <DialogBody py={4}>
          <HStack align="stretch" gap={6}>
            {/* Left side - Instructions */}
            <VStack align="stretch" gap={4} flex="1">
              <Box bg="yellow.900" p={3} borderRadius="md" borderWidth="1px" borderColor="yellow.600">
                <Text fontSize="sm" color="yellow.200" textAlign="center" fontWeight="medium">
                  ⚠️ This bootloader update is mandatory and cannot be skipped
                </Text>
              </Box>
              
              <Text fontSize="sm" color="gray.300" textAlign="center">
                Bootloader update required: v{bootloaderCheck.currentVersion} → v{bootloaderCheck.latestVersion}
              </Text>
              
              <Text fontSize="sm" color="gray.300" textAlign="center">
                Your device must be in <Text as="span" fontWeight="bold" color="yellow.300">Bootloader Mode</Text> to continue
              </Text>

              <VStack align="stretch" gap={2} bg="gray.800" p={4} borderRadius="md" fontSize="sm">
                <Text fontWeight="semibold" color="yellow.300">Required Steps:</Text>
                <Text color="white">1. Unplug your KeepKey device</Text>
                <Text color="white">2. Hold the button and plug device back in</Text>
                <Text color="white">3. Keep holding until "BOOTLOADER MODE" appears</Text>
                <Text color="white">4. Release the button</Text>
              </VStack>
              
              <Box bg="blue.900" p={3} borderRadius="md" borderWidth="1px" borderColor="blue.600">
                <Text fontSize="xs" color="blue.200" textAlign="center" fontWeight="medium">
                  💡 The update will start automatically once your device enters bootloader mode
                </Text>
              </Box>
            </VStack>

            {/* Right side - Visual instruction */}
            <Box display="flex" justifyContent="center" alignItems="center" minW="240px">
              <img
                  src={holdAndConnectSvg}
                  alt="Hold button while connecting device"
                  style={{ maxWidth: '200px', height: 'auto' }}
              />
            </Box>
          </HStack>
        </DialogBody>
        
        {/* Removed footer with "Got It" button - user must complete the action */}
      </DialogContent>
    </DialogRoot>
  )
} 