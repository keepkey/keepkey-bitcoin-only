import {
  Box,
  Button,
  HStack,
  VStack,
  Text,
  Flex,
  Icon,
} from "@chakra-ui/react";
import { useState, useContext } from "react";
import { FaCheckCircle } from "react-icons/fa";
import { invoke } from "@tauri-apps/api/core";
import { useDialog } from "../../contexts/DialogContext";
import { useTranslation } from "react-i18next";
// Safe import with conditional usage
import { OnboardingGateContext } from "../../contexts/OnboardingGateContext";
import { useOnboardingState } from "../../hooks/useOnboardingState";

// Import individual steps
import { Step0Language } from "./steps/Step0Language";
import { Step2Pin } from "./steps/Step2Pin";
import { Step3Mnemonics } from "./steps/Step3Mnemonics";
import { Step4Complete } from "./steps/Step4Complete";

interface OnboardingWizardProps {
  onClose?: () => void;
  onComplete?: () => void;
}

interface Step {
  id: string;
  label: string;
  description: string;
  component: React.ComponentType;
}

const STEPS: Step[] = [
  {
    id: "language",
    label: "Language",
    description: "Select your preferred language",
    component: Step0Language,
  },
  {
    id: "pin",
    label: "Security",
    description: "Set up your PIN and security settings",
    component: Step2Pin,
  },
  {
    id: "mnemonics",
    label: "Recovery",
    description: "Understand recovery phrases",
    component: Step3Mnemonics,
  },
  {
    id: "complete",
    label: "Complete",
    description: "You're all set!",
    component: Step4Complete,
  },
];

export function OnboardingWizard({ onClose, onComplete }: OnboardingWizardProps) {
  const [currentStep, setCurrentStep] = useState(0);
  const highlightColor = "green.500";
  const { hide } = useDialog();
  const { t } = useTranslation(['onboarding', 'common']);
  
  // Safely access OnboardingGateContext - it might not be available if rendered outside provider
  const onboardingGateContext = useContext(OnboardingGateContext);
  const setOnboardingComplete = onboardingGateContext?.setOnboardingComplete || (() => {
    console.log('OnboardingGateContext not available - using fallback');
  });
  
  const { clearCache } = useOnboardingState();

  // Override STEPS with translated values
  const translatedSteps = STEPS.map(step => ({
    ...step,
    label: t(`onboarding:steps.${step.id}`, step.label),
    description: t(`onboarding:${step.id}.description`, step.description)
  }));

  const handleNext = () => {
    if (currentStep < STEPS.length - 1) {
      setCurrentStep(currentStep + 1);
    } else {
      handleComplete();
    }
  };

  const handlePrevious = () => {
    if (currentStep > 0) {
      setCurrentStep(currentStep - 1);
    }
  };

  const handleComplete = async () => {
    console.log("=== Starting onboarding completion ===");
    try {
      // Mark onboarding as completed in the database
      console.log("Calling set_onboarding_completed...");
      await invoke("set_onboarding_completed");
      console.log("set_onboarding_completed completed successfully");

      // Clear the onboarding state cache to ensure shouldShowOnboarding updates immediately
      console.log("🚪 OnboardingWizard: Clearing onboarding state cache");
      clearCache();

      // Enable device interactions through the onboarding gate
      console.log("🚪 OnboardingWizard: Enabling device interactions");
      setOnboardingComplete(true);

      // Start device operations on the backend
      try {
        console.log("🚪 OnboardingWizard: Starting device operations on backend");
        await invoke("start_device_operations");
        console.log("🚪 OnboardingWizard: Device operations started successfully");
      } catch (error) {
        console.error("🚪 OnboardingWizard: Failed to start device operations:", error);
        // Don't fail the onboarding completion for this error
      }

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
        hide('onboarding');
        console.log('Forced onboarding dialog closure via DialogContext');
      }, 100);

      // Auto-restart backend after 10 seconds to ensure smooth transition
      console.log("Scheduling automatic backend restart in 10 seconds...");
      setTimeout(async () => {
        try {
          console.log("Auto-restarting backend after onboarding completion...");
          await invoke('restart_backend_startup');
          console.log("Backend restart initiated successfully after onboarding");
          
          // Signal backend that frontend is ready
          setTimeout(async () => {
            try {
              console.log('🎯 Signaling backend that frontend is ready after onboarding restart...');
              await invoke('frontend_ready');
              console.log('✅ Frontend ready signal sent successfully');
            } catch (error) {
              console.log('frontend_ready command failed:', error);
            }
          }, 1000);
        } catch (error) {
          console.error("Failed to auto-restart backend after onboarding:", error);
        }
      }, 10000);
    } catch (error) {
      console.error("Failed to mark onboarding as completed:", error);
      // Try to get debug info on failure
      try {
        const debugInfo = await invoke<string>('debug_onboarding_state');
        console.log("Debug info after error:", debugInfo);
      } catch (debugError) {
        console.warn("Could not get debug info:", debugError);
      }
    }
  };

  const StepComponent = STEPS[currentStep].component;
  const progress = ((currentStep + 1) / STEPS.length) * 100;

  return (
    <Box
      w="100%"
      maxW="1200px"
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
              {t('onboarding:welcome.title')}
            </Text>
            <Text fontSize="md" color="gray.400">
              {translatedSteps[currentStep].description}
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
              bg="green.500" 
              borderRadius="full"
              transition="width 0.3s"
              w={`${progress}%`}
            />
          </Box>
        </Box>

        {/* Step indicators */}
        <HStack gap={4} justify="center" p={4}>
          {translatedSteps.map((step, index) => (
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
              >
                {step.label}
              </Text>
              {index < STEPS.length - 1 && (
                <Box
                  w={8}
                  h={0.5}
                  bg={index < currentStep ? highlightColor : "gray.600"}
                  ml={2}
                />
              )}
            </Flex>
          ))}
        </HStack>

        {/* Content */}
        <Box
          p={4}
          maxH="60vh"
          overflowY="auto"
          bg="gray.800"
          display="flex"
          alignItems="center"
          justifyContent="center"
        >
          <StepComponent />
        </Box>

        {/* Footer */}
        <Box p={6} borderTopWidth="1px" borderColor="gray.700" bg="gray.850">
          <HStack justify="space-between">
            <Text fontSize="sm" color="gray.400">
              {t('common:labels.step')} {currentStep + 1} {t('common:labels.of')} {STEPS.length}
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
                {t('common:buttons.back')}
              </Button>
              <Button
                colorScheme="green"
                onClick={handleNext}
                size="lg"
              >
                {currentStep === STEPS.length - 1 ? t('onboarding:complete.startUsing') : t('common:buttons.next')}
              </Button>
            </HStack>
          </HStack>
        </Box>
      </Box>
  );
} 