import { useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { Box, VStack, HStack, Text, Button, SimpleGrid, Icon } from '@chakra-ui/react'
import { FaCircle } from 'react-icons/fa'

interface PinUnlockDialogProps {
  isOpen: boolean
  deviceId: string
  onUnlocked: () => void
  onClose: () => void
}

interface PinMatrixResult {
  success: boolean
  next_step?: string
  session_id: string
  error?: string
}

interface PinCreationSession {
  device_id: string
  session_id: string
  current_step: string
  is_active: boolean
}

export const PinUnlockDialog = ({ isOpen, deviceId, onUnlocked, onClose }: PinUnlockDialogProps) => {
  const [pinMatrix, setPinMatrix] = useState<number[]>([])
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [unlockSession, setUnlockSession] = useState<PinCreationSession | null>(null)

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

    setIsLoading(true)
    setError(null)

    try {
      // First, start PIN unlock session if we don't have one
      if (!unlockSession) {
        console.log('🔒 Starting PIN unlock session for device:', deviceId)
        const session = await invoke<PinCreationSession>('start_pin_unlock', { 
          deviceId 
        })
        console.log('🔒 PIN unlock session created:', session)
        setUnlockSession(session)
        
        // Now send the PIN
        const result = await invoke<PinMatrixResult>('send_pin_unlock_response', {
          sessionId: session.session_id,
          positions: pinMatrix
        })
        
        console.log('🔒 PIN unlock result:', result)
        
        if (result.success) {
          console.log('🔒 PIN unlock successful!')
          onUnlocked()
        } else {
          setError(result.error || 'PIN unlock failed')
          handleClear()
        }
      } else {
        // We already have a session, just send the PIN
        const result = await invoke<PinMatrixResult>('send_pin_unlock_response', {
          sessionId: unlockSession.session_id,
          positions: pinMatrix
        })
        
        console.log('🔒 PIN unlock result:', result)
        
        if (result.success) {
          console.log('🔒 PIN unlock successful!')
          onUnlocked()
        } else {
          setError(result.error || 'PIN unlock failed')
          handleClear()
        }
      }
    } catch (err) {
      console.error('🔒 PIN unlock error:', err)
      setError(err instanceof Error ? err.message : 'Failed to unlock device')
      handleClear()
    } finally {
      setIsLoading(false)
    }
  }

  const handleCancel = async () => {
    // Cancel the unlock session if it exists
    if (unlockSession) {
      try {
        await invoke('cancel_pin_creation', { sessionId: unlockSession.session_id })
      } catch (err) {
        console.error('Failed to cancel PIN unlock session:', err)
      }
    }
    
    setPinMatrix([])
    setError(null)
    setUnlockSession(null)
    onClose()
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
            Enter PIN to Unlock
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
              Your KeepKey device is locked. Enter your PIN using the matrix displayed on your device.
            </Text>

            {error && (
              <Text color="red.400" fontSize="sm" textAlign="center" p={3} bg="red.900" borderRadius="md" borderWidth="1px" borderColor="red.600">
                {error}
              </Text>
            )}

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

            <Text fontSize="xs" color="gray.500" textAlign="center">
              The numbered positions correspond to the layout shown on your KeepKey device screen.
            </Text>
          </VStack>
        </Box>
      </Box>
    </div>
  )
} 