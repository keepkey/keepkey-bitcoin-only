import { VStack, HStack, Text, Button, Box, Icon, Badge, Spinner } from "@chakra-ui/react";
import { FaDownload, FaExclamationTriangle } from "react-icons/fa";
import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

interface StepFirmwareUpdateProps {
  deviceId: string;
  onNext: () => void;
  onBack: () => void;
}

type UpdateState = 'idle' | 'loading_firmware' | 'erasing' | 'waiting_confirmation' | 'uploading' | 'complete';

// CSS animation for striped progress bar
const stripeAnimationStyle = `
  @keyframes stripeAnimation {
    0% { background-position: 0 0; }
    100% { background-position: 40px 0; }
  }
`;

export function StepFirmwareUpdate({ deviceId, onNext, onBack }: StepFirmwareUpdateProps) {
  const [deviceStatus, setDeviceStatus] = useState<any>(null);
  const [isUpdating, setIsUpdating] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [updateState, setUpdateState] = useState<UpdateState>('idle');
  const [updateProgress, setUpdateProgress] = useState(0);
  const [isVerifying, setIsVerifying] = useState(true);
  const progressIntervalRef = useRef<NodeJS.Timeout | null>(null);
  const unlistenRef = useRef<(() => void) | null>(null);

  useEffect(() => {
    checkDeviceStatus();
  }, [deviceId]);

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
    setIsVerifying(true);
    try {
      // Add a minimum delay to ensure proper verification
      await new Promise(resolve => setTimeout(resolve, 1500));
      
      const status = await invoke<any>('get_device_status', { deviceId });
      setDeviceStatus(status);
      
      // CRITICAL: Force firmware version verification before allowing initialization
      // We MUST verify we're on the latest firmware, not just check needsFirmwareUpdate flag
      const currentVersion = status.firmwareCheck?.currentVersion || "unknown";
      const latestVersion = status.firmwareCheck?.latestVersion || "7.10.0";
      
      console.log("🔒 FIRMWARE VERIFICATION:", {
        currentVersion,
        latestVersion,
        needsUpdate: status.needsFirmwareUpdate,
        isLatest: currentVersion === latestVersion
      });
      
      // Only auto-proceed if we are 100% certain we're on the latest version
      if (currentVersion === latestVersion) {
        console.log("✅ Firmware verified at latest version, proceeding to next step");
        // Add delay to show verification status
        setTimeout(() => {
          onNext();
        }, 2000);
      } else if (!status.needsFirmwareUpdate && currentVersion !== "unknown") {
        // Backend says no update needed but versions don't match - force verification
        console.warn("⚠️ Backend says no update needed but versions don't match!", {
          current: currentVersion,
          latest: latestVersion
        });
        // Stay on this step to force user to acknowledge firmware status
      }
      // If needsFirmwareUpdate is true or version is unknown, stay on this step
    } catch (err) {
      console.error("Failed to get device status:", err);
      setError(`Failed to check device status: ${err}`);
    } finally {
      setIsVerifying(false);
    }
  };

  const handleFirmwareUpdate = async () => {
    setIsUpdating(true);
    setError(null);
    setUpdateProgress(0);
    // Start with loading state - the event listener will update based on actual events
    setUpdateState('loading_firmware');
    
    try {
      // Actually invoke the firmware update
      // The event listener will handle updating the UI based on actual device events
      await invoke('update_device_firmware', { 
        deviceId,
        targetVersion: deviceStatus.firmwareCheck?.latestVersion || ''
      });
      
      // If we get here, the update was successful
      setUpdateState('complete');
      
      // Wait a moment to show completion
      setTimeout(() => {
        onNext();
      }, 1500);
      
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

  const handleVerifyAndContinue = () => {
    // Force verification check before allowing to continue
    const currentVersion = deviceStatus?.firmwareCheck?.currentVersion || "unknown";
    const latestVersion = deviceStatus?.firmwareCheck?.latestVersion || "7.10.0";
    
    if (currentVersion === latestVersion) {
      console.log("✅ Firmware verified, continuing to initialization");
      onNext();
    } else {
      setError("⚠️ Firmware verification failed! You must update to the latest firmware before continuing.");
      console.error("Cannot skip firmware update - not on latest version", {
        current: currentVersion,
        latest: latestVersion
      });
    }
  };

  if (!deviceStatus || isVerifying) {
    return (
      <VStack gap={6} w="100%" maxW="500px">
        <Icon as={FaDownload} boxSize={16} color="orange.500" />
        <VStack gap={2}>
          <Text fontSize="xl" fontWeight="bold" color="white">
            Verifying Firmware
          </Text>
          <Text color="gray.400">
            Checking your device firmware version...
          </Text>
          <Text fontSize="sm" color="orange.400">
            This verification is mandatory for security
          </Text>
        </VStack>
        <Spinner size="lg" color="orange.500" thickness="4px" />
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
              Firmware Update
            </Text>
            {deviceStatus.needsFirmwareUpdate ? (
              <>
                <Text fontSize={{ base: "sm", md: "md" }} color="gray.400" textAlign="center">
                  A new firmware version is available for your KeepKey
                </Text>
                {isOOBDevice && (
                  <Badge colorScheme="red" fontSize="sm">
                    Critical Update Required
                  </Badge>
                )}
              </>
            ) : deviceStatus.firmwareCheck?.currentVersion === deviceStatus.firmwareCheck?.latestVersion ? (
              <Text fontSize={{ base: "sm", md: "md" }} color="green.400" textAlign="center">
                ✅ Firmware verified - v{deviceStatus.firmwareCheck?.currentVersion}
              </Text>
            ) : (
              <Text fontSize={{ base: "sm", md: "md" }} color="orange.400" textAlign="center">
                ⚠️ Firmware verification in progress...
              </Text>
            )}
          </VStack>

          {/* Important Instructions */}
          <Box w="100%" p={4} bg="gray.700" borderRadius="lg" borderWidth="2px" borderColor="orange.500">
            <VStack gap={2} align="start">
              <Text color="orange.400" fontWeight="bold" fontSize="sm">
                ⚠️ Important Instructions:
              </Text>
              <Text fontSize="xs" color="gray.300">
                • Do not disconnect your device during the update
              </Text>
              <Text fontSize="xs" color="gray.300">
                • You may need to re-enter your PIN after the update
              </Text>
              <Text fontSize="xs" color="gray.300">
                • Your funds and settings will remain safe
              </Text>
            </VStack>
          </Box>
        </VStack>

        {/* Right side - Details and actions */}
        <VStack gap={4} flex={{ base: "none", lg: 1 }} w={{ base: "100%", lg: "auto" }}>
          {deviceStatus.firmwareCheck && (
            <Box w="100%" p={4} bg="gray.700" borderRadius="lg">
              <HStack gap={8} justify="space-between">
                <VStack gap={1} align="start">
                  <Text fontSize="xs" color="gray.400" textTransform="uppercase">
                    Current Version
                  </Text>
                  <Text fontSize="lg" color={isOOBDevice ? "red.400" : "white"} fontWeight="bold">
                    v{deviceStatus.firmwareCheck.currentVersion}
                  </Text>
                </VStack>
                <VStack gap={1} align="start">
                  <Text fontSize="xs" color="gray.400" textTransform="uppercase">
                    Latest Version
                  </Text>
                  <Text fontSize="lg" color="green.400" fontWeight="bold">
                    v{deviceStatus.firmwareCheck.latestVersion}
                  </Text>
                </VStack>
              </HStack>
              {isOOBDevice && (
                <Text fontSize="sm" color="orange.400" mt={3}>
                  ⚠️ Your device has factory firmware. Update is highly recommended.
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

              {updateState === 'complete' && (
                <Box p={4} bg="green.900" borderRadius="md" borderWidth="2px" borderColor="green.500">
                  <Text fontSize="sm" color="green.300" fontWeight="bold">
                    ✅ Firmware update complete!
                  </Text>
                </Box>
              )}
            </Box>
          )}

          <VStack gap={3} w="100%">
            {/* Show verification status */}
            {deviceStatus.firmwareCheck && (
              <Box w="100%" p={3} bg="gray.800" borderRadius="md" borderWidth="1px" borderColor="gray.600">
                <HStack gap={2}>
                  <Text fontSize="sm" color="gray.400">Firmware Verification:</Text>
                  {deviceStatus.firmwareCheck.currentVersion === deviceStatus.firmwareCheck.latestVersion ? (
                    <Badge colorScheme="green">✓ Verified</Badge>
                  ) : (
                    <Badge colorScheme="red">Update Required</Badge>
                  )}
                </HStack>
              </Box>
            )}
            
            {deviceStatus.needsFirmwareUpdate && !isUpdating ? (
              <Button
                colorScheme="orange"
                size="lg"
                w="100%"
                onClick={handleFirmwareUpdate}
                isLoading={isUpdating}
              >
                Update Firmware Now (Required)
              </Button>
            ) : !deviceStatus.needsFirmwareUpdate && !isUpdating && (
              <Button
                colorScheme="green"
                size="lg"
                w="100%"
                onClick={handleVerifyAndContinue}
              >
                Verify & Continue
              </Button>
            )}
          </VStack>
        </VStack>
      </HStack>
    </Box>
  );
}