import React, { useEffect, useCallback, useRef, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { useDialog } from '../contexts/DialogContext';
import { useOnboardingGate } from '../contexts/OnboardingGateContext';

interface DeviceEvent {
  type: string;
  device_id: string;
  request_id?: string;
  [key: string]: any;
}

interface PendingRequest {
  deviceId: string;
  requestId: string;
  type: 'pin' | 'button' | 'passphrase';
  operationType?: string;
}

export function useDeviceInteraction() {
  const activeRequests = useRef<Map<string, PendingRequest>>(new Map());
  const [reconnectDialogState, setReconnectDialogState] = useState<{
    isOpen: boolean;
    deviceId: string | null;
    reason: string | null;
  }>({
    isOpen: false,
    deviceId: null,
    reason: null,
  });
  
  const { show, hide } = useDialog();
  const { allowDeviceInteractions, queueDeviceEvent } = useOnboardingGate();

  useEffect(() => {
    const unlisteners: UnlistenFn[] = [];

    // DISABLED: PIN request handling to prevent duplicate PIN dialogs
    // The issue is that multiple event systems are trying to show PIN dialogs:
    // 1. This hook listening to 'device:awaiting_pin'
    // 2. Other parts of the system also showing PIN dialogs with the same ID
    // This causes duplicate dialogs to appear
    const setupPinListener = async () => {
      const unlisten = await listen<DeviceEvent>('device:awaiting_pin', (event) => {
        const { device_id, request_id, kind } = event.payload;
        
        console.log('📍 device:awaiting_pin event received (not showing dialog to prevent duplicates):', {
          device_id,
          request_id,
          kind
        });
        
        // Don't show a dialog here - let the existing PIN system handle it
        // The PIN dialog will be shown through other event handlers
      });
      unlisteners.push(unlisten);
    };

    // Listen for button requests
    const setupButtonListener = async () => {
      const unlisten = await listen<DeviceEvent>('device:awaiting_button', (event) => {
        const { device_id, request_id, label } = event.payload;
        
        if (!request_id) return;
        
        // Store the active request
        activeRequests.current.set(request_id, {
          deviceId: device_id,
          requestId: request_id,
          type: 'button',
        });
        
        // For button requests, we typically just show a notification
        // as the user needs to press the physical button on the device
        console.log(`Device ${device_id} awaiting button press: ${label || 'Confirm on device'}`);
        
        // Optionally show a toast or notification
        // You could integrate with a toast library here
      });
      unlisteners.push(unlisten);
    };

    // DISABLED: Passphrase handling is now done through unified auth dialog in DeviceUpdateManager
    // This prevents duplicate passphrase dialogs from appearing
    const setupPassphraseListener = async () => {
      const unlisten = await listen<DeviceEvent>('device:awaiting_passphrase', (event) => {
        const { device_id, request_id, cache_allowed } = event.payload;
        
        console.log('📍 device:awaiting_passphrase event received (handled by DeviceUpdateManager):', {
          device_id,
          request_id,
          cache_allowed
        });
        
        // Check onboarding gate and queue if needed
        if (!allowDeviceInteractions) {
          console.log('🚪 useDeviceInteraction: Queueing device:awaiting_passphrase - onboarding in progress');
          queueDeviceEvent({
            id: `awaiting-passphrase-${device_id}-${request_id}`,
            type: 'device:awaiting_passphrase',
            payload: event.payload,
            timestamp: Date.now(),
            deviceId: device_id
          });
          return;
        }
        
        // Don't show a dialog here - DeviceUpdateManager will handle it with the unified auth dialog
        // This prevents duplicate passphrase dialogs from appearing
      });
      unlisteners.push(unlisten);
    };

    // Listen for reconnect needs
    const setupReconnectListener = async () => {
      const unlisten = await listen<DeviceEvent>('device:needs_reconnect', (event) => {
        const { device_id, reason } = event.payload;
        
        // Show reconnect dialog
        setReconnectDialogState({
          isOpen: true,
          deviceId: device_id,
          reason: reason,
        });
      });
      unlisteners.push(unlisten);
    };

    // Listen for device reconnection
    const setupConnectedListener = async () => {
      const unlisten = await listen<DeviceEvent>('device:connected', (event) => {
        const { device_id } = event.payload;
        
        // Close reconnect dialog if open for this device
        if (reconnectDialogState.deviceId === device_id && reconnectDialogState.isOpen) {
          setReconnectDialogState({
            isOpen: false,
            deviceId: null,
            reason: null,
          });
        }
        
        // Clear any pending requests for this device
        for (const [requestId, request] of activeRequests.current.entries()) {
          if (request.deviceId === device_id) {
            activeRequests.current.delete(requestId);
          }
        }
      });
      unlisteners.push(unlisten);
    };

    // Listen for device disconnection
    const setupDisconnectedListener = async () => {
      const unlisten = await listen<DeviceEvent>('device:disconnected', (event) => {
        const { device_id } = event.payload;
        
        // Clear any pending requests for this device
        for (const [requestId, request] of activeRequests.current.entries()) {
          if (request.deviceId === device_id) {
            activeRequests.current.delete(requestId);
            
            // Close any open dialogs for this device
            if (request.type === 'pin') {
              hide(`device-pin-${device_id}-${requestId}`);
            } else if (request.type === 'passphrase') {
              hide(`passphrase-${device_id}-${requestId}`);
            }
          }
        }
      });
      unlisteners.push(unlisten);
    };

    // Listen for device errors
    const setupErrorListener = async () => {
      const unlisten = await listen<DeviceEvent>('device:error', (event) => {
        const { device_id, request_id, message } = event.payload;
        
        console.error(`Device error for ${device_id}: ${message}`);
        
        // Clear the request if it exists
        if (request_id && activeRequests.current.has(request_id)) {
          const request = activeRequests.current.get(request_id);
          activeRequests.current.delete(request_id);
          
          // Close relevant dialog
          if (request?.type === 'pin') {
            hide(`device-pin-${device_id}-${request_id}`);
          } else if (request?.type === 'passphrase') {
            hide(`passphrase-${device_id}-${request_id}`);
          }
        }
      });
      unlisteners.push(unlisten);
    };

    // Setup all listeners
    setupPinListener();
    setupButtonListener();
    setupPassphraseListener();
    setupReconnectListener();
    setupConnectedListener();
    setupDisconnectedListener();
    setupErrorListener();

    // Cleanup listeners on unmount
    return () => {
      unlisteners.forEach(unlisten => unlisten());
    };
  }, [show, hide, reconnectDialogState.deviceId, reconnectDialogState.isOpen, allowDeviceInteractions, queueDeviceEvent]);

  const closeReconnectDialog = useCallback(() => {
    setReconnectDialogState({
      isOpen: false,
      deviceId: null,
      reason: null,
    });
  }, []);

  const getActiveRequests = useCallback(() => {
    return Array.from(activeRequests.current.values());
  }, []);

  const hasActiveRequest = useCallback((deviceId: string) => {
    for (const request of activeRequests.current.values()) {
      if (request.deviceId === deviceId) {
        return true;
      }
    }
    return false;
  }, []);

  return {
    reconnectDialogState,
    closeReconnectDialog,
    getActiveRequests,
    hasActiveRequest,
  };
}