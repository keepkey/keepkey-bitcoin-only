import React, { createContext, useContext, useState, useEffect, ReactNode } from 'react';
import { useOnboardingState } from '../hooks/useOnboardingState';
import { invoke } from '@tauri-apps/api/core';

interface QueuedDeviceEvent {
  id: string;
  type: string;
  payload: any;
  timestamp: number;
  deviceId?: string;
}

interface OnboardingGateContextType {
  // Core state
  isOnboardingComplete: boolean;
  allowDeviceInteractions: boolean;
  
  // Gate control
  setOnboardingComplete: (complete: boolean) => void;
  forceAllowDeviceInteractions: () => void;
  
  // Event queuing
  queueDeviceEvent: (event: QueuedDeviceEvent) => void;
  getQueuedEvents: () => QueuedDeviceEvent[];
  clearQueuedEvents: () => void;
  replayQueuedEvents: () => QueuedDeviceEvent[];
  
  // Status
  hasQueuedEvents: boolean;
  onboardingInProgress: boolean;
}

const OnboardingGateContext = createContext<OnboardingGateContextType | undefined>(undefined);

interface OnboardingGateProviderProps {
  children: ReactNode;
}

// Maximum age for queued events (5 minutes)
const MAX_EVENT_AGE_MS = 5 * 60 * 1000;

export function OnboardingGateProvider({ children }: OnboardingGateProviderProps) {
  const [isOnboardingComplete, setIsOnboardingCompleteState] = useState(false);
  const [forceAllowed, setForceAllowed] = useState(false);
  const [queuedEvents, setQueuedEvents] = useState<QueuedDeviceEvent[]>([]);
  
  // Get onboarding state from the existing hook
  const { shouldShowOnboarding, loading: onboardingLoading } = useOnboardingState();
  
  // Determine if device interactions should be allowed
  const allowDeviceInteractions = 
    forceAllowed || // Emergency override
    (!onboardingLoading && !shouldShowOnboarding) || // Already onboarded (existing users)
    isOnboardingComplete; // Just completed onboarding (new users)
  
  const onboardingInProgress = !onboardingLoading && shouldShowOnboarding && !isOnboardingComplete;
  
  // Update onboarding complete state when external state changes
  useEffect(() => {
    if (!onboardingLoading && !shouldShowOnboarding) {
      // User is already onboarded, mark as complete
      setIsOnboardingCompleteState(true);
    }
  }, [shouldShowOnboarding, onboardingLoading]);

  // Start device operations for users who are already onboarded
  useEffect(() => {
    if (!onboardingLoading && !shouldShowOnboarding && allowDeviceInteractions) {
      // This user is already onboarded, start device operations immediately
      const startDeviceOpsForExistingUser = async () => {
        try {
          console.log('🚪 OnboardingGate: User already onboarded, starting device operations');
          await invoke('start_device_operations');
          console.log('🚪 OnboardingGate: Device operations started for existing user');
        } catch (error) {
          console.log('🚪 OnboardingGate: Failed to start device operations for existing user:', error);
        }
      };
      
      // Only do this once when the component mounts and conditions are met
      const timeoutId = setTimeout(startDeviceOpsForExistingUser, 100);
      return () => clearTimeout(timeoutId);
    }
  }, [onboardingLoading, shouldShowOnboarding, allowDeviceInteractions]);
  
  // Clean up old queued events periodically
  useEffect(() => {
    const cleanup = () => {
      const now = Date.now();
      setQueuedEvents(prev => 
        prev.filter(event => now - event.timestamp < MAX_EVENT_AGE_MS)
      );
    };
    
    const interval = setInterval(cleanup, 30000); // Cleanup every 30 seconds
    return () => clearInterval(interval);
  }, []);
  
  // Log state changes for debugging
  useEffect(() => {
    console.log('🚪 OnboardingGate state changed:', {
      isOnboardingComplete,
      allowDeviceInteractions,
      shouldShowOnboarding,
      onboardingLoading,
      forceAllowed,
      queuedEventsCount: queuedEvents.length,
      onboardingInProgress
    });
  }, [
    isOnboardingComplete, 
    allowDeviceInteractions, 
    shouldShowOnboarding, 
    onboardingLoading, 
    forceAllowed, 
    queuedEvents.length,
    onboardingInProgress
  ]);
  
  const setOnboardingComplete = (complete: boolean) => {
    console.log('🚪 OnboardingGate: Setting onboarding complete:', complete);
    setIsOnboardingCompleteState(complete);
    
    if (complete) {
      console.log('🚪 OnboardingGate: Onboarding completed, device interactions now allowed');
    }
  };
  
  const forceAllowDeviceInteractions = () => {
    console.log('🚪 OnboardingGate: Force allowing device interactions (emergency override)');
    setForceAllowed(true);
  };
  
  const queueDeviceEvent = (event: QueuedDeviceEvent) => {
    console.log('🚪 OnboardingGate: Queueing device event:', event.type, event.id);
    setQueuedEvents(prev => [...prev, { ...event, timestamp: Date.now() }]);
  };
  
  const getQueuedEvents = () => {
    return queuedEvents;
  };
  
  const clearQueuedEvents = () => {
    console.log('🚪 OnboardingGate: Clearing', queuedEvents.length, 'queued events');
    setQueuedEvents([]);
  };
  
  const replayQueuedEvents = () => {
    console.log('🚪 OnboardingGate: Replaying', queuedEvents.length, 'queued events');
    const eventsToReplay = [...queuedEvents];
    clearQueuedEvents();
    return eventsToReplay;
  };
  
  const hasQueuedEvents = queuedEvents.length > 0;
  
  const value: OnboardingGateContextType = {
    isOnboardingComplete,
    allowDeviceInteractions,
    setOnboardingComplete,
    forceAllowDeviceInteractions,
    queueDeviceEvent,
    getQueuedEvents,
    clearQueuedEvents,
    replayQueuedEvents,
    hasQueuedEvents,
    onboardingInProgress,
  };
  
  return (
    <OnboardingGateContext.Provider value={value}>
      {children}
    </OnboardingGateContext.Provider>
  );
}

export function useOnboardingGate(): OnboardingGateContextType {
  const context = useContext(OnboardingGateContext);
  if (context === undefined) {
    throw new Error('useOnboardingGate must be used within an OnboardingGateProvider');
  }
  return context;
}