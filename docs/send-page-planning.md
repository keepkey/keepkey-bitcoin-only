# Send Page Planning Document

## Overview
The **Send** page allows users to construct and broadcast Bitcoin transactions while balancing simplicity for novices and granular control for power users. The workflow spans **compose → review → sign → broadcast**. This document defines UI/UX, state, storage, and backend requirements, mirroring the level of detail found in *receive-page-planning.md*.

---
## 1. User Interface Components

### 1.1 Compose Section
| Element | Easy Mode | Advanced / Coin Control | Notes |
|---------|-----------|-------------------------|-------|
| **Recipient(s)** | Single address + amount fields. | Multiple outputs table (address, amount, label). | Future: payjoin & lightning support. |
| **Fee Selector** | Radio buttons: _slow / medium / fast_.  Displays fiat estimate. | Numeric sat/vB input + mempool stats helper. | Fee presets configurable. |
| **Input Selector** | **Slider** (0 → 100 %) that auto-picks inputs by ascending age/value. | **Coin Control Table** listing every UTXO with: checkbox _selected_, lock 🔒 toggle, value, address, confirmations. | Locked inputs are persisted in DB. |
| **Max Button** | ✔ | ✔ | Sets amount = spendable balance minus fees. |

### 1.2 Dialogs / Wizards
1. **Review Transaction**  
   - Shows inputs, outputs, fees, size, change address.  
   - Warns if sending entire balance or using low confirmations.  
   - "Edit" returns to compose.
2. **Sign on Device**  
   - Animated instructions + progress bar.  
   - Handles PIN/passphrase requests.  
   - Retries on HID reconnect.
3. **Broadcast Result**  
   - Success txid + explorer link.  
   - Failure reason with retry.

### 1.3 Navigation & Feedback
- Breadcrumbs: **Compose → Review → Sign → Broadcast**.  Greyed steps become blue when complete.
- Toasts for clipboard copy, device errors, broadcast status.

---
## 2. State Management

```ts
interface SendWalletState {
  selectedDeviceId: string;
  selectedAccount: number;           // BIP44 account
  scriptType: 'p2pkh' | 'p2sh-p2wpkh' | 'p2wpkh';
  feeRatePreset: 'slow' | 'medium' | 'fast' | 'custom';
  customFeeRate?: number;            // sat/vB
  isEasyMode: boolean;               // true = slider, false = coin control
  sliderPercent: number;             // 0–100 when easy mode
  selectedInputs: Set<string>;       // txid:vout identifiers
  lockedInputs: Set<string>;         // persisted between sessions
  outputs: Array<{ address: string; amount: string }>;  // BTC in decimals
  changeAddress?: string;            // chosen by backend
  draftTx?: UnsignedTx;              // hex or PSBT
  signProgress?: {
    step: 'waiting' | 'pin' | 'passphrase' | 'signing' | 'done';
    message?: string;
  };
  broadcastStatus?: 'idle' | 'broadcasting' | 'success' | 'error';
  broadcastTxId?: string;
  error?: string;
}
```

---
## 3. Database Storage (SQLite)

### 3.1 UTXOs Table
```sql
CREATE TABLE utxos (
    txid TEXT NOT NULL,
    vout INTEGER NOT NULL,
    device_id TEXT NOT NULL,
    account INTEGER NOT NULL,
    script_type TEXT NOT NULL,
    address TEXT NOT NULL,
    amount_sat INTEGER NOT NULL,
    confirmations INTEGER NOT NULL,
    is_locked BOOLEAN DEFAULT FALSE,
    PRIMARY KEY (txid, vout)
);
```
- `is_locked` corresponds to user-toggled 🔒 in coin control.

### 3.2 Pending Transactions
```sql
CREATE TABLE pending_txs (
    id INTEGER PRIMARY KEY,
    device_id TEXT,
    account INTEGER,
    psbt BLOB,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```
Stores drafts if user closes app mid-signing.

---
## 4. Backend Responsibilities (Rust Axum)
1. **`/api/v2/utxos`** → return fresh UTXO set per account.
2. **`/api/v2/lock_utxo`** → toggle lock state in DB.
3. **`/api/v2/tx/build`**  
   Inputs: recipient list, feeRate, optional selectedInputs.  
   Returns: unsigned PSBT, fee, size, change output.
4. **`/api/v2/tx/sign`**  
   Consumes PSBT + device_id, returns signed PSBT.
5. **`/api/v2/tx/broadcast`** → pushes raw tx, returns txid.
6. **Coin Selection Algorithm**  
   - **Easy**: BnB or knapsack using sliderPercent of spendable.
   - **Advanced**: exact set chosen by user.

---
## 5. Transaction Flow
```text
User Input → /tx/build → Review Dialog → Sign Device → /tx/sign → /tx/broadcast → Done!
```
Steps with error rollbacks:
1. **Build** fails ⇒ show error in compose.
2. **Sign** fails ⇒ keep PSBT for retry.
3. **Broadcast** fails ⇒ allow manual broadcast with raw hex.

---
## 6. Coin Control Table Columns
| Column | Description | Sortable |
|--------|-------------|----------|
| Select ✔ | Checkbox for spending | N/A |
| Lock 🔒 | Prevent future spending | N/A |
| Amount (BTC) | Value of UTXO | ✔ |
| Address | Original deposit address |  |
| Confirmations | Age indicator | ✔ |
| Label | User label (optional) | ✔ |

Context menu options: _"Copy Outpoint", "Add Label", "View on Explorer"_.

---
## 7. Privacy & Security Considerations
- Warn if mixing script types within one tx.
- Highlight high-priority inputs that deanonymise clusters.
- Change address must use same script type as inputs.
- RBF flag on by default; allow disabling.

---
## 8. Implementation Phases
1. **Phase 1: Easy Mode MVP**  
   - Single-recipient send, slider input selection, review/sign/broadcast dialogs.
2. **Phase 2: Advanced Coin Control**  
   - Full UTXO table, lock support, multi-recipient outputs.
3. **Phase 3: Persistence & Recovery**  
   - Draft PSBT storage, pending tx resume.
4. **Phase 4: Power Features**  
   - Custom change address, coinjoin, payjoin, child-pays-for-parent.

---
*Prepared: 2025-06-12*
