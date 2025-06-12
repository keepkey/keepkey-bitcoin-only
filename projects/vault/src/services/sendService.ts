import { 
  UTXO, 
  FeeEstimate, 
  BuildTxRequest, 
  BuildTxResponse, 
  BroadcastResponse,
  UnsignedTx 
} from '../types/send';
import { validate, getAddressInfo, Network } from 'bitcoin-address-validation';

const API_BASE = 'http://localhost:1646/api/v2';

export const sendService = {
  /**
   * Get UTXOs for a specific account and script type
   */
  async getUtxos(deviceId: string, account: number, scriptType: string): Promise<UTXO[]> {
    try {
      const response = await fetch(`${API_BASE}/utxos`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          device_id: deviceId,
          account,
          script_type: scriptType
        })
      });

      if (!response.ok) {
        throw new Error(`Failed to fetch UTXOs: ${response.statusText}`);
      }

      const utxos = await response.json();
      return utxos.map((utxo: any) => ({
        txid: utxo.txid,
        vout: utxo.vout,
        address: utxo.address,
        amount_sat: utxo.amount_sat,
        confirmations: utxo.confirmations,
        is_locked: utxo.is_locked || false,
        label: utxo.label
      }));
    } catch (error) {
      console.error('Error fetching UTXOs:', error);
      throw error;
    }
  },

  /**
   * Toggle lock state of a UTXO
   */
  async lockUtxo(txid: string, vout: number, locked: boolean): Promise<boolean> {
    try {
      const response = await fetch(`${API_BASE}/lock_utxo`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          txid,
          vout,
          locked
        })
      });

      return response.ok;
    } catch (error) {
      console.error('Error locking/unlocking UTXO:', error);
      return false;
    }
  },

  /**
   * Get current fee estimates
   */
  async getFeeEstimates(): Promise<FeeEstimate> {
    try {
      const response = await fetch(`${API_BASE}/fees`);
      
      if (!response.ok) {
        // Fallback to default values if API fails
        return {
          slow: 1,
          medium: 5,
          fast: 10
        };
      }

      const fees = await response.json();
      return {
        slow: fees.slow || 1,
        medium: fees.medium || 5,
        fast: fees.fast || 10
      };
    } catch (error) {
      console.error('Error fetching fee estimates:', error);
      // Return reasonable defaults
      return {
        slow: 1,
        medium: 5,
        fast: 10
      };
    }
  },

  /**
   * Build an unsigned transaction
   */
  async buildTransaction(request: BuildTxRequest): Promise<BuildTxResponse> {
    try {
      const response = await fetch(`${API_BASE}/tx/build`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          recipients: request.recipients,
          fee_rate: request.feeRate,
          selected_inputs: request.selectedInputs,
          slider_percent: request.sliderPercent
        })
      });

      const result = await response.json();

      if (!response.ok) {
        return {
          success: false,
          error: result.error || 'Failed to build transaction'
        };
      }

      return {
        success: true,
        tx: {
          psbt: result.psbt,
          fee: result.fee,
          size: result.size,
          inputs: result.inputs || [],
          outputs: result.outputs || []
        }
      };
    } catch (error) {
      console.error('Error building transaction:', error);
      return {
        success: false,
        error: error instanceof Error ? error.message : 'Unknown error'
      };
    }
  },

  /**
   * Sign a transaction with the hardware device
   */
  async signTransaction(psbt: string, deviceId: string): Promise<BuildTxResponse> {
    try {
      const response = await fetch(`${API_BASE}/tx/sign`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          psbt,
          device_id: deviceId
        })
      });

      const result = await response.json();

      if (!response.ok) {
        return {
          success: false,
          error: result.error || 'Failed to sign transaction'
        };
      }

      return {
        success: true,
        tx: {
          psbt: result.signed_psbt,
          fee: result.fee,
          size: result.size,
          inputs: result.inputs || [],
          outputs: result.outputs || []
        }
      };
    } catch (error) {
      console.error('Error signing transaction:', error);
      return {
        success: false,
        error: error instanceof Error ? error.message : 'Unknown error'
      };
    }
  },

  /**
   * Broadcast a signed transaction
   */
  async broadcastTransaction(signedPsbt: string): Promise<BroadcastResponse> {
    try {
      const response = await fetch(`${API_BASE}/tx/broadcast`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          signed_psbt: signedPsbt
        })
      });

      const result = await response.json();

      if (!response.ok) {
        return {
          success: false,
          error: result.error || 'Failed to broadcast transaction'
        };
      }

      return {
        success: true,
        txid: result.txid
      };
    } catch (error) {
      console.error('Error broadcasting transaction:', error);
      return {
        success: false,
        error: error instanceof Error ? error.message : 'Unknown error'
      };
    }
  },

  /**
   * Calculate transaction fee preview
   */
  calculateFee(utxos: UTXO[], outputs: Array<{ amount: string }>, feeRate: number): number {
    // Rough estimation: (inputs * 148 + outputs * 34 + 10) * feeRate
    const inputSize = utxos.length * 148;
    const outputSize = outputs.length * 34;
    const overhead = 10;
    const estimatedSize = inputSize + outputSize + overhead;
    return Math.ceil(estimatedSize * feeRate);
  },

  /**
   * Convert BTC to satoshis
   */
  btcToSat(btc: string): number {
    return Math.round(parseFloat(btc) * 100000000);
  },

  /**
   * Convert satoshis to BTC
   */
  satToBtc(sat: number): string {
    return (sat / 100000000).toFixed(8);
  },

  /**
   * Validate Bitcoin address using bitcoin-address-validation library
   */
  validateBitcoinAddress(address: string, network: 'mainnet' | 'testnet' | 'regtest' = 'mainnet'): { valid: boolean; error?: string; addressInfo?: any } {
    try {
      const isValid = validate(address, network as Network);
      
      if (!isValid) {
        return {
          valid: false,
          error: 'Invalid Bitcoin address format'
        };
      }

      const addressInfo = getAddressInfo(address);
      
      // Check if address belongs to the expected network
      if (addressInfo.network !== network) {
        return {
          valid: false,
          error: `Address belongs to ${addressInfo.network} network, expected ${network}`
        };
      }

      return {
        valid: true,
        addressInfo
      };
    } catch (error) {
      console.error('Error validating address:', error);
      return {
        valid: false,
        error: error instanceof Error ? error.message : 'Address validation failed'
      };
    }
  },

  /**
   * Format UTXO identifier
   */
  formatUtxoId(txid: string, vout: number): string {
    return `${txid}:${vout}`;
  },

  /**
   * Parse UTXO identifier
   */
  parseUtxoId(utxoId: string): { txid: string; vout: number } {
    const [txid, voutStr] = utxoId.split(':');
    return { txid, vout: parseInt(voutStr) };
  }
}; 