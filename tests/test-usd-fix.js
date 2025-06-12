#!/usr/bin/env node

/**
 * USD Value Fix Test
 * 
 * This test verifies that the USD values are correctly displayed in the portfolio summary
 * after fixing the portfolio summary cache invalidation issue.
 */

console.log('💵 Testing USD Value Fix');
console.log('========================');
console.log('');

const BASE_URL = 'http://localhost:1646';

async function testUsdValues() {
    console.log('🔍 Step 1: Check current balance data...');
    
    try {
        // Test balances endpoint first
        const balancesResponse = await fetch(`${BASE_URL}/api/v2/balances`);
        
        if (balancesResponse.ok) {
            const balances = await balancesResponse.json();
            console.log(`✅ Balances endpoint: ${balances.length} entries`);
            
            let totalUsdFromBalances = 0;
            let totalBtc = 0;
            
            balances.forEach((balance, index) => {
                const btcAmount = parseFloat(balance.balance || 0);
                const usdValue = parseFloat(balance.value_usd || 0);
                totalBtc += btcAmount;
                totalUsdFromBalances += usdValue;
                
                if (btcAmount > 0 || usdValue > 0) {
                    console.log(`   💎 Entry ${index + 1}: ${balance.balance} BTC → $${balance.value_usd} USD`);
                }
            });
            
            console.log(`   📊 TOTALS from balances: ${totalBtc.toFixed(8)} BTC → $${totalUsdFromBalances.toFixed(2)} USD`);
            
            // Test portfolio summary
            console.log('\n🔍 Step 2: Check portfolio summary...');
            const summaryResponse = await fetch(`${BASE_URL}/api/v2/portfolio/summary`);
            
            if (summaryResponse.ok) {
                const summary = await summaryResponse.json();
                console.log('✅ Portfolio summary endpoint successful');
                console.log(`   💰 Summary Total USD: $${summary.total_value_usd}`);
                console.log(`   🌐 Networks: ${summary.network_count}`);
                console.log(`   💎 Assets: ${summary.asset_count}`);
                
                // Compare values
                const summaryUsd = parseFloat(summary.total_value_usd || 0);
                const difference = Math.abs(summaryUsd - totalUsdFromBalances);
                
                console.log('\n🧮 USD Value Comparison:');
                console.log(`   Individual Balances Total: $${totalUsdFromBalances.toFixed(2)}`);
                console.log(`   Portfolio Summary Total:   $${summaryUsd.toFixed(2)}`);
                console.log(`   Difference:                $${difference.toFixed(2)}`);
                
                if (difference < 0.01) {
                    console.log('🎉 SUCCESS: USD values match perfectly!');
                    console.log('✅ Portfolio summary cache invalidation is working correctly');
                } else if (summaryUsd === 0 && totalUsdFromBalances > 0) {
                    console.log('❌ ISSUE: Portfolio summary shows $0.00 but balances show real values');
                    console.log('💡 This indicates the portfolio summary cache is still stale');
                } else {
                    console.log('⚠️  WARNING: USD values differ by more than $0.01');
                }
                
                // If summary is stale, try to force refresh by calling sync
                if (summaryUsd === 0 && totalUsdFromBalances > 0) {
                    console.log('\n🔄 Step 3: Attempting to force refresh...');
                    console.log('   Triggering sync to refresh portfolio summary...');
                    
                    try {
                        const syncResponse = await fetch(`${BASE_URL}/api/v2/sync-device`, {
                            method: 'POST',
                            headers: { 'Content-Type': 'application/json' }
                        });
                        
                        if (syncResponse.ok) {
                            const syncResult = await syncResponse.json();
                            console.log('✅ Sync triggered successfully');
                            
                            // Wait a moment and check again
                            console.log('   Waiting 2 seconds for cache to update...');
                            await new Promise(resolve => setTimeout(resolve, 2000));
                            
                            const newSummaryResponse = await fetch(`${BASE_URL}/api/v2/portfolio/summary`);
                            if (newSummaryResponse.ok) {
                                const newSummary = await newSummaryResponse.json();
                                const newSummaryUsd = parseFloat(newSummary.total_value_usd || 0);
                                
                                console.log(`   💰 Updated Summary Total USD: $${newSummary.total_value_usd}`);
                                
                                if (newSummaryUsd > 0) {
                                    console.log('🎉 SUCCESS: Portfolio summary now shows correct USD values!');
                                } else {
                                    console.log('❌ STILL BROKEN: Portfolio summary still shows $0.00');
                                }
                            }
                        } else {
                            console.log('❌ Sync failed:', syncResponse.status);
                        }
                    } catch (syncError) {
                        console.log('❌ Sync error:', syncError.message);
                    }
                }
                
            } else {
                console.log('❌ Portfolio summary failed:', summaryResponse.status);
                const errorText = await summaryResponse.text();
                console.log(`   Error: ${errorText}`);
            }
            
        } else {
            console.log('❌ Balances endpoint failed:', balancesResponse.status);
            const errorText = await balancesResponse.text();
            console.log(`   Error: ${errorText}`);
        }
        
    } catch (error) {
        console.log('❌ Test failed:', error.message);
    }
}

// Run the test
console.log('🚀 Starting USD value test...');
console.log('');

testUsdValues().then(() => {
    console.log('');
    console.log('🎯 Test Complete!');
    console.log('');
    console.log('💡 If USD values are still showing $0.00:');
    console.log('   1. Check that Pioneer API is returning proper USD values');
    console.log('   2. Verify that balances are being saved with correct value_usd');
    console.log('   3. Ensure portfolio summary cache is being cleared on balance updates');
    console.log('   4. Try restarting the vault to force a fresh sync');
}).catch(console.error); 