import { 
  UTXO, 
  FeeEstimate, 
  BuildTxRequest, 
  BuildTxResponse, 
  BroadcastResponse,
  UnsignedTx 
} from '../types/send';
import { validate, getAddressInfo, Network } from 'bitcoin-address-validation';

const VAULT_API_BASE = 'http://localhost:1646/api/v2';
const PIONEER_API_BASE = 'https://pioneers.dev';

export const sendService = {
  /**
   * Get Bitcoin XPUBs from vault for a specific script type
   */
  async getBitcoinXpubs(): Promise<{ legacy?: string; segwit?: string; native_segwit?: string }> {
    try {
      const response = await fetch(`${VAULT_API_BASE}/pubkeys`);
      
      if (!response.ok) {
        throw new Error(`Failed to fetch pubkeys: ${response.statusText}`);
      }

      const pubkeys = await response.json();
      
      // Find Bitcoin XPUBs
      const bitcoinXpubs = pubkeys.filter((p: any) => 
        p.scriptType && p.scriptType.includes('_xpub') &&
        p.address && (
          p.address.startsWith('xpub') || 
          p.address.startsWith('ypub') || 
          p.address.startsWith('zpub')
        ) &&
        p.context && p.context.includes('bip122:000000000019d6689c085ae165831e93')
      );

      const result: { legacy?: string; segwit?: string; native_segwit?: string } = {};
      
      for (const xpub of bitcoinXpubs) {
        if (xpub.address.startsWith('xpub')) {
          result.legacy = xpub.address;
        } else if (xpub.address.startsWith('ypub')) {
          result.segwit = xpub.address;
        } else if (xpub.address.startsWith('zpub')) {
          result.native_segwit = xpub.address;
        }
      }
      
      console.log('🔑 Available Bitcoin XPUBs:', {
        legacy: result.legacy ? 'FOUND' : 'MISSING',
        segwit: result.segwit ? 'FOUND' : 'MISSING', 
        native_segwit: result.native_segwit ? 'FOUND' : 'MISSING',
        total_pubkeys: pubkeys.length,
        bitcoin_xpubs: bitcoinXpubs.length
      });
      
      return result;
    } catch (error) {
      console.error('Error fetching Bitcoin XPUBs:', error);
      throw error;
    }
  },

  /**
   * Get the best available XPUB with fallback logic
   */
  async getBestAvailableXpub(): Promise<{ xpub: string; scriptType: string }> {
    const xpubs = await this.getBitcoinXpubs();
    
    // Priority order: Native SegWit > SegWit > Legacy
    if (xpubs.native_segwit) {
      return { xpub: xpubs.native_segwit, scriptType: 'p2wpkh' };
    }
    if (xpubs.segwit) {
      return { xpub: xpubs.segwit, scriptType: 'p2sh-p2wpkh' };
    }
    if (xpubs.legacy) {
      return { xpub: xpubs.legacy, scriptType: 'p2pkh' };
    }
    
    throw new Error('No Bitcoin XPUBs found. Device may not be frontloaded or connected properly.');
  },

  /**
   * Get UTXOs from pioneers.dev using XPUBs - now with smart fallback
   */
  async getUtxos(deviceId: string, account: number, preferredScriptType?: string): Promise<{ utxos: UTXO[]; actualScriptType: string }> {
    try {
      // Get XPUBs from vault first
      const xpubs = await this.getBitcoinXpubs();
      
      let xpub: string | undefined;
      let actualScriptType: string | undefined;
      
      // Try preferred script type first, then fall back to best available
      if (preferredScriptType) {
        switch (preferredScriptType) {
          case 'p2pkh':
            if (xpubs.legacy) {
              xpub = xpubs.legacy;
              actualScriptType = 'p2pkh';
            }
            break;
          case 'p2sh-p2wpkh':
            if (xpubs.segwit) {
              xpub = xpubs.segwit;
              actualScriptType = 'p2sh-p2wpkh';
            }
            break;
          case 'p2wpkh':
            if (xpubs.native_segwit) {
              xpub = xpubs.native_segwit;
              actualScriptType = 'p2wpkh';
            }
            break;
        }
      }
      
      // If preferred type failed or not specified, use best available
      if (!xpub || !actualScriptType) {
        const best = await this.getBestAvailableXpub();
        xpub = best.xpub;
        actualScriptType = best.scriptType;
        
        if (preferredScriptType && preferredScriptType !== actualScriptType) {
          console.warn(`⚠️ Preferred script type ${preferredScriptType} not available, using ${actualScriptType} instead`);
        }
      }

      console.log(`🔍 Using ${actualScriptType} XPUB: ${xpub.substring(0, 20)}...`);

      // Call pioneers.dev for UTXOs
      const response = await fetch(`${PIONEER_API_BASE}/api/v1/listUnspent/BTC/${xpub}`, {
        headers: { 'accept': 'application/json' }
      });

      if (!response.ok) {
        throw new Error(`Failed to fetch UTXOs from pioneers.dev: ${response.statusText}`);
      }

      const utxos = await response.json();
      
      // Convert to our UTXO format
      const formattedUtxos = utxos.map((utxo: any) => ({
        txid: utxo.txid,
        vout: utxo.vout,
        address: utxo.address || '',
        amount_sat: utxo.value,
        confirmations: utxo.confirmations || 0,
        is_locked: false, // UTXOs from pioneers.dev aren't locked by default
        label: utxo.label || undefined
      }));
      
      return { utxos: formattedUtxos, actualScriptType };
    } catch (error) {
      console.error('Error fetching UTXOs:', error);
      throw error;
    }
  },

  /**
   * Toggle lock state of a UTXO (stored locally in vault)
   */
  async lockUtxo(txid: string, vout: number, locked: boolean): Promise<boolean> {
    try {
      const response = await fetch(`${VAULT_API_BASE}/lock_utxo`, {
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
   * Get current fee estimates from pioneers.dev
   */
  async getFeeEstimates(): Promise<FeeEstimate> {
    try {
      const response = await fetch(
        `${PIONEER_API_BASE}/api/v1/GetFeeRate/bip122%3A000000000019d6689c085ae165831e93`,
        { headers: { 'accept': 'application/json' } }
      );
      
      if (!response.ok) {
        console.warn('Failed to fetch fees from pioneers.dev, using defaults');
        return {
          slow: 1,
          medium: 5,
          fast: 10
        };
      }

      const fees = await response.json();
      return {
        slow: fees.average || 1,    // Map Pioneer's 'average' to our 'slow'
        medium: fees.fast || 5,     // Map Pioneer's 'fast' to our 'medium'  
        fast: fees.fastest || 10    // Map Pioneer's 'fastest' to our 'fast'
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
      const response = await fetch(`${VAULT_API_BASE}/tx/build`, {
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
      const response = await fetch(`${VAULT_API_BASE}/tx/sign`, {
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
      const response = await fetch(`${VAULT_API_BASE}/tx/broadcast`, {
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