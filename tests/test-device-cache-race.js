#!/usr/bin/env node

/**
 * Device Cache Race Condition Test
 * 
 * This test attempts to reproduce the race condition where:
 * 1. Frontload completes successfully 
 * 2. Device ready signal sent
 * 3. API endpoints fail with "No device found in cache"
 */

console.log('🏃 Testing Device Cache Race Condition');
console.log('====================================');
console.log('');

const BASE_URL = 'http://localhost:1646';

async function testCacheRaceCondition() {
    console.log('🔍 Testing cache availability immediately after device ready...');
    
    // Test the debug endpoint to see what's in cache
    try {
        console.log('1. Checking debug cache endpoint...');
        const debugResponse = await fetch(`${BASE_URL}/api/v2/debug/cache`);
        
        if (debugResponse.ok) {
            const debugData = await debugResponse.json();
            console.log('✅ Cache debug data:', JSON.stringify(debugData, null, 2));
            
            if (debugData.device_id_in_cache) {
                console.log('🎉 SUCCESS: Device ID found in memory cache!');
                console.log(`   Device ID: ${debugData.device_id_in_cache}`);
                console.log(`   Has features: ${debugData.has_cached_features}`);
                console.log(`   Features device: ${debugData.features_device_id}`);
                
                // If cache has device, test why API endpoints fail
                console.log('\n2. Testing API endpoints with device in cache...');
                await testApiEndpoints();
                
            } else {
                console.log('❌ PROBLEM: No device ID in memory cache!');
                console.log(`   Has features: ${debugData.has_cached_features}`);
                console.log(`   Features device: ${debugData.features_device_id || 'None'}`);
                
                console.log('\n💡 This confirms the race condition:');
                console.log('   - Frontload completed successfully');
                console.log('   - But memory cache device_id is None');
                console.log('   - This means cache was cleared or not properly set');
            }
        } else {
            console.log('❌ Debug endpoint failed:', debugResponse.status);
        }
        
    } catch (error) {
        console.log('❌ Cache race test failed:', error.message);
    }
}

async function testApiEndpoints() {
    const endpoints = [
        { name: 'Portfolio Summary', url: '/api/v2/portfolio/summary' },
        { name: 'Balances', url: '/api/v2/balances' },
    ];
    
    for (const endpoint of endpoints) {
        try {
            const response = await fetch(`${BASE_URL}${endpoint.url}`);
            const data = await response.json();
            
            if (response.ok) {
                console.log(`✅ ${endpoint.name}: SUCCESS`);
                if (endpoint.name === 'Portfolio Summary') {
                    console.log(`   Total USD: $${data.total_value_usd}`);
                    console.log(`   Device ID: ${data.device_id}`);
                } else if (endpoint.name === 'Balances') {
                    console.log(`   Entries: ${data.length}`);
                }
            } else {
                console.log(`❌ ${endpoint.name}: FAILED (${response.status})`);
                console.log(`   Error: ${data.error || 'Unknown error'}`);
                
                if (data.error && data.error.includes('No device found in cache')) {
                    console.log('   🔍 This confirms the memory cache issue!');
                }
            }
        } catch (error) {
            console.log(`❌ ${endpoint.name}: ERROR - ${error.message}`);
        }
    }
}

async function testMemoryCacheTiming() {
    console.log('\n🕐 Testing memory cache timing...');
    
    const intervals = [0, 100, 500, 1000, 2000, 5000]; // Test at different intervals
    
    for (const interval of intervals) {
        if (interval > 0) {
            console.log(`   Waiting ${interval}ms...`);
            await new Promise(resolve => setTimeout(resolve, interval));
        }
        
        try {
            const debugResponse = await fetch(`${BASE_URL}/api/v2/debug/cache`);
            if (debugResponse.ok) {
                const debugData = await debugResponse.json();
                const hasDevice = !!debugData.device_id_in_cache;
                
                console.log(`   T+${interval}ms: Device in cache = ${hasDevice}`);
                
                if (hasDevice) {
                    console.log(`   📍 Device appeared at T+${interval}ms`);
                    break;
                }
            }
        } catch (error) {
            console.log(`   T+${interval}ms: Error - ${error.message}`);
        }
    }
}

// Run all tests
console.log('🚀 Starting cache race condition tests...');
console.log('');

Promise.resolve()
    .then(() => testCacheRaceCondition())
    .then(() => testMemoryCacheTiming())
    .then(() => {
        console.log('');
        console.log('🎯 Cache Race Tests Complete!');
        console.log('');
        console.log('💡 Analysis:');
        console.log('   If device_id_in_cache is null but frontload completed:');
        console.log('   → Memory cache is being cleared after frontload');
        console.log('   → Race condition between cache population and API calls');
        console.log('   → Need to ensure cache persistence through device ready signal');
        console.log('');
        console.log('   If device_id_in_cache exists but API fails:');
        console.log('   → Different DeviceCache instances being used');
        console.log('   → AppState not properly sharing cache reference');
    })
    .catch(console.error); 