import React, { useEffect, useCallback, useRef, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { useDialog } from '../contexts/DialogContext';

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
  
  const { openDialog, closeDialog } = useDialog();

  useEffect(() => {
    const unlisteners: UnlistenFn[] = [];

    // Listen for PIN requests
    const setupPinListener = async () => {
      const unlisten = await listen<DeviceEvent>('device:awaiting_pin', (event) => {
        const { device_id, request_id, kind } = event.payload;
        
        if (!request_id) return;
        
        // Store the active request
        activeRequests.current.set(request_id, {
          deviceId: device_id,
          requestId: request_id,
          type: 'pin',
          operationType: kind,
        });
        
        // Open PIN dialog with correlation
        const dialogId = `device-pin-${device_id}-${request_id}`;
        openDialog({
          id: dialogId,
          component: React.lazy(() => import('../components/DevicePinDialog').then(m => ({ default: m.DevicePinDialog }))),
          props: {
            isOpen: true,
            deviceId: device_id,
            requestId: request_id,
            operationType: kind,
            onSubmit: async (pin: string) => {
              try {
                await invoke('pin_submit', { 
                  deviceId: device_id, 
                  requestId: request_id, 
                  pin 
                });
                activeRequests.current.delete(request_id);
                closeDialog(dialogId);
              } catch (error) {
                console.error('PIN submission failed:', error);
                // Keep dialog open on error
                throw error;
              }
            },
            onCancel: async () => {
              try {
                await invoke('pin_cancel', { 
                  deviceId: device_id, 
                  requestId: request_id 
                });
                activeRequests.current.delete(request_id);
                closeDialog(dialogId);
              } catch (error) {
                console.error('PIN cancellation failed:', error);
              }
            },
            onClose: () => {
              activeRequests.current.delete(request_id);
              closeDialog(dialogId);
            }
          },
          priority: 'critical',
          persistent: true,
        });
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

    // Listen for passphrase requests
    const setupPassphraseListener = async () => {
      const unlisten = await listen<DeviceEvent>('device:awaiting_passphrase', (event) => {
        const { device_id, request_id, cache_allowed } = event.payload;
        
        if (!request_id) return;
        
        // Store the active request
        activeRequests.current.set(request_id, {
          deviceId: device_id,
          requestId: request_id,
          type: 'passphrase',
        });
        
        // Open passphrase dialog with correlation
        openDialog('passphraseDialog', {
          deviceId: device_id,
          requestId: request_id,
          cacheAllowed: cache_allowed,
          onSubmit: async (passphrase: string) => {
            try {
              await invoke('passphrase_submit', { 
                deviceId: device_id, 
                requestId: request_id, 
                passphrase 
              });
              activeRequests.current.delete(request_id);
              closeDialog('passphraseDialog');
            } catch (error) {
              console.error('Passphrase submission failed:', error);
            }
          },
          onCancel: async () => {
            try {
              await invoke('passphrase_cancel', { 
                deviceId: device_id, 
                requestId: request_id 
              });
              activeRequests.current.delete(request_id);
              closeDialog('passphraseDialog');
            } catch (error) {
              console.error('Passphrase cancellation failed:', error);
            }
          }
        });
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
              closeDialog(`device-pin-${device_id}-${requestId}`);
            } else if (request.type === 'passphrase') {
              closeDialog('passphraseDialog');
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
            closeDialog(`device-pin-${device_id}-${request_id}`);
          } else if (request?.type === 'passphrase') {
            closeDialog('passphraseDialog');
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
  }, [openDialog, closeDialog, reconnectDialogState.deviceId, reconnectDialogState.isOpen]);

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