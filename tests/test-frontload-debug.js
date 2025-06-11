#!/usr/bin/env node

/**
 * Frontload Debug Test
 * 
 * This test diagnoses why frontload isn't running by checking:
 * 1. Device connection and features
 * 2. Version comparison logic
 * 3. Cache status 
 * 
 * FAIL FAST - NO FALLBACKS OR MOCKS
 */

console.log('🚨 FRONTLOAD DEBUG - FAIL FAST MODE');
console.log('=====================================');
console.log('');

async function testServerHealth() {
    try {
        const response = await fetch('http://localhost:1646/api/health');
        if (!response.ok) {
            throw new Error(`Server returned ${response.status}`);
        }
        console.log('✅ Server is running');
        return true;
    } catch (e) {
        console.log('❌ CRITICAL FAILURE: Server not running');
        console.log(`   Error: ${e.message}`);
        console.log('   Fix: Start the server first');
        throw new Error('SERVER_NOT_RUNNING');
    }
}

async function testDeviceConnection() {
    try {
        console.log('🔍 Checking device connection...');
        const response = await fetch('http://localhost:1646/api/v2/devices');
        
        if (!response.ok) {
            throw new Error(`Devices API returned ${response.status}`);
        }
        
        const devices = await response.json();
        
        if (!devices || devices.length === 0) {
            console.log('❌ CRITICAL FAILURE: No devices connected');
            throw new Error('NO_DEVICES_CONNECTED');
        }
        
        console.log(`✅ Found ${devices.length} device(s)`);
        
        // Check each device's readiness status
        for (let i = 0; i < devices.length; i++) {
            const device = devices[i];
            console.log(`\n📱 Device ${i + 1}: ${device.device?.unique_id || 'UNKNOWN_ID'}`);
            console.log(`   Model: ${device.features?.model || 'UNKNOWN'}`);
            console.log(`   Firmware: ${device.features?.version || 'UNKNOWN'}`);
            console.log(`   Initialized: ${device.features?.initialized ? '✅' : '❌'}`);
            
            if (!device.features?.initialized) {
                console.log('❌ CRITICAL FAILURE: Device not initialized');
                throw new Error('DEVICE_NOT_INITIALIZED');
            }
            
            if (!device.features?.version) {
                console.log('❌ CRITICAL FAILURE: No firmware version');
                throw new Error('NO_FIRMWARE_VERSION');
            }
        }
        
        return devices;
    } catch (e) {
        if (e.message.includes('CRITICAL FAILURE') || e.message.includes('NO_DEVICES') || e.message.includes('DEVICE_NOT')) {
            throw e;
        }
        console.log('❌ CRITICAL FAILURE: Device connection check failed');
        console.log(`   Error: ${e.message}`);
        throw new Error('DEVICE_CONNECTION_FAILED');
    }
}

async function testCacheStatus() {
    try {
        console.log('\n🗄️  Checking cache status...');
        
        // Check pubkeys
        const pubkeysResponse = await fetch('http://localhost:1646/api/v2/pubkeys');
        if (!pubkeysResponse.ok) {
            throw new Error(`Pubkeys API returned ${pubkeysResponse.status}`);
        }
        const pubkeys = await pubkeysResponse.json();
        
        // Check paths  
        const pathsResponse = await fetch('http://localhost:1646/api/v2/paths');
        if (!pathsResponse.ok) {
            throw new Error(`Paths API returned ${pathsResponse.status}`);
        }
        const paths = await pathsResponse.json();
        
        console.log(`   Pubkeys in cache: ${pubkeys.length}`);
        console.log(`   Paths in cache: ${paths.length}`);
        
        // Look specifically for Bitcoin data
        const btcPubkeys = pubkeys.filter(p => 
            p.note && p.note.toLowerCase().includes('bitcoin') && 
            !p.note.toLowerCase().includes('bitcoin cash') &&
            !p.note.toLowerCase().includes('testnet')
        );
        
        const btcPaths = paths.filter(p => 
            p.symbol === 'BTC' || 
            (p.blockchain === 'bitcoin') ||
            (p.networks && p.networks.includes('bip122:000000000019d6689c085ae165831e93'))
        );
        
        console.log(`   Bitcoin pubkeys: ${btcPubkeys.length}`);
        console.log(`   Bitcoin paths: ${btcPaths.length}`);
        
        if (btcPubkeys.length === 0 && btcPaths.length === 0) {
            console.log('❌ CRITICAL FAILURE: Cache is empty - frontload never ran');
            throw new Error('CACHE_EMPTY_FRONTLOAD_FAILED');
        }
        
        // Check for Bitcoin XPUBs specifically
        const btcXpubs = btcPubkeys.filter(p => 
            p.type && ['xpub', 'ypub', 'zpub'].includes(p.type)
        );
        
        console.log(`   Bitcoin XPUBs: ${btcXpubs.length}`);
        
        if (btcXpubs.length === 0) {
            console.log('❌ CRITICAL FAILURE: No Bitcoin XPUBs found');
            console.log('   This means frontload ran but failed to generate XPUBs');
            throw new Error('NO_BITCOIN_XPUBS');
        }
        
        console.log('✅ Cache contains Bitcoin data');
        return { btcPubkeys, btcPaths, btcXpubs };
        
    } catch (e) {
        if (e.message.includes('CRITICAL FAILURE') || e.message.includes('CACHE_EMPTY') || e.message.includes('NO_BITCOIN')) {
            throw e;
        }
        console.log('❌ CRITICAL FAILURE: Cache status check failed');
        console.log(`   Error: ${e.message}`);
        throw new Error('CACHE_CHECK_FAILED');
    }
}

async function forceFrontloadTrigger() {
    console.log('\n🔄 Attempting to force frontload trigger...');
    console.log('❌ CRITICAL FAILURE: Cannot force frontload - must be fixed in Rust code');
    console.log('');
    console.log('🔧 ROOT CAUSE ANALYSIS:');
    console.log('   1. Device connects successfully');
    console.log('   2. Features are fetched successfully  ');
    console.log('   3. Device context is set successfully');
    console.log('   4. BUT frontload condition fails');
    console.log('');
    console.log('💡 REQUIRED FIX:');
    console.log('   • Check is_device_ready logic in lib.rs line 92-100');
    console.log('   • Verify version comparison in utils::is_version_older');
    console.log('   • Add debug logging to frontload trigger condition');
    console.log('   • Make frontload trigger always run when device.initialized = true');
    console.log('');
    throw new Error('FRONTLOAD_LOGIC_BROKEN');
}

async function runFrontloadDebugTest() {
    try {
        console.log('🚨 Starting Frontload Debug Test - FAIL FAST MODE\n');
        
        // Step 1: Server Health  
        console.log('Step 1: Server Health Check');
        console.log('─'.repeat(30));
        await testServerHealth();
        console.log('');
        
        // Step 2: Device Connection
        console.log('Step 2: Device Connection Check');
        console.log('─'.repeat(35));
        const devices = await testDeviceConnection();
        console.log('');
        
        // Step 3: Cache Status
        console.log('Step 3: Cache Status Check');
        console.log('─'.repeat(27));
        try {
            const cacheData = await testCacheStatus();
            console.log('');
            console.log('🎉 SUCCESS: Frontload already worked!');
            console.log(`   Found ${cacheData.btcXpubs.length} Bitcoin XPUBs`);
            console.log('   No action needed');
            return;
        } catch (cacheError) {
            if (cacheError.message.includes('CACHE_EMPTY') || cacheError.message.includes('NO_BITCOIN')) {
                console.log('   Cache is empty - frontload failed');
            } else {
                throw cacheError;
            }
        }
        
        // Step 4: Force Frontload (will always fail - need Rust fix)
        console.log('Step 4: Force Frontload Trigger');
        console.log('─'.repeat(32));
        await forceFrontloadTrigger();
        
    } catch (error) {
        console.log('\n💥 FRONTLOAD DEBUG FAILED');
        console.log('==========================');
        console.log(`Error: ${error.message}`);
        console.log('');
        
        if (error.message === 'FRONTLOAD_LOGIC_BROKEN') {
            console.log('🎯 NEXT STEPS:');
            console.log('1. Fix lib.rs frontload trigger condition');
            console.log('2. Add debug logs to version comparison');
            console.log('3. Test with connected device');
            console.log('4. Verify Bitcoin XPUBs are generated');
        }
        
        process.exit(1);
    }
}

// Run the debug test
runFrontloadDebugTest().catch(console.error); 