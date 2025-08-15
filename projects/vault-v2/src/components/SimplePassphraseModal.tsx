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
  const [confirmPassphrase, setConfirmPassphrase] = useState('');
  const [showPassphrase, setShowPassphrase] = useState(false);
  const [understood, setUnderstood] = useState(false);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  if (!isOpen) return null;

  const handleSubmit = async () => {
    // Basic validation
    if (!passphrase) {
      setError('Please enter a passphrase');
      return;
    }

    if (passphrase !== confirmPassphrase) {
      setError('Passphrases do not match');
      return;
    }

    if (!understood) {
      setError('Please confirm that you understand the warning');
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
      setConfirmPassphrase('');
      setUnderstood(false);
      
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
    setConfirmPassphrase('');
    setUnderstood(false);
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
        backgroundColor: 'rgba(0, 0, 0, 0.8)',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        zIndex: 9999,
      }}
    >
      <div
        style={{
          backgroundColor: 'white',
          borderRadius: '8px',
          padding: '24px',
          maxWidth: '500px',
          width: '90%',
          maxHeight: '90vh',
          overflow: 'auto',
        }}
      >
        <h2 style={{ margin: '0 0 16px 0', fontSize: '20px', fontWeight: 'bold' }}>
          Enter BIP39 Passphrase
        </h2>

        {/* Warning */}
        <div
          style={{
            padding: '12px',
            backgroundColor: '#fef3c7',
            border: '1px solid #f59e0b',
            borderRadius: '6px',
            marginBottom: '16px',
          }}
        >
          <p style={{ margin: '0 0 8px 0', fontWeight: 'bold', color: '#92400e' }}>
            Warning:
          </p>
          <p style={{ margin: 0, fontSize: '14px', color: '#92400e' }}>
            This passphrase cannot be recovered if lost! Each unique passphrase creates a
            completely different wallet. If you forget your passphrase, you will lose access
            to any funds in that wallet.
          </p>
        </div>

        {/* Error */}
        {error && (
          <div
            style={{
              padding: '12px',
              backgroundColor: '#fef2f2',
              border: '1px solid #ef4444',
              borderRadius: '6px',
              marginBottom: '16px',
            }}
          >
            <p style={{ margin: 0, fontSize: '14px', color: '#dc2626' }}>{error}</p>
          </div>
        )}

        {/* Passphrase Input */}
        <div style={{ marginBottom: '16px' }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '8px' }}>
            <label style={{ fontSize: '14px', fontWeight: '500' }}>Passphrase *</label>
            <button
              type="button"
              onClick={() => setShowPassphrase(!showPassphrase)}
              disabled={isSubmitting}
              style={{
                background: 'none',
                border: 'none',
                color: '#3b82f6',
                cursor: 'pointer',
                fontSize: '12px',
              }}
            >
              {showPassphrase ? 'Hide' : 'Show'}
            </button>
          </div>
          <input
            type={showPassphrase ? 'text' : 'password'}
            value={passphrase}
            onChange={(e) => setPassphrase(e.target.value)}
            placeholder="Enter your passphrase"
            autoComplete="off"
            disabled={isSubmitting}
            style={{
              width: '100%',
              padding: '8px 12px',
              border: '1px solid #d1d5db',
              borderRadius: '6px',
              fontSize: '14px',
            }}
          />
        </div>

        {/* Confirm Passphrase Input */}
        <div style={{ marginBottom: '16px' }}>
          <label style={{ display: 'block', fontSize: '14px', fontWeight: '500', marginBottom: '8px' }}>
            Confirm Passphrase *
          </label>
          <input
            type={showPassphrase ? 'text' : 'password'}
            value={confirmPassphrase}
            onChange={(e) => setConfirmPassphrase(e.target.value)}
            placeholder="Confirm your passphrase"
            autoComplete="off"
            disabled={isSubmitting}
            style={{
              width: '100%',
              padding: '8px 12px',
              border: '1px solid #d1d5db',
              borderRadius: '6px',
              fontSize: '14px',
            }}
          />
        </div>

        {/* Checkbox */}
        <div style={{ marginBottom: '16px' }}>
          <label style={{ display: 'flex', alignItems: 'flex-start', gap: '8px', fontSize: '14px' }}>
            <input
              type="checkbox"
              checked={understood}
              onChange={(e) => setUnderstood(e.target.checked)}
              disabled={isSubmitting}
              style={{ marginTop: '2px' }}
            />
            <span>
              I understand that this passphrase creates a completely different wallet and cannot
              be recovered if lost
            </span>
          </label>
        </div>

        {/* Tips */}
        <div
          style={{
            padding: '12px',
            backgroundColor: '#eff6ff',
            border: '1px solid #3b82f6',
            borderRadius: '6px',
            marginBottom: '24px',
          }}
        >
          <p style={{ margin: '0 0 8px 0', fontWeight: 'bold', fontSize: '14px', color: '#1e40af' }}>
            Tips for creating a strong passphrase:
          </p>
          <ul style={{ margin: 0, paddingLeft: '16px', fontSize: '12px', color: '#1e40af' }}>
            <li>Use a memorable phrase or sentence</li>
            <li>Consider using spaces between words</li>
            <li>Store a backup in a secure location</li>
            <li>Test with small amounts first</li>
          </ul>
        </div>

        {/* Buttons */}
        <div style={{ display: 'flex', gap: '12px', justifyContent: 'flex-end' }}>
          <button
            type="button"
            onClick={handleCancel}
            disabled={isSubmitting}
            style={{
              padding: '8px 16px',
              border: '1px solid #d1d5db',
              borderRadius: '6px',
              backgroundColor: 'white',
              color: '#374151',
              cursor: 'pointer',
              fontSize: '14px',
            }}
          >
            Cancel
          </button>
          <button
            type="button"
            onClick={handleSubmit}
            disabled={!passphrase || !confirmPassphrase || !understood || isSubmitting}
            style={{
              padding: '8px 16px',
              border: 'none',
              borderRadius: '6px',
              backgroundColor: !passphrase || !confirmPassphrase || !understood || isSubmitting ? '#9ca3af' : '#3b82f6',
              color: 'white',
              cursor: !passphrase || !confirmPassphrase || !understood || isSubmitting ? 'not-allowed' : 'pointer',
              fontSize: '14px',
            }}
          >
            {isSubmitting ? 'Submitting...' : 'Continue'}
          </button>
        </div>
      </div>
    </div>
  );
};

export default SimplePassphraseModal;