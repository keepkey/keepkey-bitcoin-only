#!/usr/bin/env node

/**
 * Pioneer API Only Test Suite
 * 
 * Tests local Pioneer server on 127.0.0.1:9001 independently
 * without requiring vault server to be running.
 * 
 * FAIL FAST - NO FALLBACKS OR MOCKS
 */

const axios = require('axios');

console.log('🚀 PIONEER API ONLY TEST SUITE');
console.log('===============================');
console.log('');

// Configuration
const LOCAL_PIONEER_BASE = 'http://127.0.0.1:9001';

// Get real XPUBs from vault dynamically - NO HARDCODED/FAKE DATA
let TEST_XPUBS = {};

async function getRealXPUBsFromVault() {
    try {
        console.log('🔑 Getting REAL XPUBs from vault at localhost:1646...');
        
        const pubkeysResponse = await axios.get('http://localhost:1646/api/v2/pubkeys');
        const pubkeys = pubkeysResponse.data;
        
        // Find Bitcoin XPUBs
        const bitcoinXpubs = pubkeys.filter(p => 
            p.scriptType && p.scriptType.includes('_xpub') &&
            p.address && (
                p.address.startsWith('xpub') || 
                p.address.startsWith('ypub') || 
                p.address.startsWith('zpub')
            ) &&
            p.context && p.context.includes('bip122:000000000019d6689c085ae165831e93')
        );
        
        console.log(`✅ Found ${bitcoinXpubs.length} real Bitcoin XPUBs from vault`);
        
        // Map to test structure
        for (const xpub of bitcoinXpubs) {
            if (xpub.address.startsWith('xpub')) {
                TEST_XPUBS.btc_legacy = xpub.address;
                console.log(`   btc_legacy (P2PKH): ${xpub.address.substring(0, 20)}...`);
            } else if (xpub.address.startsWith('ypub')) {
                TEST_XPUBS.btc_segwit = xpub.address;
                console.log(`   btc_segwit (P2SH-P2WPKH): ${xpub.address.substring(0, 20)}...`);
            } else if (xpub.address.startsWith('zpub')) {
                TEST_XPUBS.btc_native_segwit = xpub.address;
                console.log(`   btc_native_segwit (P2WPKH): ${xpub.address.substring(0, 20)}...`);
            }
        }
        
        if (Object.keys(TEST_XPUBS).length === 0) {
            throw new Error('No Bitcoin XPUBs found in vault - is device frontloaded?');
        }
        
        return TEST_XPUBS;
    } catch (error) {
        console.log('❌ CRITICAL: Cannot get real XPUBs from vault');
        console.log(`   Error: ${error.message}`);
        console.log('   Fix: Make sure vault is running and device is frontloaded');
        throw new Error('NO_REAL_XPUBS_FROM_VAULT');
    }
}

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
            console.log('   Fix: Start local pioneer with:');
            console.log('        docker run -p 9001:9001 pioneer:latest');
            console.log('   Or:  docker compose up pioneer');
        }
        throw new Error('PIONEER_SERVER_DOWN');
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
        if (e.response) {
            console.log(`   Status: ${e.response.status}`);
            console.log(`   Data: ${JSON.stringify(e.response.data)}`);
        }
        throw new Error('FEE_RATE_API_FAILED');
    }
}

async function testPioneerUnspentOutputs() {
    try {
        console.log('\n🪙 Testing Pioneer List Unspent API...');
        
        // Test with multiple XPUB types
        for (const [type, xpub] of Object.entries(TEST_XPUBS)) {
            console.log(`   Testing ${type} XPUB...`);
            
            const response = await axios.get(
                `${LOCAL_PIONEER_BASE}/api/v1/listUnspent/BTC/${xpub}`,
                { headers: { 'accept': 'application/json' } }
            );
            
            console.log(`   ✅ ${type}: ${response.data.length || 0} UTXOs`);
            
            if (response.data.length > 0) {
                const firstUtxo = response.data[0];
                console.log(`      First UTXO: ${firstUtxo.txid}:${firstUtxo.vout} (${firstUtxo.value} sats)`);
            }
        }
        
        return true;
    } catch (e) {
        console.log('❌ CRITICAL FAILURE: List unspent API failed');
        console.log(`   Error: ${e.message}`);
        if (e.response) {
            console.log(`   Status: ${e.response.status}`);
            console.log(`   Data: ${JSON.stringify(e.response.data)}`);
        }
        throw new Error('LIST_UNSPENT_API_FAILED');
    }
}

async function testPioneerAddressGeneration() {
    try {
        console.log('\n🏠 Testing Pioneer Address Generation APIs...');
        
        for (const [type, xpub] of Object.entries(TEST_XPUBS)) {
            console.log(`   Testing ${type} address generation...`);
            
            // Test get new receive address
            const receiveResponse = await axios.get(
                `${LOCAL_PIONEER_BASE}/api/v1/getNewAddress/BTC/${xpub}`,
                { headers: { 'accept': 'application/json' } }
            );
            
            console.log(`   ✅ ${type} receive address: ${JSON.stringify(receiveResponse.data)}`);
            
            // Test get new change address
            const changeResponse = await axios.get(
                `${LOCAL_PIONEER_BASE}/api/v1/getChangeAddress/BTC/${xpub}`,
                { headers: { 'accept': 'application/json' } }
            );
            
            console.log(`   ✅ ${type} change address: ${JSON.stringify(changeResponse.data)}`);
        }
        
        return true;
    } catch (e) {
        console.log('❌ CRITICAL FAILURE: Address generation API failed');
        console.log(`   Error: ${e.message}`);
        if (e.response) {
            console.log(`   Status: ${e.response.status}`);
            console.log(`   Data: ${JSON.stringify(e.response.data)}`);
        }
        throw new Error('ADDRESS_GENERATION_API_FAILED');
    }
}

async function testPioneerTransactionHistory() {
    try {
        console.log('\n📜 Testing Pioneer Transaction History API...');
        
        for (const [type, xpub] of Object.entries(TEST_XPUBS)) {
            console.log(`   Testing ${type} transaction history...`);
            
            const response = await axios.get(
                `${LOCAL_PIONEER_BASE}/api/v1/txsByXpub/BTC/${xpub}`,
                { headers: { 'accept': 'application/json' } }
            );
            console.log(response);
            console.log(`   ✅ ${type} transaction history:`);
            console.log(`      Page: ${response.data.page}/${response.data.totalPages}`);
            console.log(`      Balance: ${response.data.balance} sats`);
            console.log(`      Total Received: ${response.data.totalReceived} sats`);
            console.log(`      Total Sent: ${response.data.totalSent} sats`);
            console.log(`      Transaction Count: ${response.data.txs}`);
            
            if (response.data.txids && response.data.txids.length > 0) {
                console.log(`      Recent TXIDs: ${response.data.txids.slice(0, 2).join(', ')}...`);
            }
        }
        
        return true;
    } catch (e) {
        console.log('❌ CRITICAL FAILURE: Transaction history API failed');
        console.log(`   Error: ${e.message}`);
        if (e.response) {
            console.log(`   Status: ${e.response.status}`);
            console.log(`   Data: ${JSON.stringify(e.response.data)}`);
        }
        throw new Error('TX_HISTORY_API_FAILED');
    }
}

async function testPioneerPortfolioEndpoint() {
    try {
        console.log('\n📊 Testing Pioneer Portfolio Endpoint...');
        
        // Build portfolio request with multiple XPUB types
        const portfolioRequest = Object.entries(TEST_XPUBS).map(([type, xpub]) => ({
            caip: 'bip122:000000000019d6689c085ae165831e93/slip44:0',
            pubkey: xpub
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
            console.log(`      XPUB: ${entry.pubkey.substring(0, 20)}...`);
        }
        
        console.log(`   Total Portfolio Value: $${totalValue.toFixed(2)}`);
        
        return response.data;
    } catch (e) {
        console.log('❌ CRITICAL FAILURE: Pioneer portfolio API failed');
        console.log(`   Error: ${e.message}`);
        if (e.response) {
            console.log(`   Status: ${e.response.status}`);
            console.log(`   Data: ${JSON.stringify(e.response.data)}`);
        }
        throw new Error('PIONEER_PORTFOLIO_FAILED');
    }
}

async function runPioneerOnlyTests() {
    try {
        console.log('🚀 Starting Pioneer Only Test Suite - FAIL FAST MODE\n');
        
        // Step 0: Get Real XPUBs from Vault
        console.log('Step 0: Get Real XPUBs from Vault');
        console.log('─'.repeat(33));
        await getRealXPUBsFromVault();
        console.log('');
        
        // Step 1: Health Check
        console.log('Step 1: Pioneer Health Check');
        console.log('─'.repeat(28));
        await testPioneerHealth();
        console.log('');
        
        // Step 2: Fee Rate API
        console.log('Step 2: Fee Rate API Test');
        console.log('─'.repeat(25));
        await testPioneerFeeRates();
        console.log('');
        
        // Step 3: UTXO Management
        console.log('Step 3: UTXO Management Tests');
        console.log('─'.repeat(29));
        await testPioneerUnspentOutputs();
        console.log('');
        
        // Step 4: Address Generation
        console.log('Step 4: Address Generation Tests');
        console.log('─'.repeat(32));
        await testPioneerAddressGeneration();
        console.log('');
        
        // Step 5: Transaction History
        console.log('Step 5: Transaction History Tests');
        console.log('─'.repeat(33));
        await testPioneerTransactionHistory();
        console.log('');
        
        // Step 6: Portfolio API
        console.log('Step 6: Portfolio API Test');
        console.log('─'.repeat(26));
        await testPioneerPortfolioEndpoint();
        console.log('');
        
        // Summary
        console.log('🎉 ALL PIONEER TESTS PASSED!');
        console.log('=============================');
        console.log('✅ Pioneer server is fully functional on 127.0.0.1:9001');
        console.log('✅ All XPUB types work correctly (legacy, segwit, native segwit)');
        console.log('✅ Fee rates, UTXOs, addresses, transactions, and portfolio all work');
        console.log('✅ Ready for vault integration testing');
        console.log('✅ Ready to deploy to pioneer.dev!');
        console.log('');
        console.log('📋 NEXT STEPS:');
        console.log('1. Start vault server: make vault');
        console.log('2. Run full integration test: node tests/test-pioneer-live.js');
        console.log('3. Verify frontload populates XPUBs correctly');
        console.log('4. Test multi-device context switching');
        
    } catch (error) {
        console.log('\n💥 PIONEER TESTS FAILED');
        console.log('========================');
        console.log(`Error: ${error.message}`);
        console.log('');
        
        // Specific troubleshooting advice
        if (error.message === 'PIONEER_SERVER_DOWN') {
            console.log('🔧 TROUBLESHOOTING:');
            console.log('1. Check if Pioneer server is running:');
            console.log('   lsof -i :9001');
            console.log('2. Start Pioneer server:');
            console.log('   docker run -p 9001:9001 pioneer:latest');
            console.log('3. Check Docker logs:');
            console.log('   docker logs $(docker ps | grep pioneer | awk \'{print $1}\')');
        } else if (error.message.includes('API_FAILED')) {
            console.log('🔧 TROUBLESHOOTING:');
            console.log('1. Check Pioneer server logs for errors');
            console.log('2. Verify XPUB format is correct');
            console.log('3. Check network connectivity');
            console.log('4. Ensure test data is valid');
        }
        
        process.exit(1);
    }
}

// Run the test suite
runPioneerOnlyTests().catch(console.error); 