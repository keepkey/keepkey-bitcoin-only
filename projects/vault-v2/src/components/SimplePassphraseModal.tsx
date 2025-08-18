import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useTypedTranslation } from '../hooks/useTypedTranslation';

interface SimplePassphraseModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSubmit?: () => void;
  deviceId?: string;
}

export const SimplePassphraseModal: React.FC<SimplePassphraseModalProps> = ({
  isOpen,
  onClose,
  onSubmit,
  deviceId,
}) => {
  const { t } = useTypedTranslation('dialogs');
  const [passphrase, setPassphrase] = useState('');
  const [showPassphrase, setShowPassphrase] = useState(false);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [hasSubmittedForSession, setHasSubmittedForSession] = useState(false);
  const [awaitingDeviceConfirmation, setAwaitingDeviceConfirmation] = useState(false);

  // Reset session state when modal opens (new passphrase request)
  useEffect(() => {
    if (isOpen) {
      console.log('🔐 [SimplePassphraseModal] Modal opened, resetting session state');
      setHasSubmittedForSession(false);
      setAwaitingDeviceConfirmation(false);
      setError(null);
      // Don't clear passphrase here - user might want to retry with same passphrase
    }
  }, [isOpen]);

  console.log('🔐 [SimplePassphraseModal] Component rendered with props:', {
    isOpen,
    deviceId,
    hasOnClose: !!onClose,
    hasOnSubmit: !!onSubmit,
    hasSubmittedForSession,
    awaitingDeviceConfirmation
  });

  if (!isOpen) {
    console.log('🔐 [SimplePassphraseModal] Not rendering because isOpen is false');
    return null;
  }

  console.log('🔐 [SimplePassphraseModal] ✅ Rendering passphrase modal');

  const handleSubmit = async () => {
    // Prevent duplicate submissions for the same session
    if (hasSubmittedForSession) {
      console.log('🔐 [SimplePassphraseModal] Preventing duplicate passphrase submission');
      setError(t('passphrase.alreadySubmitted', 'Passphrase already submitted. Please confirm on your device.'));
      return;
    }

    // Basic validation
    if (!passphrase) {
      setError(t('passphrase.pleaseEnter', 'Please enter a passphrase'));
      return;
    }

    if (!deviceId) {
      setError(t('passphrase.noDeviceId', 'No device ID available'));
      return;
    }

    setIsSubmitting(true);
    setError(null);

    try {
      console.log('🔐 [SimplePassphraseModal] Sending passphrase for device:', deviceId);
      await invoke('send_passphrase', {
        passphrase,
        deviceId,
      });

      // Mark as submitted for this session
      setHasSubmittedForSession(true);
      setAwaitingDeviceConfirmation(true);
      
      // Clear sensitive data
      setPassphrase('');
      
      // Show confirmation message instead of closing immediately
      setError(null);
      console.log('🔐 [SimplePassphraseModal] Passphrase sent successfully, awaiting device confirmation');
      
      // Don't close the modal immediately - wait for device confirmation
      // The modal will be closed by the parent component when operation completes
      
      // Call onSubmit callback if provided (for PassphraseSettings flow)
      if (onSubmit) {
        onSubmit();
      }
    } catch (err) {
      console.error('🔐 [SimplePassphraseModal] Failed to send passphrase:', err);
      const errorMessage = err instanceof Error ? err.message : 'Failed to send passphrase';
      
      // Only allow retry if it's not an "Unexpected message" error
      if (errorMessage.includes('Unexpected message')) {
        setError(t('passphrase.deviceNotReady', 'Device not ready for passphrase. Please try the operation again.'));
        setHasSubmittedForSession(true); // Prevent further attempts
        
        // Close the modal after a short delay to let user see the error message
        setTimeout(() => {
          onClose();
        }, 2000);
      } else if (errorMessage.includes('timed out')) {
        setError(t('passphrase.timedOut', 'Operation timed out. Please confirm on your device or try again.'));
        // Allow retry for timeout
        setHasSubmittedForSession(false);
        setAwaitingDeviceConfirmation(false);
      } else {
        setError(errorMessage);
        // Allow retry for other errors
        setHasSubmittedForSession(false);
        setAwaitingDeviceConfirmation(false);
      }
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleCancel = () => {
    setPassphrase('');
    setError(null);
    onClose();
  };

  return (
    <div
      style={{
        position: 'fixed',
        top: 0,
        left: 0,
        right: 0,
        bottom: 0,
        backgroundColor: 'rgba(0, 0, 0, 0.5)',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        zIndex: 9999,
      }}
    >
      <div
        style={{
          backgroundColor: '#2D3748',
          borderRadius: '16px',
          padding: '32px',
          maxWidth: '500px',
          width: '90%',
          maxHeight: '90vh',
          overflow: 'auto',
          border: '1px solid #4A5568',
        }}
      >
        <h2 style={{ margin: '0 0 16px 0', fontSize: '20px', fontWeight: 'bold', color: 'white' }}>
          {awaitingDeviceConfirmation ? t('passphrase.confirmOnDevice', 'Confirm on Device') : t('passphrase.title')}
        </h2>

        {/* Instruction */}
        <div
          style={{
            padding: '12px',
            backgroundColor: '#1A202C',
            border: '1px solid #4299E1',
            borderRadius: '8px',
            marginBottom: '16px',
            textAlign: 'center',
          }}
        >
          <p style={{ margin: 0, fontSize: '14px', color: '#90CDF4' }}>
            {awaitingDeviceConfirmation 
              ? t('passphrase.confirmOnDevice')
              : t('passphrase.afterSubmitting')}
          </p>
        </div>

        {/* Error */}
        {error && (
          <div
            style={{
              padding: '12px',
              backgroundColor: '#2D1B1B',
              border: '1px solid #E53E3E',
              borderRadius: '8px',
              marginBottom: '16px',
            }}
          >
            <p style={{ margin: 0, fontSize: '14px', color: '#FEB2B2' }}>{error}</p>
          </div>
        )}

        {/* Passphrase Input */}
        <div style={{ marginBottom: '16px' }}>
          <input
            type={showPassphrase ? 'text' : 'password'}
            value={passphrase}
            onChange={(e) => setPassphrase(e.target.value)}
            placeholder={t('passphrase.placeholder')}
            autoComplete="off"
            disabled={isSubmitting || awaitingDeviceConfirmation}
            style={{
              width: '100%',
              padding: '12px 16px',
              border: '2px solid #4A5568',
              borderRadius: '8px',
              fontSize: '16px',
              backgroundColor: '#1A202C',
              color: 'white',
              outline: 'none',
            }}
            onFocus={(e) => {
              e.target.style.borderColor = '#4299E1';
            }}
            onBlur={(e) => {
              e.target.style.borderColor = '#4A5568';
            }}
          />
        </div>

        {/* Warning */}
        <div
          style={{
            padding: '12px',
            backgroundColor: '#2D1B10',
            border: '1px solid #D69E2E',
            borderRadius: '8px',
            marginBottom: '24px',
          }}
        >
          <p style={{ margin: '0 0 8px 0', fontWeight: 'bold', color: '#F6E05E' }}>
            {t('passphrase.useWithCaution')}
          </p>
          <p style={{ margin: 0, fontSize: '14px', color: '#FBD38D' }}>
            {t('passphrase.warningLoseAccess')} {t('passphrase.learnMore')}
          </p>
        </div>

        {/* Buttons */}
        <div style={{ display: 'flex', gap: '12px', justifyContent: 'center', marginTop: '24px' }}>
          <button
            type="button"
            onClick={handleCancel}
            disabled={isSubmitting}
            style={{
              padding: '12px 24px',
              border: '1px solid #4A5568',
              borderRadius: '8px',
              backgroundColor: '#2D3748',
              color: '#E2E8F0',
              cursor: 'pointer',
              fontSize: '14px',
              fontWeight: '500',
            }}
          >
            {t('passphrase.buttons.cancel')}
          </button>
          <button
            type="button"
            onClick={handleSubmit}
            disabled={!passphrase || isSubmitting || hasSubmittedForSession}
            style={{
              padding: '12px 24px',
              border: 'none',
              borderRadius: '8px',
              backgroundColor: !passphrase || isSubmitting || hasSubmittedForSession ? '#4A5568' : '#D69E2E',
              color: 'white',
              cursor: !passphrase || isSubmitting || hasSubmittedForSession ? 'not-allowed' : 'pointer',
              fontSize: '14px',
              fontWeight: '500',
            }}
          >
            {isSubmitting ? t('passphrase.submitting') : 
             awaitingDeviceConfirmation ? t('passphrase.awaitingConfirmation') : 
             hasSubmittedForSession ? t('passphrase.alreadySubmittedButton') : 
             t('passphrase.buttons.submitPassphrase')}
          </button>
        </div>
      </div>
    </div>
  );
};

export default SimplePassphraseModal;