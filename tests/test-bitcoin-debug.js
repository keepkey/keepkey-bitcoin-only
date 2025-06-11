#!/usr/bin/env node

/**
 * Bitcoin XPUB Display Test
 * 
 * This script displays all Bitcoin XPUBs from the KeepKey device with nice labels
 * for different script types (Legacy, SegWit, Native SegWit).
 * 
 * Expected Bitcoin XPUBs:
 * - Legacy (P2PKH): m/44'/0'/0' -> xpub...
 * - SegWit (P2SH-P2WPKH): m/49'/0'/0' -> ypub...
 * - Native SegWit (P2WPKH): m/84'/0'/0' -> zpub...
 */

console.log('🔑 Bitcoin XPUB Display Test');
console.log('============================');
console.log('');

// Bitcoin XPUB configuration with nice labels
const BTC_XPUB_CONFIGS = {
    legacy: {
        label: '🏛️  Legacy (P2PKH)',
        network: 'bip122:000000000019d6689c085ae165831e93',
        path: "m/44'/0'/0'",
        pathArray: [2147483692, 2147483648, 2147483648],
        scriptType: 'p2pkh',
        expectedPrefix: 'xpub',
        description: 'Original Bitcoin address format',
        addressExample: '1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2'
    },
    segwit: {
        label: '🔗 SegWit (P2SH-P2WPKH)',
        network: 'bip122:000000000019d6689c085ae165831e93',
        path: "m/49'/0'/0'",
        pathArray: [2147483697, 2147483648, 2147483648],
        scriptType: 'p2sh-p2wpkh',
        expectedPrefix: 'ypub',
        description: 'SegWit wrapped in P2SH',
        addressExample: '37VucYSaXLCAsxYyAPfbSi9eh4iEcbShgf'
    },
    nativeSegwit: {
        label: '⚡ Native SegWit (P2WPKH)',
        network: 'bip122:000000000019d6689c085ae165831e93',
        path: "m/84'/0'/0'",
        pathArray: [2147483732, 2147483648, 2147483648],
        scriptType: 'p2wpkh',
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
        console.log('💡 Please start the server:');
        console.log('   cd projects/vault && bun run build');
        console.log('   or');
        console.log('   cd projects/kkcli && cargo run -- server --port 1646');
        return false;
    }
}

// Check if devices are connected
async function testDeviceConnection() {
    try {
        console.log('🔍 Checking for connected KeepKey devices...');
        const response = await fetch('http://localhost:1646/api/v2/devices');
        const devices = await response.json();
        
        if (!devices || devices.length === 0) {
            console.log('❌ No KeepKey devices found');
            console.log('💡 Please:');
            console.log('   • Connect your KeepKey device via USB');
            console.log('   • Ensure device is unlocked and ready');
            console.log('   • Wait for device recognition');
            return false;
        }
        
        console.log(`✅ Found ${devices.length} connected device(s)`);
        devices.forEach((device, index) => {
            console.log(`   📱 Device ${index + 1}:`);
            console.log(`      Model: ${device.features?.model || 'Unknown'}`);
            console.log(`      Label: ${device.features?.label || 'No label'}`);
            console.log(`      Firmware: ${device.features?.version || 'Unknown'}`);
            console.log(`      Initialized: ${device.features?.initialized ? '✅' : '❌'}`);
        });
        
        return true;
    } catch (e) {
        console.log('❌ Failed to check device connection:', e.message);
        return false;
    }
}

// Display all Bitcoin XPUBs with nice formatting
async function displayBitcoinXPUBs() {
    try {
        console.log('🔍 Fetching Bitcoin XPUBs from KeepKey...');
        console.log('');
        
        const response = await fetch('http://localhost:1646/api/v2/pubkeys');
        const data = await response.json();
        
        // Filter for Bitcoin XPUBs only
        const btcXpubs = data.filter(item => {
            const isBitcoin = item.networks && item.networks.includes('bip122:000000000019d6689c085ae165831e93');
            const isXpub = item.type && ['xpub', 'ypub', 'zpub'].includes(item.type);
            const isBitcoinNote = item.note && item.note.toLowerCase().includes('bitcoin') && 
                                 !item.note.toLowerCase().includes('bitcoin cash') &&
                                 !item.note.toLowerCase().includes('testnet');
            
            return isBitcoin && isXpub && isBitcoinNote;
        });
        
        console.log(`📊 Found ${btcXpubs.length} Bitcoin XPUBs\n`);
        
        if (btcXpubs.length === 0) {
            console.log('❌ No Bitcoin XPUBs found!');
            console.log('💡 This means:');
            console.log('   • Device may not be connected');
            console.log('   • Frontload process hasn\'t run');
            console.log('   • XPUBs not generated properly');
            return false;
        }
        
        // Group XPUBs by script type for better display
        const xpubsByType = {
            'p2pkh': [],
            'p2sh-p2wpkh': [],
            'p2wpkh': []
        };
        
        btcXpubs.forEach(xpub => {
            if (xpubsByType[xpub.scriptType]) {
                xpubsByType[xpub.scriptType].push(xpub);
            }
        });
        
        // Display each XPUB type with nice formatting
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
                    console.log(`   Type: ${xpub.type}`);
                    console.log(`   Public Key: ${xpub.pubkey || 'N/A'}`);
                    console.log(`   Path: ${xpub.path}`);
                    console.log(`   Note: ${xpub.note}`);
                    
                    // Verify XPUB prefix matches expected
                    if (xpub.pubkey && xpub.pubkey.startsWith(config.expectedPrefix)) {
                        console.log(`   ✅ Correct ${config.expectedPrefix} format`);
                    } else if (xpub.pubkey) {
                        console.log(`   ⚠️  Unexpected prefix: ${xpub.pubkey.substring(0, 4)} (expected ${config.expectedPrefix})`);
                    }
                    
                    // Show truncated pubkey for readability
                    if (xpub.pubkey && xpub.pubkey.length > 50) {
                        console.log(`   📋 XPUB: ${xpub.pubkey.substring(0, 40)}...${xpub.pubkey.substring(-8)}`);
                        console.log(`   📋 Full: ${xpub.pubkey}`);
                    } else {
                        console.log(`   📋 XPUB: ${xpub.pubkey || 'NOT FOUND'}`);
                    }
                });
            } else {
                console.log(`\n❌ No XPUB found for ${config.scriptType}`);
                console.log(`   💡 Expected: ${config.expectedPrefix}...`);
                console.log(`   🔧 Check frontload process for this derivation path`);
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
        console.log(`Legacy (P2PKH): ${xpubsByType['p2pkh'].length}`);
        console.log(`SegWit (P2SH-P2WPKH): ${xpubsByType['p2sh-p2wpkh'].length}`);
        console.log(`Native SegWit (P2WPKH): ${xpubsByType['p2wpkh'].length}`);
        
        if (totalFound === totalExpected) {
            console.log('\n🎉 All Bitcoin XPUBs found successfully!');
            console.log('✅ Ready for balance queries and transactions');
        } else {
            console.log('\n⚠️  Missing Bitcoin XPUBs detected');
            console.log('🔧 Run frontload process to generate missing XPUBs');
        }
        
        return totalFound > 0;
        
    } catch (e) {
        console.log('❌ Failed to fetch Bitcoin XPUBs:', e.message);
        return false;
    }
}

// Test individual XPUB generation via direct API call
async function testXPUBGeneration() {
    console.log('\n🧪 Testing Direct XPUB Generation');
    console.log('─'.repeat(40));
    
    // First check if devices are connected
    try {
        const deviceResponse = await fetch('http://localhost:1646/api/v2/devices');
        const devices = await deviceResponse.json();
        
        if (!devices || devices.length === 0) {
            console.log('❌ No KeepKey devices connected');
            console.log('💡 Please connect your KeepKey device and try again');
            return false;
        }
        
        console.log(`✅ Found ${devices.length} connected device(s)`);
        
        // Also try to generate addresses which should create XPUBs
        console.log('\n🔄 Testing Bitcoin address generation (creates XPUBs)...');
        
        for (const [configKey, config] of Object.entries(BTC_XPUB_CONFIGS)) {
            try {
                console.log(`\n📍 Testing ${config.label}...`);
                
                const response = await fetch('http://localhost:1646/api/v1/addresses/utxo', {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json',
                    },
                    body: JSON.stringify({
                        coin: 'Bitcoin',
                        address_n: config.pathArray,
                        script_type: config.scriptType,
                        show_display: false
                    })
                });
                
                if (response.ok) {
                    const result = await response.json();
                    console.log(`   ✅ Address generated: ${result.address}`);
                    console.log(`   📍 Derivation path: ${config.path}`);
                    console.log(`   🔧 This should create XPUB in database`);
                } else {
                    const errorText = await response.text();
                    console.log(`   ❌ Address generation failed: ${response.status}`);
                    console.log(`   📝 Error: ${errorText}`);
                }
                
            } catch (e) {
                console.log(`   ❌ API call failed: ${e.message}`);
            }
        }
        
        // Wait a moment then re-check for XPUBs
        console.log('\n⏳ Waiting 2 seconds for database update...');
        await new Promise(resolve => setTimeout(resolve, 2000));
        
        console.log('\n🔄 Re-checking for generated XPUBs...');
        return await displayBitcoinXPUBs();
        
    } catch (e) {
        console.log(`❌ Failed to check devices: ${e.message}`);
        return false;
    }
}

// Main test function
async function runBitcoinXPUBTest() {
    console.log('🔑 Starting Bitcoin XPUB Display Test...\n');
    
    // Test 1: Server Health
    console.log('Step 1: Check Server Health');
    console.log('─'.repeat(30));
    const serverOk = await testServerHealth();
    console.log('');
    
    if (!serverOk) {
        console.log('❌ Cannot proceed - server not available');
        console.log('💡 Start the KeepKey server first');
        process.exit(1);
    }
    
    // Test 2: Device Connection
    console.log('Step 2: Check Device Connection');
    console.log('─'.repeat(35));
    const deviceOk = await testDeviceConnection();
    console.log('');
    
    if (!deviceOk) {
        console.log('❌ Cannot proceed - no devices connected');
        console.log('💡 Connect your KeepKey device and try again');
        process.exit(1);
    }
    
    // Test 3: Display All Bitcoin XPUBs
    console.log('Step 3: Display Bitcoin XPUBs');
    console.log('─'.repeat(30));
    const xpubsFound = await displayBitcoinXPUBs();
    
    // Test 4: Generate XPUBs if missing
    if (!xpubsFound) {
        console.log('Step 4: Generate Bitcoin XPUBs');
        console.log('─'.repeat(32));
        const generated = await testXPUBGeneration();
        
        if (generated) {
            console.log('\n🎉 Bitcoin XPUBs successfully generated and retrieved!');
        } else {
            console.log('\n❌ Failed to generate Bitcoin XPUBs');
            console.log('💡 Check device connection and try again');
        }
    }
    
    console.log('\n🎯 Test Complete!');
    console.log('');
    console.log('💡 Next Steps:');
    console.log('   • Use these XPUBs for balance queries');
    console.log('   • Import XPUBs into watch-only wallets');
    console.log('   • Monitor Bitcoin addresses for transactions');
    console.log('');
}

// Run the Bitcoin XPUB test
runBitcoinXPUBTest().catch(console.error); 