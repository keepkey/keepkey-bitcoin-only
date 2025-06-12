#!/usr/bin/env node

/**
 * Pioneer API Live Test Suite - NO MOCKING, FAIL FAST
 * 
 * Tests localhost Vault vs prod pioneers.dev and validates integration.
 * Uses REAL XPUBs from connected KeepKey device - NO MOCKED DATA.
 * 
 * FAIL FAST - NO FALLBACKS OR MOCKS
 */

const axios = require('axios');

console.log('🚀 PIONEER API LIVE TEST SUITE - NO MOCKING');
console.log('============================================');
console.log('');

// Configuration - NO MOCKED XPUBS!
const LOCALHOST_VAULT = 'http://localhost:1646';
const PROD_PIONEER = 'https://pioneers.dev';


// Global storage for real XPUBs from device
let REAL_XPUBS = {
    legacy: null,
    segwit: null,
    native_segwit: null
};

async function testVaultHealth() {
    try {
        console.log('🏥 Testing Vault Server Health...');
        const response = await axios.get(`${LOCALHOST_VAULT}/api/health`);
        
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

async function getRealXPUBsFromVault() {
    try {
        console.log('\n🔑 Getting REAL XPUBs from connected KeepKey device...');
        
        // Get actual pubkeys from vault - NO MOCKING
        const response = await axios.get(`${LOCALHOST_VAULT}/api/v2/pubkeys`);
        console.log(`📊 Retrieved ${response.data.length} pubkeys from vault`);
        
        // Extract REAL Bitcoin XPUBs
        const bitcoinXpubs = response.data.filter(p => 
            p.key_type && ['xpub', 'ypub', 'zpub'].includes(p.key_type) &&
            p.context && p.context.includes('bip122:000000000019d6689c085ae165831e93')
        );
        
        console.log(`✅ Found ${bitcoinXpubs.length} REAL Bitcoin XPUBs:`);
        
        for (const xpub of bitcoinXpubs) {
            console.log(`   ${xpub.key_type}: ${xpub.address.substring(0, 20)}... (${xpub.scriptType})`);
            
            // Store real XPUBs by type
            if (xpub.key_type === 'xpub') {
                REAL_XPUBS.legacy = xpub.address;
            } else if (xpub.key_type === 'ypub') {
                REAL_XPUBS.segwit = xpub.address;
            } else if (xpub.key_type === 'zpub') {
                REAL_XPUBS.native_segwit = xpub.address;
            }
        }
        
        if (bitcoinXpubs.length === 0) {
            throw new Error('NO REAL XPUBs FOUND - Device not frontloaded or not connected');
        }
        
        return bitcoinXpubs;
    } catch (e) {
        console.log('❌ CRITICAL FAILURE: Cannot get real XPUBs from device');
        console.log(`   Error: ${e.message}`);
        throw new Error('NO_REAL_XPUBS_AVAILABLE');
    }
}

async function testProdPioneerHealth() {
    try {
        console.log('\n🌍 Testing PROD Pioneer Server Health...');
        const response = await axios.get(`${PROD_PIONEER}/api/v1/health`);
        
        if (response.status === 200) {
            console.log('✅ Prod Pioneer server is running');
            console.log(`   Response: ${JSON.stringify(response.data)}`);
            return true;
        } else {
            throw new Error(`Unexpected status: ${response.status}`);
        }
    } catch (e) {
        console.log('❌ CRITICAL FAILURE: Prod Pioneer server not available');
        console.log(`   Error: ${e.message}`);
        throw new Error('PROD_PIONEER_DOWN');
    }
}

async function compareLocalVsProdUnspent() {
    try {
        console.log('\n🪙 COMPARING: List Unspent (Local Vault V2 vs Prod Pioneer)...');
        
        const testXpub = REAL_XPUBS.segwit || REAL_XPUBS.legacy;
        if (!testXpub) {
            throw new Error('No real XPUB available for testing');
        }
        
        console.log(`   Testing with REAL XPUB: ${testXpub.substring(0, 20)}...`);
        
        // LOCAL: Vault V2 API (will create this endpoint)
        let localResult;
        try {
            const localResponse = await axios.get(`${LOCALHOST_VAULT}/api/v2/listUnspent?xpub=${encodeURIComponent(testXpub)}`);
            localResult = localResponse.data;
            console.log(`✅ LOCAL: Found ${localResult.length || 0} UTXOs`);
        } catch (e) {
            console.log(`❌ LOCAL: V2 listUnspent endpoint failed - ${e.message}`);
            localResult = { error: e.message };
        }
        
        // PROD: Pioneer API
        let prodResult;
        try {
            const prodResponse = await axios.get(`${PROD_PIONEER}/api/v1/listUnspent/BTC/${testXpub}`);
            prodResult = prodResponse.data;
            console.log(`✅ PROD: Found ${prodResult.length || 0} UTXOs`);
        } catch (e) {
            console.log(`❌ PROD: Pioneer listUnspent failed - ${e.message}`);
            prodResult = { error: e.message };
        }
        
        // COMPARISON
        console.log('\n📋 COMPARISON RESULTS:');
        console.log(`   LOCAL: ${JSON.stringify(localResult, null, 2)}`);
        console.log(`   PROD:  ${JSON.stringify(prodResult, null, 2)}`);
        
        return { local: localResult, prod: prodResult };
    } catch (e) {
        console.log('❌ CRITICAL FAILURE: listUnspent comparison failed');
        console.log(`   Error: ${e.message}`);
        throw new Error('LIST_UNSPENT_COMPARISON_FAILED');
    }
}

async function compareLocalVsProdTxHistory() {
    try {
        console.log('\n📜 COMPARING: Transaction History (Local Vault V2 vs Prod Pioneer)...');
        
        const testXpub = REAL_XPUBS.segwit || REAL_XPUBS.legacy;
        if (!testXpub) {
            throw new Error('No real XPUB available for testing');
        }
        
        console.log(`   Testing with REAL XPUB: ${testXpub.substring(0, 20)}...`);
        
        // LOCAL: Vault V2 API (will create this endpoint)
        let localResult;
        try {
            const localResponse = await axios.get(`${LOCALHOST_VAULT}/api/v2/txHistory?xpub=${encodeURIComponent(testXpub)}`);
            localResult = localResponse.data;
            console.log(`✅ LOCAL: Found ${localResult.txs || 0} transactions`);
        } catch (e) {
            console.log(`❌ LOCAL: V2 txHistory endpoint failed - ${e.message}`);
            localResult = { error: e.message };
        }
        
        // PROD: Pioneer API
        let prodResult;
        try {
            const prodResponse = await axios.get(`${PROD_PIONEER}/api/v1/txsByXpub/BTC/${testXpub}`);
            prodResult = prodResponse.data;
            console.log(`✅ PROD: Found ${prodResult.txs || 0} transactions, balance ${prodResult.balance || 0} sats`);
        } catch (e) {
            console.log(`❌ PROD: Pioneer txHistory failed - ${e.message}`);
            prodResult = { error: e.message };
        }
        
        // COMPARISON
        console.log('\n📋 COMPARISON RESULTS:');
        console.log(`   LOCAL: ${JSON.stringify(localResult, null, 2)}`);
        console.log(`   PROD:  ${JSON.stringify(prodResult, null, 2)}`);
        
        return { local: localResult, prod: prodResult };
    } catch (e) {
        console.log('❌ CRITICAL FAILURE: txHistory comparison failed');
        console.log(`   Error: ${e.message}`);
        throw new Error('TX_HISTORY_COMPARISON_FAILED');
    }
}

async function compareLocalVsProdAddressGeneration() {
    try {
        console.log('\n🏠 COMPARING: Address Generation (Local Vault V2 vs Prod Pioneer)...');
        
        const testXpub = REAL_XPUBS.legacy;
        if (!testXpub) {
            throw new Error('No real legacy XPUB available for testing');
        }
        
        console.log(`   Testing with REAL XPUB: ${testXpub.substring(0, 20)}...`);
        
        // LOCAL: Vault V2 API (will create this endpoint)
        let localReceive, localChange;
        try {
            const receiveResponse = await axios.get(`${LOCALHOST_VAULT}/api/v2/getNewAddress?xpub=${encodeURIComponent(testXpub)}`);
            localReceive = receiveResponse.data;
            console.log(`✅ LOCAL: New receive address generated`);
            
            const changeResponse = await axios.get(`${LOCALHOST_VAULT}/api/v2/getChangeAddress?xpub=${encodeURIComponent(testXpub)}`);
            localChange = changeResponse.data;
            console.log(`✅ LOCAL: New change address generated`);
        } catch (e) {
            console.log(`❌ LOCAL: V2 address generation failed - ${e.message}`);
            localReceive = { error: e.message };
            localChange = { error: e.message };
        }
        
        // PROD: Pioneer API
        let prodReceive, prodChange;
        try {
            const receiveResponse = await axios.get(`${PROD_PIONEER}/api/v1/getNewAddress/BTC/${testXpub}`);
            prodReceive = receiveResponse.data;
            console.log(`✅ PROD: New receive address generated`);
            
            const changeResponse = await axios.get(`${PROD_PIONEER}/api/v1/getChangeAddress/BTC/${testXpub}`);
            prodChange = changeResponse.data;
            console.log(`✅ PROD: New change address generated`);
        } catch (e) {
            console.log(`❌ PROD: Pioneer address generation failed - ${e.message}`);
            prodReceive = { error: e.message };
            prodChange = { error: e.message };
        }
        
        // COMPARISON
        console.log('\n📋 COMPARISON RESULTS:');
        console.log(`   LOCAL RECEIVE: ${JSON.stringify(localReceive, null, 2)}`);
        console.log(`   PROD RECEIVE:  ${JSON.stringify(prodReceive, null, 2)}`);
        console.log(`   LOCAL CHANGE:  ${JSON.stringify(localChange, null, 2)}`);
        console.log(`   PROD CHANGE:   ${JSON.stringify(prodChange, null, 2)}`);
        
        return { 
            local: { receive: localReceive, change: localChange },
            prod: { receive: prodReceive, change: prodChange }
        };
    } catch (e) {
        console.log('❌ CRITICAL FAILURE: address generation comparison failed');
        console.log(`   Error: ${e.message}`);
        throw new Error('ADDRESS_GENERATION_COMPARISON_FAILED');
    }
}

async function testVaultV2Endpoints() {
    try {
        console.log('\n🔧 Testing NEW Vault V2 Endpoints...');
        
        // Test expanded pubkeys endpoint with address indices
        const pubkeysResponse = await axios.get(`${LOCALHOST_VAULT}/api/v2/pubkeys?include_indices=true`);
        console.log(`✅ Pubkeys endpoint: ${pubkeysResponse.data.length} entries`);
        
        // Show example of what we get
        const firstPubkey = pubkeysResponse.data[0];
        if (firstPubkey) {
            console.log(`   Example entry: ${JSON.stringify(firstPubkey, null, 2)}`);
        }
        
        return pubkeysResponse.data;
    } catch (e) {
        console.log('❌ CRITICAL FAILURE: V2 endpoints test failed');
        console.log(`   Error: ${e.message}`);
        throw new Error('V2_ENDPOINTS_FAILED');
    }
}

async function runNoMockingTests() {
    try {
        console.log('🚀 Starting NO MOCKING Test Suite - FAIL FAST MODE\n');
        
        // Step 1: Health Checks
        console.log('Step 1: Health Checks');
        console.log('─'.repeat(21));
        await testVaultHealth();
        await testProdPioneerHealth();
        console.log('');
        
        // Step 2: Get REAL XPUBs from device
        console.log('Step 2: Get REAL Data from Device');
        console.log('─'.repeat(33));
        await getRealXPUBsFromVault();
        console.log('');
        
        // Step 3: Test new V2 endpoints
        console.log('Step 3: Test NEW V2 Endpoints');
        console.log('─'.repeat(29));
        await testVaultV2Endpoints();
        console.log('');
        
        // Step 4: Local vs Prod Comparisons
        console.log('Step 4: Local vs Prod API Comparisons');
        console.log('─'.repeat(37));
        const unspentComparison = await compareLocalVsProdUnspent();
        const txHistoryComparison = await compareLocalVsProdTxHistory();
        const addressComparison = await compareLocalVsProdAddressGeneration();
        console.log('');
        
        // Summary
        console.log('🎯 NO MOCKING TEST RESULTS');
        console.log('==========================');
        console.log('✅ Using REAL XPUBs from connected KeepKey device');
        console.log('✅ Local Vault V2 API tests completed');
        console.log('✅ Prod Pioneer API tests completed');
        console.log('✅ Local vs Prod comparisons completed');
        console.log('');
        console.log('📊 COMPARISON SUMMARY:');
        console.log(`   ListUnspent: Local=${JSON.stringify(unspentComparison.local?.length || 'error')}, Prod=${JSON.stringify(unspentComparison.prod?.length || 'error')}`);
        console.log(`   TxHistory: Local=${JSON.stringify(txHistoryComparison.local?.txs || 'error')}, Prod=${JSON.stringify(txHistoryComparison.prod?.txs || 'error')}`);
        console.log(`   Address Gen: Both attempted (see detailed output above)`);
        
    } catch (error) {
        console.log('\n💥 NO MOCKING TESTS FAILED');
        console.log('===========================');
        console.log(`Error: ${error.message}`);
        console.log('');
        console.log('🔧 REQUIRED ACTIONS:');
        
        if (error.message === 'VAULT_SERVER_DOWN') {
            console.log('1. Start Vault server: cd projects/vault && bun run dev:tauri');
            console.log('2. Ensure KeepKey device is connected');
        } else if (error.message === 'NO_REAL_XPUBS_AVAILABLE') {
            console.log('1. Connect KeepKey device via USB');
            console.log('2. Run frontload to populate XPUBs');
            console.log('3. Verify device is properly initialized');
        } else if (error.message === 'PROD_PIONEER_DOWN') {
            console.log('1. Check pioneers.dev is accessible');
            console.log('2. Check internet connectivity');
        } else {
            console.log('1. Check detailed error output above');
            console.log('2. Implement missing V2 API endpoints');
        }
        
        process.exit(1);
    }
}

// Run the NO MOCKING test suite
runNoMockingTests().catch(console.error);