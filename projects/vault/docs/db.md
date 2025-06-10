index.db Planning Document
Unencrypted, low-latency metadata store for KeepKey Desktop

1 Purpose & Scope
Goal	Outcome
Instant startup	App can render account list, balances, and recent activity without waiting for vault unlock.
Sync journal	Acts as durable cache for on-chain sync cursors, block heights, server paging tokens, etc.
Public-only data	Stores nothing that would let an attacker spend funds or break 2FA.

2 File Location
OS	Path
macOS / Linux	~/.keepkey/index.db
Windows	%APPDATA%\KeepKey\index.db

Directory created on first run (std::fs::create_dir_all()).

3 Technical Requirements
Plain SQLite 3 – no SQLCipher, WAL enabled.

Single-writer, many-reader – sync task writes; UI threads read.

Migrations – handled by rusqlite_migration.

Size ceiling – < 200 MB even with years of history (prune old blocks).

No secrets – only xpubs, addresses, tx metadata, user prefs.

4 Schema v1
sql
Copy
Edit
PRAGMA journal_mode=WAL;
PRAGMA foreign_keys = ON;

CREATE TABLE accounts (
id           INTEGER PRIMARY KEY,
wallet_fp    TEXT NOT NULL,      -- 4-byte fingerprint (hex)
kind         TEXT NOT NULL,      -- 'keepkey' | 'digital'
xpub         TEXT NOT NULL,
label        TEXT,
added_ts     INTEGER NOT NULL    -- epoch seconds
);

CREATE UNIQUE INDEX idx_accounts_fp_xpub ON accounts(wallet_fp, xpub);

CREATE TABLE addresses (
id           INTEGER PRIMARY KEY,
account_id   INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
address      TEXT NOT NULL UNIQUE,
deriv_path   TEXT NOT NULL,      -- "m/84'/0'/0'/0/15"
first_seen   INTEGER             -- block height
);

CREATE INDEX idx_addresses_account ON addresses(account_id);

CREATE TABLE txs (
txid         TEXT PRIMARY KEY,
account_id   INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
block_height INTEGER,
direction    INTEGER NOT NULL,   -- +sats (recv) / -sats (send)
amount       INTEGER NOT NULL,   -- satoshis (always positive)
fee          INTEGER,            -- satoshis
timestamp    INTEGER             -- tx time per node
);

CREATE INDEX idx_txs_account_block ON txs(account_id, block_height);

CREATE TABLE meta (
key TEXT PRIMARY KEY,
val TEXT
);
Design notes
direction keeps sums simple (SUM(direction * amount) → balance).

meta stores "btc_last_height", "eth_last_block", theme prefs, etc.

All INTEGER timestamps are UTC epoch seconds.

5 Rust Access Layer (index.rs)
rust
Copy
Edit
pub struct IndexDb(Connection);

impl IndexDb {
pub fn open() -> Result<Self> {
let p = data_dir().join("index.db");
let conn = Connection::open_with_flags(
p,
OpenFlags::SQLITE_OPEN_READ_WRITE | SQLITE_OPEN_CREATE,
)?;
conn.pragma_update(None, "journal_mode", &"WAL")?;
MIGRATIONS.to_latest(&conn)?;   // rusqlite_migration
Ok(Self(conn))
}

    /* ---------- high-level ops ---------- */

    pub fn upsert_account(&self, fp: &str, xpub: &str, kind: &str, label: Option<&str>) { … }

    pub fn insert_txs_batch(&self, acc_id: i64, rows: &[TxRow]) { … }  // wrap BEGIN/COMMIT

    pub fn latest_height(&self, chain: &str) -> Option<u64> { … }

    pub fn list_balances(&self) -> Vec<AccountBalance> { … }
}
Return plain structs; Tauri command layer converts to JSON.

6 Startup & Runtime Flow
text
Copy
Edit
┌───────── App launch ─────────┐
│                              │
│ 1  open IndexDB              │
│ 2  fetch accounts, balances  │  (UI renders ≈50 ms)
│ 3  start sync task           │
│     ├─ determine last_height │
│     ├─ fetch delta blocks    │
│     └─ insert_txs_batch      │
│ 4  if/when vault unlocks     │
│     └─ merge private data    │
└──────────────────────────────┘
7 Performance Targets (M1/M2, WAL on SSD)
Operation	Rows	≤ Target
Cold start (open + SELECT balances)	–	50 ms
Batch insert blocks	10 000 TXs	< 0.9 s
PK lookup (txid)	1	< 150 µs
Daily VACUUM (after prune)	–	< 2 s

Use PRAGMA synchronous=NORMAL during sync; revert to EXTRA on idle.

8 Migration Strategy
Version table managed by rusqlite_migration.

v2 candidate: add coin_type column for multisig or alt-chains.

Backward-compatible reads in UI; writer always targets latest schema.

9 Backup & Restore
Both DBs bundled in Settings → “Export data” (zip).

Restore wizard: copy index.db first → optional vault import.

For cloud sync later: let user upload encrypted vault.db only; index.db can be rebuilt from chain.

10 Testing Checklist
Unit – CRUD round-trips, foreign-key cascade.

Load – Simulate 1 M TXs insert; assert < time budget.

Fuzz – Random set of inserts + deletes; verify PRAGMA integrity_check passes.

Cold-start – Benchmark open-&-query on CI.

Next Steps
Implement index.rs with migration v1.

Wire new Tauri commands: db_open_index, index_get_balances, index_insert_txs.

Modify sync worker to use insert_txs_batch.

Front-end: render read-only dashboard when vault absent.

Once merged, you have an ultra-fast public cache while the secure vault remains optional.