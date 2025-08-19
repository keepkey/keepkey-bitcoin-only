import { useState, useEffect, useCallback } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { Button, Text, HStack, Icon, VStack, Box, Spinner, SimpleGrid, Heading } from '@chakra-ui/react'
import { FaCircle, FaExclamationTriangle, FaTimes, FaCheckCircle, FaSync, FaBackspace, FaUsb } from 'react-icons/fa'
import { useTypedTranslation } from '../hooks/useTypedTranslation'

interface PinUnlockDialogProps {
  isOpen: boolean
  deviceId: string
  onUnlocked: () => void
  onClose: () => void
  isManagementOperation?: boolean // true when creating/removing PIN, false for normal unlock
}

// The KeepKey device shows this scrambled layout on its screen:
// 7 8 9
// 4 5 6
// 1 2 3
// We need to send these exact numbers when the user clicks each position
const PIN_MATRIX_LAYOUT = [7, 8, 9, 4, 5, 6, 1, 2, 3] as const

export const PinUnlockDialog = ({ isOpen, deviceId, onUnlocked, onClose, isManagementOperation = false }: PinUnlockDialogProps) => {
  const { t } = useTypedTranslation('dialogs')
  const [pinPositions, setPinPositions] = useState<number[]>([])
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [step, setStep] = useState<'verifying' | 'trigger' | 'enter' | 'submitting' | 'success' | 'reconnect'>('verifying')
  const [retryCount, setRetryCount] = useState(0)
  const [deviceReadyStatus, setDeviceReadyStatus] = useState<string>('Checking device...')

  // Debug component lifecycle
  useEffect(() => {
    console.log('🔒 PinUnlockDialog mounted, isOpen:', isOpen)
    return () => {
      console.log('🔒 PinUnlockDialog unmounting')
    }
  }, [])

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
      setDeviceReadyStatus('Preparing device for PIN entry...')
      console.log('🔍 Preparing PIN unlock for device:', deviceId)
      
      // Check if device is already in PIN flow
      const isInPinFlow = await invoke('check_device_in_pin_flow', { deviceId })
      if (isInPinFlow) {
        console.log('🔐 Device already in PIN flow, ready for PIN entry')
        setDeviceReadyStatus('Device ready - PIN matrix should be visible on device')
        setStep('enter')
        setIsLoading(false)
        return
      }
      
      // Since the backend already triggered PIN when emitting device:pin-request-triggered,
      // we should just go straight to enter mode
      console.log('🔐 Device PIN was already triggered by backend, ready for entry')
      setDeviceReadyStatus('Device ready - PIN matrix should be visible on device')
      setStep('enter')
      
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
      setDeviceReadyStatus('Checking device lock status...')
      console.log('🔐 Triggering PIN request for device:', deviceId)
      
      // Check if device is already in PIN flow to avoid duplicate requests
      const isInPinFlow = await invoke('check_device_in_pin_flow', { deviceId })
      if (isInPinFlow) {
        console.log('🔐 Device already in PIN flow, skipping trigger')
        setDeviceReadyStatus('Device ready - PIN matrix should be visible on device')
        setStep('enter')
        return
      }
      
      const result = await invoke('trigger_pin_request', { deviceId })
      
      if (result === true) {
        // Check if we actually got a PIN flow started
        // The backend now returns true for PassphraseRequest and Address responses too
        // which means the device is already unlocked
        
        // Small delay to check if device is actually showing PIN
        await new Promise(resolve => setTimeout(resolve, 500))
        
        // If the device was already unlocked (PassphraseRequest or Address response),
        // the backend would have unmarked the PIN flow
        const isInPinFlow = await invoke('check_device_in_pin_flow', { deviceId }).catch(() => false)
        
        if (isInPinFlow) {
          console.log('✅ PIN trigger successful, device should be showing PIN matrix')
          setStep('enter')
          setDeviceReadyStatus('PIN matrix ready')
          setError(null) // Clear any previous errors
        } else {
          console.log('✅ Device is already unlocked, no PIN needed')
          setStep('success')
          setDeviceReadyStatus('Device already unlocked')
          // Call onUnlocked callback since device is already unlocked
          setTimeout(() => {
            onUnlocked()
          }, 1000)
        }
      } else {
        throw new Error('PIN trigger returned unexpected result')
      }
      
    } catch (err: any) {
      console.error('❌ PIN trigger failed:', err)
      
      const errorStr = String(err).toLowerCase()
      
      // Check if device is already showing PIN matrix (expected "failure")
      if (errorStr.includes('unknown message') || errorStr.includes('failure: unknown message')) {
        console.log('🔐 Device is already in PIN mode (expected behavior), proceeding to PIN entry')
        setStep('enter')
        setDeviceReadyStatus('PIN matrix ready')
        setError(null)
        return
      }
      
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

  const handlePinButtonClick = useCallback((position: number) => {
    if (pinPositions.length >= 9) return // Max 9 digits
    
    console.log(`Button clicked: position ${position}`)
    setPinPositions(prev => [...prev, position])
  }, [pinPositions.length])

  const handleBackspace = useCallback(() => {
    setPinPositions(prev => prev.slice(0, -1))
  }, [])

  const handleSubmitPin = async () => {
    if (pinPositions.length === 0) {
      setError('Please enter your PIN')
      return
    }

    try {
      setStep('submitting')
      setError(null)
      console.log('🔐 Submitting PIN with positions:', pinPositions)
      
      const result = await invoke('send_pin_matrix_ack', { 
        deviceId, 
        positions: pinPositions 
      })
      
      if (result === true) {
        // PIN submitted successfully
        console.log('✅ PIN submitted successfully')
        
        // Only show reconnect prompt for PIN management operations (create/remove)
        // For normal unlock, just show success and close
        if (isManagementOperation) {
          console.log('🔄 PIN management operation - showing reconnect prompt')
          setStep('reconnect')
          // Don't auto-close - user needs to reconnect device
        } else {
          console.log('🔓 Normal PIN unlock - closing immediately to allow passphrase dialog')
          setStep('success')
          
          // Small delay to ensure backend has time to send passphrase_request event
          setTimeout(() => {
            console.log('🔒 Closing PIN dialog after successful submission')
            onUnlocked()
          }, 100)
        }
      } else {
        throw new Error('PIN verification failed')
      }
      
    } catch (err: any) {
      const errorStr = String(err)
      
      // Check if this is actually a success case (PassphraseRequest)
      // This should not happen anymore with the backend fix, but keeping for safety
      if (errorStr.includes('PassphraseRequest')) {
        console.log('✅ PIN accepted, device is requesting passphrase')
        // PIN was correct, device now needs passphrase
        // Close this dialog as PIN was successful
        onUnlocked()
        return
      }
      
      console.error('❌ PIN submission failed:', err)
      
      // This is a real PIN validation error - show it clearly
      if (errorStr.toLowerCase().includes('incorrect') || errorStr.toLowerCase().includes('invalid') || errorStr.toLowerCase().includes('wrong')) {
        setError('Incorrect PIN. Please check your device screen and try again.')
        setRetryCount(prev => prev + 1)
        
        // If too many failed attempts, warn user about device lockout
        if (retryCount >= 2) {
          setError('Incorrect PIN. Warning: Too many failed attempts may temporarily lock your device!')
        }
      } else if (errorStr.toLowerCase().includes('device not found')) {
        setError('Device disconnected during PIN entry. Please reconnect and try again.')
      } else if (errorStr.toLowerCase().includes('locked') || errorStr.toLowerCase().includes('too many')) {
        setError('Device is temporarily locked due to too many failed PIN attempts. Please wait and try again later.')
        setStep('trigger')
        return
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

  // Generate PIN dots for display
  const pinDots = Array.from({ length: Math.max(4, pinPositions.length) }, (_, i) => (
    <Box
      key={i}
      w="10px"
      h="10px"
      borderRadius="full"
      bg={i < pinPositions.length ? "green.400" : "gray.600"}
      opacity={i < pinPositions.length ? 1 : 0.5}
      transition="all 0.2s"
    />
  ))

  if (!isOpen) return null

  return (
    <Box
      position="fixed"
      top={0}
      left={0}
      right={0}
      bottom={0}
      bg="blackAlpha.800"
      zIndex={99999}
      display="flex"
      alignItems="center"
      justifyContent="center"
    >
      <Box
        bg="gray.800"
        color="white"
        borderRadius="xl"
        boxShadow="2xl"
        borderWidth="1px"
        borderColor="gray.700"
        overflow="hidden"
        maxW="400px"
        w="90%"
      >
        {/* Header */}
        <Box bg="gray.850" p={4} position="relative">
          <Heading fontSize="xl" fontWeight="bold" color="white" textAlign="center">
            {t('pin.unlock.title')}
          </Heading>
          <Button
            position="absolute"
            right={2}
            top="50%"
            transform="translateY(-50%)"
            size="sm"
            variant="ghost"
            onClick={onClose}
            color="gray.400"
            _hover={{ color: "white", bg: "gray.700" }}
            borderRadius="md"
          >
            <Icon as={FaTimes} />
          </Button>
        </Box>

        <Box p={5}>
          <VStack gap={4}>
            
            {step === 'verifying' && (
              <VStack gap={3} py={4}>
                <Spinner size="lg" color="blue.400" />
                <Text fontSize="md" fontWeight="semibold">
                  {t('pin.unlock.preparingDevice')}
                </Text>
                <Text fontSize="sm" color="gray.400">
                  {deviceReadyStatus}
                </Text>
              </VStack>
            )}

            {step === 'trigger' && (
              <VStack gap={3} py={4}>
                <Text color="gray.300" fontSize="sm">
                  {t('pin.unlock.deviceReadyForPin')}
                </Text>
                {isLoading ? (
                  <Spinner size="md" color="blue.400" />
                ) : (
                  <Button onClick={triggerPinRequest} colorScheme="blue" size="md">
                    {t('pin.unlock.requestPinMatrix')}
                  </Button>
                )}
              </VStack>
            )}

            {step === 'enter' && (
              <VStack gap={3} w="full">
                <VStack gap={1}>
                  <Text color="gray.300" fontSize="sm" textAlign="center">
                    {t('pin.unlock.lookAtDevice')}
                  </Text>
                  <Text color="gray.400" fontSize="xs" textAlign="center">
                    {t('pin.unlock.clickPositions')}
                  </Text>
                </VStack>

                {/* PIN dots display */}
                <Box
                  p={3}
                  bg="gray.750"
                  borderRadius="lg"
                  borderWidth="1px"
                  borderColor="gray.600"
                >
                  <HStack gap={1.5} justify="center">
                    {pinDots}
                  </HStack>
                </Box>

                {/* PIN Grid - matches device layout */}
                <Box>
                  <SimpleGrid
                    columns={3}
                    gap={2}
                    w="200px"
                    mx="auto"
                  >
                    {PIN_MATRIX_LAYOUT.map((number, index) => (
                      <Button
                        key={index}
                        onClick={() => handlePinButtonClick(number)}
                        size="md"
                        h="50px"
                        bg="gray.700"
                        borderColor="gray.600"
                        borderWidth="1px"
                        color="gray.300"
                        fontSize="lg"
                        fontWeight="bold"
                        position="relative"
                        _hover={{
                          bg: "gray.600",
                          borderColor: "green.500",
                          transform: "scale(1.05)",
                        }}
                        _active={{
                          bg: "gray.500",
                          transform: "scale(0.95)",
                        }}
                        transition="all 0.15s"
                        disabled={pinPositions.length >= 9}
                      >
                        <Icon as={FaCircle} boxSize={3} />
                      </Button>
                    ))}
                  </SimpleGrid>
                  <Text fontSize="10px" color="gray.600" textAlign="center" mt={1}>
                    {t('pin.unlock.numbersMatchNote')}
                  </Text>
                </Box>

                {/* Action buttons */}
                <HStack gap={2} w="full" pt={2}>
                  <Button 
                    onClick={handleBackspace}
                    variant="outline"
                    size="sm"
                    borderColor="gray.600"
                    color="gray.300"
                    _hover={{
                      bg: "gray.700",
                      borderColor: "gray.500",
                    }}
                    disabled={pinPositions.length === 0}
                    flex={1}
                  >
                    <Icon as={FaBackspace} mr={1} />
                    {t('pin.unlock.buttons.back')}
                  </Button>
                  <Button 
                    onClick={handleClearPin}
                    variant="outline"
                    size="sm"
                    borderColor="gray.600"
                    color="gray.300"
                    _hover={{
                      bg: "gray.700",
                      borderColor: "gray.500",
                    }}
                    disabled={pinPositions.length === 0}
                    flex={1}
                  >
                    {t('pin.unlock.buttons.clear')}
                  </Button>
                  <Button
                    onClick={handleSubmitPin}
                    colorScheme="green"
                    size="sm"
                    _hover={{
                      transform: "translateY(-1px)",
                      boxShadow: "lg",
                    }}
                    transition="all 0.2s"
                    disabled={pinPositions.length === 0}
                    flex={2}
                  >
                    {t('pin.unlock.buttons.unlock')}
                  </Button>
                </HStack>

                <Text fontSize="xs" color="gray.500" textAlign="center">
                  {t('pin.unlock.useScrambledLayout')}
                </Text>
              </VStack>
            )}

            {step === 'submitting' && (
              <VStack gap={3} py={4}>
                <Spinner size="lg" color="blue.400" />
                <Text fontSize="md" fontWeight="semibold">
                  {t('pin.unlock.verifyingPin')}
                </Text>
                <Text fontSize="sm" color="gray.400">
                  {t('pin.unlock.pleaseWait')}
                </Text>
              </VStack>
            )}

            {step === 'success' && (
              <VStack gap={3} py={4}>
                <Icon as={FaCheckCircle} color="green.400" boxSize={10} />
                <Text fontSize="md" fontWeight="semibold" color="green.400">
                  {t('pin.unlock.deviceUnlocked')}
                </Text>
              </VStack>
            )}

            {step === 'reconnect' && (
              <VStack gap={4} py={4}>
                <Icon as={FaUsb} color="blue.400" boxSize={10} />
                <VStack gap={2}>
                  <Text fontSize="md" fontWeight="semibold" color="white">
                    Please Reconnect Your KeepKey
                  </Text>
                  <Text fontSize="sm" color="gray.300" textAlign="center">
                    Your PIN was successfully entered. Please unplug your KeepKey and plug it back in to complete the unlock process.
                  </Text>
                </VStack>
                <Box
                  p={3}
                  bg="blue.900"
                  borderColor="blue.700"
                  border="1px solid"
                  borderRadius="md"
                  w="full"
                >
                  <VStack gap={2} align="start">
                    <Text fontSize="sm" fontWeight="semibold" color="blue.200">
                      Steps:
                    </Text>
                    <Text fontSize="xs" color="blue.100">
                      1. Unplug your KeepKey from the USB port
                    </Text>
                    <Text fontSize="xs" color="blue.100">
                      2. Wait 2-3 seconds
                    </Text>
                    <Text fontSize="xs" color="blue.100">
                      3. Plug your KeepKey back in
                    </Text>
                    <Text fontSize="xs" color="blue.100">
                      4. The device will be ready to use
                    </Text>
                  </VStack>
                </Box>
                <Button
                  onClick={onUnlocked}
                  colorScheme="blue"
                  size="md"
                  w="full"
                >
                  I've Reconnected My Device
                </Button>
              </VStack>
            )}

            {/* Error display */}
            {error && step !== 'success' && (
              <Box bg="red.900" borderColor="red.700" border="1px solid" p={3} borderRadius="md" w="full">
                <HStack gap={2} align="start">
                  <Icon as={FaExclamationTriangle} color="red.400" mt={0.5} boxSize={4} />
                  <VStack align="start" gap={1} flex={1}>
                    <Text fontSize="sm" color="red.100">{error}</Text>
                    <Button 
                      size="xs" 
                      colorScheme="red" 
                      variant="outline" 
                      onClick={handleRetry}
                    >
                      <Icon as={FaSync} mr={1} />
                      {retryCount >= 2 ? t('pin.unlock.fullRetry') : t('pin.unlock.tryAgain')}
                    </Button>
                  </VStack>
                </HStack>
              </Box>
            )}

          </VStack>
        </Box>
      </Box>
    </Box>
  )
} 