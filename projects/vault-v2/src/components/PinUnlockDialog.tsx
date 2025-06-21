import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { Button, Grid, Text, HStack, Icon, VStack, Box, Spinner } from '@chakra-ui/react'
import { FaCircle, FaExclamationTriangle, FaTimes, FaCheckCircle, FaSync } from 'react-icons/fa'

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
  const [step, setStep] = useState<'verifying' | 'trigger' | 'enter' | 'submitting' | 'success'>('verifying')
  const [retryCount, setRetryCount] = useState(0)
  const [deviceReadyStatus, setDeviceReadyStatus] = useState<string>('Checking device...')

  // Reset state when dialog opens/closes
  useEffect(() => {
    if (isOpen) {
      setPinPositions([])
      setError(null)
      setStep('verifying')
      setRetryCount(0)
      setDeviceReadyStatus('Checking device...')
      verifyDeviceReadiness()
    }
  }, [isOpen, deviceId])

  const verifyDeviceReadiness = async () => {
    try {
      setIsLoading(true)
      setError(null)
      setDeviceReadyStatus('Verifying device is ready for PIN...')
      console.log('🔍 Verifying device readiness for PIN unlock:', deviceId)
      
      // Use the dedicated device PIN readiness check
      const isPinReady = await invoke('check_device_pin_ready', { deviceId })
      console.log('📊 Device PIN ready status:', isPinReady)
      
      if (!isPinReady) {
        // Device is not ready or no longer needs PIN unlock
        console.log('✅ Device no longer needs PIN unlock or is not ready, closing dialog')
        onUnlocked()
        return
      }
      
      // Device is ready for PIN unlock attempt
      setDeviceReadyStatus('Device ready - requesting PIN matrix...')
      await triggerPinRequest()
      
    } catch (err: any) {
      console.error('❌ Device readiness verification failed:', err)
      setError(`Device not ready: ${err}`)
      setStep('trigger')
      setDeviceReadyStatus('Device not ready')
    } finally {
      setIsLoading(false)
    }
  }

  const triggerPinRequest = async () => {
    try {
      setIsLoading(true)
      setError(null)
      setDeviceReadyStatus('Requesting PIN matrix from device...')
      console.log('🔐 Triggering PIN request for device:', deviceId)
      
      const result = await invoke('trigger_pin_request', { deviceId })
      
      if (result === true) {
        // PIN request was successfully triggered - device should now be showing PIN matrix
        console.log('✅ PIN trigger successful, device should be showing PIN matrix')
        setStep('enter')
        setDeviceReadyStatus('PIN matrix ready')
      } else {
        throw new Error('PIN trigger returned unexpected result')
      }
      
    } catch (err: any) {
      console.error('❌ PIN trigger failed:', err)
      
      const errorStr = String(err).toLowerCase()
      
      // Check if this is a device communication issue
      if (errorStr.includes('device not found') || errorStr.includes('not connected')) {
        setError('Device disconnected. Please reconnect your KeepKey and try again.')
        setStep('trigger')
      } else if (errorStr.includes('device already in use') || errorStr.includes('claimed')) {
        setError('Device is being used by another application. Please close other wallet software and try again.')
        setStep('trigger')
      } else if (errorStr.includes('timeout')) {
        setError('Device communication timeout. Please check your connection and try again.')
        setStep('trigger')
      } else {
        // For other errors, allow retry but show the specific error
        const userFriendlyError = errorStr.includes('failed to trigger pin request') 
          ? 'Unable to request PIN from device. Please check your device screen and try again.'
          : `Communication error: ${err}`
        
        setError(userFriendlyError)
        setStep('trigger')
      }
      
      setDeviceReadyStatus('PIN request failed')
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
      
      // PIN submitted successfully - show success feedback briefly before closing
      console.log('✅ PIN submitted successfully')
      setStep('success')
      
      // Auto-close after brief success display
      setTimeout(() => {
        onUnlocked()
      }, 1000)
      
    } catch (err: any) {
      console.error('❌ PIN submission failed:', err)
      
      // This is a real PIN validation error - show it clearly
      const errorStr = String(err)
      if (errorStr.toLowerCase().includes('incorrect') || errorStr.toLowerCase().includes('invalid') || errorStr.toLowerCase().includes('wrong')) {
        setError('Incorrect PIN. Please check your device screen and try again.')
      } else if (errorStr.toLowerCase().includes('device not found')) {
        setError('Device disconnected during PIN entry. Please reconnect and try again.')
      } else {
        setError(`PIN verification failed: ${err}`)
      }
      
      // Reset to PIN entry step and clear entered PIN
      setPinPositions([])
      setStep('enter')
    }
  }

  const handleClearPin = () => {
    setPinPositions([])
  }

  const handleRetry = () => {
    setError(null)
    setRetryCount(prev => prev + 1)
    
    if (retryCount >= 2) {
      // After multiple retries, go back to device verification
      setStep('verifying')
      verifyDeviceReadiness()
    } else {
      // Simple retry of PIN trigger
      setStep('trigger')
      triggerPinRequest()
    }
  }

  const getStatusMessage = () => {
    switch (step) {
      case 'verifying':
        return deviceReadyStatus
      case 'trigger':
        return isLoading ? 'Requesting PIN from device...' : 'Ready to request PIN matrix'
      case 'enter':
        return 'Device is showing PIN matrix'
      case 'submitting':
        return 'Verifying PIN...'
      case 'success':
        return 'PIN accepted - device unlocked!'
      default:
        return 'Processing...'
    }
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
          
          {step === 'verifying' && (
            <VStack gap={4}>
              <Spinner size="lg" color="blue.400" />
              <Text textAlign="center" fontSize="lg" fontWeight="bold">
                Preparing Device
              </Text>
              <Text fontSize="sm" color="gray.400" textAlign="center">
                {deviceReadyStatus}
              </Text>
            </VStack>
          )}

          {step === 'trigger' && (
            <VStack gap={4}>
              <Text textAlign="center">
                {isLoading ? 'Requesting PIN from device...' : 'Ready to unlock device'}
              </Text>
              <Text fontSize="sm" color="gray.400" textAlign="center">
                Your device will display a PIN matrix when ready
              </Text>
              {isLoading ? (
                <Spinner size="md" color="blue.400" />
              ) : (
                <Button onClick={triggerPinRequest} colorScheme="blue">
                  Request PIN Matrix
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
              <Spinner size="lg" color="blue.400" />
              <Text textAlign="center">Verifying PIN...</Text>
              <Text fontSize="sm" color="gray.400" textAlign="center">
                Please wait while your device processes the PIN
              </Text>
            </VStack>
          )}

          {step === 'success' && (
            <VStack gap={4}>
              <Icon as={FaCheckCircle} color="green.400" boxSize={12} />
              <Text textAlign="center" fontSize="lg" fontWeight="bold" color="green.400">
                Device Unlocked!
              </Text>
              <Text fontSize="sm" color="gray.400" textAlign="center">
                Your KeepKey is now ready to use
              </Text>
            </VStack>
          )}

          {/* Status message */}
          {step !== 'success' && step !== 'submitting' && (
            <Box bg="gray.700" borderRadius="md" p={3} w="full">
              <Text fontSize="sm" color="gray.300" textAlign="center">
                Status: {getStatusMessage()}
              </Text>
            </Box>
          )}

          {/* Error display */}
          {error && step !== 'success' && (
            <Box bg="red.900" borderColor="red.700" border="1px solid" p={4} borderRadius="md" w="full">
              <HStack gap={3} align="start">
                <Icon as={FaExclamationTriangle} color="red.400" mt={1} />
                <VStack align="start" gap={2} flex={1}>
                  <Text fontSize="sm" color="red.100">{error}</Text>
                  <Button 
                    size="sm" 
                    colorScheme="red" 
                    variant="outline" 
                    onClick={handleRetry}
                  >
                    <Icon as={FaSync} mr={2} />
                    {retryCount >= 2 ? 'Full Retry' : 'Try Again'}
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