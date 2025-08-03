import { VStack, HStack, Text, Button, Box, Icon, Progress, Badge, Alert } from "@chakra-ui/react";
import { FaDownload, FaExclamationTriangle } from "react-icons/fa";
import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";

interface StepFirmwareUpdateProps {
  deviceId: string;
  onNext: () => void;
  onBack: () => void;
}

type UpdateState = 'idle' | 'waiting_confirmation' | 'complete';

export function StepFirmwareUpdate({ deviceId, onNext, onBack }: StepFirmwareUpdateProps) {
  const [deviceStatus, setDeviceStatus] = useState<any>(null);
  const [isUpdating, setIsUpdating] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [updateState, setUpdateState] = useState<UpdateState>('idle');

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
    setUpdateState('waiting_confirmation');
    
    try {
      // Actually invoke the firmware update
      // This will handle all the device communication including:
      // - Loading firmware
      // - Getting device features
      // - Firmware erase
      // - Waiting for user confirmation
      // - Uploading firmware
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
                  <VStack gap={3} align="start">
                    <HStack gap={2}>
                      <Icon as={FaExclamationTriangle} color="orange.300" boxSize={5} />
                      <Text fontSize="md" color="orange.300" fontWeight="bold">
                        Firmware update in progress...
                      </Text>
                    </HStack>
                    <Text fontSize="sm" color="orange.200" fontWeight="semibold">
                      Please check your KeepKey device screen!
                    </Text>
                    <VStack gap={1} align="start" pl={4}>
                      <Text fontSize="xs" color="orange.200">
                        • Press the button to confirm the firmware update
                      </Text>
                      <Text fontSize="xs" color="orange.200">
                        • Do not disconnect your device
                      </Text>
                      <Text fontSize="xs" color="orange.200" fontStyle="italic">
                        • If you see "verify backup" screen, you can safely ignore it
                      </Text>
                    </VStack>
                    <Text fontSize="xs" color="gray.400" mt={2}>
                      This process may take a few minutes...
                    </Text>
                  </VStack>
                </Box>
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
            {deviceStatus.needsFirmwareUpdate && !isUpdating && (
              <Button
                colorScheme="orange"
                size="lg"
                w="100%"
                onClick={handleFirmwareUpdate}
                isLoading={isUpdating}
              >
                Update Firmware Now
              </Button>
            )}
          </VStack>
        </VStack>
      </HStack>
    </Box>
  );
}