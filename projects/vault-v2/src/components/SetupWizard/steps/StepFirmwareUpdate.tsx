import { VStack, Text, Button, Box, Icon, Progress, Alert, Badge } from "@chakra-ui/react";
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
      await invoke('update_firmware', { deviceId });
      
      // Simulate progress (in real implementation, listen to progress events)
      const progressInterval = setInterval(() => {
        setUpdateProgress(prev => {
          if (prev >= 90) {
            clearInterval(progressInterval);
            return 100;
          }
          return prev + 5;
        });
      }, 500);

      // Wait for update to complete
      setTimeout(() => {
        clearInterval(progressInterval);
        setUpdateProgress(100);
        onNext();
      }, 10000); // Firmware updates take longer
      
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
    <VStack gap={6} w="100%" maxW="500px">
      <Icon as={FaDownload} boxSize={16} color="orange.500" />
      
      <VStack gap={2}>
        <Text fontSize="2xl" fontWeight="bold" color="white">
          Firmware Update
        </Text>
        {deviceStatus.needsFirmwareUpdate ? (
          <>
            <Text fontSize="md" color="gray.400" textAlign="center">
              A new firmware version is available for your KeepKey
            </Text>
            {isOOBDevice && (
              <Badge colorScheme="red" fontSize="sm">
                Critical Update Required
              </Badge>
            )}
          </>
        ) : (
          <Text fontSize="md" color="green.400" textAlign="center">
            Your firmware is up to date!
          </Text>
        )}
      </VStack>

      {deviceStatus.firmwareCheck && (
        <Box w="100%" p={4} bg="gray.700" borderRadius="lg">
          <VStack gap={2} align="start">
            <Text fontSize="sm" color="gray.300">
              Current Version: v{deviceStatus.firmwareCheck.currentVersion}
            </Text>
            <Text fontSize="sm" color="gray.300">
              Latest Version: v{deviceStatus.firmwareCheck.latestVersion}
            </Text>
            {isOOBDevice && (
              <Text fontSize="sm" color="orange.400">
                ⚠️ Your device has factory firmware. Update is highly recommended.
              </Text>
            )}
          </VStack>
        </Box>
      )}

      {error && (
        <Alert status="error" borderRadius="md">
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

      <Box w="100%" p={4} bg="gray.700" borderRadius="lg" borderWidth="2px" borderColor="orange.500">
        <VStack gap={3}>
          <Text color="orange.400" fontWeight="bold" fontSize="sm">
            ⚠️ Important Instructions:
          </Text>
          <Text fontSize="sm" color="gray.300">
            • Do not disconnect your device during the update
          </Text>
          <Text fontSize="sm" color="gray.300">
            • You may need to re-enter your PIN after the update
          </Text>
          <Text fontSize="sm" color="gray.300">
            • Your funds and settings will remain safe
          </Text>
        </VStack>
      </Box>

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
  );
}