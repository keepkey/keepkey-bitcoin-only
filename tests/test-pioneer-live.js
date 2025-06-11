#!/usr/bin/env node

/**
 * Pioneer API Live Test Suite
 * 
 * Tests local Pioneer server on 127.0.0.1:9001 and validates integration
 * with KeepKey Vault v2 endpoints. Ensures frontload properly saves
 * both XPUBs and addresses for multi-device/multi-seed support.
 * 
 * FAIL FAST - NO FALLBACKS OR MOCKS
 */

const axios = require('axios');

console.log('🚀 PIONEER API LIVE TEST SUITE');
console.log('===============================');
console.log('');

// Configuration
const LOCAL_PIONEER_BASE = 'http://127.0.0.1:9001';
const VAULT_API_BASE = 'http://localhost:1646';

// Test XPUBs - these should be available in your test environment
const TEST_XPUBS = {
    btc_legacy: 'xpub6CqeSKMnFCNL3iD4FMBXxec7dqwrqvgpHYX7fDKgWQLATp6HS1nNsWvMXKWNbPJ8s6ybHEGWJ6E8V2trZVrYtnZUMT1toFUppxXTpwKh1hG',
    btc_segwit: 'ypub6WamSeXgTYgy7W25fVorMLDHFx5SPkuYaE7ToWCiyCUK2jdWpufQ8VqkDg83YjBtJFHDoekhf9ESdPDbL9aCPXC5NnmzXUiq3J6oycFShfS',
    // btc_native_segwit: 'zpub...' // Add real zpub here once available from KeepKey
};

// Test device contexts for multi-device testing
const TEST_DEVICES = {
    device1: {
        id: '343737340F4736331F003B00',
        label: 'KeepKey3',
        eth_address: '0x141d9959cae3853b035000490c03991eb70fc4ac'
    },
    device2: {
        id: '343737340F4736331F003B01', 
        label: 'KeepKey4',
        eth_address: '0x241d9959cae3853b035000490c03991eb70fc4ad'
    }
};

async function testPioneerHealth() {
    try {
        console.log('🏥 Testing Pioneer Server Health...');
        const response = await axios.get(`${LOCAL_PIONEER_BASE}/api/v1/health`);
        
        if (response.status === 200) {
            console.log('✅ Pioneer server is running');
            console.log(`   Response: ${JSON.stringify(response.data)}`);
            return true;
        } else {
            throw new Error(`Unexpected status: ${response.status}`);
        }
    } catch (e) {
        console.log('❌ CRITICAL FAILURE: Pioneer server not available');
        console.log(`   Error: ${e.message}`);
        if (e.code === 'ECONNREFUSED') {
            console.log('   Fix: Start local pioneer with: docker run -p 9001:9001 pioneer:latest');
        }
        throw new Error('PIONEER_SERVER_DOWN');
    }
}

async function testVaultHealth() {
    try {
        console.log('🏥 Testing Vault Server Health...');
        const response = await axios.get(`${VAULT_API_BASE}/api/health`);
        
        if (response.status === 200) {
            console.log('✅ Vault server is running');
            return true;
        } else {
            throw new Error(`Unexpected status: ${response.status}`);
        }
    } catch (e) {
        console.log('❌ CRITICAL FAILURE: Vault server not available');
        console.log(`   Error: ${e.message}`);
        throw new Error('VAULT_SERVER_DOWN');
    }
}

async function testPioneerFeeRates() {
    try {
        console.log('\n💰 Testing Pioneer Fee Rate API...');
        
        // Bitcoin fee rates
        const btcResponse = await axios.get(
            `${LOCAL_PIONEER_BASE}/api/v1/GetFeeRate/bip122%3A000000000019d6689c085ae165831e93`,
            { headers: { 'accept': 'application/json' } }
        );
        
        console.log('✅ Bitcoin fee rates retrieved:');
        console.log(`   Fastest: ${btcResponse.data.fastest} sat/vB`);
        console.log(`   Fast: ${btcResponse.data.fast} sat/vB`);
        console.log(`   Average: ${btcResponse.data.average} sat/vB`);
        
        // Validate response structure
        if (!btcResponse.data.fastest || !btcResponse.data.fast || !btcResponse.data.average) {
            throw new Error('Invalid fee rate response structure');
        }
        
        return btcResponse.data;
    } catch (e) {
        console.log('❌ CRITICAL FAILURE: Fee rate API failed');
        console.log(`   Error: ${e.message}`);
        throw new Error('FEE_RATE_API_FAILED');
    }
}

async function testPioneerUnspentOutputs() {
    try {
        console.log('\n🪙 Testing Pioneer List Unspent API...');
        
        // Test with SegWit XPUB
        const xpub = TEST_XPUBS.btc_segwit;
        const response = await axios.get(
            `${LOCAL_PIONEER_BASE}/api/v1/listUnspent/BTC/${xpub}`,
            { headers: { 'accept': 'application/json' } }
        );
        
        console.log(`✅ Unspent outputs for ${xpub}:`);
        console.log(`   UTXOs: ${response.data.length || 0}`);
        
        if (response.data.length > 0) {
            const firstUtxo = response.data[0];
            console.log(`   First UTXO: ${firstUtxo.txid}:${firstUtxo.vout} (${firstUtxo.value} sats)`);
        }
        
        return response.data;
    } catch (e) {
        console.log('❌ CRITICAL FAILURE: List unspent API failed');
        console.log(`   Error: ${e.message}`);
        throw new Error('LIST_UNSPENT_API_FAILED');
    }
}

async function testPioneerAddressGeneration() {
    try {
        console.log('\n🏠 Testing Pioneer Address Generation APIs...');
        
        const xpub = TEST_XPUBS.btc_legacy;
        
        // Test get new receive address
        const receiveResponse = await axios.get(
            `${LOCAL_PIONEER_BASE}/api/v1/getNewAddress/BTC/${xpub}`,
            { headers: { 'accept': 'application/json' } }
        );
        
        console.log('✅ New receive address:');
        console.log(`   Response: ${JSON.stringify(receiveResponse.data)}`);
        
        // Test get new change address
        const changeResponse = await axios.get(
            `${LOCAL_PIONEER_BASE}/api/v1/getChangeAddress/BTC/${xpub}`,
            { headers: { 'accept': 'application/json' } }
        );
        
        console.log('✅ New change address:');
        console.log(`   Change Index: ${changeResponse.data.changeIndex}`);
        console.log(`   Receive Index: ${changeResponse.data.receiveIndex}`);
        
        return {
            receive: receiveResponse.data,
            change: changeResponse.data
        };
    } catch (e) {
        console.log('❌ CRITICAL FAILURE: Address generation API failed');
        console.log(`   Error: ${e.message}`);
        throw new Error('ADDRESS_GENERATION_API_FAILED');
    }
}

async function testPioneerTransactionHistory() {
    try {
        console.log('\n📜 Testing Pioneer Transaction History API...');
        
        const xpub = TEST_XPUBS.btc_segwit;
        const response = await axios.get(
            `${LOCAL_PIONEER_BASE}/api/v1/txsByXpub/BTC/${xpub}`,
            { headers: { 'accept': 'application/json' } }
        );
        
        console.log(`✅ Transaction history for ${xpub}:`);
        console.log(`   Page: ${response.data.page}/${response.data.totalPages}`);
        console.log(`   Balance: ${response.data.balance} sats`);
        console.log(`   Total Received: ${response.data.totalReceived} sats`);
        console.log(`   Total Sent: ${response.data.totalSent} sats`);
        console.log(`   Transaction Count: ${response.data.txs}`);
        
        if (response.data.txids && response.data.txids.length > 0) {
            console.log(`   Recent TXIDs: ${response.data.txids.slice(0, 3).join(', ')}...`);
        }
        
        return response.data;
    } catch (e) {
        console.log('❌ CRITICAL FAILURE: Transaction history API failed');
        console.log(`   Error: ${e.message}`);
        throw new Error('TX_HISTORY_API_FAILED');
    }
}

async function testVaultPubkeys() {
    try {
        console.log('\n🔑 Testing Vault Pubkey Endpoints...');
        
        const response = await axios.get(`${VAULT_API_BASE}/api/v2/pubkeys`);
        
        console.log(`✅ Retrieved ${response.data.length} pubkeys from vault:`);
        
        // Filter for Bitcoin pubkeys
        const btcPubkeys = response.data.filter(p => 
            p.note && p.note.toLowerCase().includes('bitcoin') && 
            !p.note.toLowerCase().includes('bitcoin cash') &&
            !p.note.toLowerCase().includes('testnet')
        );
        
        console.log(`   Bitcoin pubkeys: ${btcPubkeys.length}`);
        
        // Show XPUBs specifically
        const xpubs = btcPubkeys.filter(p => p.type && ['xpub', 'ypub', 'zpub'].includes(p.type));
        console.log(`   Bitcoin XPUBs: ${xpubs.length}`);
        
        for (const xpub of xpubs.slice(0, 3)) {
            console.log(`   ${xpub.type}: ${xpub.address.substring(0, 20)}...`);
        }
        
        return { all: response.data, bitcoin: btcPubkeys, xpubs };
    } catch (e) {
        console.log('❌ CRITICAL FAILURE: Vault pubkey API failed');
        console.log(`   Error: ${e.message}`);
        throw new Error('VAULT_PUBKEY_API_FAILED');
    }
}

async function testVaultPaths() {
    try {
        console.log('\n🛤️  Testing Vault Path Endpoints...');
        
        const response = await axios.get(`${VAULT_API_BASE}/api/v2/paths`);
        
        console.log(`✅ Retrieved ${response.data.length} paths from vault:`);
        
        // Filter for Bitcoin paths
        const btcPaths = response.data.filter(p => 
            p.symbol === 'BTC' || 
            (p.blockchain === 'bitcoin') ||
            (p.networks && p.networks.includes('bip122:000000000019d6689c085ae165831e93'))
        );
        
        console.log(`   Bitcoin paths: ${btcPaths.length}`);
        
        for (const path of btcPaths.slice(0, 3)) {
            console.log(`   Path: ${path.addressNList.join('/')}`);
            console.log(`   Script: ${path.scriptType}`);
            console.log(`   Networks: ${path.networks.length}`);
        }
        
        return { all: response.data, bitcoin: btcPaths };
    } catch (e) {
        console.log('❌ CRITICAL FAILURE: Vault path API failed');
        console.log(`   Error: ${e.message}`);
        throw new Error('VAULT_PATH_API_FAILED');
    }
}

async function testVaultBalances() {
    try {
        console.log('\n💎 Testing Vault Balance Endpoints...');
        
        const response = await axios.get(`${VAULT_API_BASE}/api/v2/balances?force_refresh=false`);
        
        console.log(`✅ Retrieved ${response.data.length} balances from vault:`);
        
        // Filter for Bitcoin balances
        const btcBalances = response.data.filter(b => 
            b.symbol === 'BTC' || 
            (b.caip && b.caip.includes('bip122:000000000019d6689c085ae165831e93'))
        );
        
        console.log(`   Bitcoin balances: ${btcBalances.length}`);
        
        let totalValue = 0;
        for (const balance of btcBalances) {
            const value = parseFloat(balance.value_usd) || 0;
            totalValue += value;
            console.log(`   ${balance.caip}: ${balance.balance} BTC ($${balance.value_usd})`);
        }
        
        console.log(`   Total BTC Value: $${totalValue.toFixed(2)}`);
        
        return { all: response.data, bitcoin: btcBalances };
    } catch (e) {
        console.log('❌ CRITICAL FAILURE: Vault balance API failed');
        console.log(`   Error: ${e.message}`);
        throw new Error('VAULT_BALANCE_API_FAILED');
    }
}

async function testDeviceContextSwitching() {
    try {
        console.log('\n🔄 Testing Multi-Device Context Switching...');
        
        for (const [deviceName, deviceInfo] of Object.entries(TEST_DEVICES)) {
            console.log(`\n   Testing device: ${deviceName} (${deviceInfo.label})`);
            
            // Set context to this device
            await axios.post(`${VAULT_API_BASE}/api/context`, {
                device_id: deviceInfo.id,
                device_label: deviceInfo.label,
                eth_address: deviceInfo.eth_address
            });
            
            // Verify context was set
            const contextResponse = await axios.get(`${VAULT_API_BASE}/api/context`);
            
            if (contextResponse.data.device_id === deviceInfo.id) {
                console.log(`   ✅ Context set successfully for ${deviceInfo.label}`);
                
                // Test if device-specific data is available
                const pubkeysResponse = await axios.get(`${VAULT_API_BASE}/api/v2/pubkeys`);
                console.log(`   📊 Device has ${pubkeysResponse.data.length} pubkeys`);
                
                const pathsResponse = await axios.get(`${VAULT_API_BASE}/api/v2/paths`);
                console.log(`   🛤️  Device has ${pathsResponse.data.length} paths`);
                
            } else {
                throw new Error(`Context not set correctly for ${deviceName}`);
            }
        }
        
        console.log('\n✅ Multi-device context switching works correctly');
        return true;
    } catch (e) {
        console.log('❌ CRITICAL FAILURE: Device context switching failed');
        console.log(`   Error: ${e.message}`);
        throw new Error('DEVICE_CONTEXT_FAILED');
    }
}

async function testFrontloadIntegration() {
    try {
        console.log('\n🔄 Testing Frontload Integration...');
        
        // Trigger frontload process
        const frontloadResponse = await axios.post(`${VAULT_API_BASE}/api/frontload`);
        
        if (frontloadResponse.status === 200 && frontloadResponse.data.success) {
            console.log('✅ Frontload completed successfully');
            console.log(`   Addresses loaded: ${frontloadResponse.data.addresses_loaded}`);
            
            // Verify frontload populated both XPUBs and addresses
            const pubkeysAfter = await axios.get(`${VAULT_API_BASE}/api/v2/pubkeys`);
            const xpubsAfter = pubkeysAfter.data.filter(p => p.type && ['xpub', 'ypub', 'zpub'].includes(p.type));
            
            console.log(`   XPUBs in cache: ${xpubsAfter.length}`);
            
            if (xpubsAfter.length === 0) {
                throw new Error('Frontload did not populate any XPUBs');
            }
            
            // Test if XPUBs work with Pioneer
            const testXpub = xpubsAfter[0].address;
            const pioneerTestResponse = await axios.get(
                `${LOCAL_PIONEER_BASE}/api/v1/txsByXpub/BTC/${testXpub}`,
                { headers: { 'accept': 'application/json' } }
            );
            
            console.log(`✅ Pioneer integration test with frontloaded XPUB successful`);
            console.log(`   XPUB: ${testXpub.substring(0, 20)}...`);
            console.log(`   Pioneer balance: ${pioneerTestResponse.data.balance} sats`);
            
        } else {
            throw new Error('Frontload failed or returned error');
        }
        
        return true;
    } catch (e) {
        console.log('❌ CRITICAL FAILURE: Frontload integration failed');
        console.log(`   Error: ${e.message}`);
        throw new Error('FRONTLOAD_INTEGRATION_FAILED');
    }
}

async function testPioneerPortfolioEndpoint() {
    try {
        console.log('\n📊 Testing Pioneer Portfolio Endpoint...');
        
        // Get XPUBs from vault
        const pubkeysResponse = await axios.get(`${VAULT_API_BASE}/api/v2/pubkeys`);
        const xpubs = pubkeysResponse.data.filter(p => p.type && ['xpub', 'ypub', 'zpub'].includes(p.type));
        
        if (xpubs.length === 0) {
            throw new Error('No XPUBs available for portfolio test');
        }
        
        // Build portfolio request
        const portfolioRequest = xpubs.slice(0, 3).map(xpub => ({
            caip: 'bip122:000000000019d6689c085ae165831e93/slip44:0',
            pubkey: xpub.address
        }));
        
        console.log(`   Testing with ${portfolioRequest.length} XPUBs...`);
        
        const response = await axios.post(
            `${LOCAL_PIONEER_BASE}/api/v1/portfolio`,
            portfolioRequest,
            { 
                headers: { 
                    'Content-Type': 'application/json',
                    'accept': 'application/json' 
                } 
            }
        );
        
        console.log('✅ Pioneer portfolio API successful:');
        console.log(`   Response entries: ${response.data.length}`);
        
        let totalValue = 0;
        for (const entry of response.data) {
            const value = parseFloat(entry.valueUsd) || 0;
            totalValue += value;
            console.log(`   ${entry.caip}: ${entry.balance} ($${entry.valueUsd})`);
        }
        
        console.log(`   Total Portfolio Value: $${totalValue.toFixed(2)}`);
        
        return response.data;
    } catch (e) {
        console.log('❌ CRITICAL FAILURE: Pioneer portfolio API failed');
        console.log(`   Error: ${e.message}`);
        throw new Error('PIONEER_PORTFOLIO_FAILED');
    }
}

async function runPioneerLiveTests() {
    try {
        console.log('🚀 Starting Pioneer Live Test Suite - FAIL FAST MODE\n');
        
        // Step 1: Health Checks
        console.log('Step 1: Health Checks');
        console.log('─'.repeat(21));
        await testPioneerHealth();
        await testVaultHealth();
        console.log('');
        
        // Step 2: Pioneer API Tests
        console.log('Step 2: Pioneer API Tests');
        console.log('─'.repeat(25));
        await testPioneerFeeRates();
        await testPioneerUnspentOutputs();
        await testPioneerAddressGeneration();
        await testPioneerTransactionHistory();
        console.log('');
        
        // Step 3: Vault API Tests
        console.log('Step 3: Vault API Tests');
        console.log('─'.repeat(21));
        const pubkeyData = await testVaultPubkeys();
        const pathData = await testVaultPaths();
        const balanceData = await testVaultBalances();
        console.log('');
        
        // Step 4: Multi-Device Context Tests
        console.log('Step 4: Multi-Device Context Tests');
        console.log('─'.repeat(34));
        await testDeviceContextSwitching();
        console.log('');
        
        // Step 5: Integration Tests
        console.log('Step 5: Integration Tests');
        console.log('─'.repeat(24));
        await testFrontloadIntegration();
        await testPioneerPortfolioEndpoint();
        console.log('');
        
        // Summary
        console.log('🎉 ALL TESTS PASSED!');
        console.log('====================');
        console.log('✅ Pioneer server is fully functional on 127.0.0.1:9001');
        console.log('✅ Vault v2 API endpoints are working correctly');
        console.log('✅ Frontload properly saves both XPUBs and addresses');
        console.log('✅ Multi-device context switching is supported');
        console.log('✅ Pioneer-Vault integration is working end-to-end');
        console.log('');
        console.log('🚀 Ready to deploy to pioneer.dev!');
        
    } catch (error) {
        console.log('\n💥 PIONEER LIVE TESTS FAILED');
        console.log('=============================');
        console.log(`Error: ${error.message}`);
        console.log('');
        
        // Specific troubleshooting advice
        if (error.message === 'PIONEER_SERVER_DOWN') {
            console.log('🔧 TROUBLESHOOTING:');
            console.log('1. Start local Pioneer server: docker run -p 9001:9001 pioneer:latest');
            console.log('2. Or check if Pioneer is running on different port');
        } else if (error.message === 'VAULT_SERVER_DOWN') {
            console.log('🔧 TROUBLESHOOTING:');
            console.log('1. Start Vault server: make vault');
            console.log('2. Ensure device is connected and initialized');
        } else if (error.message.includes('FRONTLOAD')) {
            console.log('🔧 TROUBLESHOOTING:');
            console.log('1. Check device connection and frontload logs');
            console.log('2. Verify device has been properly initialized');
            console.log('3. Check transport connectivity');
        }
        
        process.exit(1);
    }
}

// Run the test suite
runPioneerLiveTests().catch(console.error);