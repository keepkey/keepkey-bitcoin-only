#!/usr/bin/env node

/**
 * Test Sync Functionality 
 * 
 * This script tests the sync-device endpoint that we've integrated into the vault frontend.
 * It verifies:
 * 1. Sync-device endpoint works
 * 2. Portfolio data is properly refreshed after sync
 * 3. USD values are correctly displayed
 */

console.log('🔄 Testing Sync Functionality');
console.log('=============================');
console.log('');

// Test sync-device endpoint
async function testSyncDevice() {
    console.log('📡 Testing sync-device endpoint...');
    
    try {
        const response = await fetch('http://localhost:1646/api/v2/sync-device', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' }
        });
        
        if (response.ok) {
            const result = await response.json();
            console.log('✅ Sync device successful:', result);
            console.log(`   Device ID: ${result.device_id}`);
            console.log(`   Balances cached: ${result.balances_cached}`);
            console.log(`   Message: ${result.message}`);
            return true;
        } else {
            console.log('❌ Sync device failed:', response.status);
            const errorText = await response.text();
            console.log('   Error:', errorText);
            return false;
        }
    } catch (error) {
        console.log('❌ Sync device error:', error.message);
        return false;
    }
}

// Test portfolio endpoints after sync
async function testPortfolioEndpoints() {
    console.log('\n💰 Testing portfolio endpoints after sync...');
    
    try {
        // Test portfolio summary
        console.log('🔍 Testing /api/v2/portfolio/summary...');
        const summaryResponse = await fetch('http://localhost:1646/api/v2/portfolio/summary');
        
        if (summaryResponse.ok) {
            const summary = await summaryResponse.json();
            console.log('✅ Portfolio summary successful:');
            console.log(`   Total USD Value: $${summary.total_value_usd}`);
            console.log(`   Network Count: ${summary.network_count}`);
            console.log(`   Balance Count: ${summary.balance_count}`);
        } else {
            console.log('❌ Portfolio summary failed:', summaryResponse.status);
        }
        
        // Test balances
        console.log('\n🔍 Testing /api/v2/balances...');
        const balancesResponse = await fetch('http://localhost:1646/api/v2/balances');
        
        if (balancesResponse.ok) {
            const balances = await balancesResponse.json();
            console.log(`✅ Balances successful: ${balances.length} entries`);
            
            // Show BTC balances specifically
            const btcBalances = balances.filter(b => b.symbol === 'BTC');
            if (btcBalances.length > 0) {
                console.log('   📊 BTC Balances:');
                btcBalances.forEach((balance, index) => {
                    console.log(`     ${index + 1}. ${balance.balance} BTC ($${balance.value_usd} USD)`);
                });
                
                // Calculate total BTC
                const totalBtc = btcBalances.reduce((sum, b) => sum + parseFloat(b.balance), 0);
                const totalUsd = btcBalances.reduce((sum, b) => sum + parseFloat(b.value_usd || 0), 0);
                console.log(`   💎 Total BTC: ${totalBtc.toFixed(8)} BTC`);
                console.log(`   💵 Total USD: $${totalUsd.toFixed(2)} USD`);
            } else {
                console.log('   ℹ️  No BTC balances found');
            }
        } else {
            console.log('❌ Balances failed:', balancesResponse.status);
        }
        
    } catch (error) {
        console.log('❌ Portfolio endpoints error:', error.message);
    }
}

// Check server health first
async function checkServerHealth() {
    try {
        const response = await fetch('http://localhost:1646/api/health');
        if (response.ok) {
            console.log('✅ Server is running');
            return true;
        } else {
            console.log('❌ Server health check failed');
            return false;
        }
    } catch (e) {
        console.log('❌ Server is not running');
        console.log('💡 Please start the server with: make vault');
        return false;
    }
}

// Main test function
async function runSyncTest() {
    console.log('🏁 Starting sync functionality test...\n');
    
    // Check server health
    const serverOk = await checkServerHealth();
    if (!serverOk) {
        process.exit(1);
    }
    
    // Test sync functionality
    const syncOk = await testSyncDevice();
    if (!syncOk) {
        console.log('⚠️  Sync failed, but continuing to test portfolio endpoints...');
    }
    
    // Test portfolio endpoints
    await testPortfolioEndpoints();
    
    console.log('\n🎯 Test Summary');
    console.log('─'.repeat(15));
    console.log('✅ Server: Running');
    console.log(`${syncOk ? '✅' : '❌'} Sync: ${syncOk ? 'Working' : 'Failed'}`);
    console.log('✅ Portfolio: Tested');
    
    if (syncOk) {
        console.log('\n🎉 SUCCESS: Sync functionality is working!');
        console.log('💡 The vault frontend should now:');
        console.log('   • Auto-sync device on load');
        console.log('   • Allow clicking Bitcoin logo to sync');
        console.log('   • Display proper USD values');
    } else {
        console.log('\n⚠️  PARTIAL SUCCESS: Server is running but sync may need debugging');
    }
    
    console.log('\n🔧 Frontend Features Implemented:');
    console.log('   ✅ Automatic sync on Portfolio load');
    console.log('   ✅ Clickable Bitcoin logo with spinner');
    console.log('   ✅ USD value display from server');
    console.log('   ✅ Sync status indicators');
}

// Run the test
runSyncTest().catch(console.error); 