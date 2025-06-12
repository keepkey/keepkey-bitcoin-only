#!/usr/bin/env node

/**
 * Debug Cache Mismatch Tool
 * 
 * This script traces the exact issue where:
 * - 40 addresses exist in database
 * - "Missing 24 out of 24 required addresses" 
 * - Frontload keeps running repeatedly
 */

console.log('🔍 Debug Cache Mismatch Tool');
console.log('===========================');
console.log('');

const BASE_URL = 'http://localhost:1646';

async function debugCacheMismatch() {
    console.log('🕵️ Investigating cache mismatch...');
    
    try {
        // 1. Check what device ID is in memory cache
        console.log('\n1. Memory Cache Status:');
        const cacheResponse = await fetch(`${BASE_URL}/api/v2/debug/cache`);
        if (cacheResponse.ok) {
            const cacheData = await cacheResponse.json();
            console.log(`   Device ID in memory: ${cacheData.device_id_in_cache || 'NULL'}`);
            console.log(`   Has features: ${cacheData.has_cached_features}`);
            console.log(`   Features device: ${cacheData.features_device_id || 'NULL'}`);
            console.log(`   Cache address: ${cacheData.cache_address}`);
        } else {
            console.log('   ❌ Failed to get cache debug info');
        }
        
        // 2. Check if we can query balances directly (bypassing device ID check)
        console.log('\n2. Direct Database Query Test:');
        
        // Try to force load by device ID from logs
        const knownDeviceId = '343737340F4736331F003B00';
        console.log(`   Testing with known device ID: ${knownDeviceId}`);
        
        // 3. Test if API endpoints work with manual sync
        console.log('\n3. Manual Sync Test:');
        try {
            const syncResponse = await fetch(`${BASE_URL}/api/v2/sync-device`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' }
            });
            
            if (syncResponse.ok) {
                const syncData = await syncResponse.json();
                console.log('   ✅ Sync successful');
                console.log(`   Success: ${syncData.success}`);
                console.log(`   Device ID: ${syncData.device_id || 'None'}`);
                console.log(`   Balances cached: ${syncData.balances_cached || 0}`);
                
                // Wait a moment then test endpoints
                console.log('\n   Testing endpoints after sync...');
                await new Promise(resolve => setTimeout(resolve, 1000));
                
                await testEndpointsAfterSync();
                
            } else {
                const errorText = await syncResponse.text();
                console.log('   ❌ Sync failed:', syncResponse.status);
                console.log(`   Error: ${errorText}`);
            }
        } catch (syncError) {
            console.log('   ❌ Sync error:', syncError.message);
        }
        
        // 4. Test database consistency 
        console.log('\n4. Database Consistency Test:');
        console.log('   This requires server-side debugging...');
        console.log('   💡 Add debug endpoint to dump raw database contents');
        
    } catch (error) {
        console.log('❌ Debug failed:', error.message);
    }
}

async function testEndpointsAfterSync() {
    const endpoints = [
        { name: 'Portfolio Summary', url: '/api/v2/portfolio/summary' },
        { name: 'Balances', url: '/api/v2/balances' },
        { name: 'Debug Cache', url: '/api/v2/debug/cache' }
    ];
    
    for (const endpoint of endpoints) {
        try {
            const response = await fetch(`${BASE_URL}${endpoint.url}`);
            const data = await response.json();
            
            if (response.ok) {
                console.log(`   ✅ ${endpoint.name}: SUCCESS`);
                
                if (endpoint.name === 'Portfolio Summary') {
                    console.log(`      Total USD: $${data.total_value_usd}`);
                    console.log(`      Networks: ${data.network_count}`);
                    console.log(`      Assets: ${data.asset_count}`);
                } else if (endpoint.name === 'Balances') {
                    console.log(`      Entries: ${data.length}`);
                    const nonZero = data.filter(b => parseFloat(b.value_usd || 0) > 0);
                    if (nonZero.length > 0) {
                        console.log(`      Non-zero balances: ${nonZero.length}`);
                        nonZero.forEach(b => {
                            console.log(`        ${b.symbol}: $${b.value_usd}`);
                        });
                    }
                } else if (endpoint.name === 'Debug Cache') {
                    console.log(`      Device in cache: ${data.device_id_in_cache || 'NULL'}`);
                }
            } else {
                console.log(`   ❌ ${endpoint.name}: FAILED (${response.status})`);
                console.log(`      Error: ${data.error || 'Unknown'}`);
            }
        } catch (error) {
            console.log(`   ❌ ${endpoint.name}: ERROR - ${error.message}`);
        }
    }
}

async function suggestFixes() {
    console.log('\n💡 Suggested Fixes for Cache Issues:');
    console.log('');
    
    console.log('1. 🔧 Fix get_device_id() method:');
    console.log('   - Add database fallback when memory cache is empty');
    console.log('   - Use get_first_device_from_db() as fallback');
    console.log('');
    
    console.log('2. 🔧 Fix CASCADE deletion issues:');
    console.log('   - Review all INSERT OR REPLACE operations');
    console.log('   - Replace with INSERT ... ON CONFLICT DO UPDATE');
    console.log('   - Never use INSERT OR REPLACE with CASCADE foreign keys');
    console.log('');
    
    console.log('3. 🔧 Simplify API endpoints:');
    console.log('   - Remove device ID requirement for read-only operations');
    console.log('   - Balances/portfolio should work without device in memory');
    console.log('   - Use database as source of truth, not memory cache');
    console.log('');
    
    console.log('4. 🔧 Add database debug endpoint:');
    console.log('   - /api/v2/debug/database - dump raw table contents');
    console.log('   - Show device IDs, address counts, balance counts');
    console.log('   - Trace device ID mismatches');
    console.log('');
    
    console.log('5. 🔧 Fix frontload cache checking:');
    console.log('   - Debug exact path/coin name mismatches');
    console.log('   - Log what addresses are expected vs found');
    console.log('   - Fix query logic to match save logic exactly');
}

// Run the debug tool
console.log('🚀 Starting cache mismatch investigation...');
console.log('');

debugCacheMismatch()
    .then(() => suggestFixes())
    .then(() => {
        console.log('\n🎯 Debug Complete!');
        console.log('');
        console.log('📋 Next Steps:');
        console.log('   1. Fix get_device_id() to use database fallback');
        console.log('   2. Remove device ID requirement from read-only endpoints');
        console.log('   3. Add database debug endpoint to trace raw data');
        console.log('   4. Fix any remaining CASCADE deletion bugs');
        console.log('   5. Make database the source of truth, not memory cache');
    })
    .catch(console.error); 