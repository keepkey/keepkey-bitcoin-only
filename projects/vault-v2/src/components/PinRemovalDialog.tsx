import { useState, useCallback } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { Button, Text, HStack, Icon, VStack, Box, Spinner, SimpleGrid, Heading } from '@chakra-ui/react'
import { FaCircle, FaBackspace } from 'react-icons/fa'

interface PinRemovalDialogProps {
  isOpen: boolean
  deviceId: string
  onSuccess: () => void
  onClose: () => void
}

// The KeepKey device shows this scrambled layout on its screen:
// 7 8 9
// 4 5 6
// 1 2 3
// We need to send these exact numbers when the user clicks each position
const PIN_MATRIX_LAYOUT = [7, 8, 9, 4, 5, 6, 1, 2, 3] as const

export const PinRemovalDialog = ({ isOpen, deviceId, onSuccess, onClose }: PinRemovalDialogProps) => {
  const [pinPositions, setPinPositions] = useState<number[]>([])
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const handlePinButtonClick = useCallback((position: number) => {
    if (pinPositions.length >= 9) return // Max 9 digits
    
    setPinPositions(prev => [...prev, position])
    setError(null)
  }, [pinPositions])

  const handleBackspace = useCallback(() => {
    setPinPositions(prev => prev.slice(0, -1))
    setError(null)
  }, [])

  const handleClear = useCallback(() => {
    setPinPositions([])
    setError(null)
  }, [])

  const handleSubmit = async () => {
    if (pinPositions.length === 0) {
      setError('Please enter your PIN')
      return
    }

    setIsLoading(true)
    setError(null)

    try {
      console.log('🔐 Submitting PIN for removal with positions:', pinPositions)
      
      // Send PIN positions to backend for removal operation
      const result = await invoke('send_pin_for_removal', {
        deviceId,
        positions: new Uint8Array(pinPositions)
      })
      
      console.log('✅ PIN removal result:', result)
      
      if (result === true) {
        // Success - PIN was correct and removal completed
        onSuccess()
      } else {
        setError('Failed to disable PIN protection')
      }
      
    } catch (err: any) {
      console.error('❌ PIN removal submission failed:', err)
      
      const errorStr = String(err).toLowerCase()
      
      if (errorStr.includes('incorrect pin')) {
        setError('Incorrect PIN. Please try again.')
        setPinPositions([]) // Clear PIN for retry
      } else if (errorStr.includes('device not found') || errorStr.includes('not connected')) {
        setError('Device disconnected. Please reconnect and try again.')
        onClose()
      } else if (errorStr.includes('not in pin removal flow')) {
        setError('PIN removal flow was interrupted. Please try again.')
        onClose()
      } else {
        setError(`Error: ${err}`)
      }
    } finally {
      setIsLoading(false)
    }
  }

  if (!isOpen) return null

  return (
    <Box
      position="fixed"
      top="0"
      left="0"
      right="0"
      bottom="0"
      bg="blackAlpha.600"
      display="flex"
      alignItems="center"
      justifyContent="center"
      zIndex={9999}
    >
      <Box
        bg="white"
        borderRadius="md"
        p={6}
        maxW="400px"
        w="90%"
        boxShadow="xl"
      >
        <VStack gap={4}>
          <Heading size="md">Enter PIN to Disable Protection</Heading>
          
          <Text fontSize="sm" color="gray.600" textAlign="center">
            Enter your current PIN on the computer to disable PIN protection
          </Text>

          {/* PIN dots display */}
          <HStack gap={2} justify="center" minH="32px">
            {Array.from({ length: 9 }).map((_, i) => (
              <Icon
                key={i}
                as={FaCircle}
                boxSize={i < pinPositions.length ? 3 : 2}
                color={i < pinPositions.length ? "blue.500" : "gray.300"}
              />
            ))}
          </HStack>

          {/* PIN matrix buttons */}
          <SimpleGrid columns={3} gap={2} width="200px">
            {PIN_MATRIX_LAYOUT.map((num, index) => (
              <Button
                key={index}
                onClick={() => handlePinButtonClick(num)}
                disabled={isLoading || pinPositions.length >= 9}
                size="lg"
                height="60px"
                width="60px"
                fontSize="lg"
                variant="outline"
                colorScheme="blue"
              >
                •
              </Button>
            ))}
          </SimpleGrid>

          {/* Control buttons */}
          <HStack gap={2} width="100%">
            <Button
              onClick={handleClear}
              disabled={isLoading || pinPositions.length === 0}
              size="sm"
              variant="outline"
            >
              Clear
            </Button>
            
            <Button
              onClick={handleBackspace}
              disabled={isLoading || pinPositions.length === 0}
              size="sm"
              variant="outline"
            >
              <Icon as={FaBackspace} />
            </Button>
            
            <Button
              onClick={handleSubmit}
              disabled={isLoading || pinPositions.length === 0}
              colorScheme="blue"
              size="sm"
              flex="1"
            >
              {isLoading ? <Spinner size="sm" /> : 'Submit PIN'}
            </Button>
            
            <Button
              onClick={onClose}
              disabled={isLoading}
              size="sm"
              variant="ghost"
            >
              Cancel
            </Button>
          </HStack>

          {/* Error message */}
          {error && (
            <Text color="red.500" fontSize="sm" textAlign="center">
              {error}
            </Text>
          )}
        </VStack>
      </Box>
    </Box>
  )
}