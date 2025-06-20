# TXID Schema & Address-Usage Integration Plan

Date: 2025-06-11
Author: Cascade

---

## 1  Objectives

1.  **Persist transaction IDs (TXIDs) per address** in the local SQLite cache.
2.  Provide quick lookup of
    * all TXIDs for an address,
    * addresses involved in a TXID (optional future work),
    * aggregated usage stats (firstSeen, lastSeen, totalReceived/Sent).
3.  Support incremental sync via Pioneer (or other indexers) with idempotent updates.
4.  Migrate existing DB without data loss.
5.  Keep front-load / sync performant (batch inserts, indices).

---

## 2  Existing DB Overview

| Table | Purpose |
|-------|---------|
| `addresses` | Cached addresses per path/asset. |
| `paths` | Derivation path metadata. |
| `features` | Device features per device. |

No TXID storage exists; balance & tx-count are fetched on demand.

---

## 3  Proposed Schema Additions

```sql
-- new table
CREATE TABLE txids (
  id            INTEGER PRIMARY KEY,
  txid          TEXT    NOT NULL,   -- 64-char hex
  address_id    INTEGER NOT NULL,   -- FK → addresses.id
  block_height  INTEGER,            -- NULL for mempool
  received_at   INTEGER,            -- Unix epoch secs (first seen)
  value         TEXT,               -- satoshis as string to preserve 64-bit
  direction     TEXT,               -- 'in' | 'out' | 'self'
  UNIQUE(txid, address_id)
);

-- indices
CREATE INDEX idx_txids_address_id        ON txids(address_id);
CREATE INDEX idx_txids_block_height      ON txids(block_height);
CREATE INDEX idx_txids_address_block_tx  ON txids(address_id, block_height DESC);
```

Optional future table `tx_cache` to store raw JSON for full Tx lookup.

---

## 4  Data Flow

1. **Sync call** to Pioneer (example payload in ticket) returns `txids` array.  
2. For each address:
   1. Upsert row in `addresses` (already exists).
   2. Bulk-insert TXIDs via `INSERT OR IGNORE` into `txids`.
   3. Update `addresses.last_synced_height` (add column) & balances.
3. Repeat until `usedTokens == totalPages`.

Batch insertion example (rusqlite):
```rust
let tx = conn.transaction()?;
let mut stmt = tx.prepare_cached("INSERT OR IGNORE INTO txids (txid, address_id, block_height, received_at, value, direction) VALUES (?,?,?,?,?,?)")?;
for entry in entries {
    stmt.execute(params![entry.txid, addr_id, entry.height, entry.time, entry.value, entry.direction])?;
}
tx.commit()?;
```

---

## 5  Migration Strategy

1. **DDL Gate**: bump `user_version` in SQLite.  
2. On startup, detect old version → run `ALTER TABLE`/`CREATE TABLE` statements inside a transaction.  
3. Add try-catch and fallback to prompt user if schema corrupt.

---

## 6  Backend API Changes

### 6.1  Rust `DeviceCache`

* Add `save_txids(&self, address_id: i64, txs: Vec<TxidRow>)`.
* Add `get_txids(&self, address_id, limit, offset)`.
* Add `get_tx_summary(&self, address_id)` (aggregate sums & counts).

### 6.2  Frontload / Sync

* Create `TxSyncWorker` that iterates cached addresses lacking `addrTxCount` rows.
* Respect Pioneer pagination (`page`, `totalPages`).
* Store `usedTokens` to resume later when API limits hit.

### 6.3  REST Endpoints

| Route | Method | Description |
|-------|--------|-------------|
| `/api/address/:addr/txids` | GET | paginated list of txids |
| `/api/address/:addr` | GET | returns address + balance + tx-count (already) |

---

## 7  Performance Considerations

* Use **single SQLite connection / pool** (see optimisation doc).  
* Wrap bulk inserts in one transaction → ~50-100× faster.  
* Index carefully; watch write amplification when syncing thousands of txids.

---

## 8  Testing Plan

1. **Unit tests** for `DeviceCache::save_txids` ensuring idempotency.  
2. **Integration test** using mocked Pioneer response (`test-pioneer-only.js`) verifying:
   * correct row count
   * duplicates ignored
   * pagination handling.
3. **Migration test**: load pre-migration DB, run app, verify schema upgraded.

---

## 9  Timeline & Ownership

| Task | Owner | ETA |
|------|-------|-----|
| Schema migration (Rust) | ⚙️  backend | 2 d |
| DeviceCache impl | ⚙️  backend | 2 d |
| TxSyncWorker | ⚙️  backend | 3 d |
| REST endpoints | ⚙️  backend | 2 d |
| React UI list (optional) | 🎨 frontend | 2 d |
| Tests + CI pipeline | 🧪 dev-infra | 2 d |
| Buffer / Review | 🔍 team | 1 d |

_Total: **~2 weeks** elapsed time._

---

## 10  Open Questions

1. Should we cache full raw Tx JSON for offline detail view?  
2. Multi-asset support: same `txids` table can work, but need `asset_id` column?  
3. How to handle reorgs? (tx drops / block_height changes)  
4. Privacy: store xpub-derived addresses only or allow arbitrary addresses query?

---

> "Shipping beats perfection." — This plan aims for an MVP that can evolve with future indexer integrations.
