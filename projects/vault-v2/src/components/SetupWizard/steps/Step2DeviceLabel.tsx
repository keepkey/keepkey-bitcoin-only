import { VStack, Text, Input, Button, HStack, Box } from "@chakra-ui/react";
import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";

interface Step2DeviceLabelProps {
  deviceId: string;
  wizardData: any;
  updateWizardData: (data: any) => void;
  onNext: () => void;
  onBack: () => void;
}

export function Step2DeviceLabel({ 
  deviceId, 
  wizardData, 
  updateWizardData, 
  onNext,
  onBack 
}: Step2DeviceLabelProps) {
  const [label, setLabel] = useState(wizardData.deviceLabel || "");
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleSubmit = async () => {
    setIsLoading(true);
    setError(null);
    
    try {
      if (label.trim()) {
        await invoke('set_device_label', { deviceId, label: label.trim() });
      }
      updateWizardData({ deviceLabel: label.trim() || 'KeepKey' });
      onNext();
    } catch (err) {
      console.error("Failed to set device label:", err);
      setError(`Failed to set device label: ${err}`);
    } finally {
      setIsLoading(false);
    }
  };

  const handleSkip = () => {
    updateWizardData({ deviceLabel: 'KeepKey' });
    onNext();
  };

  return (
    <VStack gap={6} w="100%" maxW="400px" mx="auto">
      <VStack gap={2}>
        <Text fontSize="2xl" fontWeight="bold" color="white" textAlign="center">
          Name Your Device
        </Text>
        <Text fontSize="md" color="gray.400" textAlign="center">
          Give your KeepKey a friendly name to identify it easily
        </Text>
      </VStack>

      <Box w="100%">
        <Input
          placeholder="My KeepKey"
          value={label}
          onChange={(e) => setLabel(e.target.value)}
          size="lg"
          bg="gray.700"
          borderColor="gray.600"
          _hover={{ borderColor: "gray.500" }}
          _focus={{ borderColor: "orange.500", boxShadow: "0 0 0 1px orange.500" }}
          color="white"
          isDisabled={isLoading}
          onKeyPress={(e) => {
            if (e.key === 'Enter' && label.trim()) {
              handleSubmit();
            }
          }}
        />
        {error && (
          <Text color="red.400" fontSize="sm" mt={2}>
            {error}
          </Text>
        )}
      </Box>

      <VStack gap={3} w="100%">
        <Button
          colorScheme="orange"
          size="lg"
          w="100%"
          onClick={handleSubmit}
          isLoading={isLoading}
          loadingText="Setting label..."
          isDisabled={!label.trim()}
        >
          Set Device Name
        </Button>
        
        <Button
          variant="ghost"
          size="lg"
          w="100%"
          onClick={handleSkip}
          isDisabled={isLoading}
          color="gray.400"
          _hover={{ color: "white", bg: "gray.700" }}
        >
          Skip for Now
        </Button>
      </VStack>
    </VStack>
  );
}