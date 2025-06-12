#!/usr/bin/env node

/**
 * Bitcoin XPUB, Balance & Portfolio Test
 * 
 * This script tests and displays:
 * 1. Bitcoin XPUBs from KeepKey device (Legacy, SegWit, Native SegWit)
 * 2. Balance queries for each XPUB via Pioneer API
 * 3. Portfolio summary with real BTC values
 * 
 * Expected Bitcoin XPUBs:
 * - Legacy (P2PKH): m/44'/0'/0' -> xpub...
 * - SegWit (P2SH-P2WPKH): m/49'/0'/0' -> ypub...
 * - Native SegWit (P2WPKH): m/84'/0'/0' -> zpub...
 */

console.log('🔑 Bitcoin XPUB, Balance & Portfolio Test');
console.log('=========================================');
console.log('');

// Bitcoin XPUB configuration with nice labels
const BTC_XPUB_CONFIGS = {
    legacy: {
        label: '🏛️  Legacy (P2PKH)',
        network: 'bip122:000000000019d6689c085ae165831e93',
        path: "m/44'/0'/0'",
        pathArray: [2147483692, 2147483648, 2147483648],
        scriptType: 'p2pkh_xpub',  // Use correct script type
        expectedPrefix: 'xpub',
        description: 'Original Bitcoin address format',
        addressExample: '1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2'
    },
    segwit: {
        label: '🔗 SegWit (P2SH-P2WPKH)',
        network: 'bip122:000000000019d6689c085ae165831e93',
        path: "m/49'/0'/0'",
        pathArray: [2147483697, 2147483648, 2147483648],
        scriptType: 'p2sh-p2wpkh_xpub',  // Use correct script type
        expectedPrefix: 'ypub',
        description: 'SegWit wrapped in P2SH',
        addressExample: '37VucYSaXLCAsxYyAPfbSi9eh4iEcbShgf'
    },
    nativeSegwit: {
        label: '⚡ Native SegWit (P2WPKH)',
        network: 'bip122:000000000019d6689c085ae165831e93',
        path: "m/84'/0'/0'",
        pathArray: [2147483732, 2147483648, 2147483648],
        scriptType: 'p2wpkh_xpub',  // Use correct script type
        expectedPrefix: 'zpub',
        description: 'Native SegWit (Bech32)',
        addressExample: 'bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4'
    }
};

// Check if the server is running
async function testServerHealth() {
    try {
        const response = await fetch('http://localhost:1646/api/health');
        if (response.ok) {
            console.log('✅ KeepKey Server is running');
            return true;
        } else {
            console.log('❌ Server health check failed');
            return false;
        }
    } catch (e) {
        console.log('❌ Server is not running or not accessible');
        console.log('💡 Please start the server with: make vault');
        return false;
    }
}

// Check if device is working by testing pubkeys endpoint
async function ensureDeviceIsWorking() {
    try {
        console.log('🔍 Checking if device is working via pubkeys...');
        
        // Test pubkeys endpoint directly since that's what matters
        const pubkeysResponse = await fetch('http://localhost:1646/api/v2/pubkeys');
        
        if (pubkeysResponse.ok) {
            const pubkeys = await pubkeysResponse.json();
            console.log(`✅ Device is working - found ${pubkeys.length} pubkeys`);
            
            // Try to sync device to cache for balance/portfolio endpoints
            console.log('🔄 Attempting to sync device for balance endpoints...');
            try {
                const syncResponse = await fetch('http://localhost:1646/api/v2/sync-device', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' }
                });
                
                if (syncResponse.ok) {
                    const syncData = await syncResponse.json();
                    console.log(`✅ Device synced: ${syncData.device_id}`);
                    console.log(`   Balances cached: ${syncData.balances_cached}`);
                } else {
                    console.log('⚠️  Sync failed, but device is working - continuing with pubkey tests');
                }
            } catch (e) {
                console.log('⚠️  Sync failed, but device is working - continuing with pubkey tests');
            }
            
            return true;
        } else {
            console.log('❌ Pubkeys endpoint failed');
            return false;
        }
    } catch (e) {
        console.log('❌ Failed to check device via pubkeys:', e.message);
        return false;
    }
}

// Display all Bitcoin XPUBs with nice formatting using correct v2 API
async function displayBitcoinXPUBs() {
    try {
        console.log('🔍 Fetching Bitcoin XPUBs from vault...');
        console.log('');
        
        // Use correct v2 API endpoint
        const response = await fetch('http://localhost:1646/api/v2/pubkeys');
        const data = await response.json();
        
        console.log(`📊 Raw pubkeys response: ${data.length} entries`);
        
        // Filter for Bitcoin XPUBs using correct field names
        const btcXpubs = data.filter(item => {
            const isBitcoin = item.context && item.context.includes('bip122:000000000019d6689c085ae165831e93');
            const isXpub = item.scriptType && item.scriptType.includes('_xpub');
            const hasValidAddress = item.address && (
                item.address.startsWith('xpub') || 
                item.address.startsWith('ypub') || 
                item.address.startsWith('zpub')
            );
            
            return isBitcoin && isXpub && hasValidAddress;
        });
        
        console.log(`📊 Found ${btcXpubs.length} Bitcoin XPUBs\n`);
        
        if (btcXpubs.length === 0) {
            console.log('❌ No Bitcoin XPUBs found!');
            console.log('💡 This means:');
            console.log('   • Device may not be frontloaded');
            console.log('   • Run frontload process first');
            console.log('   • XPUBs not generated properly');
            return { xpubs: [], summary: null };
        }
        
        // Group XPUBs by script type for better display
        const xpubsByType = {
            'p2pkh_xpub': [],
            'p2sh-p2wpkh_xpub': [],
            'p2wpkh_xpub': []
        };
        
        btcXpubs.forEach(xpub => {
            if (xpubsByType[xpub.scriptType]) {
                xpubsByType[xpub.scriptType].push(xpub);
            }
        });
        
        // Display each XPUB type with nice formatting
        const foundXpubs = [];
        Object.entries(BTC_XPUB_CONFIGS).forEach(([configKey, config]) => {
            const xpubs = xpubsByType[config.scriptType] || [];
            
            console.log(`${config.label}`);
            console.log('─'.repeat(50));
            console.log(`📍 Derivation Path: ${config.path}`);
            console.log(`📝 Description: ${config.description}`);
            console.log(`🏷️  Address Example: ${config.addressExample}`);
            
            if (xpubs.length > 0) {
                xpubs.forEach((xpub, index) => {
                    console.log(`\n🔑 XPUB #${index + 1}:`);
                    console.log(`   Type: ${xpub.key_type || xpub.type || 'Unknown'}`);
                    console.log(`   Script Type: ${xpub.scriptType}`);
                    console.log(`   Path: ${xpub.path}`);
                    console.log(`   Note: ${xpub.note}`);
                    
                    // Verify XPUB prefix matches expected
                    if (xpub.address && xpub.address.startsWith(config.expectedPrefix)) {
                        console.log(`   ✅ Correct ${config.expectedPrefix} format`);
                    } else if (xpub.address) {
                        console.log(`   ⚠️  Unexpected prefix: ${xpub.address.substring(0, 4)} (expected ${config.expectedPrefix})`);
                    }
                    
                    // Show truncated XPUB for readability
                    if (xpub.address && xpub.address.length > 50) {
                        console.log(`   📋 XPUB: ${xpub.address.substring(0, 40)}...${xpub.address.substring(-8)}`);
                    } else {
                        console.log(`   📋 XPUB: ${xpub.address || 'NOT FOUND'}`);
                    }
                    
                    // Store for balance testing
                    foundXpubs.push({
                        config: config,
                        xpub: xpub.address,
                        type: config.expectedPrefix,
                        scriptType: config.scriptType
                    });
                });
            } else {
                console.log(`\n❌ No XPUB found for ${config.scriptType}`);
                console.log(`   💡 Expected: ${config.expectedPrefix}...`);
                console.log(`   🔧 Run frontload process for this derivation path`);
            }
            
            console.log('\n');
        });
        
        // Summary statistics
        const totalExpected = Object.keys(BTC_XPUB_CONFIGS).length;
        const totalFound = Object.values(xpubsByType).reduce((sum, arr) => sum + arr.length, 0);
        
        console.log('📈 Bitcoin XPUB Summary');
        console.log('─'.repeat(30));
        console.log(`Total Expected: ${totalExpected}`);
        console.log(`Total Found: ${totalFound}`);
        console.log(`Legacy (P2PKH): ${xpubsByType['p2pkh_xpub'].length}`);
        console.log(`SegWit (P2SH-P2WPKH): ${xpubsByType['p2sh-p2wpkh_xpub'].length}`);
        console.log(`Native SegWit (P2WPKH): ${xpubsByType['p2wpkh_xpub'].length}`);
        
        const summary = {
            totalExpected,
            totalFound,
            legacy: xpubsByType['p2pkh_xpub'].length,
            segwit: xpubsByType['p2sh-p2wpkh_xpub'].length,
            nativeSegwit: xpubsByType['p2wpkh_xpub'].length,
            allFound: totalFound === totalExpected
        };
        
        if (summary.allFound) {
            console.log('\n🎉 All Bitcoin XPUBs found successfully!');
            console.log('✅ Ready for balance queries and transactions');
        } else {
            console.log('\n⚠️  Missing Bitcoin XPUBs detected');
            console.log('🔧 Run frontload process to generate missing XPUBs');
        }
        
        return { xpubs: foundXpubs, summary };
        
    } catch (e) {
        console.log('❌ Failed to fetch Bitcoin XPUBs:', e.message);
        return { xpubs: [], summary: null };
    }
}

// Test balance queries for each XPUB
async function testBalanceQueries(foundXpubs) {
    if (foundXpubs.length === 0) {
        console.log('⚠️  No XPUBs available for balance testing');
        return { balances: [], totalValue: 0 };
    }
    
    console.log('💰 Testing Balance Queries');
    console.log('─'.repeat(27));
    console.log(`Checking balances for ${foundXpubs.length} XPUBs via Pioneer API...\n`);
    
    const balanceResults = [];
    let totalValueUsd = 0;
    
    for (const xpubData of foundXpubs) {
        try {
            console.log(`📊 ${xpubData.config.label}`);
            console.log(`🔑 ${xpubData.type}: ${xpubData.xpub.substring(0, 30)}...`);
            
            // Query balance via Pioneer API (localhost:9001)
            const response = await fetch(`http://localhost:9001/api/v1/listUnspent/BTC/${xpubData.xpub}`, {
                headers: { 'accept': 'application/json' }
            });
            
            if (response.ok) {
                const utxos = await response.json();
                
                // Calculate total balance from UTXOs
                const totalSats = utxos.reduce((sum, utxo) => sum + parseInt(utxo.value), 0);
                const btcAmount = totalSats / 100000000; // Convert to BTC
                
                console.log(`   💎 UTXOs: ${utxos.length}`);
                console.log(`   💰 Balance: ${totalSats.toLocaleString()} sats (${btcAmount.toFixed(8)} BTC)`);
                
                if (utxos.length > 0) {
                    console.log(`   📍 First UTXO: ${utxos[0].txid.substring(0, 16)}...`);
                    console.log(`   🔗 Confirmations: ${utxos[0].confirmations || 'Unknown'}`);
                    
                    // Get USD value (approximate at $43k/BTC)
                    const usdValue = btcAmount * 43000;
                    totalValueUsd += usdValue;
                    console.log(`   💵 ~$${usdValue.toFixed(2)} USD`);
                }
                
                balanceResults.push({
                    type: xpubData.type,
                    xpub: xpubData.xpub,
                    utxos: utxos.length,
                    balanceSats: totalSats,
                    balanceBtc: btcAmount,
                    usdValue: btcAmount * 43000
                });
                
                console.log('   ✅ Balance query successful\n');
            } else {
                console.log(`   ❌ Balance query failed: ${response.status}`);
                console.log(`   📝 Response: ${await response.text()}\n`);
                
                balanceResults.push({
                    type: xpubData.type,
                    xpub: xpubData.xpub,
                    utxos: 0,
                    balanceSats: 0,
                    balanceBtc: 0,
                    usdValue: 0,
                    error: `HTTP ${response.status}`
                });
            }
            
        } catch (e) {
            console.log(`   ❌ API call failed: ${e.message}\n`);
            
            balanceResults.push({
                type: xpubData.type,
                xpub: xpubData.xpub,
                utxos: 0,
                balanceSats: 0,
                balanceBtc: 0,
                usdValue: 0,
                error: e.message
            });
        }
    }
    
    return { balances: balanceResults, totalValue: totalValueUsd };
}

// Test portfolio endpoints
async function testPortfolioEndpoints() {
    console.log('📊 Testing Portfolio Endpoints');
    console.log('─'.repeat(31));
    
    try {
        // Test balance endpoint
        console.log('🔍 Testing /api/v2/balances endpoint...');
        const balanceResponse = await fetch('http://localhost:1646/api/v2/balances');
        
        if (balanceResponse.ok) {
            const balances = await balanceResponse.json();
            console.log(`✅ Balances endpoint: ${balances.length} entries`);
            
            balances.forEach((balance, index) => {
                if (index < 3) { // Show first 3
                    console.log(`   💎 ${balance.symbol || 'BTC'}: ${balance.balance} (${balance.value_usd} USD)`);
                }
            });
            
            if (balances.length > 3) {
                console.log(`   ... and ${balances.length - 3} more`);
            }
        } else {
            const errorText = await balanceResponse.text();
            console.log(`❌ Balances endpoint failed: ${balanceResponse.status}`);
            console.log(`   Error: ${errorText}`);
        }
        
        // Test portfolio summary endpoint
        console.log('\n🔍 Testing /api/v2/portfolio/summary endpoint...');
        const portfolioResponse = await fetch('http://localhost:1646/api/v2/portfolio/summary');
        
        if (portfolioResponse.ok) {
            const portfolio = await portfolioResponse.json();
            console.log('✅ Portfolio summary endpoint successful');
            console.log(`   💰 Total Value: $${portfolio.total_value_usd} USD`);
            console.log(`   🌐 Networks: ${portfolio.network_count}`);
            console.log(`   💎 Assets: ${portfolio.asset_count}`);
            console.log(`   📅 Last Updated: ${new Date(portfolio.last_updated * 1000).toLocaleString()}`);
        } else {
            const errorText = await portfolioResponse.text();
            console.log(`❌ Portfolio summary failed: ${portfolioResponse.status}`);
            console.log(`   Error: ${errorText}`);
        }
        
    } catch (e) {
        console.log(`❌ Portfolio endpoint testing failed: ${e.message}`);
    }
}

// Main test function
async function runBitcoinTest() {
    console.log('🔑 Starting Bitcoin XPUB, Balance & Portfolio Test...\n');
    
    // Test 1: Server Health
    console.log('Step 1: Check Server Health');
    console.log('─'.repeat(30));
    const serverOk = await testServerHealth();
    console.log('');
    
    if (!serverOk) {
        console.log('❌ Cannot proceed - server not available');
        console.log('💡 Start the KeepKey server with: make vault');
        process.exit(1);
    }
    
    // Test 2: Ensure Device is Working
    console.log('Step 2: Ensure Device is Working');
    console.log('─'.repeat(35));
    const deviceOk = await ensureDeviceIsWorking();
    console.log('');
    
    if (!deviceOk) {
        console.log('❌ Cannot proceed - device not working');
        console.log('💡 Connect your KeepKey device and try again');
        process.exit(1);
    }
    
    // Test 3: Display All Bitcoin XPUBs
    console.log('Step 3: Display Bitcoin XPUBs');
    console.log('─'.repeat(30));
    const { xpubs: foundXpubs, summary } = await displayBitcoinXPUBs();
    console.log('');
    
    // Test 4: Balance Queries
    if (foundXpubs.length > 0) {
        console.log('Step 4: Test Balance Queries');
        console.log('─'.repeat(29));
        const { balances, totalValue } = await testBalanceQueries(foundXpubs);
        console.log('');
        
        // Test 5: Portfolio Endpoints
        console.log('Step 5: Test Portfolio Endpoints');
        console.log('─'.repeat(33));
        await testPortfolioEndpoints();
        console.log('');
        
        // Final Summary
        console.log('🎯 Test Summary');
        console.log('─'.repeat(15));
        console.log(`✅ XPUBs Found: ${foundXpubs.length}/3`);
        console.log(`✅ Balances Checked: ${balances.length}`);
        console.log(`💰 Total Portfolio Value: ~$${totalValue.toFixed(2)} USD`);
        
        const hasBalance = balances.some(b => b.balanceSats > 0);
        if (hasBalance) {
            console.log('🎉 SUCCESS: Found real Bitcoin balances!');
        } else {
            console.log('ℹ️  No Bitcoin balances found (addresses may be empty)');
        }
    } else {
        console.log('⚠️  No XPUBs found - cannot test balances');
        console.log('💡 Run frontload process first');
    }
    
    console.log('\n🎯 Test Complete!');
    console.log('');
    console.log('💡 Next Steps:');
    console.log('   • Use XPUBs for watch-only wallet imports');
    console.log('   • Monitor addresses for incoming transactions');
    console.log('   • Use vault Send page for outgoing transactions');
    console.log('');
}

// Run the comprehensive Bitcoin test
runBitcoinTest().catch(console.error); 