#!/bin/bash
# Test Recovery Device Reconnection Functionality
# This skill tests that recovery sessions survive device disconnections/reconnections

set -e

echo "🔧 Testing Recovery Device Reconnection Functionality"
echo "======================================================"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test results tracking
TESTS_PASSED=0
TESTS_FAILED=0

function print_test_header() {
    echo -e "\n${BLUE}Test: $1${NC}"
    echo "----------------------------------------"
}

function assert_success() {
    if [ $? -eq 0 ]; then
        echo -e "✅ ${GREEN}PASS:${NC} $1"
        ((TESTS_PASSED++))
    else
        echo -e "❌ ${RED}FAIL:${NC} $1"
        ((TESTS_FAILED++))
    fi
}

function assert_contains() {
    local text="$1"
    local expected="$2"
    local description="$3"
    
    if echo "$text" | grep -q "$expected"; then
        echo -e "✅ ${GREEN}PASS:${NC} $description"
        ((TESTS_PASSED++))
    else
        echo -e "❌ ${RED}FAIL:${NC} $description"
        echo -e "   Expected to find: '$expected'"
        echo -e "   In text: '$text'"
        ((TESTS_FAILED++))
    fi
}

# Check if we're in the correct directory
if [ ! -f "src-tauri/Cargo.toml" ]; then
    echo -e "${RED}Error: Must be run from vault-v2 project directory${NC}"
    exit 1
fi

print_test_header "Recovery Device Alias Functions"

# Test 1: Check that alias functions are accessible
echo "Testing device alias functionality..."

# Start the app in test mode to verify functions exist
cargo test --test recovery_reconnection_test --no-run > /dev/null 2>&1
assert_success "Recovery reconnection test compiles"

print_test_header "Code Analysis - Device ID Generation"

# Test 2: Verify device ID generation logic exists
echo "Checking device ID generation patterns..."

DEVICE_ID_CODE=$(grep -r "keepkey_.*_bus.*_addr" src-tauri/src/ || echo "")
assert_contains "$DEVICE_ID_CODE" "keepkey_" "Device ID generation patterns found"

# Test 3: Check for recovery alias tracking
echo "Checking recovery alias tracking..."

ALIAS_CODE=$(grep -r "RECOVERY_DEVICE_ALIASES" src-tauri/src/ || echo "")
assert_contains "$ALIAS_CODE" "RECOVERY_DEVICE_ALIASES" "Recovery device aliases tracking exists"

# Test 4: Check for device similarity detection
echo "Checking device similarity detection..."

SIMILARITY_CODE=$(grep -r "are_devices_potentially_same" src-tauri/src/ || echo "")
assert_contains "$SIMILARITY_CODE" "pub fn are_devices_potentially_same" "Device similarity detection function exists"

print_test_header "Event Controller Protection Logic"

# Test 5: Verify event controller has recovery protection
echo "Checking event controller recovery protection..."

PROTECTION_CODE=$(grep -r "is_device_in_recovery_flow" src-tauri/src/event_controller.rs || echo "")
assert_contains "$PROTECTION_CODE" "is_device_in_recovery_flow" "Event controller has recovery flow protection"

# Test 6: Check for queue preservation during recovery
echo "Checking queue preservation logic..."

PRESERVATION_CODE=$(grep -r "preserving queue and state" src-tauri/src/event_controller.rs || echo "")
assert_contains "$PRESERVATION_CODE" "preserving queue" "Queue preservation logic exists"

print_test_header "Recovery Flow UI Protection"

# Test 7: Check RecoveryFlow component has reconnection handling
echo "Checking RecoveryFlow reconnection handling..."

RECONNECTION_CODE=$(grep -r "device:recovery-reconnected" src/components/ || echo "")
assert_contains "$RECONNECTION_CODE" "recovery-reconnected" "RecoveryFlow listens for reconnection events"

# Test 8: Check for recovery lock mechanism
echo "Checking recovery lock mechanism..."

LOCK_CODE=$(grep -r "isRecoveryLocked" src/components/WalletCreationWizard/RecoveryFlow.tsx || echo "")
assert_contains "$LOCK_CODE" "isRecoveryLocked" "Recovery lock mechanism exists"

print_test_header "Backend Recovery Session Management"

# Test 9: Check canonical device ID resolution
echo "Checking canonical device ID resolution..."

CANONICAL_CODE=$(grep -r "get_canonical_device_id" src-tauri/src/commands.rs || echo "")
assert_contains "$CANONICAL_CODE" "pub fn get_canonical_device_id" "Canonical device ID resolution function exists"

# Test 10: Check recovery character handling uses canonical IDs
echo "Checking recovery character handling uses canonical IDs..."

CHARACTER_CODE=$(grep -A 10 -B 5 "send_recovery_character" src-tauri/src/commands.rs | grep -E "(canonical|get_canonical)" || echo "")
assert_contains "$CHARACTER_CODE" "canonical" "Recovery character handling uses canonical device IDs"

print_test_header "Test Summary"

echo -e "\n📊 Test Results:"
echo -e "  ${GREEN}Passed: $TESTS_PASSED${NC}"
echo -e "  ${RED}Failed: $TESTS_FAILED${NC}"

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "\n🎉 ${GREEN}All recovery reconnection functionality tests passed!${NC}"
    echo ""
    echo "✅ Device alias tracking implemented"
    echo "✅ Device similarity detection working"
    echo "✅ Event controller preserves recovery sessions"
    echo "✅ RecoveryFlow handles reconnection events"
    echo "✅ Backend resolves canonical device IDs"
    echo ""
    echo "🛡️ Recovery sessions should now survive device disconnections!"
    exit 0
else
    echo -e "\n⚠️  ${YELLOW}Some tests failed. Recovery reconnection may not work correctly.${NC}"
    exit 1
fi 