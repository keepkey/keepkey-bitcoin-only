import { VStack, HStack, Text, Button, Box, Icon, Image, Alert, Progress } from "@chakra-ui/react";
import { FaShieldAlt, FaExclamationTriangle } from "react-icons/fa";
import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import holdAndConnectSvg from '../../../assets/svg/hold-and-connect.svg';

interface StepBootloaderUpdateProps {
  deviceId: string;
  onNext: () => void;
  onBack: () => void;
}

// CSS animation for striped progress bar
const stripeAnimationStyle = `
  @keyframes stripeAnimation {
    0% { background-position: 0 0; }
    100% { background-position: 40px 0; }
  }
`;

export function StepBootloaderUpdate({ deviceId, onNext, onBack }: StepBootloaderUpdateProps) {
  const [deviceStatus, setDeviceStatus] = useState<any>(null);
  const [isUpdating, setIsUpdating] = useState(false);
  const [updateProgress, setUpdateProgress] = useState(0);
  const [showBootloaderInstructions, setShowBootloaderInstructions] = useState(false);
  const progressIntervalRef = useRef<NodeJS.Timeout | null>(null);

  useEffect(() => {
    // Only check device status if we have a deviceId
    if (deviceId) {
      checkDeviceStatus();
    }
    
    // If showing bootloader instructions, periodically check if device entered bootloader mode
    let intervalId: NodeJS.Timeout | null = null;
    if (showBootloaderInstructions && deviceId) {
      intervalId = setInterval(() => {
        checkDeviceStatus();
      }, 2000); // Check every 2 seconds
    }
    
    return () => {
      if (intervalId) {
        clearInterval(intervalId);
      }
    };
  }, [deviceId, showBootloaderInstructions]);

  const checkDeviceStatus = async () => {
    try {
      const status = await invoke<any>('get_device_status', { deviceId });
      
      // Check if status is null or undefined
      if (!status) {
        console.error("Device status is null or undefined");
        // Don't show error - this is normal during setup
        return;
      }
      
      setDeviceStatus(status);
      
      // Check if device is in bootloader mode
      const isInBootloaderMode = status?.features?.bootloader_mode || status?.features?.bootloaderMode || false;
      console.log('Bootloader mode check:', {
        hasFeatures: !!status?.features,
        bootloader_mode: status?.features?.bootloader_mode,
        bootloaderMode: status?.features?.bootloaderMode,
        isInBootloaderMode,
        needsBootloaderUpdate: status?.needsBootloaderUpdate
      });
      
      // CRITICAL: Force bootloader version verification
      const currentBootloaderVersion = status?.bootloaderCheck?.currentVersion || "unknown";
      const latestBootloaderVersion = status?.bootloaderCheck?.latestVersion || "2.1.5";
      
      console.log("🔒 BOOTLOADER VERIFICATION:", {
        currentVersion: currentBootloaderVersion,
        latestVersion: latestBootloaderVersion,
        needsUpdate: status?.needsBootloaderUpdate,
        isLatest: currentBootloaderVersion === latestBootloaderVersion
      });
      
      // Version 2.0.0 is OLD and MUST be updated!
      const isOldBootloader = currentBootloaderVersion === "2.0.0" || 
                             currentBootloaderVersion.startsWith("1.") ||
                             currentBootloaderVersion === "2.0.1" ||
                             currentBootloaderVersion === "2.0.2" ||
                             currentBootloaderVersion === "2.0.3" ||
                             currentBootloaderVersion === "2.0.4";
      
      if (isOldBootloader) {
        console.log("📦 Bootloader update required: v" + currentBootloaderVersion + " → v2.1.5");
        // Require the update before proceeding
        if (!isInBootloaderMode) {
          console.log("Device needs bootloader update but not in bootloader mode");
          setShowBootloaderInstructions(true);
        } else if (isInBootloaderMode && showBootloaderInstructions) {
          console.log("Device is now in bootloader mode for update");
          setShowBootloaderInstructions(false);
        }
        return; // Stay on this step - update required
      }
      
      // Only skip if bootloader is verified to be on latest version
      if (currentBootloaderVersion === latestBootloaderVersion) {
        console.log("✅ Bootloader verified at latest version", latestBootloaderVersion, ", proceeding to next step");
        onNext();
        return;
      } else if (!status?.needsBootloaderUpdate && currentBootloaderVersion !== "unknown") {
        // Backend says no update but versions don't match - this is suspicious
        console.warn("⚠️ Backend says no bootloader update needed but versions don't match!", {
          current: currentBootloaderVersion,
          latest: latestBootloaderVersion
        });
        // Stay on this step to force user acknowledgment
        return;
      } else if (!isInBootloaderMode) {
        // Device needs bootloader update but is not in bootloader mode
        console.log("Device needs bootloader update but not in bootloader mode");
        setShowBootloaderInstructions(true);
      } else if (isInBootloaderMode && showBootloaderInstructions) {
        // Device has entered bootloader mode, hide instructions
        console.log("Device is now in bootloader mode");
        setShowBootloaderInstructions(false);
      }
    } catch (err) {
      console.error("Failed to get device status:", err);
      // Don't show error - this is normal during setup
    }
  };

  const handleBootloaderUpdate = async () => {
    // First check if device is in bootloader mode
    const isInBootloaderMode = deviceStatus?.features?.bootloader_mode || deviceStatus?.features?.bootloaderMode || false;
    
    if (!isInBootloaderMode) {
      setShowBootloaderInstructions(true);
      return;
    }
    
    setIsUpdating(true);
    setUpdateProgress(0);
    
    // Start the 60-second progress animation
    let progress = 0;
    progressIntervalRef.current = setInterval(() => {
      progress += (100 / 60); // 100% over 60 seconds
      if (progress >= 100) {
        progress = 100;
        if (progressIntervalRef.current) {
          clearInterval(progressIntervalRef.current);
        }
      }
      setUpdateProgress(progress);
    }, 1000); // Update every second
    
    try {
      // Start bootloader update
      const success = await invoke('update_device_bootloader', {
        deviceId,
        targetVersion: deviceStatus.bootloaderCheck?.latestVersion || ''
      });
      
      // Clear the interval when done
      if (progressIntervalRef.current) {
        clearInterval(progressIntervalRef.current);
      }
      setUpdateProgress(100);
      
      // Wait a moment to show completion
      setTimeout(() => {
        onNext();
      }, 500);
      
    } catch (err) {
      console.error("Failed to update bootloader:", err);
      const errorMsg = String(err);
      
      // Check if the error is because device is not in bootloader mode
      if (errorMsg.includes('bootloader mode') || errorMsg.includes('Bootloader mode')) {
        setShowBootloaderInstructions(true);
      }
      // Don't show any errors to the user
      setIsUpdating(false);
      // Clear the interval on error
      if (progressIntervalRef.current) {
        clearInterval(progressIntervalRef.current);
      }
    }
  };

  // Cleanup interval on unmount
  useEffect(() => {
    return () => {
      if (progressIntervalRef.current) {
        clearInterval(progressIntervalRef.current);
      }
    };
  }, []);


  if (!deviceStatus) {
    return (
      <VStack gap={6} w="100%" maxW="500px">
        <Text color="gray.400">Checking device status...</Text>
      </VStack>
    );
  }

  // If showing bootloader instructions
  if (showBootloaderInstructions) {
    return (
      <Box w="100%">
        <HStack 
          gap={{ base: 4, md: 8 }} 
          align="flex-start"
          flexDirection={{ base: "column", lg: "row" }}
          w="100%"
        >
          {/* Left side - Instructions and image */}
          <VStack gap={4} flex={{ base: "none", lg: 1 }} w={{ base: "100%", lg: "auto" }}>
            <HStack gap={2}>
              <Icon as={FaExclamationTriangle} color="yellow.500" boxSize={6} />
              <Text fontSize="xl" fontWeight="bold" color="white">
                Enter Bootloader Mode
              </Text>
            </HStack>

            <Text fontSize="sm" color="gray.300" textAlign="center">
              To update the bootloader, your device must be in Bootloader Mode
            </Text>

            <Box display="flex" justifyContent="center" py={2}>
              <Image
                src={holdAndConnectSvg}
                alt="Hold button while connecting device"
                maxW={{ base: "200px", md: "240px" }}
                height="auto"
              />
            </Box>
          </VStack>

          {/* Right side - Steps and actions */}
          <VStack gap={4} flex={{ base: "none", lg: 1 }} w={{ base: "100%", lg: "auto" }}>
            <VStack align="stretch" gap={1} bg="gray.700" p={4} borderRadius="md" fontSize="sm" w="100%">
              <Text fontWeight="semibold" color="yellow.300">Quick Steps:</Text>
              <Text color="gray.200">1. Unplug your KeepKey device</Text>
              <Text color="gray.200">2. Hold the button and plug it back in</Text>
              <Text color="gray.200">3. Keep holding until "BOOTLOADER MODE" appears</Text>
              <Text color="gray.200">4. Release the button</Text>
            </VStack>

          </VStack>
        </HStack>
      </Box>
    );
  }

  // Check if this is an old bootloader that MUST be updated
  const currentBootloaderVersion = deviceStatus?.bootloaderCheck?.currentVersion || "unknown";
  const isOldBootloader = currentBootloaderVersion === "2.0.0" || 
                         currentBootloaderVersion.startsWith("1.") ||
                         currentBootloaderVersion === "2.0.1" ||
                         currentBootloaderVersion === "2.0.2" ||
                         currentBootloaderVersion === "2.0.3" ||
                         currentBootloaderVersion === "2.0.4";

  return (
    <Box w="100%">
      <HStack 
        gap={{ base: 4, md: 8 }} 
        align="flex-start"
        flexDirection={{ base: "column", lg: "row" }}
      >
        {/* Left side - Icon and status */}
        <VStack gap={4} flex={{ base: "none", lg: 1 }} w={{ base: "100%", lg: "auto" }}>
          <Icon as={FaShieldAlt} 
                boxSize={{ base: 12, md: 16 }} 
                color="blue.500" />
          
          <VStack gap={2}>
            <Text fontSize={{ base: "xl", md: "2xl" }} fontWeight="bold" color="white" textAlign="center">
              Bootloader Update
            </Text>
            {isOldBootloader ? (
              <>
                <Text fontSize={{ base: "sm", md: "md" }} color="blue.400" textAlign="center">
                  Update available for bootloader v{currentBootloaderVersion}
                </Text>
                <Text fontSize="xs" color="gray.400" textAlign="center">
                  Let's update to the latest version for the best experience
                </Text>
              </>
            ) : deviceStatus.needsBootloaderUpdate ? (
              <Text fontSize={{ base: "sm", md: "md" }} color="gray.400" textAlign="center">
                Your KeepKey bootloader needs to be updated!
              </Text>
            ) : (
              <Text fontSize={{ base: "sm", md: "md" }} color="green.400" textAlign="center">
                Your bootloader is up to date!
              </Text>
            )}
          </VStack>
          
          {/* Update info box for old bootloaders */}
          {isOldBootloader && (
            <Box w="100%" p={4} bg="blue.900" borderRadius="lg" borderWidth="2px" borderColor="blue.500">
              <VStack gap={2} align="start">
                <Text color="blue.300" fontWeight="bold" fontSize="sm">
                  Update Required:
                </Text>
                <Text fontSize="xs" color="blue.200">
                  • Your device has bootloader v{currentBootloaderVersion}
                </Text>
                <Text fontSize="xs" color="blue.200">
                  • We'll update it to v2.1.5 for improved security
                </Text>
                <Text fontSize="xs" color="blue.200">
                  • This is a required step in the setup process
                </Text>
              </VStack>
            </Box>
          )}
        </VStack>

        {/* Right side - Details and actions */}
        <VStack gap={4} flex={{ base: "none", lg: 1 }} w={{ base: "100%", lg: "auto" }}>
          {deviceStatus.bootloaderCheck && (
            <Box w="100%" p={4} bg="gray.700" borderRadius="lg">
              <HStack gap={8} justify="space-between">
                <VStack gap={1} align="start">
                  <Text fontSize="xs" color="gray.400" textTransform="uppercase">
                    Current Version
                  </Text>
                  <Text fontSize="lg" color="white" fontWeight="bold">
                    v{deviceStatus.bootloaderCheck.currentVersion}
                  </Text>
                </VStack>
                <VStack gap={1} align="start">
                  <Text fontSize="xs" color="gray.400" textTransform="uppercase">
                    Latest Version
                  </Text>
                  <Text fontSize="lg" color="green.400" fontWeight="bold">
                    v{deviceStatus.bootloaderCheck.latestVersion}
                  </Text>
                </VStack>
              </HStack>
            </Box>
          )}


          {isUpdating && (
            <Box w="100%">
              <style>{stripeAnimationStyle}</style>
              <Text fontSize="sm" color="gray.400" mb={2}>
                Updating bootloader... Do not disconnect your device
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
            </Box>
          )}

          <VStack gap={3} w="100%">
            {(deviceStatus.needsBootloaderUpdate || isOldBootloader) && !isUpdating && (
              <Button
                colorScheme="blue"
                size="lg"
                w="100%"
                onClick={handleBootloaderUpdate}
                isLoading={isUpdating}
              >
                Update Bootloader
              </Button>
            )}
            {!deviceStatus.needsBootloaderUpdate && !isOldBootloader && (
              <Text fontSize="sm" color="green.400" textAlign="center">
                ✅ Bootloader verified - v{currentBootloaderVersion}
              </Text>
            )}
          </VStack>
        </VStack>
      </HStack>
    </Box>
  );
}