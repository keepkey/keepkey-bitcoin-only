import { VStack, HStack, Text, Button, Box, Icon, Progress, Alert, Badge } from "@chakra-ui/react";
import { FaDownload } from "react-icons/fa";
import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

interface StepFirmwareUpdateProps {
  deviceId: string;
  onNext: () => void;
  onBack: () => void;
}

export function StepFirmwareUpdate({ deviceId, onNext, onBack }: StepFirmwareUpdateProps) {
  const [deviceStatus, setDeviceStatus] = useState<any>(null);
  const [isUpdating, setIsUpdating] = useState(false);
  const [updateProgress, setUpdateProgress] = useState(0);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    checkDeviceStatus();
  }, [deviceId]);

  const checkDeviceStatus = async () => {
    try {
      const status = await invoke<any>('get_device_status', { deviceId });
      setDeviceStatus(status);
      
      // If firmware doesn't need update, skip to next step
      if (!status.needsFirmwareUpdate) {
        console.log("Firmware is up to date, skipping to next step");
        onNext();
      }
    } catch (err) {
      console.error("Failed to get device status:", err);
      setError(`Failed to check device status: ${err}`);
    }
  };

  const handleFirmwareUpdate = async () => {
    setIsUpdating(true);
    setError(null);
    
    try {
      // Start firmware update
      await invoke('update_device_firmware', { 
        deviceId,
        targetVersion: deviceStatus.firmwareCheck?.latestVersion || ''
      });
      
      // In real implementation, listen to progress events
      // For now, skip to next step after the update
      onNext();
      
    } catch (err) {
      console.error("Failed to update firmware:", err);
      setError(`Failed to update firmware: ${err}`);
      setIsUpdating(false);
    }
  };

  const handleSkip = () => {
    console.log("Skipping firmware update");
    onNext();
  };

  if (!deviceStatus) {
    return (
      <VStack gap={6} w="100%" maxW="500px">
        <Text color="gray.400">Checking device status...</Text>
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
            ) : (
              <Text fontSize={{ base: "sm", md: "md" }} color="green.400" textAlign="center">
                Your firmware is up to date!
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
            <Alert status="error" borderRadius="md" w="100%">
              {error}
            </Alert>
          )}

          {isUpdating && (
            <Box w="100%">
              <Text fontSize="sm" color="gray.400" mb={2}>
                Updating firmware... Do not disconnect your device
              </Text>
              <Progress value={updateProgress} size="lg" colorScheme="orange" />
              <Text fontSize="xs" color="gray.500" mt={2}>
                This may take a few minutes. Your device will restart when complete.
              </Text>
            </Box>
          )}

          <VStack gap={3} w="100%">
            {deviceStatus.needsFirmwareUpdate && !isUpdating && (
              <>
                <Button
                  colorScheme="orange"
                  size="lg"
                  w="100%"
                  onClick={handleFirmwareUpdate}
                  isLoading={isUpdating}
                >
                  Update Firmware Now
                </Button>
                {!isOOBDevice && (
                  <Button
                    variant="ghost"
                    size="lg"
                    w="100%"
                    onClick={handleSkip}
                    color="gray.400"
                    _hover={{ color: "white", bg: "gray.700" }}
                  >
                    Remind Me Later
                  </Button>
                )}
              </>
            )}
          </VStack>
        </VStack>
      </HStack>
    </Box>
  );
}