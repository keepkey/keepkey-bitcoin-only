import React, { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface SimplePassphraseModalProps {
  isOpen: boolean;
  onClose: () => void;
  deviceId?: string;
}

export const SimplePassphraseModal: React.FC<SimplePassphraseModalProps> = ({
  isOpen,
  onClose,
  deviceId,
}) => {
  const [passphrase, setPassphrase] = useState('');
  const [showPassphrase, setShowPassphrase] = useState(false);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  if (!isOpen) return null;

  const handleSubmit = async () => {
    // Basic validation
    if (!passphrase) {
      setError('Please enter a passphrase');
      return;
    }

    if (!deviceId) {
      setError('No device ID available');
      return;
    }

    setIsSubmitting(true);
    setError(null);

    try {
      await invoke('send_passphrase', {
        passphrase,
        deviceId,
      });

      // Clear sensitive data
      setPassphrase('');
      
      onClose();
    } catch (err) {
      console.error('Failed to send passphrase:', err);
      setError(err instanceof Error ? err.message : 'Failed to send passphrase');
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
          Enter Your Passphrase
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
            After submitting your passphrase, use the button on the KeepKey to approve it.
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
            placeholder="Enter your passphrase"
            autoComplete="off"
            disabled={isSubmitting}
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
            Use with caution
          </p>
          <p style={{ margin: 0, fontSize: '14px', color: '#FBD38D' }}>
            If you forget your passphrase, you will lose access to your funds. Learn more
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
            Cancel
          </button>
          <button
            type="button"
            onClick={handleSubmit}
            disabled={!passphrase || isSubmitting}
            style={{
              padding: '12px 24px',
              border: 'none',
              borderRadius: '8px',
              backgroundColor: !passphrase || isSubmitting ? '#4A5568' : '#D69E2E',
              color: 'white',
              cursor: !passphrase || isSubmitting ? 'not-allowed' : 'pointer',
              fontSize: '14px',
              fontWeight: '500',
            }}
          >
            {isSubmitting ? 'Submitting...' : 'Submit Passphrase'}
          </button>
        </div>
      </div>
    </div>
  );
};

export default SimplePassphraseModal;