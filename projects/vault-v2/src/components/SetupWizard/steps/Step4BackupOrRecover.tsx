import { Box, VStack, Text, Button, Icon } from "@chakra-ui/react";
import { FaShieldAlt } from "react-icons/fa";
import { RecoveryFlow } from "../../WalletCreationWizard/RecoveryFlow";
import { RecoverySettings } from "../../WalletCreationWizard/RecoverySettings";
import { invoke } from "@tauri-apps/api/core";
import { useState } from "react";

interface Step4BackupOrRecoverProps {
  deviceId: string;
  wizardData: any;
  updateWizardData: (data: any) => void;
  onNext: () => void;
  onBack: () => void;
  flowType: 'create' | 'recover' | null;
}

export function Step4BackupOrRecover({ 
  deviceId, 
  wizardData, 
  updateWizardData, 
  onNext,
  onBack,
  flowType 
}: Step4BackupOrRecoverProps) {
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [showRecoverySettings, setShowRecoverySettings] = useState(true);

  // Create flow - show backup phrase
  if (flowType === 'create') {
    const handleBackupComplete = async () => {
      setIsLoading(true);
      try {
        await invoke('complete_wallet_creation', { deviceId });
        onNext();
      } catch (err) {
        console.error("Failed to complete wallet creation:", err);
        setError(`Failed to complete wallet creation: ${err}`);
      } finally {
        setIsLoading(false);
      }
    };

    return (
      <VStack gap={6} w="100%" maxW="500px">
        <Icon as={FaShieldAlt} boxSize={16} color="orange.500" />
        
        <VStack gap={2}>
          <Text fontSize="2xl" fontWeight="bold" color="white">
            Backup Your Recovery Phrase
          </Text>
          <Text fontSize="md" color="gray.400" textAlign="center">
            Your recovery phrase is now displayed on your KeepKey device
          </Text>
        </VStack>

        <Box 
          p={4} 
          bg="gray.700" 
          borderRadius="lg" 
          borderWidth="2px"
          borderColor="orange.500"
          w="100%"
        >
          <VStack gap={3}>
            <Text color="orange.400" fontWeight="bold">
              ⚠️ Important Instructions:
            </Text>
            <Text fontSize="sm" color="gray.300">
              1. Write down each word exactly as shown on your device
            </Text>
            <Text fontSize="sm" color="gray.300">
              2. Store your recovery phrase in a safe place
            </Text>
            <Text fontSize="sm" color="gray.300">
              3. Never share it with anyone or store it digitally
            </Text>
            <Text fontSize="sm" color="gray.300">
              4. This is your only way to recover your funds
            </Text>
          </VStack>
        </Box>

        {error && (
          <Text color="red.400" fontSize="sm">
            {error}
          </Text>
        )}

        <Button
          colorScheme="orange"
          size="lg"
          w="100%"
          onClick={handleBackupComplete}
          isLoading={isLoading}
          loadingText="Completing setup..."
        >
          I Have Written Down My Recovery Phrase
        </Button>
      </VStack>
    );
  }

  // Recover flow - show recovery options then recovery flow
  if (flowType === 'recover') {
    if (showRecoverySettings && !wizardData.recoverySettings) {
      const handleRecoverySettingsComplete = (settings: any) => {
        updateWizardData({ recoverySettings: settings });
        setShowRecoverySettings(false);
      };

      return (
        <RecoverySettings
          onComplete={handleRecoverySettingsComplete}
          onBack={onBack}
          isLoading={false}
          error={null}
        />
      );
    }

    const handleRecoveryComplete = async () => {
      setIsLoading(true);
      try {
        await invoke('complete_recovery', { deviceId });
        onNext();
      } catch (err) {
        console.error("Failed to complete recovery:", err);
        setError(`Failed to complete recovery: ${err}`);
      } finally {
        setIsLoading(false);
      }
    };

    const handleRecoveryError = (error: string) => {
      setError(error);
    };

    return (
      <RecoveryFlow
        deviceId={deviceId}
        wordCount={wizardData.recoverySettings?.wordCount || 12}
        passphraseProtection={wizardData.recoverySettings?.usePassphrase || false}
        deviceLabel={wizardData.deviceLabel || 'KeepKey Recovery'}
        onComplete={handleRecoveryComplete}
        onError={handleRecoveryError}
        onBack={() => {
          setShowRecoverySettings(true);
          updateWizardData({ recoverySettings: null });
        }}
      />
    );
  }

  return null;
}