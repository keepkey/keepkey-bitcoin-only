import { Box } from "@chakra-ui/react";
import { DevicePin } from "../../WalletCreationWizard/DevicePin";

interface Step3PinProps {
  deviceId: string;
  wizardData: any;
  updateWizardData: (data: any) => void;
  onNext: () => void;
  onBack: () => void;
}

export function Step3Pin({ 
  deviceId, 
  wizardData, 
  updateWizardData, 
  onNext,
  onBack 
}: Step3PinProps) {
  
  const handlePinComplete = (pinSession: any) => {
    updateWizardData({ pinSession });
    onNext();
  };

  return (
    <Box w="100%">
      <DevicePin
        deviceId={deviceId}
        deviceLabel={wizardData.deviceLabel || 'KeepKey'}
        mode="create"
        onComplete={handlePinComplete}
        onBack={onBack}
        isLoading={false}
        error={null}
      />
    </Box>
  );
}