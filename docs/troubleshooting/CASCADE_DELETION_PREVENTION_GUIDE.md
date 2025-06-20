# 🚨 CRITICAL: CASCADE DELETION PREVENTION GUIDE 🚨

**Date Created:** 2025-06-12  
**Last Updated:** 2025-06-12  
**Severity:** CRITICAL - Data Loss Prevention  

## Overview

This document provides essential guidelines to prevent accidental CASCADE deletions that cause cache clearing bugs in the KeepKey application. The issue was first identified and resolved on June 9th, 2025, but regressed due to incomplete fixes. This guide ensures it never happens again.

## ⚠️ THE GOLDEN RULE ⚠️

**NEVER USE `INSERT OR REPLACE` ON TABLES WITH CASCADE FOREIGN KEY CONSTRAINTS!**

## What is the Problem?

### Background
Our database schema uses foreign key constraints with `ON DELETE CASCADE` to maintain referential integrity:

```sql
-- Example from schema.sql
CREATE TABLE cached_addresses (
    ...
    device_id TEXT NOT NULL,
    ...
    FOREIGN KEY (device_id) REFERENCES devices(device_id) ON DELETE CASCADE
);

CREATE TABLE cached_balances (
    ...
    device_id TEXT NOT NULL,
    ...
    FOREIGN KEY (device_id) REFERENCES devices(device_id) ON DELETE CASCADE
);

CREATE TABLE portfolio_summaries (
    ...
    device_id TEXT NOT NULL UNIQUE,
    ...
    FOREIGN KEY (device_id) REFERENCES devices(device_id) ON DELETE CASCADE
);
```

### The Dangerous Pattern
When you use `INSERT OR REPLACE` on these tables, SQLite performs:
1. **DELETE** the existing row (if unique constraint conflict exists)
2. **INSERT** the new row

The DELETE operation triggers CASCADE deletion of ALL related data!

### Affected Tables
**All tables with CASCADE foreign keys to `devices` table:**
- `cached_addresses`
- `cached_balances` 
- `portfolio_summaries`

**Safe tables (no CASCADE constraints):**
- `config` - ✅ Safe to use `INSERT OR REPLACE`
- `devices` - ✅ Safe (fixed with proper UPSERT)
- `networks` - ✅ Safe (no CASCADE constraints)
- `paths` - ✅ Safe (no CASCADE constraints)

## ✅ CORRECT SOLUTIONS

### For cached_addresses
```sql
-- ❌ WRONG (causes CASCADE deletion)
INSERT OR REPLACE INTO cached_addresses 
  (device_id, coin, script_type, derivation_path, address, pubkey, created_at)
  VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)

-- ✅ CORRECT (safe UPSERT)
INSERT INTO cached_addresses 
  (device_id, coin, script_type, derivation_path, address, pubkey, created_at)
  VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
  ON CONFLICT(device_id, coin, script_type, derivation_path) DO UPDATE SET
    address = excluded.address,
    pubkey = excluded.pubkey,
    created_at = excluded.created_at
```

### For cached_balances
```sql
-- ❌ WRONG (causes CASCADE deletion)
INSERT OR REPLACE INTO cached_balances 
  (device_id, caip, pubkey, balance, price_usd, value_usd, symbol, network_id, last_updated)
  VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)

-- ✅ CORRECT (safe UPSERT)
INSERT INTO cached_balances 
  (device_id, caip, pubkey, balance, price_usd, value_usd, symbol, network_id, last_updated)
  VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
  ON CONFLICT(device_id, caip, pubkey) DO UPDATE SET
    balance = excluded.balance,
    price_usd = excluded.price_usd,
    value_usd = excluded.value_usd,
    symbol = excluded.symbol,
    network_id = excluded.network_id,
    last_updated = excluded.last_updated
```

### For portfolio_summaries
```sql
-- ❌ WRONG (causes CASCADE deletion)
INSERT OR REPLACE INTO portfolio_summaries 
  (device_id, total_value_usd, network_count, asset_count, last_updated)
  VALUES (?1, ?2, ?3, ?4, ?5)

-- ✅ CORRECT (safe UPSERT)
INSERT INTO portfolio_summaries 
  (device_id, total_value_usd, network_count, asset_count, last_updated)
  VALUES (?1, ?2, ?3, ?4, ?5)
  ON CONFLICT(device_id) DO UPDATE SET
    total_value_usd = excluded.total_value_usd,
    network_count = excluded.network_count,
    asset_count = excluded.asset_count,
    last_updated = excluded.last_updated
```

### For devices table (already fixed)
```sql
-- ✅ CORRECT (safe UPSERT for parent table)
INSERT INTO devices 
  (device_id, label, vendor, major_version, minor_version, patch_version,
   revision, firmware_hash, bootloader_hash, features_json, last_seen, created_at)
  VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11,
          COALESCE((SELECT created_at FROM devices WHERE device_id = ?1), ?11))
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

## 🔍 HOW TO IDENTIFY THE PROBLEM

### Symptoms
- Cache appears to work during frontload
- Cache data disappears after startup
- API endpoints report "No device found in cache"
- Database file exists but appears empty on subsequent runs
- Error messages like "FAIL FAST: No cached addresses found for device!"

### Debugging Steps
1. Check for `INSERT OR REPLACE` statements in code
2. Verify foreign key constraints with CASCADE
3. Check logs for successful cache population followed by cache misses
4. Manually inspect database before/after server restart

## 🛡️ PREVENTION MEASURES

### Code Review Checklist
- [ ] No `INSERT OR REPLACE` used on tables with CASCADE foreign keys
- [ ] All UPSERT operations use `INSERT ... ON CONFLICT DO UPDATE`
- [ ] Proper UNIQUE constraints defined for UPSERT targets
- [ ] Comments added warning about CASCADE dangers

### Testing Protocol
1. **Before any database-related changes:**
   - Run full cache population
   - Restart application
   - Verify cache persistence
   - Check API endpoints return cached data

2. **CI/CD Integration:**
   - Add automated tests that verify cache persistence across restarts
   - Include database schema validation
   - Test foreign key constraint behavior

### Documentation Requirements
Every database function that could affect cached data MUST include:

```rust
/// Function description
/// 
/// 🚨 CRITICAL WARNING: ON DELETE CASCADE DANGER 🚨
/// 
/// This function [description of what it does with database]
/// The [table_name] table has: FOREIGN KEY (device_id) REFERENCES devices(device_id) ON DELETE CASCADE
/// When "INSERT OR REPLACE" encounters a UNIQUE constraint conflict, it:
/// 1. DELETES the existing row (triggering CASCADE deletion of ALL related data!)
/// 2. INSERTS the new row
/// 
/// This was the root cause of the cache clearing bug documented in:
/// /docs/debugging/resolved_cache_bug_summary_20250609.md
/// 
/// SOLUTION: Use "INSERT ... ON CONFLICT DO UPDATE" instead, which updates in-place
/// without triggering CASCADE deletions.
/// 
/// ⚠️ NEVER USE "INSERT OR REPLACE" ON TABLES WITH CASCADE FOREIGN KEYS! ⚠️
```

## 📁 AFFECTED FILES

### Successfully Fixed
- ✅ `projects/kkcli/src/server/cache/device_cache.rs`
  - Fixed `save_address()` method
  - Fixed `save_balances()` method  
  - Fixed `save_portfolio_summary()` method
  - Fixed `save_features()` method (from June 9th fix)

- ✅ `projects/vault/src-tauri/src/cache/device_cache.rs`
  - Fixed `save_address()` method
  - Fixed `save_balances()` method
  - Fixed `save_portfolio_summary()` method
  - Fixed `save_features()` method (from June 9th fix)

### Database Schema Files
- `projects/kkcli/src/server/cache/schema.sql` - Contains CASCADE constraints
- `projects/vault/src-tauri/src/cache/schema.sql` - Contains CASCADE constraints

## 🚨 EMERGENCY RESPONSE

If cache clearing occurs again:

1. **Immediate Investigation:**
   ```bash
   # Search for INSERT OR REPLACE in codebase
   grep -r "INSERT OR REPLACE" --include="*.rs" .
   
   # Check for CASCADE constraints
   grep -r "ON DELETE CASCADE" --include="*.sql" .
   ```

2. **Quick Fix Pattern:**
   Replace any `INSERT OR REPLACE` with proper UPSERT using the examples above

3. **Verification:**
   - Test cache persistence across application restarts
   - Verify all API endpoints return cached data
   - Check database integrity

## 📚 REFERENCES

- **Original Bug Report:** `/docs/debugging/resolved_cache_bug_summary_20250609.md`
- **SQLite UPSERT Documentation:** https://www.sqlite.org/lang_UPSERT.html
- **Foreign Key Documentation:** https://www.sqlite.org/foreignkeys.html

## 🎯 CONCLUSION

The CASCADE deletion bug is entirely preventable by following this simple rule:

**Use `INSERT ... ON CONFLICT DO UPDATE` instead of `INSERT OR REPLACE` on any table with CASCADE foreign key constraints.**

This guide must be referenced and followed for ALL future database modifications to prevent data loss and cache clearing issues.

---

**Remember: An engineer is only as good as their tools. This documentation IS a tool. Keep it updated and refer to it often!** 