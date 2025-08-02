import {
  Box,
  Button,
  HStack,
  VStack,
  Text,
  Flex,
  Icon,
} from "@chakra-ui/react";
import { useState } from "react";
import { FaCheckCircle } from "react-icons/fa";
import { invoke } from "@tauri-apps/api/core";
import { useDialog } from "../../contexts/DialogContext";

// Import individual steps
import { Step0Welcome } from "./steps/Step0Welcome";
import { StepBootloaderUpdate } from "./steps/StepBootloaderUpdate";
import { StepFirmwareUpdate } from "./steps/StepFirmwareUpdate";
import { Step1CreateOrRecover } from "./steps/Step1CreateOrRecover";
import { Step2DeviceLabel } from "./steps/Step2DeviceLabel";
import { Step3Pin } from "./steps/Step3Pin";
import { Step4BackupOrRecover } from "./steps/Step4BackupOrRecover";
import { Step5Complete } from "./steps/Step5Complete";

interface SetupWizardProps {
  deviceId: string;
  onClose?: () => void;
  onComplete?: () => void;
}

interface Step {
  id: string;
  label: string;
  description: string;
  component: React.ComponentType<any>;
}

// Define steps based on flow type
const CREATE_STEPS: Step[] = [
  {
    id: "welcome",
    label: "Welcome",
    description: "Welcome to KeepKey Bitcoin-Only",
    component: Step0Welcome,
  },
  {
    id: "bootloader",
    label: "Bootloader",
    description: "Verify and update bootloader if needed",
    component: StepBootloaderUpdate,
  },
  {
    id: "firmware",
    label: "Firmware",
    description: "Verify and update firmware if needed",
    component: StepFirmwareUpdate,
  },
  {
    id: "create-or-recover",
    label: "Setup Type",
    description: "Choose your setup method",
    component: Step1CreateOrRecover,
  },
  {
    id: "device-label",
    label: "Device Name",
    description: "Name your device",
    component: Step2DeviceLabel,
  },
  {
    id: "pin",
    label: "Security",
    description: "Set up your PIN",
    component: Step3Pin,
  },
  {
    id: "backup",
    label: "Backup",
    description: "Backup your recovery phrase",
    component: Step4BackupOrRecover,
  },
  {
    id: "complete",
    label: "Complete",
    description: "Setup complete!",
    component: Step5Complete,
  },
];

const RECOVER_STEPS: Step[] = [
  {
    id: "welcome",
    label: "Welcome",
    description: "Welcome to KeepKey Bitcoin-Only",
    component: Step0Welcome,
  },
  {
    id: "bootloader",
    label: "Bootloader",
    description: "Verify and update bootloader if needed",
    component: StepBootloaderUpdate,
  },
  {
    id: "firmware",
    label: "Firmware",
    description: "Verify and update firmware if needed",
    component: StepFirmwareUpdate,
  },
  {
    id: "create-or-recover",
    label: "Setup Type",
    description: "Choose your setup method",
    component: Step1CreateOrRecover,
  },
  {
    id: "recover",
    label: "Recovery",
    description: "Enter your recovery phrase",
    component: Step4BackupOrRecover,
  },
  {
    id: "device-label",
    label: "Device Name",
    description: "Name your device",
    component: Step2DeviceLabel,
  },
  {
    id: "pin",
    label: "Security",
    description: "Set up your PIN",
    component: Step3Pin,
  },
  {
    id: "complete",
    label: "Complete",
    description: "Recovery complete!",
    component: Step5Complete,
  },
];

export function SetupWizard({ deviceId, onClose, onComplete }: SetupWizardProps) {
  const [currentStep, setCurrentStep] = useState(0);
  const [flowType, setFlowType] = useState<'create' | 'recover' | null>(null);
  const [wizardData, setWizardData] = useState<{
    deviceLabel?: string;
    pinSession?: any;
    recoverySettings?: any;
  }>({});
  
  const highlightColor = "orange.500"; // Bitcoin orange
  const { hide } = useDialog();

  // Determine which steps to use based on flow type
  const STEPS = flowType === 'recover' ? RECOVER_STEPS : CREATE_STEPS;

  const handleNext = () => {
    if (currentStep < STEPS.length - 1) {
      setCurrentStep(currentStep + 1);
    } else {
      handleComplete();
    }
  };

  const handlePrevious = () => {
    if (currentStep > 0) {
      // If going back from a step after flow type is chosen, reset flow type
      const createOrRecoverIndex = STEPS.findIndex(step => step.id === 'create-or-recover');
      if (currentStep > createOrRecoverIndex && STEPS[currentStep].id !== 'create-or-recover') {
        // Going back to or before the create-or-recover step
        if (currentStep - 1 <= createOrRecoverIndex) {
          setFlowType(null);
        }
      }
      setCurrentStep(currentStep - 1);
    }
  };

  const handleComplete = async () => {
    console.log("=== Starting setup completion ===");
    try {
      // Mark setup as completed
      console.log("Calling set_onboarding_completed...");
      await invoke("set_onboarding_completed");
      console.log("set_onboarding_completed completed successfully");

      // Call the completion callback if provided
      if (onComplete) {
        console.log("Calling onComplete callback");
        onComplete();
      }

      // Use multiple methods to ensure the dialog closes
      if (onClose) {
        console.log("Calling onClose callback");
        onClose();
      }

      // Use the dialog context directly to force close after a short delay
      setTimeout(() => {
        hide('setup-wizard');
        console.log('Forced setup wizard closure via DialogContext');
      }, 100);
    } catch (error) {
      console.error("Failed to mark setup as completed:", error);
    }
  };

  const handleFlowTypeSelection = (type: 'create' | 'recover') => {
    setFlowType(type);
    handleNext();
  };

  const updateWizardData = (data: Partial<typeof wizardData>) => {
    setWizardData(prev => ({ ...prev, ...data }));
  };

  const StepComponent = STEPS[currentStep].component;
  const progress = ((currentStep + 1) / STEPS.length) * 100;

  // Props to pass to step components
  const stepProps = {
    deviceId,
    wizardData,
    updateWizardData,
    onNext: handleNext,
    onBack: handlePrevious,
    onFlowTypeSelect: handleFlowTypeSelection,
    flowType,
  };

  return (
    <Box
      w="100%"
      maxW="800px"
      bg="gray.800"
      borderRadius="xl"
      boxShadow="xl"
      borderWidth="1px"
      borderColor="gray.700"
      overflow="hidden"
    >
      {/* Header */}
      <Box 
        p={6} 
        borderBottomWidth="1px" 
        borderColor="gray.700"
        bg="gray.850"
      >
        <VStack gap={4}>
          <Text fontSize="2xl" fontWeight="bold" color={highlightColor}>
            KeepKey Bitcoin Setup
          </Text>
          <Text fontSize="md" color="gray.400">
            {STEPS[currentStep].description}
          </Text>
        </VStack>
      </Box>

      {/* Progress */}
      <Box px={6} py={2}>
        <Box 
          h="4px" 
          bg="gray.700" 
          borderRadius="full"
          overflow="hidden"
        >
          <Box 
            h="100%" 
            bg={highlightColor}
            borderRadius="full"
            transition="width 0.3s"
            w={`${progress}%`}
          />
        </Box>
      </Box>

      {/* Step indicators */}
      <HStack gap={4} justify="center" p={4}>
        {STEPS.map((step, index) => (
          <Flex key={step.id} align="center">
            <Box
              w={8}
              h={8}
              borderRadius="full"
              bg={index <= currentStep ? highlightColor : "gray.600"}
              display="flex"
              alignItems="center"
              justifyContent="center"
              transition="all 0.3s"
            >
              {index < currentStep ? (
                <Icon as={FaCheckCircle} color="white" boxSize={4} />
              ) : (
                <Text color="white" fontSize="sm" fontWeight="bold">
                  {index + 1}
                </Text>
              )}
            </Box>
            <Text
              ml={2}
              fontSize="sm"
              fontWeight={index === currentStep ? "bold" : "normal"}
              color={index <= currentStep ? highlightColor : "gray.400"}
              display={{ base: "none", md: "block" }}
            >
              {step.label}
            </Text>
            {index < STEPS.length - 1 && (
              <Box
                w={8}
                h={0.5}
                bg={index < currentStep ? highlightColor : "gray.600"}
                ml={2}
                display={{ base: "none", md: "block" }}
              />
            )}
          </Flex>
        ))}
      </HStack>

      {/* Content */}
      <Box
        p={8}
        minH="400px"
        bg="gray.800"
        display="flex"
        alignItems="center"
        justifyContent="center"
      >
        <StepComponent {...stepProps} />
      </Box>

      {/* Footer */}
      <Box p={6} borderTopWidth="1px" borderColor="gray.700" bg="gray.850">
        <HStack justify="space-between">
          <Text fontSize="sm" color="gray.400">
            Step {currentStep + 1} of {STEPS.length}
          </Text>
          <HStack gap={4}>
            <Button
              variant="outline"
              onClick={handlePrevious}
              disabled={currentStep === 0}
              borderColor="gray.600"
              color="gray.300"
              _hover={{ bg: "gray.700" }}
            >
              Previous
            </Button>
            {/* Only show Next button if not on flow selection step or if flow is selected */}
            {(STEPS[currentStep].id !== 'create-or-recover' || flowType) && (
              <Button
                colorScheme="orange"
                onClick={handleNext}
                size="lg"
              >
                {currentStep === STEPS.length - 1 ? "Complete Setup" : "Next"}
              </Button>
            )}
          </HStack>
        </HStack>
      </Box>
    </Box>
  );
}