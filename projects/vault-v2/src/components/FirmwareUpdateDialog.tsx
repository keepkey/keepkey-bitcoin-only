import { VStack, Text, Button, Box, Icon, HStack, Badge } from '@chakra-ui/react'
import { 
  DialogRoot,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogBody,
  DialogFooter,
  DialogCloseTrigger
} from './ui/dialog'
import { FaDownload, FaShieldAlt } from 'react-icons/fa'
import { useState } from 'react'
import type { FirmwareCheck } from '../types/device'
import { useTypedTranslation } from '../hooks/useTypedTranslation'

interface FirmwareUpdateDialogProps {
  isOpen: boolean
  firmwareCheck: {
    currentVersion: string
    latestVersion: string
    needsUpdate: boolean
  }
  onUpdateStart: () => void
  onSkip: () => void
  onRemindLater: () => void
  onClose: () => void
  isLoading?: boolean
  deviceStatus?: { 
    bootloaderCheck?: { 
      currentVersion: string 
      latestVersion: string
      needsUpdate: boolean
    } 
  }
}

export const FirmwareUpdateDialog = ({ 
  isOpen, 
  firmwareCheck,
  onUpdateStart,
  onSkip,
  onRemindLater,
  onClose,
  isLoading = false,
  deviceStatus  // Add deviceStatus prop to access bootloader info
}: FirmwareUpdateDialogProps) => {
  const { t } = useTypedTranslation('dialogs')
  const [isUpdating, setIsUpdating] = useState(false)
  
  const handleUpdate = () => {
    setIsUpdating(true)
    onUpdateStart()
  }
  
  // Check if this is an OOB bootloader that cannot be skipped
  const isOOBBootloader = deviceStatus?.bootloaderCheck?.currentVersion?.startsWith("1.") || 
                         firmwareCheck.currentVersion.startsWith("1.0.")
  const canSkip = !isOOBBootloader
  
  return (
    <DialogRoot open={isOpen} onOpenChange={({ open }) => !open && onClose()}>
      <DialogContent 
        maxW="lg" 
        bg="gray.900" 
        color="white"
        borderColor="blue.600"
      >
        <DialogHeader borderBottomWidth="1px" borderColor="gray.700" pb={4}>
          <DialogTitle color="white" display="flex" alignItems="center" gap={2}>
            <Icon as={FaDownload} color="blue.400" />
            {t('firmwareUpdate.title')}
            {(firmwareCheck.needsUpdate || isOOBBootloader) && (
              <Badge colorScheme={isOOBBootloader ? "red" : "orange"} ml={2}>
                {isOOBBootloader ? t('firmwareUpdate.criticalUpdate') : t('firmwareUpdate.updateAvailable')}
              </Badge>
            )}
          </DialogTitle>
          <DialogCloseTrigger color="gray.400" _hover={{ color: "white" }} />
        </DialogHeader>
        
        <DialogBody py={6}>
          <VStack align="stretch" gap={6}>
            <Box 
              bg={isOOBBootloader ? "red.900" : "blue.900"}
              borderRadius="md"
              borderWidth="1px"
              borderColor={isOOBBootloader ? "red.600" : "blue.600"}
              p={4}
            >
              <HStack gap={3} align="start">
                <Icon as={FaShieldAlt} color={isOOBBootloader ? "red.400" : "blue.400"} mt={1} />
                <Box>
                  <Text fontWeight="bold">
                    {isOOBBootloader ? t('firmwareUpdate.criticalRequired') : t('firmwareUpdate.newAvailable')}
                  </Text>
                  <Text fontSize="sm" mt={1}>
                    {isOOBBootloader 
                      ? t('firmwareUpdate.criticalDescription')
                      : t('firmwareUpdate.description')
                    }
                  </Text>
                </Box>
              </HStack>
            </Box>
            
            <VStack align="stretch" gap={3}>
              <HStack justify="space-between">
                <Box>
                  <Text fontSize="sm" color="gray.400">{t('firmwareUpdate.currentVersion')}</Text>
                  <Text fontSize="lg" fontWeight="semibold">{firmwareCheck.currentVersion}</Text>
                </Box>
                <Icon as={FaDownload} color="gray.600" boxSize={6} />
                <Box>
                  <Text fontSize="sm" color="gray.400">{t('firmwareUpdate.newVersion')}</Text>
                  <Text fontSize="lg" fontWeight="semibold" color="green.400">
                    {firmwareCheck.latestVersion}
                  </Text>
                </Box>
              </HStack>
              
              <Box>
                <Text fontSize="sm" color="gray.400">{t('firmwareUpdate.estimatedTime')}</Text>
                <Text>{t('firmwareUpdate.estimatedDuration')}</Text>
              </Box>
            </VStack>
            
            <VStack align="stretch" gap={2} pt={2}>
              <Text fontSize="sm" fontWeight="bold" color={isOOBBootloader ? "red.400" : "yellow.400"}>
                {t('firmwareUpdate.important')}
              </Text>
              <Text fontSize="sm" color="gray.300">
                • {t('firmwareUpdate.warnings.doNotDisconnect')}
              </Text>
              <Text fontSize="sm" color="gray.300">
                • {t('firmwareUpdate.warnings.reenterPin')}
              </Text>
              <Text fontSize="sm" color="gray.300">
                • {t('firmwareUpdate.warnings.fundsRemainSafe')}
              </Text>
              {isOOBBootloader && (
                <Text fontSize="sm" color="red.300" fontWeight="bold">
                  • {t('firmwareUpdate.warnings.cannotSkip')}
                </Text>
              )}
            </VStack>
          </VStack>
        </DialogBody>
        
        <DialogFooter borderTopWidth="1px" borderColor="gray.700" pt={4}>
          <HStack width="full" gap={3}>
            {canSkip && (
              <Button
                variant="outline"
                colorScheme="gray"
                onClick={onSkip}
                flex={1}
                disabled={isUpdating || isLoading}
              >
                Skip
              </Button>
            )}
            {canSkip && (
              <Button
                variant="outline"
                colorScheme="blue"
                onClick={onRemindLater}
                flex={1}
                disabled={isUpdating || isLoading}
              >
                Remind Later
              </Button>
            )}
            <Button
              colorScheme={isOOBBootloader ? "red" : "blue"}
              onClick={handleUpdate}
              loading={isUpdating || isLoading}
              loadingText="Updating..."
              flex={canSkip ? 2 : 1}
            >
              {isOOBBootloader ? "Update Now (Required)" : "Update Now"}
            </Button>
          </HStack>
        </DialogFooter>
      </DialogContent>
    </DialogRoot>
  )
} 