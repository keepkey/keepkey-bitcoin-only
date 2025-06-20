import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { Box, VStack, HStack, Text, Button, SimpleGrid, Icon } from '@chakra-ui/react'
import { FaCircle } from 'react-icons/fa'

interface PinUnlockDialogProps {
  isOpen: boolean
  deviceId: string
  onUnlocked: () => void
  onClose: () => void
}

interface PinRequestEvent {
  device_id: string
  request_id: string
  session_id: string
  pin_type: string
  message: string
}

export const PinUnlockDialog = ({ isOpen, deviceId, onUnlocked, onClose }: PinUnlockDialogProps) => {
  const [pinMatrix, setPinMatrix] = useState<number[]>([])
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [currentPinRequest, setCurrentPinRequest] = useState<PinRequestEvent | null>(null)
  const [isWaitingForPinRequest, setIsWaitingForPinRequest] = useState(false)

  // Listen for PIN request events from the device
  useEffect(() => {
    if (!isOpen) return

    console.log('🔐 Setting up PIN request event listener for device:', deviceId)
    
    const unlisten = listen<PinRequestEvent>('device:pin-request', (event) => {
      console.log('🔐 Received PIN request event:', event.payload)
      
      // Only handle PIN requests for our device
      if (event.payload.device_id === deviceId) {
        console.log('🔐 PIN request is for our device, showing PIN dialog')
        setCurrentPinRequest(event.payload)
        setIsWaitingForPinRequest(false)
        setError(null)
        setPinMatrix([])
      }
    })

    return () => {
      unlisten.then(fn => fn())
    }
  }, [isOpen, deviceId])

  // Trigger a PIN request when dialog opens
  useEffect(() => {
    if (isOpen && !currentPinRequest && !isWaitingForPinRequest) {
      console.log('🔐 Dialog opened - triggering PIN request for device:', deviceId)
      setIsWaitingForPinRequest(true)
      setError(null)
      
      // Set a timeout to show retry button if no PIN event received
      const timeout = setTimeout(() => {
        if (!currentPinRequest) {
          console.log('🔐 Timeout waiting for PIN request event')
          setIsWaitingForPinRequest(false)
          setError('No PIN request received from device. Please try again.')
        }
      }, 5000) // 5 second timeout
      
      invoke('trigger_pin_request', { deviceId })
        .then(() => {
          console.log('🔐 PIN request triggered successfully')
        })
        .catch(err => {
          console.error('🔐 Failed to trigger PIN request:', err)
          setError('Failed to request PIN from device')
          setIsWaitingForPinRequest(false)
          clearTimeout(timeout)
        })
      
      return () => clearTimeout(timeout)
    }
  }, [isOpen, deviceId, currentPinRequest, isWaitingForPinRequest])

  // Clean up state when dialog closes
  useEffect(() => {
    if (!isOpen) {
      setCurrentPinRequest(null)
      setPinMatrix([])
      setError(null)
      setIsWaitingForPinRequest(false)
    }
  }, [isOpen])

  const handleMatrixClick = (position: number) => {
    if (pinMatrix.length < 9) { // Max PIN length
      setPinMatrix([...pinMatrix, position])
    }
  }

  const handleClear = () => {
    setPinMatrix([])
    setError(null)
  }

  const handleSubmit = async () => {
    if (pinMatrix.length === 0) {
      setError('Please enter your PIN')
      return
    }

    if (!currentPinRequest) {
      setError('No active PIN request. Please try again.')
      return
    }

    setIsLoading(true)
    setError(null)

    try {
      console.log('🔐 Sending PIN response for session:', currentPinRequest.session_id)
      
      const success = await invoke<boolean>('send_pin_matrix_ack', {
        sessionId: currentPinRequest.session_id,
        positions: pinMatrix
      })
      
      if (success) {
        console.log('🔐 PIN sent successfully!')
        onUnlocked()
      } else {
        setError('Failed to send PIN to device')
        handleClear()
      }
    } catch (err) {
      console.error('🔐 PIN submission error:', err)
      setError(err instanceof Error ? err.message : 'Failed to unlock device')
      handleClear()
    } finally {
      setIsLoading(false)
    }
  }

  const handleCancel = () => {
    setPinMatrix([])
    setError(null)
    setCurrentPinRequest(null)
    setIsWaitingForPinRequest(false)
    onClose()
  }

  const handleRetryPinRequest = () => {
    console.log('🔄 Retrying PIN request...')
    setCurrentPinRequest(null)
    setIsWaitingForPinRequest(true)
    setError(null)
    setPinMatrix([])
    
    invoke('trigger_pin_request', { deviceId })
      .then(() => {
        console.log('🔄 PIN request retry triggered successfully')
      })
      .catch(err => {
        console.error('🔄 Failed to retry PIN request:', err)
        setError('Failed to request PIN from device')
        setIsWaitingForPinRequest(false)
      })
  }

  if (!isOpen) return null

  return (
    <div style={{ 
      position: 'fixed', 
      top: 0, 
      left: 0, 
      right: 0, 
      bottom: 0, 
      backgroundColor: 'rgba(0, 0, 0, 0.8)', 
      display: 'flex', 
      alignItems: 'center', 
      justifyContent: 'center', 
      zIndex: 9999 
    }}>
      <Box
        maxW="500px"
        bg="gray.800"
        borderRadius="xl"
        boxShadow="xl"
        borderWidth="1px"
        borderColor="gray.700"
        overflow="hidden"
        w="90%"
      >
        <Box bg="gray.850" p={6}>
          <Text fontSize="2xl" fontWeight="bold" color="white" textAlign="center">
            {currentPinRequest ? 'Enter PIN to Unlock' : 'Requesting PIN...'}
          </Text>
        </Box>
        
        <Box p={6}>
          <VStack gap={6}>
            <Text 
              color="gray.400" 
              textAlign="center"
              fontSize="md"
              lineHeight="1.6"
            >
              {isWaitingForPinRequest 
                ? 'Requesting PIN from your KeepKey device...'
                : currentPinRequest 
                  ? currentPinRequest.message + '. Look at your device and click the corresponding positions below.'
                  : 'Please wait while we request a PIN from your device.'
              }
            </Text>

            {error && (
              <Text color="red.400" fontSize="sm" textAlign="center" p={3} bg="red.900" borderRadius="md" borderWidth="1px" borderColor="red.600">
                {error}
              </Text>
            )}

            {currentPinRequest && (
              <>
                {/* PIN Dots Display */}
                <VStack gap={2}>
                  <Box
                    p={4}
                    bg="gray.700"
                    borderRadius="lg"
                    borderWidth="2px"
                    borderColor={pinMatrix.length > 0 ? "blue.500" : "gray.600"}
                    transition="all 0.2s"
                  >
                    <HStack gap={2} justify="center">
                      {Array.from({ length: Math.max(4, pinMatrix.length) }, (_, i) => (
                        <Box
                          key={i}
                          w="12px"
                          h="12px"
                          borderRadius="full"
                          bg={i < pinMatrix.length ? "blue.400" : "gray.500"}
                          opacity={i < pinMatrix.length ? 1 : 0.5}
                        />
                      ))}
                    </HStack>
                  </Box>
                  
                  {pinMatrix.length === 0 && (
                    <Text fontSize="xs" color="blue.400" textAlign="center">
                      Look at your KeepKey device and click the corresponding positions
                    </Text>
                  )}
                </VStack>

                {/* PIN Matrix */}
                <SimpleGrid
                  columns={3}
                  gap={4}
                  w="250px"
                >
                  {[1, 2, 3, 4, 5, 6, 7, 8, 9].map((position) => (
                    <Button
                      key={position}
                      onClick={() => handleMatrixClick(position)}
                      size="lg"
                      h="60px"
                      bg="gray.700"
                      borderColor="gray.600"
                      borderWidth="1px"
                      color="gray.300"
                      _hover={{
                        bg: "gray.600",
                        borderColor: "blue.500",
                        transform: "scale(1.05)",
                      }}
                      _active={{
                        bg: "gray.500",
                        transform: "scale(0.95)",
                      }}
                      transition="all 0.2s"
                      disabled={isLoading || pinMatrix.length >= 9}
                    >
                      <Icon as={FaCircle} boxSize={4} />
                    </Button>
                  ))}
                </SimpleGrid>

                {/* Action Buttons */}
                <VStack gap={4} w="full">
                  <HStack gap={4} w="full">
                    <Button
                      onClick={handleClear}
                      variant="outline"
                      size="lg"
                      flex={1}
                      borderColor="gray.600"
                      color="gray.300"
                      fontSize="md"
                      fontWeight="semibold"
                      _hover={{
                        bg: "gray.700",
                        borderColor: "gray.500",
                      }}
                      disabled={isLoading || pinMatrix.length === 0}
                    >
                      Clear
                    </Button>
                    
                    <Button
                      onClick={handleCancel}
                      variant="outline"
                      size="lg"
                      flex={1}
                      borderColor="gray.600"
                      color="gray.300"
                      fontSize="md"
                      fontWeight="semibold"
                      _hover={{
                        bg: "gray.700",
                        borderColor: "gray.500",
                      }}
                      disabled={isLoading}
                    >
                      Cancel
                    </Button>
                    
                    <Button
                      onClick={handleSubmit}
                      colorScheme="blue"
                      size="lg"
                      flex={1}
                      fontSize="md"
                      fontWeight="semibold"
                      _hover={{
                        transform: "translateY(-1px)",
                        boxShadow: "lg",
                      }}
                      transition="all 0.2s"
                      disabled={isLoading || pinMatrix.length === 0}
                    >
                      {isLoading ? 'Unlocking...' : 'Unlock'}
                    </Button>
                  </HStack>
                </VStack>
              </>
            )}

            {/* Retry button if no PIN request received or error occurred */}
            {!currentPinRequest && (!isWaitingForPinRequest || error) && (
              <Button
                onClick={handleRetryPinRequest}
                variant="outline"
                size="md"
                w="full"
                borderColor="yellow.600"
                color="yellow.300"
                fontSize="sm"
                fontWeight="medium"
                _hover={{
                  bg: "yellow.900",
                  borderColor: "yellow.500",
                }}
                disabled={isLoading || isWaitingForPinRequest}
              >
                🔄 Retry PIN Request
              </Button>
            )}

            <Text fontSize="xs" color="gray.500" textAlign="center">
              The numbered positions correspond to the layout shown on your KeepKey device screen.
            </Text>
          </VStack>
        </Box>
      </Box>
    </div>
  )
} 