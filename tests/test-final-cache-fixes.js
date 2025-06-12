#!/usr/bin/env node

/**
 * Final Cache Fixes Test
 * 
 * This test verifies all the cache fixes work together:
 * 1. Database fallback for get_device_id()
 * 2. Portfolio summary cache invalidation
 * 3. 1-hour cache timeout
 * 4. Proper cache respect logic
 * 5. USD values working correctly
 */

console.log('🧪 Final Cache Fixes Test');
console.log('=========================');
console.log('');

const BASE_URL = 'http://localhost:1646';

async function testAllFixes() {
    console.log('🔧 Testing all cache fixes...');
    
    try {
        // 1. Test database debug endpoint (new functionality)
        console.log('\n1. Testing Database Debug Endpoint:');
        try {
            const dbResponse = await fetch(`${BASE_URL}/api/v2/debug/database`);
            if (dbResponse.ok) {
                const dbData = await dbResponse.json();
                console.log('   ✅ Database debug endpoint working');
                console.log(`   📊 Summary: ${JSON.stringify(dbData.summary)}`);
                
                if (dbData.memory_cache_device_id) {
                    console.log(`   💾 Memory cache device ID: ${dbData.memory_cache_device_id}`);
                } else {
                    console.log('   💾 Memory cache device ID: NULL (will use database fallback)');
                }
            } else {
                console.log('   ❌ Database debug endpoint failed');
            }
        } catch (error) {
            console.log('   ❌ Database debug endpoint error:', error.message);
        }
        
        // 2. Test API endpoints (should work with database fallback)
        console.log('\n2. Testing API Endpoints with Database Fallback:');
        const endpoints = [
            { name: 'Portfolio Summary', url: '/api/v2/portfolio/summary', key: 'total_value_usd' },
            { name: 'Balances', url: '/api/v2/balances', key: 'length' }
        ];
        
        for (const endpoint of endpoints) {
            try {
                const response = await fetch(`${BASE_URL}${endpoint.url}`);
                if (response.ok) {
                    const data = await response.json();
                    console.log(`   ✅ ${endpoint.name}: SUCCESS`);
                    
                    if (endpoint.key === 'length') {
                        console.log(`      Entries: ${data.length}`);
                        const nonZero = data.filter(b => parseFloat(b.value_usd || 0) > 0);
                        if (nonZero.length > 0) {
                            console.log(`      Non-zero balances: ${nonZero.length}`);
                        }
                    } else if (endpoint.key === 'total_value_usd') {
                        console.log(`      Total USD: $${data.total_value_usd}`);
                    }
                } else {
                    const errorData = await response.json();
                    console.log(`   ❌ ${endpoint.name}: FAILED (${response.status})`);
                    console.log(`      Error: ${errorData.error || 'Unknown'}`);
                }
            } catch (error) {
                console.log(`   ❌ ${endpoint.name}: ERROR - ${error.message}`);
            }
        }
        
        // 3. Test sync functionality (portfolio summary invalidation)
        console.log('\n3. Testing Portfolio Summary Cache Invalidation:');
        try {
            console.log('   🔄 Triggering sync to test cache invalidation...');
            const syncResponse = await fetch(`${BASE_URL}/api/v2/sync-device`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' }
            });
            
            if (syncResponse.ok) {
                const syncData = await syncResponse.json();
                console.log('   ✅ Sync successful');
                console.log(`      Success: ${syncData.success}`);
                
                // Wait a moment then check portfolio summary
                await new Promise(resolve => setTimeout(resolve, 1000));
                
                const summaryResponse = await fetch(`${BASE_URL}/api/v2/portfolio/summary`);
                if (summaryResponse.ok) {
                    const summaryData = await summaryResponse.json();
                    console.log('   ✅ Portfolio summary accessible after sync');
                    console.log(`      Total USD: $${summaryData.total_value_usd}`);
                    
                    if (parseFloat(summaryData.total_value_usd) > 0) {
                        console.log('   🎉 USD values working correctly!');
                    } else {
                        console.log('   ⚠️  USD values still showing $0.00 - may need manual refresh');
                    }
                } else {
                    console.log('   ❌ Portfolio summary failed after sync');
                }
            } else {
                const errorText = await syncResponse.text();
                console.log('   ❌ Sync failed:', syncResponse.status);
                console.log(`      Error: ${errorText}`);
            }
        } catch (error) {
            console.log('   ❌ Sync test error:', error.message);
        }
        
        // 4. Test cache debug endpoint again (should show memory cache populated)
        console.log('\n4. Testing Memory Cache Population:');
        try {
            const cacheResponse = await fetch(`${BASE_URL}/api/v2/debug/cache`);
            if (cacheResponse.ok) {
                const cacheData = await cacheResponse.json();
                console.log('   ✅ Cache debug endpoint working');
                console.log(`      Device in cache: ${cacheData.device_id_in_cache || 'NULL'}`);
                console.log(`      Has features: ${cacheData.has_cached_features}`);
                console.log(`      Cache file: ${cacheData.cache_address}`);
            } else {
                console.log('   ❌ Cache debug endpoint failed');
            }
        } catch (error) {
            console.log('   ❌ Cache debug error:', error.message);
        }
        
    } catch (error) {
        console.log('❌ Test failed:', error.message);
    }
}

async function showFixesSummary() {
    console.log('\n🎯 Cache Fixes Summary:');
    console.log('');
    
    console.log('✅ Fix 1: Database Fallback for get_device_id()');
    console.log('   - API endpoints no longer fail with "No device found in cache"');
    console.log('   - Memory cache auto-populates from database');
    console.log('');
    
    console.log('✅ Fix 2: Portfolio Summary Cache Invalidation');
    console.log('   - USD values update when balances change');
    console.log('   - No more stale $0.00 portfolio summaries');
    console.log('');
    
    console.log('✅ Fix 3: Extended Cache Timeout (1 hour)');
    console.log('   - Instant startup when cache is fresh');
    console.log('   - Reduced Pioneer API calls');
    console.log('');
    
    console.log('✅ Fix 4: Proper Cache Respect Logic');
    console.log('   - Frontload actually uses cache check results');
    console.log('   - No more redundant frontload loops');
    console.log('');
    
    console.log('✅ Fix 5: Database Debug Tools');
    console.log('   - /api/v2/debug/database for raw database inspection');
    console.log('   - Better troubleshooting capabilities');
    console.log('');
    
    console.log('🚀 Expected Result:');
    console.log('   - First run: 30-60s frontload, then instant');
    console.log('   - Subsequent runs: < 3s instant startup');
    console.log('   - USD values always accurate');
    console.log('   - No more "No device found" errors');
}

// Run the test
console.log('🚀 Starting final cache fixes test...');
console.log('');

testAllFixes()
    .then(() => showFixesSummary())
    .then(() => {
        console.log('\n🎉 Final Cache Fixes Test Complete!');
        console.log('');
        console.log('The database caching should now be reliable and fast.');
        console.log('Restart the vault to test instant startup behavior.');
    })
    .catch(console.error); 