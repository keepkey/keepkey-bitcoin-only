#!/usr/bin/env node

/**
 * Bitcoin (BTC) Debug Test
 * 
 * This script focuses specifically on Bitcoin to debug why it shows:
 * "Addresses without XPUBs (will break balance queries)" for multiple script types.
 * 
 * Expected Bitcoin Configuration:
 * - Network: bip122:000000000019d6689c085ae165831e93 (Bitcoin genesis hash)
 * - Path: m/44'/0'/0' = [2147483692, 2147483648, 2147483648] (legacy)
 * - Path: m/49'/0'/0' = [2147483697, 2147483648, 2147483648] (segwit)
 * - Path: m/84'/0'/0' = [2147483732, 2147483648, 2147483648] (native segwit)
 * - Coin Type: 0 (SLIP-44)
 * - Coin Name: "Bitcoin" (should be sent to KeepKey device)
 * - Expected CAIP: bip122:000000000019d6689c085ae165831e93/slip44:0
 */

console.log('🚧 Bitcoin (BTC) Debug Test');
console.log('============================');
console.log('');

// Bitcoin specific configuration for all script types
const BTC_CONFIGS = {
    legacy: {
        network: 'bip122:000000000019d6689c085ae165831e93',
        coinType: 0,
        pathHardened: [2147483692, 2147483648, 2147483648], // m/44'/0'/0'
        pathReadable: "m/44'/0'/0'",
        scriptType: 'p2pkh',
        expectedCoinName: 'Bitcoin',
        expectedCAIP: 'bip122:000000000019d6689c085ae165831e93/slip44:0',
        expectedSymbol: 'BTC',
        blockchain: 'bitcoin',
        note: 'Bitcoin account 0 legacy'
    },
    segwit: {
        network: 'bip122:000000000019d6689c085ae165831e93',
        coinType: 0,
        pathHardened: [2147483697, 2147483648, 2147483648], // m/49'/0'/0'
        pathReadable: "m/49'/0'/0'",
        scriptType: 'p2sh-p2wpkh',
        expectedCoinName: 'Bitcoin',
        expectedCAIP: 'bip122:000000000019d6689c085ae165831e93/slip44:0',
        expectedSymbol: 'BTC',
        blockchain: 'bitcoin',
        note: 'Bitcoin account 0 segwit'
    },
    nativeSegwit: {
        network: 'bip122:000000000019d6689c085ae165831e93',
        coinType: 0,
        pathHardened: [2147483732, 2147483648, 2147483648], // m/84'/0'/0'
        pathReadable: "m/84'/0'/0'",
        scriptType: 'p2wpkh',
        expectedCoinName: 'Bitcoin',
        expectedCAIP: 'bip122:000000000019d6689c085ae165831e93/slip44:0',
        expectedSymbol: 'BTC',
        blockchain: 'bitcoin',
        note: 'Bitcoin account 0 Native Segwit'
    }
};

console.log('📋 Bitcoin Configuration:');
Object.entries(BTC_CONFIGS).forEach(([type, config]) => {
    console.log(`   ${type}:`);
    console.log(`     Network: ${config.network}`);
    console.log(`     Path: ${config.pathReadable}`);
    console.log(`     Script Type: ${config.scriptType}`);
    console.log(`     Note: ${config.note}`);
});
console.log('');

// Check if the server is running
async function testServerHealth() {
    try {
        const response = await fetch('http://localhost:1646/health');
        if (response.ok) {
            console.log('✅ Server is running');
            return true;
        } else {
            console.log('❌ Server health check failed');
            return false;
        }
    } catch (e) {
        console.log('❌ Server is not running or not accessible');
        console.log('   Please start the server: cd projects/keepkey-rust/projects/kkcli && cargo run -- server --port 1646');
        return false;
    }
}

// Test paths endpoint for Bitcoin entries
async function testPathsEndpoint() {
    try {
        console.log('🔍 Testing /v2/paths endpoint for Bitcoin...');
        const response = await fetch('http://localhost:1646/v2/paths');
        const data = await response.json();
        
        console.log(`📊 Total paths in database: ${data.length}`);
        
        // Look for Bitcoin specific paths
        const btcPaths = data.filter(path => {
            const hasNetwork = path.networks.includes(BTC_CONFIGS.legacy.network);
            const hasCoinType = path.addressNList && path.addressNList[1] === BTC_CONFIGS.legacy.coinType;
            const hasSymbol = path.symbol === 'BTC';
            const hasBlockchain = path.blockchain === 'bitcoin';
            
            return hasNetwork && (hasCoinType || hasSymbol || hasBlockchain);
        });
        
        console.log(`🎯 Bitcoin paths found: ${btcPaths.length}`);
        
        if (btcPaths.length > 0) {
            btcPaths.forEach((path, index) => {
                console.log(`   📍 BTC Path #${index + 1}:`);
                console.log(`      ID: ${path.id}`);
                console.log(`      Note: ${path.note}`);
                console.log(`      Blockchain: ${path.blockchain}`);
                console.log(`      Symbol: ${path.symbol}`);
                console.log(`      Script Type: ${path.scriptType}`);
                console.log(`      Networks: ${path.networks.join(', ')}`);
                console.log(`      Address List: [${path.addressNList.join(', ')}]`);
                console.log(`      Path (readable): m/${path.addressNList.map(n => (n >= 0x80000000 ? (n - 0x80000000) + "'" : n)).join('/')}`);
                
                // Check which script type this represents
                if (path.addressNList[0] === 2147483692) { // 44'
                    console.log(`      ✅ Legacy (p2pkh) script type detected`);
                } else if (path.addressNList[0] === 2147483697) { // 49'
                    console.log(`      ✅ Segwit (p2sh-p2wpkh) script type detected`);
                } else if (path.addressNList[0] === 2147483732) { // 84'
                    console.log(`      ✅ Native Segwit (p2wpkh) script type detected`);
                } else {
                    console.log(`      ❓ Unknown script type with purpose: ${path.addressNList[0] - 0x80000000}'`);
                }
            });
        } else {
            console.log('   ❌ No Bitcoin paths found!');
        }
        
        return btcPaths.length > 0;
    } catch (e) {
        console.log('❌ Failed to fetch v2/paths endpoint:', e.message);
        return false;
    }
}

// Test pubkeys endpoint for Bitcoin entries
async function testPubkeysEndpoint() {
    try {
        console.log('🔍 Testing /v2/pubkeys endpoint for Bitcoin...');
        const response = await fetch('http://localhost:1646/v2/pubkeys');
        const data = await response.json();
        
        console.log(`📊 Total pubkeys in API: ${data.length}`);
        
        // Look for Bitcoin pubkeys/addresses
        const btcPubkeys = data.filter(pubkey => {
            const hasNetwork = pubkey.networks.includes(BTC_CONFIGS.legacy.network);
            const isBTC = pubkey.note.toLowerCase().includes('bitcoin') && 
                         !pubkey.note.toLowerCase().includes('bitcoin cash') &&
                         !pubkey.note.toLowerCase().includes('bitcoin testnet');
            
            return hasNetwork && isBTC;
        });
        
        console.log(`🎯 Bitcoin pubkeys found: ${btcPubkeys.length}`);
        
        let xpubCount = 0;
        let addressCount = 0;
        const scriptTypes = { p2pkh: 0, 'p2sh-p2wpkh': 0, p2wpkh: 0 };
        
        if (btcPubkeys.length > 0) {
            btcPubkeys.forEach((pubkey, index) => {
                console.log(`   🔑 BTC Pubkey #${index + 1}:`);
                console.log(`      Note: ${pubkey.note}`);
                console.log(`      Type: ${pubkey.type}`);
                console.log(`      Script Type: ${pubkey.scriptType}`);
                console.log(`      Networks: ${pubkey.networks.join(', ')}`);
                console.log(`      Address: ${pubkey.address ? pubkey.address.substring(0, 30) + '...' : 'N/A'}`);
                console.log(`      Pubkey: ${pubkey.pubkey ? pubkey.pubkey.substring(0, 30) + '...' : 'N/A'}`);
                console.log(`      Path: ${pubkey.path}`);
                
                // Count types
                if (['xpub', 'ypub', 'zpub'].includes(pubkey.type)) {
                    xpubCount++;
                    console.log(`      ✅ XPUB detected for balance queries`);
                } else {
                    addressCount++;
                    console.log(`      ⚠️  Address only - no XPUB for balance queries`);
                }
                
                // Count script types
                if (scriptTypes.hasOwnProperty(pubkey.scriptType)) {
                    scriptTypes[pubkey.scriptType]++;
                }
            });
            
            console.log(`\n   📊 Bitcoin Summary:`);
            console.log(`      XPUBs: ${xpubCount}`);
            console.log(`      Addresses only: ${addressCount}`);
            console.log(`      Script types: p2pkh=${scriptTypes.p2pkh}, p2sh-p2wpkh=${scriptTypes['p2sh-p2wpkh']}, p2wpkh=${scriptTypes.p2wpkh}`);
            
            if (addressCount > 0 && xpubCount === 0) {
                console.log(`      ❌ CRITICAL: All Bitcoin entries are addresses without XPUBs!`);
                console.log(`      💡 This will break balance queries - need XPUB generation`);
            }
        } else {
            console.log('   ❌ No Bitcoin pubkeys found!');
        }
        
        return btcPubkeys.length > 0;
    } catch (e) {
        console.log('❌ Failed to fetch v2/pubkeys endpoint:', e.message);
        return false;
    }
}

// Test balances endpoint for Bitcoin entries
async function testBalancesEndpoint() {
    try {
        console.log('🔍 Testing /v2/balances endpoint for Bitcoin...');
        const response = await fetch('http://localhost:1646/v2/balances');
        const data = await response.json();
        
        console.log(`📊 Total balances in API: ${data.length}`);
        
        // Look for Bitcoin balances
        const btcBalances = data.filter(balance => 
            balance.symbol === 'BTC' && balance.caip.includes('slip44:0')
        );
        
        console.log(`🎯 Bitcoin balances found: ${btcBalances.length}`);
        
        if (btcBalances.length > 0) {
            btcBalances.forEach((balance, index) => {
                console.log(`   💰 BTC Balance #${index + 1}:`);
                console.log(`      CAIP: ${balance.caip}`);
                console.log(`      Symbol: ${balance.symbol}`);
                console.log(`      Balance: ${balance.balance}`);
                console.log(`      Price USD: $${balance.priceUsd}`);
                console.log(`      Value USD: $${balance.valueUsd}`);
                console.log(`      Pubkey: ${balance.pubkey ? balance.pubkey.substring(0, 30) + '...' : 'N/A'}`);
                console.log(`      Network ID: ${balance.networkId}`);
                
                // Validate CAIP format
                if (balance.caip === BTC_CONFIGS.legacy.expectedCAIP) {
                    console.log(`      ✅ Correct CAIP format`);
                } else {
                    console.log(`      ❌ Wrong CAIP: ${balance.caip} (expected ${BTC_CONFIGS.legacy.expectedCAIP})`);
                }
            });
        } else {
            console.log('   ❌ No Bitcoin balances found!');
            console.log('   💡 This confirms balance queries are failing due to missing XPUBs');
        }
        
        return btcBalances.length > 0;
    } catch (e) {
        console.log('❌ Failed to fetch v2/balances endpoint:', e.message);
        return false;
    }
}

// Main test function
async function runBitcoinDebugTest() {
    console.log('🧪 Running Bitcoin Debug Test...\n');
    
    // Test 1: Server Health
    console.log('Test 1: Server Health');
    console.log('---------------------');
    const serverOk = await testServerHealth();
    console.log('');
    
    if (!serverOk) {
        console.log('❌ Cannot proceed with tests - server not available');
        process.exit(1);
    }
    
    // Test 2: Paths Configuration
    console.log('Test 2: Paths Configuration');
    console.log('---------------------------');
    const pathsOk = await testPathsEndpoint();
    console.log('');
    
    // Test 3: Pubkeys Generation
    console.log('Test 3: Pubkeys Generation');
    console.log('--------------------------');
    const pubkeysOk = await testPubkeysEndpoint();
    console.log('');
    
    // Test 4: Balance Fetching
    console.log('Test 4: Balance Fetching');
    console.log('------------------------');
    const balancesOk = await testBalancesEndpoint();
    console.log('');
    
    // Summary
    console.log('🎯 Bitcoin Debug Summary');
    console.log('========================');
    console.log(`Server Health: ${serverOk ? '✅' : '❌'}`);
    console.log(`Paths Configuration: ${pathsOk ? '✅' : '❌'}`);
    console.log(`Pubkeys Generation: ${pubkeysOk ? '✅' : '❌'}`);
    console.log(`Balance Fetching: ${balancesOk ? '✅' : '❌'}`);
    console.log('');
    
    // Critical diagnosis
    if (pubkeysOk && !balancesOk) {
        console.log('🚨 CRITICAL ISSUE: Bitcoin has addresses but no balances');
        console.log('   • Root cause: Missing XPUB generation for balance queries');
        console.log('   • Solution: Fix frontload.rs to generate XPUBs for Bitcoin script types');
        console.log('   • Impact: All Bitcoin balance queries are failing');
    }
    
    console.log('\n💡 Bitcoin-specific fixes needed:');
    console.log('   • Ensure frontload generates XPUBs (not just addresses) for all Bitcoin script types');
    console.log('   • Verify coin name "Bitcoin" maps correctly in device communication');
    console.log('   • Check that all three derivation paths (44\', 49\', 84\') generate XPUBs');
    console.log('   • Restart server after fixes to regenerate data');
}

// Run the Bitcoin debug test
runBitcoinDebugTest().catch(console.error); 