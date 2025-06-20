import { useState } from 'react'
import { invoke } from '@tauri-apps/api/core'

interface PinUnlockDialogProps {
  isOpen: boolean
  deviceId: string
  onUnlocked: () => void
  onClose: () => void
}

interface PinMatrixResult {
  success: boolean
  next_step?: string
  session_id: string
  error?: string
}

interface PinCreationSession {
  device_id: string
  session_id: string
  current_step: string
  is_active: boolean
}

export const PinUnlockDialog = ({ isOpen, deviceId, onUnlocked, onClose }: PinUnlockDialogProps) => {
  const [pinMatrix, setPinMatrix] = useState<number[]>([])
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [unlockSession, setUnlockSession] = useState<PinCreationSession | null>(null)

  const handleMatrixClick = (position: number) => {
    if (pinMatrix.length < 9) { // Max PIN length
      setPinMatrix([...pinMatrix, position])
    }
  }

  const handleClear = () => {
    setPinMatrix([])
    setError(null)
  }

  const handleSubmit = async () => {
    if (pinMatrix.length === 0) {
      setError('Please enter your PIN')
      return
    }

    setIsLoading(true)
    setError(null)

    try {
      // First, start PIN unlock session if we don't have one
      if (!unlockSession) {
        console.log('🔒 Starting PIN unlock session for device:', deviceId)
        const session = await invoke<PinCreationSession>('start_pin_unlock', { 
          deviceId 
        })
        console.log('🔒 PIN unlock session created:', session)
        setUnlockSession(session)
        
        // Now send the PIN
        const result = await invoke<PinMatrixResult>('send_pin_unlock_response', {
          sessionId: session.session_id,
          positions: pinMatrix
        })
        
        console.log('🔒 PIN unlock result:', result)
        
        if (result.success) {
          console.log('🔒 PIN unlock successful!')
          onUnlocked()
        } else {
          setError(result.error || 'PIN unlock failed')
          handleClear()
        }
      } else {
        // We already have a session, just send the PIN
        const result = await invoke<PinMatrixResult>('send_pin_unlock_response', {
          sessionId: unlockSession.session_id,
          positions: pinMatrix
        })
        
        console.log('🔒 PIN unlock result:', result)
        
        if (result.success) {
          console.log('🔒 PIN unlock successful!')
          onUnlocked()
        } else {
          setError(result.error || 'PIN unlock failed')
          handleClear()
        }
      }
    } catch (err) {
      console.error('🔒 PIN unlock error:', err)
      setError(err instanceof Error ? err.message : 'Failed to unlock device')
      handleClear()
    } finally {
      setIsLoading(false)
    }
  }

  const handleCancel = async () => {
    // Cancel the unlock session if it exists
    if (unlockSession) {
      try {
        await invoke('cancel_pin_creation', { sessionId: unlockSession.session_id })
      } catch (err) {
        console.error('Failed to cancel PIN unlock session:', err)
      }
    }
    
    setPinMatrix([])
    setError(null)
    setUnlockSession(null)
    onClose()
  }

  if (!isOpen) return null

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-white rounded-lg shadow-xl p-6 w-full max-w-md mx-4">
        <div className="mb-6">
          <h2 className="text-xl font-semibold text-gray-900 mb-2">
            Enter PIN to Unlock
          </h2>
          <p className="text-sm text-gray-600">
            Your KeepKey device is locked. Enter your PIN using the matrix displayed on your device.
          </p>
        </div>

        {error && (
          <div className="mb-4 p-3 bg-red-50 border border-red-200 rounded text-red-700 text-sm">
            {error}
          </div>
        )}

        {/* PIN Display */}
        <div className="mb-6">
          <div className="text-sm text-gray-600 mb-2">PIN entered:</div>
          <div className="flex items-center min-h-[2rem]">
            {pinMatrix.map((_, index) => (
              <span key={index} className="w-3 h-3 bg-gray-800 rounded-full mr-2"></span>
            ))}
            {pinMatrix.length === 0 && (
              <span className="text-gray-400 text-sm">No digits entered</span>
            )}
          </div>
        </div>

        {/* PIN Matrix */}
        <div className="mb-6">
          <div className="text-sm text-gray-600 mb-3">
            Look at your KeepKey device and click the positions shown:
          </div>
          <div className="grid grid-cols-3 gap-2 max-w-48 mx-auto">
            {[1, 2, 3, 4, 5, 6, 7, 8, 9].map((position) => (
              <button
                key={position}
                onClick={() => handleMatrixClick(position)}
                disabled={isLoading}
                className="aspect-square bg-gray-100 hover:bg-gray-200 disabled:hover:bg-gray-100 disabled:opacity-50 rounded border-2 border-gray-300 text-lg font-semibold text-gray-700 transition-colors"
              >
                •
              </button>
            ))}
          </div>
        </div>

        {/* Action Buttons */}
        <div className="flex gap-3">
          <button
            onClick={handleClear}
            disabled={isLoading || pinMatrix.length === 0}
            className="flex-1 px-4 py-2 border border-gray-300 text-gray-700 rounded hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
          >
            Clear
          </button>
          <button
            onClick={handleCancel}
            disabled={isLoading}
            className="flex-1 px-4 py-2 border border-gray-300 text-gray-700 rounded hover:bg-gray-50 disabled:opacity-50"
          >
            Cancel
          </button>
          <button
            onClick={handleSubmit}
            disabled={isLoading || pinMatrix.length === 0}
            className="flex-1 px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {isLoading ? 'Unlocking...' : 'Unlock'}
          </button>
        </div>

        <div className="mt-4 text-xs text-gray-500 text-center">
          The numbers on the buttons correspond to the positions shown on your KeepKey device screen.
        </div>
      </div>
    </div>
  )
} 