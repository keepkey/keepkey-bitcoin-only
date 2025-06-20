import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { Button, Grid, Text, HStack, Icon, VStack, Box } from '@chakra-ui/react'
import { FaCircle, FaExclamationTriangle, FaTimes } from 'react-icons/fa'

interface PinUnlockDialogProps {
  isOpen: boolean
  deviceId: string
  onUnlocked: () => void
  onClose: () => void
}

export const PinUnlockDialog = ({ isOpen, deviceId, onUnlocked, onClose }: PinUnlockDialogProps) => {
  const [pinPositions, setPinPositions] = useState<number[]>([])
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [step, setStep] = useState<'trigger' | 'enter' | 'submitting'>('trigger')

  // Reset state when dialog opens/closes
  useEffect(() => {
    if (isOpen) {
      setPinPositions([])
      setError(null)
      setStep('trigger')
      triggerPinRequest()
    }
  }, [isOpen, deviceId])

  const triggerPinRequest = async () => {
    try {
      setIsLoading(true)
      setError(null)
      console.log('🔐 Triggering PIN request for device:', deviceId)
      
      await invoke('trigger_pin_request', { deviceId })
      
      // If we get here, the PIN request was triggered successfully
      // Device should now be showing PIN matrix
      setStep('enter')
      
    } catch (err: any) {
      console.error('❌ Failed to trigger PIN request:', err)
      setError(`Failed to request PIN: ${err}`)
    } finally {
      setIsLoading(false)
    }
  }

  const handlePinButtonClick = (position: number) => {
    if (pinPositions.length >= 9) return // Max 9 digits
    
    setPinPositions(prev => [...prev, position])
  }

  const handleSubmitPin = async () => {
    if (pinPositions.length === 0) {
      setError('Please enter your PIN')
      return
    }

    try {
      setStep('submitting')
      setError(null)
      console.log('🔐 Submitting PIN with positions:', pinPositions)
      
      await invoke('send_pin_matrix_ack', { 
        deviceId, 
        positions: pinPositions 
      })
      
      // PIN submitted successfully
      console.log('✅ PIN submitted successfully')
      onUnlocked()
      
    } catch (err: any) {
      console.error('❌ PIN submission failed:', err)
      setError(`PIN verification failed: ${err}`)
      setStep('enter')
    }
  }

  const handleClearPin = () => {
    setPinPositions([])
  }

  const handleRetry = () => {
    setError(null)
    setStep('trigger')
    triggerPinRequest()
  }

  if (!isOpen) return null

  return (
    <Box
      position="fixed"
      top={0}
      left={0}
      right={0}
      bottom={0}
      bg="blackAlpha.800"
      zIndex={9999}
      display="flex"
      alignItems="center"
      justifyContent="center"
    >
      <Box
        bg="gray.800"
        color="white"
        border="1px solid"
        borderColor="gray.600"
        borderRadius="xl"
        p={6}
        maxW="md"
        w="90%"
        maxH="90vh"
        overflowY="auto"
      >
        {/* Header */}
        <HStack justify="space-between" mb={6}>
          <Text fontSize="xl" fontWeight="bold">Unlock Device</Text>
          <Button
            size="sm"
            variant="ghost"
            onClick={onClose}
            color="gray.400"
            _hover={{ color: "white" }}
          >
            <Icon as={FaTimes} />
          </Button>
        </HStack>

        <VStack gap={6}>
          
          {step === 'trigger' && (
            <VStack gap={4}>
              <Text textAlign="center">
                {isLoading ? 'Requesting PIN from device...' : 'Ready to unlock'}
              </Text>
              <Text fontSize="sm" color="gray.400" textAlign="center">
                Check your device screen for the PIN matrix
              </Text>
              {!isLoading && (
                <Button onClick={triggerPinRequest} colorScheme="blue">
                  Request PIN
                </Button>
              )}
            </VStack>
          )}

          {step === 'enter' && (
            <VStack gap={4} w="full">
              <Text textAlign="center" fontSize="lg" fontWeight="bold">
                Enter PIN from Device
              </Text>
              <Text fontSize="sm" color="gray.400" textAlign="center">
                Look at your device screen and click the positions shown
              </Text>

              {/* PIN dots display */}
              <HStack gap={2} justify="center" minH="30px">
                {Array.from({ length: Math.max(6, pinPositions.length) }, (_, i) => (
                  <Icon
                    key={i}
                    as={FaCircle}
                    color={i < pinPositions.length ? "blue.400" : "gray.600"}
                    w={3}
                    h={3}
                  />
                ))}
              </HStack>

              {/* 3x3 PIN matrix */}
              <Grid templateColumns="repeat(3, 1fr)" gap={3} w="200px">
                {Array.from({ length: 9 }, (_, i) => {
                  const position = i + 1
                  return (
                    <Button
                      key={position}
                      onClick={() => handlePinButtonClick(position)}
                      variant="outline"
                      borderColor="gray.500"
                      color="white"
                      _hover={{ bg: "gray.700", borderColor: "blue.400" }}
                      _active={{ bg: "blue.600" }}
                      size="lg"
                      fontSize="xl"
                      disabled={pinPositions.length >= 9}
                    >
                      •
                    </Button>
                  )
                })}
              </Grid>

              {/* Action buttons */}
              <HStack gap={3} w="full" justify="center">
                <Button 
                  onClick={handleClearPin}
                  variant="ghost"
                  size="sm"
                  disabled={pinPositions.length === 0}
                >
                  Clear
                </Button>
                <Button
                  onClick={handleSubmitPin}
                  colorScheme="blue"
                  disabled={pinPositions.length === 0}
                >
                  Unlock Device
                </Button>
              </HStack>
            </VStack>
          )}

          {step === 'submitting' && (
            <VStack gap={4}>
              <Text textAlign="center">Verifying PIN...</Text>
              <Text fontSize="sm" color="gray.400" textAlign="center">
                Please wait while your device processes the PIN
              </Text>
            </VStack>
          )}

          {/* Error display */}
          {error && (
            <Box bg="red.900" borderColor="red.700" border="1px solid" p={4} borderRadius="md" w="full">
              <HStack gap={3} align="start">
                <Icon as={FaExclamationTriangle} color="red.400" mt={1} />
                <VStack align="start" gap={2} flex={1}>
                  <Text fontSize="sm" color="red.100">{error}</Text>
                  <Button size="sm" colorScheme="red" variant="outline" onClick={handleRetry}>
                    Try Again
                  </Button>
                </VStack>
              </HStack>
            </Box>
          )}

        </VStack>
      </Box>
    </Box>
  )
} 