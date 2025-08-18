import { useState, useEffect } from 'react'
import { 
  Button, 
  Text, 
  HStack, 
  Icon, 
  VStack, 
  Box, 
  SimpleGrid, 
  Heading 
} from '@chakra-ui/react'
import { 
  FaCircle, 
  FaTimes, 
  FaBackspace 
} from 'react-icons/fa'
import {
  DialogRoot,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogBody,
  DialogFooter,
} from "./ui/dialog"

interface PinEntryDialogProps {
  isOpen: boolean
  onSubmit: (pinPositions: number[]) => void | Promise<void>
  onClose: () => void
  title?: string
  description?: string
}

// The KeepKey device shows this scrambled layout on its screen:
// 7 8 9
// 4 5 6
// 1 2 3
// We need to send these exact numbers when the user clicks each position
const PIN_MATRIX_LAYOUT = [7, 8, 9, 4, 5, 6, 1, 2, 3] as const

export const PinEntryDialog = ({ 
  isOpen, 
  onSubmit, 
  onClose, 
  title = "Enter PIN",
  description = "Enter your PIN using the layout shown on your KeepKey device"
}: PinEntryDialogProps) => {
  const [pinPositions, setPinPositions] = useState<number[]>([])
  const [isSubmitting, setIsSubmitting] = useState(false)

  // Reset state when dialog opens/closes
  useEffect(() => {
    if (isOpen) {
      setPinPositions([])
      setIsSubmitting(false)
    }
  }, [isOpen])

  const handlePinPadClick = (position: number) => {
    if (pinPositions.length < 9) {
      setPinPositions([...pinPositions, position])
    }
  }

  const handleBackspace = () => {
    if (pinPositions.length > 0) {
      setPinPositions(pinPositions.slice(0, -1))
    }
  }

  const handleClear = () => {
    setPinPositions([])
  }

  const handleSubmit = async () => {
    if (pinPositions.length === 0) return
    
    setIsSubmitting(true)
    try {
      await onSubmit(pinPositions)
    } catch (error) {
      console.error('PIN submission error:', error)
    } finally {
      setIsSubmitting(false)
    }
  }

  const handleCancel = () => {
    setPinPositions([])
    onClose()
  }

  if (!isOpen) return null

  return (
    <DialogRoot size="md" placement="center" open={isOpen}>
      <DialogContent maxW="450px">
        <DialogHeader>
          <DialogTitle>{title}</DialogTitle>
        </DialogHeader>
        
        <DialogBody>
          <VStack gap={4}>
            <Text fontSize="sm" color="gray.600" textAlign="center">
              {description}
            </Text>

            {/* PIN Display */}
            <HStack justify="center" gap={2} minH="40px">
              {pinPositions.length === 0 ? (
                <Text fontSize="sm" color="gray.500">Enter PIN...</Text>
              ) : (
                pinPositions.map((_, index) => (
                  <Icon key={index} as={FaCircle} boxSize={2} color="blue.500" />
                ))
              )}
            </HStack>

            {/* PIN Pad */}
            <Box 
              bg="gray.50" 
              borderRadius="lg" 
              p={4}
              border="1px solid"
              borderColor="gray.200"
            >
              <SimpleGrid columns={3} gap={2}>
                {PIN_MATRIX_LAYOUT.map((value, index) => (
                  <Button
                    key={index}
                    onClick={() => handlePinPadClick(value)}
                    size="lg"
                    h="60px"
                    w="60px"
                    variant="outline"
                    disabled={isSubmitting || pinPositions.length >= 9}
                    _hover={{ bg: 'blue.50', borderColor: 'blue.400' }}
                    _active={{ bg: 'blue.100' }}
                  >
                    <FaCircle size="12px" />
                  </Button>
                ))}
              </SimpleGrid>
            </Box>

            {/* PIN Info */}
            <Text fontSize="xs" color="gray.500" textAlign="center">
              Look at your KeepKey screen for the number layout
            </Text>
          </VStack>
        </DialogBody>

        <DialogFooter>
          <HStack justify="space-between" w="full">
            <HStack>
              <Button
                variant="ghost"
                onClick={handleBackspace}
                disabled={pinPositions.length === 0 || isSubmitting}
                size="sm"
              >
                <Icon as={FaBackspace} mr={2} />
                Backspace
              </Button>
              <Button
                variant="ghost"
                onClick={handleClear}
                disabled={pinPositions.length === 0 || isSubmitting}
                size="sm"
              >
                Clear
              </Button>
            </HStack>
            
            <HStack>
              <Button
                variant="ghost"
                onClick={handleCancel}
                disabled={isSubmitting}
              >
                Cancel
              </Button>
              <Button
                colorScheme="blue"
                onClick={handleSubmit}
                disabled={pinPositions.length === 0 || isSubmitting}
                loading={isSubmitting}
              >
                Submit
              </Button>
            </HStack>
          </HStack>
        </DialogFooter>
      </DialogContent>
    </DialogRoot>
  )
}