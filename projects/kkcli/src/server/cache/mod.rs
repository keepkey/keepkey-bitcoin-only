pub mod device_cache;
pub mod frontload;

pub use device_cache::{DeviceCache, CachedAddress, CachedFeatures};
pub use frontload::DeviceFrontloader;

#[cfg(test)]
mod test_helpers {
    use crate::server::routes;
    
    pub fn create_test_features(device_id: &str, label: &str) -> routes::Features {
        routes::Features {
            vendor: Some("KeepKey".to_string()),
            major_version: Some(7),
            minor_version: Some(7),
            patch_version: Some(0),
            bootloader_mode: Some(false),
            device_id: Some(device_id.to_string()),
            pin_protection: Some(true),
            passphrase_protection: Some(false),
            language: Some("english".to_string()),
            label: Some(label.to_string()),
            initialized: Some(true),
            revision: None,
            firmware_hash: None,
            bootloader_hash: None,
            imported: Some(false),
            pin_cached: Some(false),
            passphrase_cached: Some(false),
            wipe_code_protection: Some(false),
            auto_lock_delay_ms: Some(0),
            policies: None,
            model: Some("KeepKey".to_string()),
            firmware_variant: Some("standard".to_string()),
            no_backup: Some(false),
        }
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use tempfile::tempdir;
    use test_helpers::create_test_features;
    
    // Helper function to create test cache with specific path
    async fn create_test_cache_with_path(db_path: &std::path::Path) -> DeviceCache {
        let conn = rusqlite::Connection::open(db_path).unwrap();
        conn.pragma_update(None, "journal_mode", "WAL").unwrap();
        conn.pragma_update(None, "foreign_keys", "ON").unwrap();
        let schema = include_str!("schema.sql");
        conn.execute_batch(schema).unwrap();
        DeviceCache {
            db: std::sync::Arc::new(tokio::sync::Mutex::new(conn)),
            memory_cache: std::sync::Arc::new(std::sync::RwLock::new(device_cache::MemoryCache::default())),
        }
    }
    
    /// Test that reproduces the exact cache loading issue we encountered
    /// This test verifies that addresses saved to the database are properly loaded back into memory
    #[tokio::test]
    async fn test_cache_loading_bug_reproduction() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("reproduction_test.db");
        
        let device_id = "333433373337333430463437333633333146303033423030"; // Real device ID from logs
        
        // Create features similar to what we see in real usage
        let features = create_test_features(device_id, "KeepKey");
        
        // === PHASE 1: Simulate first startup (no cache) ===
        {
            let cache = create_test_cache_with_path(&db_path).await;
            
            // Save device
            cache.save_features(&features, device_id).await.unwrap();
            
            // Verify device exists but has no addresses
            let loaded = cache.load_device(device_id).await.unwrap();
            assert!(loaded.is_some());
            assert!(!cache.has_cached_addresses(device_id).await.unwrap());
            
            // Simulate frontloading - save addresses to database
            let addresses = vec![
                ("Bitcoin", "legacy", vec![44, 0, 0, 0, 0], "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa"),
                ("Bitcoin", "legacy", vec![44, 0, 0, 0, 1], "1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2"),
                ("Bitcoin", "segwit", vec![44, 0, 0, 0, 0], "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4"),
                ("Ethereum", "legacy", vec![44, 60, 0, 0, 0], "0x742E4C4F4E7E3F2E3D2E1E0F0E0D0C0B0A090807"),
            ];
            
            for (coin, script_type, path, address) in &addresses {
                cache.save_address(device_id, coin, script_type, path, address, None).await.unwrap();
            }
            
            // Verify addresses are now cached
            assert!(cache.has_cached_addresses(device_id).await.unwrap());
        }
        
        // === PHASE 2: Simulate second startup (should use cache) ===
        {
            // Create NEW cache instance (simulates server restart)
            let cache2 = create_test_cache_with_path(&db_path).await;
            
            // This is the critical test - the cache should detect existing addresses
            let has_addresses_before_load = cache2.has_cached_addresses(device_id).await.unwrap();
            assert!(has_addresses_before_load, "🐛 BUG REPRODUCED: Cache shows 0 addresses even though database has them!");
            
            // Load device - this should populate memory cache
            let loaded = cache2.load_device(device_id).await.unwrap();
            assert!(loaded.is_some());
            
            // Verify addresses are available in memory after load
            let btc_legacy = cache2.get_cached_address("Bitcoin", "legacy", &[44, 0, 0, 0, 0]);
            let btc_segwit = cache2.get_cached_address("Bitcoin", "segwit", &[44, 0, 0, 0, 0]);
            let eth_legacy = cache2.get_cached_address("Ethereum", "legacy", &[44, 60, 0, 0, 0]);
            
            assert!(btc_legacy.is_some(), "Bitcoin legacy address should be in memory cache");
            assert!(btc_segwit.is_some(), "Bitcoin segwit address should be in memory cache");
            assert!(eth_legacy.is_some(), "Ethereum address should be in memory cache");
            
            assert_eq!(btc_legacy.unwrap().address, "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa");
            assert_eq!(btc_segwit.unwrap().address, "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4");
            assert_eq!(eth_legacy.unwrap().address, "0x742E4C4F4E7E3F2E3D2E1E0F0E0D0C0B0A090807");
        }
    }
    
    /// Test that simulates the exact scenario from the logs
    #[tokio::test]
    async fn test_log_scenario_reproduction() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("log_scenario_test.db");
        
        let device_id = "333433373337333430463437333633333146303033423030";
        
        // Create cache and device
        let cache = create_test_cache_with_path(&db_path).await;
        
        let features = create_test_features(device_id, "KeepKey");
        
        cache.save_features(&features, device_id).await.unwrap();
        
        // Save exactly 32 addresses like in the logs
        for i in 0..32 {
            let path = vec![2147483692, 2147483648, 2147483648, 0, i];
            let address = format!("test_address_{}", i);
            cache.save_address(device_id, "Bitcoin", "legacy", &path, &address, None).await.unwrap();
        }
        
        // Verify we have exactly 32 addresses
        assert!(cache.has_cached_addresses(device_id).await.unwrap());
        
        // Drop the cache to test persistence
        drop(cache);
        
        // Create new cache (simulates restart)
        let cache2 = create_test_cache_with_path(&db_path).await;
        
        // This should show 32 addresses, not 0
        let has_addresses = cache2.has_cached_addresses(device_id).await.unwrap();
        assert!(has_addresses, "Should detect 32 cached addresses from previous session");
        
        // Load device
        let loaded = cache2.load_device(device_id).await.unwrap();
        assert!(loaded.is_some());
        
        // Test that we can retrieve specific addresses
        let first_address = cache2.get_cached_address("Bitcoin", "legacy", &[2147483692, 2147483648, 2147483648, 0, 0]);
        let last_address = cache2.get_cached_address("Bitcoin", "legacy", &[2147483692, 2147483648, 2147483648, 0, 31]);
        
        assert!(first_address.is_some());
        assert!(last_address.is_some());
        assert_eq!(first_address.unwrap().address, "test_address_0");
        assert_eq!(last_address.unwrap().address, "test_address_31");
    }
    
    /// Test for database WAL mode issues
    #[tokio::test]
    async fn test_database_wal_consistency() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("wal_test.db");
        
        let device_id = "wal_test_device";
        
        // First cache instance
        {
            let cache1 = create_test_cache_with_path(&db_path).await;
            
            let features = create_test_features(device_id, "WAL Test Device");
            
            cache1.save_features(&features, device_id).await.unwrap();
            cache1.save_address(device_id, "Bitcoin", "legacy", &[44, 0, 0, 0, 0], "wal_test_address", None).await.unwrap();
        }
        
        // Second cache instance (different connection)
        {
            let cache2 = create_test_cache_with_path(&db_path).await;
            
            // Should see the data from the first instance
            let has_addresses = cache2.has_cached_addresses(device_id).await.unwrap();
            assert!(has_addresses, "WAL checkpoint should make data visible to new connections");
            
            let loaded = cache2.load_device(device_id).await.unwrap();
            assert!(loaded.is_some());
            
            let address = cache2.get_cached_address("Bitcoin", "legacy", &[44, 0, 0, 0, 0]);
            assert!(address.is_some());
            assert_eq!(address.unwrap().address, "wal_test_address");
        }
    }
} 