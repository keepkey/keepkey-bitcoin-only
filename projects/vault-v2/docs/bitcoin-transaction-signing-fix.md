# Bitcoin Transaction Signing: Invalid Prevhash Fix Documentation

## Executive Summary

This document details the resolution of the "Encountered invalid prevhash 2" error that was preventing KeepKey hardware wallets from signing Bitcoin transactions. The issue involved multiple interconnected problems across the JavaScript frontend, Rust backend, and transaction data handling.

## The Problem

When attempting to sign Bitcoin transactions, the KeepKey device would consistently return:
```
Device communication error: Failure: Encountered invalid prevhash 2
```

This error occurs when the device cannot properly validate the previous transaction data for security verification.

## Root Causes Identified

### 1. SegWit Transaction Format Mishandling

**Issue**: The transaction hex being fetched from the blockchain was in SegWit format (containing witness data), but the parser was attempting to read it as a legacy transaction.

**Example of problematic hex**:
```
0100000000010192dc12975c6e4ceb7678e2972c6a300091d799ab94281a47adbc8bb673bfbf3b...
```

Breaking this down:
- `01000000` = version 1
- `0001` = SegWit marker (0x00) and flag (0x01)
- The parser incorrectly interpreted `0001` as the input count

**Solution**: Updated the transaction parser to:
- Detect SegWit marker and flag
- Properly skip witness data when present
- Correctly parse the actual input count

### 2. Incorrect Script Type Requirements

**Issue**: The system was attempting to fetch previous transaction hex for ALL inputs, but KeepKey only requires this for legacy (P2PKH) inputs.

**HDWallet Behavior** (reference implementation):
- Legacy inputs (P2PKH): REQUIRE full previous transaction hex
- SegWit inputs (P2SH-P2WPKH, P2WPKH): Do NOT need hex

**Solution**: 
- Only fetch transaction hex for legacy (p2pkh) inputs
- Skip hex fetching for SegWit inputs (p2sh-p2wpkh, p2wpkh)

### 3. Script Type Naming Inconsistency

**Issue**: Frontend was using `p2sh` while backend expected `p2sh-p2wpkh`.

**Solution**: Standardized script type names across the stack:
- `p2pkh` → Legacy
- `p2sh-p2wpkh` → SegWit wrapped in P2SH (not just `p2sh`)
- `p2wpkh` → Native SegWit

### 4. Change Address Derivation Bug

**Issue**: Change addresses were mixing derivation paths from different script types, potentially sending funds to addresses outside the gap limit.

**Example of the bug**:
```javascript
// Wrong: Using P2SH change index with P2WPKH path
{
  "address_n_list": [2147483732, 2147483648, 2147483648, 1, 58],  // Index 58 from P2SH
  "script_type": "p2wpkh"  // But using native SegWit type!
}
```

**Solution**: 
- Always use native SegWit (p2wpkh) for change addresses
- Use correct BIP84 derivation path: `m/84'/0'/0'/1/x`
- Ensure change address index matches the script type

## Implementation Details

### Frontend Changes (TypeScript/React)

#### 1. Transaction Building (`createUnsignedUxtoTx.ts`)

```typescript
// Only fetch hex for legacy inputs
if (scriptType === 'p2pkh') {
  // Legacy inputs REQUIRE the full previous transaction
  hex = await PioneerAPI.getRawTransaction(hash);
} else {
  // SegWit inputs (p2sh-p2wpkh, p2wpkh) do NOT need hex
  console.log(`⚡ SegWit input detected (${scriptType}) - no hex needed`);
}
```

#### 2. Change Address Logic

```typescript
// Always use native segwit (p2wpkh) for change addresses
const changeScriptType = 'p2wpkh';
const changeXpub = relevantPubkeys.find(pk => pk.scriptType === 'p2wpkh')?.pubkey;
const path = `m/84'/0'/0'/1/${changeAddressIndex}`;  // BIP84 path
```

### Backend Changes (Rust)

#### 1. Transaction Parser (`commands.rs`)

```rust
// Check for SegWit marker and flag
let mut is_segwit = false;
let input_count = {
    let first_byte = read_varint(&mut cursor)?;
    if first_byte == 0 {
        // This might be SegWit marker (0x00) followed by flag (0x01)
        let flag = read_u8(&mut cursor)?;
        if flag == 1 {
            is_segwit = true;
            // Now read the actual input count
            read_varint(&mut cursor)?
        }
    } else {
        first_byte
    }
};

// Skip witness data if present
if is_segwit {
    for _ in 0..input_count {
        let witness_count = read_varint(&mut cursor)?;
        for _ in 0..witness_count {
            let witness_len = read_varint(&mut cursor)? as usize;
            // Skip witness bytes
            let mut witness_data = vec![0u8; witness_len];
            read_exact(&mut cursor, &mut witness_data)?;
        }
    }
}
```

#### 2. Input Validation (`device/queue.rs`)

```rust
// Only legacy (p2pkh) inputs require previous transaction hex
let needs_hex = input.script_type == "p2pkh";

if needs_hex && input.prev_tx_hex.is_none() {
    return Err(format!("Legacy input {} missing required previous transaction hex", idx));
} else if !needs_hex {
    println!("⚡ SegWit input {} ({}): no hex required", idx, input.script_type);
}
```

## Testing & Verification

### Test Scenarios

1. **Legacy Input Transaction**: 
   - Input type: P2PKH
   - Hex required: YES
   - Result: ✅ Successfully signs

2. **SegWit Input Transaction**:
   - Input type: P2SH-P2WPKH or P2WPKH
   - Hex required: NO
   - Result: ✅ Successfully signs

3. **Mixed Input Transaction**:
   - Multiple input types
   - Hex fetched only for legacy inputs
   - Result: ✅ Successfully signs

### Verification Steps

1. Check that hex is only fetched for legacy inputs in browser console
2. Verify Rust backend logs show "SegWit input: no hex required" for non-legacy inputs
3. Confirm change addresses always use native SegWit (BIP84) paths
4. Validate successful transaction signing and broadcasting

## Migration to Mempool.space API

As part of this fix, we also migrated from Blockstream API to Mempool.space for improved reliability:

```typescript
// Primary API
const response = await axios.get(
  `https://mempool.space/api/tx/${txid}/hex`
);

// Fallback to Blockstream if needed
if (error) {
  const fallback = await axios.get(
    `https://blockstream.info/api/tx/${txid}/hex`
  );
}
```

Benefits:
- Better uptime and reliability
- Richer transaction data
- More responsive API
- Better rate limits

## Key Learnings

1. **Hardware Wallet Security**: KeepKey validates previous transactions differently for legacy vs SegWit inputs as a security measure
2. **Transaction Format Evolution**: Bitcoin's SegWit upgrade added complexity that must be handled at the parsing level
3. **HDWallet Compatibility**: Following the HDWallet reference implementation patterns ensures device compatibility
4. **Script Type Consistency**: Maintaining consistent script type naming across the entire stack is crucial

## References

- [BIP141 - SegWit](https://github.com/bitcoin/bips/blob/master/bip-0141.mediawiki)
- [BIP84 - Native SegWit Addresses](https://github.com/bitcoin/bips/blob/master/bip-0084.mediawiki)
- [HDWallet Bitcoin Implementation](https://github.com/shapeshift/hdwallet/blob/master/packages/hdwallet-keepkey/src/bitcoin.ts)
- [KeepKey Protocol Documentation](https://github.com/keepkey/keepkey-firmware/wiki)

## Conclusion

The "invalid prevhash 2" error was caused by a combination of:
1. Incorrect SegWit transaction parsing
2. Unnecessary hex fetching for SegWit inputs
3. Script type naming inconsistencies
4. Change address derivation bugs

All issues have been resolved, and the system now correctly handles both legacy and SegWit transactions according to KeepKey's security requirements.