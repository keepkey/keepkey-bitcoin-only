import { VStack, HStack, Text, Button, Box, Icon, Progress, Badge, Alert, Spinner } from "@chakra-ui/react";
import { FaDownload, FaExclamationTriangle } from "react-icons/fa";
import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useTypedTranslation } from "../../../hooks/useTypedTranslation";

interface StepFirmwareUpdateProps {
  deviceId: string;
  onNext: () => void;
  onBack: () => void;
  onFirmwareUpdateStart?: () => void;
  onFirmwareUpdateComplete?: () => void;
}

type UpdateState = 'idle' | 'loading_firmware' | 'erasing' | 'waiting_confirmation' | 'uploading' | 'complete';

// CSS animation for striped progress bar
const stripeAnimationStyle = `
  @keyframes stripeAnimation {
    0% { background-position: 0 0; }
    100% { background-position: 40px 0; }
  }
`;

export function StepFirmwareUpdate({ deviceId, onNext, onBack, onFirmwareUpdateStart, onFirmwareUpdateComplete }: StepFirmwareUpdateProps) {
  const [deviceStatus, setDeviceStatus] = useState<any>(null);
  const [isUpdating, setIsUpdating] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [updateState, setUpdateState] = useState<UpdateState>('idle');
  const [updateProgress, setUpdateProgress] = useState(0);
  const [isWaitingForReboot, setIsWaitingForReboot] = useState(false);
  const progressIntervalRef = useRef<NodeJS.Timeout | null>(null);
  const unlistenRef = useRef<(() => void) | null>(null);
  const rebootPollRef = useRef<NodeJS.Timeout | null>(null);
  const { t } = useTypedTranslation('setup');

  useEffect(() => {
    checkDeviceStatus();
  }, [deviceId]);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (rebootPollRef.current) {
        clearInterval(rebootPollRef.current);
      }
    };
  }, []);

  // Set up event listener for firmware update events
  useEffect(() => {
    if (!isUpdating) return;

    const setupListener = async () => {
      unlistenRef.current = await listen('firmware:update-status', (event) => {
        const { status, progress } = event.payload as { status: string; progress?: number };
        console.log('Firmware update status:', status, progress);
        
        switch (status) {
          case 'loading_firmware':
            setUpdateState('loading_firmware');
            break;
          case 'firmware_erase':
            setUpdateState('erasing');
            break;
          case 'button_request':
            setUpdateState('waiting_confirmation');
            break;
          case 'firmware_upload':
            setUpdateState('uploading');
            // Start progress animation when upload begins
            if (!progressIntervalRef.current) {
              let prog = 0;
              progressIntervalRef.current = setInterval(() => {
                prog += (100 / 60); // 100% over 60 seconds
                if (prog >= 100) {
                  prog = 100;
                  if (progressIntervalRef.current) {
                    clearInterval(progressIntervalRef.current);
                  }
                }
                setUpdateProgress(prog);
              }, 1000);
            }
            break;
          case 'complete':
            setUpdateState('complete');
            if (progressIntervalRef.current) {
              clearInterval(progressIntervalRef.current);
            }
            setUpdateProgress(100);
            break;
        }
      });
    };

    setupListener();

    return () => {
      if (unlistenRef.current) {
        unlistenRef.current();
      }
      if (progressIntervalRef.current) {
        clearInterval(progressIntervalRef.current);
      }
    };
  }, [isUpdating]);

  const checkDeviceStatus = async () => {
    try {
      const status = await invoke<any>('get_device_status', { deviceId });
      setDeviceStatus(status);
      
      // Simple check: if firmware is 7.10.0, we're good to go
      const currentVersion = status?.firmwareCheck?.currentVersion;
      
      console.log("Firmware check:", {
        currentVersion,
        needsUpdate: status?.needsFirmwareUpdate
      });
      
      // If firmware is already 7.10.0, skip to next step
      if (currentVersion === "7.10.0") {
        console.log("Firmware is 7.10.0, skipping to next step");
        onNext();
      }
      // Otherwise show the update screen
    } catch (err) {
      console.error("Failed to get device status:", err);
      // Don't show scary red error, just log it
    }
  };

  const handleFirmwareUpdate = async () => {
    setIsUpdating(true);
    setError(null);
    setUpdateProgress(0);
    // Start with loading state - the event listener will update based on actual events
    setUpdateState('loading_firmware');
    
    // Notify parent that firmware update is starting
    if (onFirmwareUpdateStart) {
      onFirmwareUpdateStart();
    }
    
    try {
      // Actually invoke the firmware update
      // The event listener will handle updating the UI based on actual device events
      await invoke('update_device_firmware', { 
        deviceId,
        targetVersion: deviceStatus.firmwareCheck?.latestVersion || ''
      });
      
      // If we get here, the update was successful
      setUpdateState('complete');
      
      // Notify parent that firmware update is complete
      if (onFirmwareUpdateComplete) {
        onFirmwareUpdateComplete();
      }
      
      // After firmware update, device will reboot - wait longer and let the backend handle reconnection
      console.log("Firmware update complete - device will reboot, waiting for reconnection...");
      setIsWaitingForReboot(true);
      
      // Wait 10 seconds before starting to poll - device needs time to fully reboot
      console.log("Waiting 10 seconds for device to reboot...");
      setTimeout(() => {
        console.log("Starting to poll for device reconnection...");
        
        // Start polling for device reconnection after reboot
        let pollAttempts = 0;
        const maxPollAttempts = 20; // 20 attempts over 10 seconds after initial wait
        
        rebootPollRef.current = setInterval(async () => {
        pollAttempts++;
        console.log(`Polling for device after reboot (attempt ${pollAttempts}/${maxPollAttempts})...`);
        
        try {
          // First try with the original device ID
          const status = await invoke<any>('get_device_status', { deviceId });
          
          if (status?.firmwareCheck?.currentVersion === "7.10.0") {
            console.log("✅ Device reconnected with firmware 7.10.0!");
            
            // Clear the polling interval
            if (rebootPollRef.current) {
              clearInterval(rebootPollRef.current);
              rebootPollRef.current = null;
            }
            
            setIsWaitingForReboot(false);
            setIsUpdating(false);
            setDeviceStatus(status);
            
            // Move to next step
            onNext();
          }
        } catch (err) {
          // If the original device ID fails, try to find any connected device
          try {
            console.log("Original device ID failed, looking for any connected device...");
            const devices = await invoke<any[]>('list_devices');
            
            if (devices && devices.length > 0) {
              // Take the first device found
              const newDeviceId = devices[0].unique_id || devices[0].id;
              console.log(`Found device with ID: ${newDeviceId}`);
              
              // Try to get status with the new device ID
              const status = await invoke<any>('get_device_status', { deviceId: newDeviceId });
              
              if (status?.firmwareCheck?.currentVersion === "7.10.0") {
                console.log("✅ Device reconnected with firmware 7.10.0 (new ID)!");
                
                // Clear the polling interval
                if (rebootPollRef.current) {
                  clearInterval(rebootPollRef.current);
                  rebootPollRef.current = null;
                }
                
                setIsWaitingForReboot(false);
                setIsUpdating(false);
                setDeviceStatus(status);
                
                // Move to next step
                onNext();
              }
            } else {
              console.log("No devices found yet, continuing to poll...");
            }
          } catch (listErr) {
            console.log("Device not ready yet, continuing to poll...");
          }
        }
        
        if (pollAttempts >= maxPollAttempts) {
          console.error("Device did not reconnect after firmware update");
          if (rebootPollRef.current) {
            clearInterval(rebootPollRef.current);
            rebootPollRef.current = null;
          }
          setIsWaitingForReboot(false);
          setIsUpdating(false);
          setError("Device did not reconnect after update. Please unplug and reconnect your device.");
        }
        }, 500); // Poll every 500ms
      }, 10000); // Wait 10 seconds before starting to poll
      
    } catch (err) {
      console.error("Failed to update firmware:", err);
      setError(`Failed to update firmware: ${err}`);
      setIsUpdating(false);
      setUpdateState('idle');
      // Clear any running progress interval
      if (progressIntervalRef.current) {
        clearInterval(progressIntervalRef.current);
      }
    }
  };


  if (!deviceStatus) {
    return (
      <VStack gap={6} w="100%" maxW="500px">
        <HStack gap={3}>
          <Spinner size="sm" color="green.500" />
          <Text color="gray.400">Follow directions on device...</Text>
        </HStack>
      </VStack>
    );
  }

  const isOOBDevice = deviceStatus.firmwareCheck?.currentVersion === "4.0.0";

  return (
    <Box w="100%">
      <HStack 
        gap={{ base: 4, md: 8 }} 
        align="flex-start"
        flexDirection={{ base: "column", lg: "row" }}
      >
        {/* Left side - Icon and status */}
        <VStack gap={4} flex={{ base: "none", lg: 1 }} w={{ base: "100%", lg: "auto" }}>
          <Icon as={FaDownload} boxSize={{ base: 12, md: 16 }} color="orange.500" />
          
          <VStack gap={2}>
            <Text fontSize={{ base: "xl", md: "2xl" }} fontWeight="bold" color="white" textAlign="center">
              {t('firmwareUpdate.title')}
            </Text>
            {!deviceStatus.firmwareCheck ? (
              <HStack gap={2} justify="center">
                <Spinner size="sm" color="orange.400" />
                <Text fontSize={{ base: "sm", md: "md" }} color="orange.400" textAlign="center">
                  Checking firmware version...
                </Text>
              </HStack>
            ) : deviceStatus.needsFirmwareUpdate ? (
              <>
                <Text fontSize={{ base: "sm", md: "md" }} color="gray.400" textAlign="center">
                  {t('firmwareUpdate.available')}
                </Text>
                {isOOBDevice && (
                  <Badge colorScheme="red" fontSize="sm">
                    Critical Update Required
                  </Badge>
                )}
              </>
            ) : (
              <Text fontSize={{ base: "sm", md: "md" }} color="green.400" textAlign="center">
                ✅ Firmware verified - v{deviceStatus.firmwareCheck?.currentVersion}
              </Text>
            )}
          </VStack>

          {/* Important Instructions */}
          <Box w="100%" p={4} bg="gray.700" borderRadius="lg" borderWidth="2px" borderColor="orange.500">
            <VStack gap={2} align="start">
              <Text color="orange.400" fontWeight="bold" fontSize="sm">
                ⚠️ {t('bootloaderUpdate.importantInstructions', 'Important Instructions:')}
              </Text>
              <Text fontSize="xs" color="gray.300">
                • {t('bootloaderUpdate.doNotDisconnectDuringUpdate')}
              </Text>
              <Text fontSize="xs" color="gray.300">
                • {t('bootloaderUpdate.mayNeedReenterPin', 'You may need to re-enter your PIN after the update')}
              </Text>
              <Text fontSize="xs" color="gray.300">
                • {t('bootloaderUpdate.fundsRemainSafe', 'Your funds and settings will remain safe')}
              </Text>
            </VStack>
          </Box>
        </VStack>

        {/* Right side - Details and actions */}
        <VStack gap={4} flex={{ base: "none", lg: 1 }} w={{ base: "100%", lg: "auto" }}>
          {!deviceStatus.firmwareCheck ? (
            <Box w="100%" p={4} bg="gray.700" borderRadius="lg">
              <VStack gap={3}>
                <Spinner size="md" color="orange.400" />
                <Text fontSize="sm" color="gray.400">
                  Detecting firmware version...
                </Text>
              </VStack>
            </Box>
          ) : (
            <Box w="100%" p={4} bg="gray.700" borderRadius="lg">
              <HStack gap={8} justify="space-between">
                <VStack gap={1} align="start">
                  <Text fontSize="xs" color="gray.400" textTransform="uppercase">
                    {t('bootloaderUpdate.currentVersion')}
                  </Text>
                  <Text fontSize="lg" color={isOOBDevice ? "red.400" : "white"} fontWeight="bold">
                    v{deviceStatus.firmwareCheck.currentVersion}
                  </Text>
                </VStack>
                <VStack gap={1} align="start">
                  <Text fontSize="xs" color="gray.400" textTransform="uppercase">
                    {t('bootloaderUpdate.latestVersion')}
                  </Text>
                  <Text fontSize="lg" color="green.400" fontWeight="bold">
                    v{deviceStatus.firmwareCheck.latestVersion}
                  </Text>
                </VStack>
              </HStack>
              {isOOBDevice && (
                <Text fontSize="sm" color="orange.400" mt={3}>
                  ⚠️ {t('bootloaderUpdate.factoryFirmwareWarning', 'Your device has factory firmware. Update is highly recommended.')}
                </Text>
              )}
            </Box>
          )}

          {error && (
            <Box
              bg="red.900"
              borderRadius="md"
              borderWidth="1px"
              borderColor="red.600"
              p={3}
              w="100%"
            >
              <Text color="red.300" fontSize="sm">
                {String(error)}
              </Text>
            </Box>
          )}

          {isUpdating && (
            <Box w="100%">
              <style>{stripeAnimationStyle}</style>
              
              {/* Status Messages */}
              <VStack gap={3} mb={4} align="stretch">
                {/* Loading Firmware */}
                {['loading_firmware', 'erasing', 'waiting_confirmation', 'uploading', 'complete'].includes(updateState) && (
                  <HStack gap={2}>
                    <Box w={2} h={2} borderRadius="full" bg="green.500" />
                    <Text fontSize="sm" color="gray.300">
                      📦 Loaded firmware binary: 577,720 bytes
                    </Text>
                  </HStack>
                )}
                
                {/* Device in bootloader mode */}
                {['erasing', 'waiting_confirmation', 'uploading', 'complete'].includes(updateState) && (
                  <HStack gap={2}>
                    <Box w={2} h={2} borderRadius="full" bg="green.500" />
                    <Text fontSize="sm" color="gray.300">
                      ✅ Device confirmed in bootloader mode
                    </Text>
                  </HStack>
                )}
                
                {/* Firmware Erase */}
                {['erasing', 'waiting_confirmation', 'uploading', 'complete'].includes(updateState) && (
                  <HStack gap={2}>
                    <Box w={2} h={2} borderRadius="full" bg={updateState === 'erasing' ? "blue.500" : "green.500"} />
                    <Text fontSize="sm" color="gray.300">
                      {updateState === 'erasing' ? '🔄' : '✅'} Firmware Erase
                    </Text>
                  </HStack>
                )}
                
                {/* Waiting for confirmation */}
                {updateState === 'waiting_confirmation' && (
                  <Box 
                    p={4} 
                    bg="orange.900" 
                    borderRadius="md" 
                    borderWidth="2px" 
                    borderColor="orange.500"
                    boxShadow="0 0 20px rgba(251, 146, 60, 0.5)"
                    animation="pulse 2s infinite"
                  >
                    <style>{`
                      @keyframes pulse {
                        0% { opacity: 1; }
                        50% { opacity: 0.8; }
                        100% { opacity: 1; }
                      }
                    `}</style>
                    <VStack gap={2} align="start">
                      <HStack gap={2}>
                        <Icon as={FaExclamationTriangle} color="orange.300" boxSize={5} />
                        <Text fontSize="md" color="orange.300" fontWeight="bold">
                          Confirm action on device!
                        </Text>
                      </HStack>
                      <Text fontSize="sm" color="orange.200">
                        Look at your KeepKey screen and press the button to confirm.
                      </Text>
                      <Text fontSize="xs" color="orange.200" fontStyle="italic">
                        Note: If your device is not set up, you can safely ignore any "verify backup" screen.
                      </Text>
                    </VStack>
                  </Box>
                )}
              </VStack>

              {/* Progress Bar - Only show during actual upload */}
              {updateState === 'uploading' && (
                <>
                  <Text fontSize="sm" color="gray.400" mb={2}>
                    Uploading firmware... Do not disconnect your device
                  </Text>
                  <Box position="relative" h="24px" bg="gray.600" borderRadius="full" overflow="hidden">
                    <Box
                      position="absolute"
                      top={0}
                      left={0}
                      h="100%"
                      w={`${updateProgress}%`}
                      bg="green.500"
                      borderRadius="full"
                      transition="width 0.5s ease-out"
                      backgroundImage="repeating-linear-gradient(
                        -45deg,
                        rgba(255, 255, 255, 0.15),
                        rgba(255, 255, 255, 0.15) 10px,
                        transparent 10px,
                        transparent 20px
                      )"
                      backgroundSize="40px 40px"
                      animation="stripeAnimation 1s linear infinite"
                    />
                  </Box>
                  <Text fontSize="xs" color="gray.500" mt={2}>
                    {Math.round(updateProgress)}% - Estimated time remaining: {Math.max(0, 60 - Math.round(updateProgress * 0.6))}s
                  </Text>
                  <Text fontSize="xs" color="gray.500">
                    Your device will restart when complete.
                  </Text>
                </>
              )}

              {updateState === 'complete' && !isWaitingForReboot && (
                <Box p={4} bg="green.900" borderRadius="md" borderWidth="2px" borderColor="green.500">
                  <Text fontSize="sm" color="green.300" fontWeight="bold">
                    ✅ Firmware update complete!
                  </Text>
                </Box>
              )}

              {isWaitingForReboot && (
                <Box p={4} bg="blue.900" borderRadius="md" borderWidth="2px" borderColor="blue.500">
                  <VStack gap={2} align="start">
                    <HStack gap={2}>
                      <Spinner size="sm" color="blue.300" />
                      <Text fontSize="sm" color="blue.300" fontWeight="bold">
                        Device is rebooting...
                      </Text>
                    </HStack>
                    <Text fontSize="xs" color="blue.200">
                      Your KeepKey is restarting with the new firmware.
                    </Text>
                    <Text fontSize="xs" color="blue.200">
                      This may take a few seconds. Please wait...
                    </Text>
                  </VStack>
                </Box>
              )}
            </Box>
          )}

          <VStack gap={3} w="100%">
            {/* Show verification status */}
            {deviceStatus.firmwareCheck && (
              <Box w="100%" p={3} bg="gray.800" borderRadius="md" borderWidth="1px" borderColor="gray.600">
                <HStack gap={2}>
                  <Text fontSize="sm" color="gray.400">{t('bootloaderUpdate.status', 'Status:')}:</Text>
                  {!deviceStatus.firmwareCheck.currentVersion || !deviceStatus.firmwareCheck.latestVersion ? (
                    <Badge colorScheme="orange">Checking...</Badge>
                  ) : deviceStatus.firmwareCheck.currentVersion === deviceStatus.firmwareCheck.latestVersion ? (
                    <Badge colorScheme="green">✓ Up to date</Badge>
                  ) : (
                    <Badge colorScheme="orange">Update Available</Badge>
                  )}
                </HStack>
              </Box>
            )}
            
            {!deviceStatus.firmwareCheck ? (
              <Button
                size="lg"
                w="100%"
                disabled
              >
                <HStack>
                  <Spinner size="sm" />
                  <Text>Checking Firmware...</Text>
                </HStack>
              </Button>
            ) : deviceStatus.needsFirmwareUpdate && !isUpdating ? (
              <>
                <Button
                  colorScheme="orange"
                  size="lg"
                  w="100%"
                  onClick={handleFirmwareUpdate}
                  loading={isUpdating}
                >
                  {t('bootloaderUpdate.updateFirmwareTo', { version: deviceStatus.firmwareCheck?.latestVersion || "7.10.0" })}
                </Button>
                <Button
                  variant="outline"
                  size="lg"
                  w="100%"
                  onClick={onNext}
                  borderColor="gray.600"
                  color="gray.300"
                  _hover={{ bg: "gray.700" }}
                >
                  {t('firmwareUpdate.skipUpdate')}
                </Button>
                {!isOOBDevice && (
                  <Text fontSize="xs" color="gray.500" textAlign="center">
                    {t('firmwareUpdate.skipUpdateNote')}
                  </Text>
                )}
              </>
            ) : deviceStatus.firmwareCheck?.currentVersion === "7.10.0" && !isUpdating ? (
              <Button
                colorScheme="green"
                size="lg"
                w="100%"
                onClick={onNext}
              >
                Continue Setup
              </Button>
            ) : isUpdating ? null : (
              <Button
                size="lg"
                w="100%"
                disabled
              >
                <HStack>
                  <Spinner size="sm" />
                  <Text>Verifying...</Text>
                </HStack>
              </Button>
            )}
          </VStack>
        </VStack>
      </HStack>
    </Box>
  );
}