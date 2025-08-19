import { useDialog } from '../contexts/DialogContext';
import { useCallback } from 'react';
import React from 'react';
import { SetupWizard } from '../components/SetupWizard';
import { NoDeviceDialog } from '../components/NoDeviceDialog';

// Import dialog components dynamically
const dialogComponents = {
  onboarding: () => import('../components/OnboardingWizard/OnboardingWizard').then(m => ({ default: m.OnboardingWizard })),
  settings: () => import('../components/SettingsDialog').then(m => ({ default: m.SettingsDialog })),
  walletCreation: () => import('../components/WalletCreationWizard/WalletCreationWizard').then(m => ({ default: m.WalletCreationWizard })),
  noDevice: () => import('../components/NoDeviceDialog').then(m => ({ default: m.NoDeviceDialog })),
  // Add more dialog imports as needed
};

export function useCommonDialogs() {
  const { show, hide, hideAll, requestAppFocus, releaseAppFocus } = useDialog();
  
  const showOnboarding = useCallback((props?: any) => {
    show({
      id: 'onboarding',
      component: React.lazy(dialogComponents.onboarding), // Use lazy loading to avoid circular dependencies
      props,
      priority: 'high',
      persistent: true,
      onOpen: () => {
        requestAppFocus();
      },
      onClose: () => {
        releaseAppFocus();
      },
    });
  }, [show, requestAppFocus, releaseAppFocus]);
  
  const showSettings = useCallback((props?: any) => {
    show({
      id: 'settings',
      component: React.lazy(dialogComponents.settings),
      props,
      priority: 'normal',
    });
  }, [show]);

  const showWalletCreation = useCallback((props?: {
    deviceId?: string;
    onWizardComplete?: () => void;
    onWizardClose?: () => void;
  }) => {
    console.log(`🏦 [useCommonDialogs] showWalletCreation called with props:`, props);
    console.trace('🏦 [useCommonDialogs] Call stack trace:');
    
    show({
      id: 'setup-wizard',
      component: SetupWizard, // Using new SetupWizard component
      props: {
        ...props,
        deviceId: props?.deviceId || 'mock-device-id',
        onComplete: () => {
          console.log(`🏦 [useCommonDialogs] SetupWizard onComplete called`);
          if (props?.onWizardComplete) props.onWizardComplete();
          hide('setup-wizard');
        },
        onClose: () => {
          console.log(`🏦 [useCommonDialogs] SetupWizard onClose called`);
          if (props?.onWizardClose) props.onWizardClose();
          hide('setup-wizard');
        }
      },
      priority: 'critical', // High priority since it's device setup
      persistent: true, // User must complete flow
      onOpen: () => {
        requestAppFocus();
      },
      onClose: () => {
        releaseAppFocus();
      },
    });
  }, [show, hide, requestAppFocus, releaseAppFocus]);

  const showNoDevice = useCallback((props?: {
    onRetry?: () => void;
  }) => {
    show({
      id: 'no-device-found',
      component: NoDeviceDialog,
      props,
      priority: 'high',
      persistent: false,
    });
  }, [show]);
  
  // TODO: Implement these dialogs
  const showError = useCallback((title: string, message: string) => {
    console.error('Error dialog not implemented yet:', title, message);
  }, []);
  
  const showConfirm = useCallback((title: string, message: string, onConfirm: () => void) => {
    console.warn('Confirm dialog not implemented yet:', title, message);
    // For now, just call the confirm callback
    onConfirm();
  }, []);
  
  return {
    showOnboarding,
    showSettings,
    showWalletCreation,
    showNoDevice,
    showError,
    showConfirm,
    hideDialog: hide,
    hideAllDialogs: hideAll,
  };
} 