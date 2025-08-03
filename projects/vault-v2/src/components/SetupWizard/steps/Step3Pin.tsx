import { Box } from "@chakra-ui/react";
import { DevicePinHorizontal } from "../../WalletCreationWizard/DevicePinHorizontal";

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
    console.log("Step3Pin: handlePinComplete called!");
    console.log("Step3Pin: PIN completed, session:", pinSession);
    console.log("Step3Pin: Session details:", JSON.stringify(pinSession, null, 2));
    
    // Update wizard data
    updateWizardData({ pinSession });
    console.log("Step3Pin: Updated wizard data with pinSession");
    
    // Call onNext immediately
    console.log("Step3Pin: About to call onNext()...");
    onNext();
    console.log("Step3Pin: onNext() has been called");
  };

  return (
    <Box w="100%">
      <DevicePinHorizontal
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