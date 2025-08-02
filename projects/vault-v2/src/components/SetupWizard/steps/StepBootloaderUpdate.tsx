import { VStack, HStack, Text, Button, Box, Icon, Progress, Alert, Image } from "@chakra-ui/react";
import { FaShieldAlt, FaExclamationTriangle } from "react-icons/fa";
import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import holdAndConnectSvg from '../../../assets/svg/hold-and-connect.svg';

interface StepBootloaderUpdateProps {
  deviceId: string;
  onNext: () => void;
  onBack: () => void;
}

export function StepBootloaderUpdate({ deviceId, onNext, onBack }: StepBootloaderUpdateProps) {
  const [deviceStatus, setDeviceStatus] = useState<any>(null);
  const [isUpdating, setIsUpdating] = useState(false);
  const [updateProgress, setUpdateProgress] = useState(0);
  const [error, setError] = useState<string | null>(null);
  const [showBootloaderInstructions, setShowBootloaderInstructions] = useState(false);

  useEffect(() => {
    checkDeviceStatus();
    
    // If showing bootloader instructions, periodically check if device entered bootloader mode
    let intervalId: NodeJS.Timeout | null = null;
    if (showBootloaderInstructions) {
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
      setDeviceStatus(status);
      
      // Check if device is in bootloader mode
      const isInBootloaderMode = status.features?.bootloader_mode || status.features?.bootloaderMode || false;
      console.log('Bootloader mode check:', {
        bootloader_mode: status.features?.bootloader_mode,
        bootloaderMode: status.features?.bootloaderMode,
        isInBootloaderMode,
        needsBootloaderUpdate: status.needsBootloaderUpdate
      });
      
      // If bootloader doesn't need update, skip to next step
      if (!status.needsBootloaderUpdate) {
        console.log("Bootloader is up to date, skipping to next step");
        onNext();
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
      setError(`Failed to check device status: ${err}`);
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
    setError(null);
    
    try {
      // Start bootloader update
      const success = await invoke('update_device_bootloader', {
        deviceId,
        targetVersion: deviceStatus.bootloaderCheck?.latestVersion || ''
      });
      
      // In real implementation, listen to progress events
      // For now, skip to next step after the update
      onNext();
      
    } catch (err) {
      console.error("Failed to update bootloader:", err);
      const errorMsg = String(err);
      
      // Check if the error is because device is not in bootloader mode
      if (errorMsg.includes('bootloader mode') || errorMsg.includes('Bootloader mode')) {
        setShowBootloaderInstructions(true);
        setError(null);
      } else {
        setError(`Failed to update bootloader: ${errorMsg}`);
      }
      setIsUpdating(false);
    }
  };

  const handleSkip = () => {
    console.log("Skipping bootloader update");
    onNext();
  };

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

            <Text fontSize="xs" color="blue.300" textAlign="center">
              Once in bootloader mode, click "Check Again" to continue
            </Text>

            <VStack gap={3} w="100%">
              <Button
                colorScheme="blue"
                size="lg"
                w="100%"
                onClick={() => {
                  setShowBootloaderInstructions(false);
                  checkDeviceStatus();
                }}
              >
                Check Again
              </Button>
              <Button
                variant="ghost"
                size="lg"
                w="100%"
                onClick={handleSkip}
                color="gray.400"
                _hover={{ color: "white", bg: "gray.700" }}
              >
                Skip Bootloader Update
              </Button>
            </VStack>
          </VStack>
        </HStack>
      </Box>
    );
  }

  return (
    <Box w="100%">
      <HStack 
        gap={{ base: 4, md: 8 }} 
        align="flex-start"
        flexDirection={{ base: "column", lg: "row" }}
      >
        {/* Left side - Icon and status */}
        <VStack gap={4} flex={{ base: "none", lg: 1 }} w={{ base: "100%", lg: "auto" }}>
          <Icon as={FaShieldAlt} boxSize={{ base: 12, md: 16 }} color="blue.500" />
          
          <VStack gap={2}>
            <Text fontSize={{ base: "xl", md: "2xl" }} fontWeight="bold" color="white" textAlign="center">
              Bootloader Update
            </Text>
            {deviceStatus.needsBootloaderUpdate ? (
              <Text fontSize={{ base: "sm", md: "md" }} color="gray.400" textAlign="center">
                Your KeepKey bootloader needs to be updated for optimal security
              </Text>
            ) : (
              <Text fontSize={{ base: "sm", md: "md" }} color="green.400" textAlign="center">
                Your bootloader is up to date!
              </Text>
            )}
          </VStack>
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

          {error && (
            <Alert status="error" borderRadius="md" w="100%">
              {error}
            </Alert>
          )}

          {isUpdating && (
            <Box w="100%">
              <Text fontSize="sm" color="gray.400" mb={2}>
                Updating bootloader... Do not disconnect your device
              </Text>
              <Progress value={updateProgress} size="lg" colorScheme="blue" />
            </Box>
          )}

          <VStack gap={3} w="100%">
            {deviceStatus.needsBootloaderUpdate && !isUpdating && (
              <>
                <Button
                  colorScheme="blue"
                  size="lg"
                  w="100%"
                  onClick={handleBootloaderUpdate}
                  isLoading={isUpdating}
                >
                  Update Bootloader
                </Button>
                <Button
                  variant="ghost"
                  size="lg"
                  w="100%"
                  onClick={handleSkip}
                  color="gray.400"
                  _hover={{ color: "white", bg: "gray.700" }}
                >
                  Skip for Now
                </Button>
              </>
            )}
          </VStack>
        </VStack>
      </HStack>
    </Box>
  );
}