import { VStack, Text, Button, Box, Icon, Progress, Alert } from "@chakra-ui/react";
import { FaShieldAlt } from "react-icons/fa";
import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

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

  useEffect(() => {
    checkDeviceStatus();
  }, [deviceId]);

  const checkDeviceStatus = async () => {
    try {
      const status = await invoke<any>('get_device_status', { deviceId });
      setDeviceStatus(status);
      
      // If bootloader doesn't need update, skip to next step
      if (!status.needsBootloaderUpdate) {
        console.log("Bootloader is up to date, skipping to next step");
        onNext();
      }
    } catch (err) {
      console.error("Failed to get device status:", err);
      setError(`Failed to check device status: ${err}`);
    }
  };

  const handleBootloaderUpdate = async () => {
    setIsUpdating(true);
    setError(null);
    
    try {
      // Start bootloader update
      await invoke('update_bootloader', { deviceId });
      
      // Simulate progress (in real implementation, listen to progress events)
      const progressInterval = setInterval(() => {
        setUpdateProgress(prev => {
          if (prev >= 90) {
            clearInterval(progressInterval);
            return 100;
          }
          return prev + 10;
        });
      }, 500);

      // Wait for update to complete
      setTimeout(() => {
        clearInterval(progressInterval);
        setUpdateProgress(100);
        onNext();
      }, 5000);
      
    } catch (err) {
      console.error("Failed to update bootloader:", err);
      setError(`Failed to update bootloader: ${err}`);
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

  return (
    <VStack gap={6} w="100%" maxW="500px">
      <Icon as={FaShieldAlt} boxSize={16} color="blue.500" />
      
      <VStack gap={2}>
        <Text fontSize="2xl" fontWeight="bold" color="white">
          Bootloader Update
        </Text>
        {deviceStatus.needsBootloaderUpdate ? (
          <Text fontSize="md" color="gray.400" textAlign="center">
            Your KeepKey bootloader needs to be updated for optimal security
          </Text>
        ) : (
          <Text fontSize="md" color="green.400" textAlign="center">
            Your bootloader is up to date!
          </Text>
        )}
      </VStack>

      {deviceStatus.bootloaderCheck && (
        <Box w="100%" p={4} bg="gray.700" borderRadius="lg">
          <VStack gap={2} align="start">
            <Text fontSize="sm" color="gray.300">
              Current Version: v{deviceStatus.bootloaderCheck.currentVersion}
            </Text>
            <Text fontSize="sm" color="gray.300">
              Latest Version: v{deviceStatus.bootloaderCheck.latestVersion}
            </Text>
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
  );
}