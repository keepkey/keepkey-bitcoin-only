# Resolved: KeepKey CLI Cached Addresses Disappearing Bug

**Date Resolved:** 2025-06-09

## 1. Problem Description

The `kkcli` server was consistently failing on startup with the error:
`FAIL FAST: No cached addresses found for device <DEVICE_ID>! DB is present but empty. Stopping.`

This occurred even when addresses were manually inserted into the `cached_addresses` table in the `device_cache.db` SQLite database. The server logs indicated it was using the correct database file and device ID, yet it could not find the pre-existing cached address data.

## 2. Debugging Process & Methodology

A systematic, scientific approach was taken to diagnose the root cause:

1.  **Initial Checks:**
    *   Verified the exact database file path (`/Users/highlander/.keepkey/kkcli/device_cache.db`) and device ID used at runtime by adding detailed logging. Both were confirmed to be correct.
    *   Manually inserted test addresses into `cached_addresses` and confirmed their presence via direct SQLite queries *before* server startup.

2.  **Schema Review:**
    *   The `schema.sql` file was inspected. All `CREATE TABLE` statements used `IF NOT EXISTS`, and `CREATE INDEX` also used `IF NOT EXISTS`. No `DROP TABLE` or other destructive statements were found. This ruled out the schema migration process as the cause of data loss.

3.  **Codebase Search for Destructive SQL:**
    *   The Rust codebase was searched for any SQL statements like `DELETE FROM cached_addresses`, `TRUNCATE TABLE cached_addresses`, or `DROP TABLE cached_addresses`. None were found.

4.  **Database Internals:**
    *   **Journal Mode:** Checked with `PRAGMA journal_mode;` which returned `delete`. This is a standard mode and compatible with the server's operations.
    *   **File Permissions:** Verified using `ls -l`; the user running the server had appropriate read/write permissions.

5.  **Hypothesis: Foreign Key Cascade Deletion:**
    *   The `cached_addresses` table has a foreign key constraint:
        ```sql
        FOREIGN KEY (device_id) REFERENCES devices(device_id) ON DELETE CASCADE
        ```
    *   It was hypothesized that an operation on the parent `devices` table might be triggering this cascade.
    *   **Test:**
        *   Inserted data into `cached_addresses`.
        *   Queried `devices` and `cached_addresses` *before* server start (data present).
        *   Started the server (it failed with the error).
        *   Queried `devices` and `cached_addresses` *after* server start. **Result:** `devices` still had the entry, but `cached_addresses` was now empty.

6.  **Pinpointing the Culprit:**
    *   The `grep_search` tool was used to find all code interacting with the `devices` table in `device_cache.rs`.
    *   The function `DeviceCache.save_features` was identified as using the following SQL:
        ```sql
        INSERT OR REPLACE INTO devices ...
        ```
    *   **Diagnosis:** The `INSERT OR REPLACE` statement, when encountering an existing `device_id` (which is `UNIQUE`), first *deletes* the old row and then *inserts* a new one. The deletion of the old `devices` row triggered the `ON DELETE CASCADE` for `cached_addresses`, wiping the manually inserted data.

## 3. Solution Implemented

The SQL statement in `DeviceCache.save_features` (located in `/Users/highlander/WebstormProjects/keepkey-stack/projects/keepkey-rust/projects/kkcli/src/server/cache/device_cache.rs`) was modified to use an "UPSERT" mechanism:

**Old (Problematic) Code:**
```sql
INSERT OR REPLACE INTO devices 
 (device_id, ..., created_at)
 VALUES (?1, ..., COALESCE((SELECT created_at FROM devices WHERE device_id = ?1), ?11))
```

**New (Corrected) Code:**
```sql
INSERT INTO devices 
 (device_id, ..., created_at)
 VALUES (?1, ..., COALESCE((SELECT created_at FROM devices WHERE device_id = ?1), ?11))
 ON CONFLICT(device_id) DO UPDATE SET
   label = excluded.label,
   vendor = excluded.vendor,
   major_version = excluded.major_version,
   minor_version = excluded.minor_version,
   patch_version = excluded.patch_version,
   revision = excluded.revision,
   firmware_hash = excluded.firmware_hash,
   bootloader_hash = excluded.bootloader_hash,
   features_json = excluded.features_json,
   last_seen = excluded.last_seen
```
This `INSERT ... ON CONFLICT DO UPDATE` statement updates the existing device record in place if a conflict on `device_id` occurs, or inserts a new record if it doesn't exist. Crucially, it avoids deleting the parent row, thereby preventing the cascade deletion of associated `cached_addresses`.

## 4. Verification

After applying the fix:
1.  Test data was re-inserted into `cached_addresses`.
2.  The `kkcli server` was started.
3.  The server successfully initialized, loaded the 3 cached addresses, and did not produce the "FAIL FAST" error.

This confirmed the solution effectively resolved the bug.