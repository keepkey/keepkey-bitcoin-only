#!/usr/bin/env bash
# Auto-reexec with bash if script is run with sh or another shell
if [ -z "$BASH_VERSION" ]; then
  exec bash "$0" "$@"
fi

# KeepKey REST E2E Test Suite - Simple Bitcoin API with Frontloading
# Tests real device communication and frontloaded data
set -euo pipefail

# Enhanced Colors and Formatting
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
WHITE='\033[1;37m'
BOLD='\033[1m'
DIM='\033[2m'
NC='\033[0m' # No Color

# Unicode symbols for better visual feedback
CHECKMARK="✅"
CROSSMARK="❌"
WARNING="⚠️"
ROCKET="🚀"
DEVICE="📱"
KEY="🔑"
COIN="💰"
SIGNATURE="✍️"
NETWORK="🌐"
MAGNIFY="🔍"
DATABASE="💾"

# Dependency checks
command -v curl >/dev/null 2>&1 || { echo -e "${RED}${CROSSMARK} curl required${NC}"; exit 1; }
command -v jq >/dev/null 2>&1 || { echo -e "${RED}${CROSSMARK} jq required${NC}"; exit 1; }

BASE_URL="${BASE_URL:-http://localhost:1646/api/bitcoin}"
VERBOSE=${VERBOSE:-1}

# Track test failures
FAILED_TESTS=0

# Enhanced logging with colors and symbols
log() {
  if [[ "$VERBOSE" == "1" ]]; then
    echo -e "${DIM}${MAGNIFY} $1${NC}"
  fi
}

header() {
  echo -e "${BOLD}${CYAN}$1${NC}"
}

subheader() {
  echo -e "${BOLD}${BLUE}$1${NC}"
}

pass() { 
  echo -e "${GREEN}${CHECKMARK} ${BOLD}$1${NC}"
}

fail() { 
  echo -e "${RED}${CROSSMARK} ${BOLD}$1${NC}"
  ((FAILED_TESTS++))
}

warn() {
  echo -e "${YELLOW}${WARNING} ${BOLD}$1${NC}"
}

info() {
  echo -e "${CYAN}ℹ️  $1${NC}"
}

# Test helper to verify CAIP-2 network format
check_caip2_format() {
  local networks="$1"
  if echo "$networks" | jq -e '.networks[] | select(startswith("bip122:"))' >/dev/null 2>&1; then
    pass "Networks use proper CAIP-2 format"
  else
    fail "Networks not in CAIP-2 format"
  fi
}

echo -e "${BOLD}${WHITE}"
echo "╔════════════════════════════════════════════════════════════════╗"
echo "║         ${ROCKET} KeepKey Bitcoin API Test Suite - RUD over DRY ${ROCKET}         ║"
echo "║                                                                ║"
echo "║  Testing against: $BASE_URL"
echo "║  Focus: Frontloading, Real Data, Simplicity                   ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo -e "${NC}"

########################################
# 1. Infrastructure & Health Tests
########################################
header "📋 INFRASTRUCTURE TESTS"

subheader "🔧 Bitcoin Health Check"
log "GET /health"
health_response=$(curl -s "$BASE_URL/health")
health_status=$(curl -s -o /dev/null -w "%{http_code}" "$BASE_URL/health")
log "Response: $health_response"
log "Status: $health_status"

if [[ $(echo "$health_response" | jq -r '.status' 2>/dev/null) == "ok" ]]; then
  device_connected=$(echo "$health_response" | jq -r '.device_connected' 2>/dev/null)
  frontloaded=$(echo "$health_response" | jq -r '.frontloaded' 2>/dev/null)
  
  pass "Health endpoint responding"
  
  if [[ "$device_connected" == "true" ]]; then
    pass "KeepKey device detected"
  else
    fail "No KeepKey device detected - cannot test real functionality!"
  fi
  
  if [[ "$frontloaded" == "true" ]]; then
    pass "Data is frontloaded"
  else
    info "Data not yet frontloaded (will test frontload endpoint)"
  fi
else
  fail "Health check failed"
fi

########################################
# 2. Network Tests (CAIP-2 Format)
########################################
header "${NETWORK} NETWORK TESTS"

subheader "🌐 Supported Networks (CAIP-2)"
log "GET /networks"
networks_response=$(curl -s "$BASE_URL/networks")
networks_status=$(curl -s -o /dev/null -w "%{http_code}" "$BASE_URL/networks")
log "Response: $networks_response"
log "Status: $networks_status"

if echo "$networks_response" | jq -e '.networks | length > 0' >/dev/null 2>&1; then
  network_count=$(echo "$networks_response" | jq -r '.networks | length')
  networks_list=$(echo "$networks_response" | jq -r '.networks | join(", ")')
  pass "Networks endpoint returns $network_count networks"
  info "Networks: $networks_list"
  check_caip2_format "$networks_response"
else
  fail "Networks endpoint returned invalid data"
fi

# Use proper CAIP-2 format for Bitcoin mainnet (matches default-paths.json)
NETWORK="bip122:000000000019d6689c085ae165831e93"
NETWORK_DISPLAY="Bitcoin Mainnet (CAIP-2)"

########################################
# 3. Path Validation Tests
########################################
header "${KEY} PATH VALIDATION TESTS"

declare -a TEST_PATHS=("m/44'/0'/0'" "m/49'/0'/0'" "m/84'/0'/0'" "m/99'/0'/0'")
declare -a PATH_NAMES=("Legacy (BIP44)" "SegWit P2SH (BIP49)" "Native SegWit (BIP84)" "Invalid")
declare -a EXPECTED_VALID=(true true true false)

for i in "${!TEST_PATHS[@]}"; do
  path="${TEST_PATHS[$i]}"
  name="${PATH_NAMES[$i]}"
  expected="${EXPECTED_VALID[$i]}"
  
  subheader "🔍 Testing $name Path: $path"
  log "POST /parse-path path=$path"
  
  path_response=$(curl -s -X POST "$BASE_URL/parse-path" \
    -H 'Content-Type: application/json' \
    -d "{\"path\":\"$path\"}")
  path_status=$(curl -s -o /dev/null -w "%{http_code}" -X POST "$BASE_URL/parse-path" \
    -H 'Content-Type: application/json' \
    -d "{\"path\":\"$path\"}")
  
  log "Response: $path_response"
  log "Status: $path_status"
  
  actual_valid=$(echo "$path_response" | jq -r '.valid' 2>/dev/null)
  if [[ "$actual_valid" == "$expected" ]]; then
    pass "Path validation for $name correctly returned $expected"
  else
    fail "Path validation for $name expected $expected, got $actual_valid"
  fi
done

########################################
# 4. Frontloading Tests
########################################
header "${DATABASE} FRONTLOADING TESTS"

subheader "💾 Frontload Bitcoin Addresses"
log "POST /frontload"

frontload_response=$(curl -s -X POST "$BASE_URL/frontload")
frontload_status=$(curl -s -o /dev/null -w "%{http_code}" -X POST "$BASE_URL/frontload")

log "Response: $frontload_response"
log "Status: $frontload_status"

if [[ "$frontload_status" == "200" ]]; then
  success=$(echo "$frontload_response" | jq -r '.success' 2>/dev/null)
  addresses_loaded=$(echo "$frontload_response" | jq -r '.addresses_loaded' 2>/dev/null)
  message=$(echo "$frontload_response" | jq -r '.message' 2>/dev/null)
  
  if [[ "$success" == "true" && "$addresses_loaded" -gt 0 ]]; then
    pass "Frontload successful: $addresses_loaded addresses loaded"
    info "$message"
  else
    fail "Frontload failed: $message (need real device connected!)"
  fi
else
  fail "Frontload returned error status $frontload_status"
fi

########################################
# 5. Pubkey Tests (From Frontloaded or Device)
########################################
header "${KEY} PUBKEY TESTS - REAL DATA REQUIRED"

info "Testing pubkey retrieval from frontloaded data or live device"
warn "These tests MUST return real data - 503 errors are FAILURES!"

subheader "${KEY} Public Key Retrieval Tests"
for i in "${!TEST_PATHS[@]:0:3}"; do  # Only test valid paths
  path="${TEST_PATHS[$i]}"
  name="${PATH_NAMES[$i]}"
  
  log "POST /pubkey path=$path network=$NETWORK_DISPLAY"
  
  pubkey_response=$(curl -s -X POST "$BASE_URL/pubkey" \
    -H 'Content-Type: application/json' \
    -d "{\"path\":\"$path\",\"network\":\"$NETWORK\"}")
  pubkey_status=$(curl -s -o /dev/null -w "%{http_code}" -X POST "$BASE_URL/pubkey" \
    -H 'Content-Type: application/json' \
    -d "{\"path\":\"$path\",\"network\":\"$NETWORK\"}")
  
  log "Response: $pubkey_response"
  log "Status: $pubkey_status"
  
  if [[ "$pubkey_status" == "200" ]]; then
    # Got real data!
    address=$(echo "$pubkey_response" | jq -r '.address' 2>/dev/null)
    xpub=$(echo "$pubkey_response" | jq -r '.xpub' 2>/dev/null)
    script_type=$(echo "$pubkey_response" | jq -r '.script_type' 2>/dev/null)
    
    if [[ "$address" != "null" && "$address" != "" && "$address" != "FRONTLOAD_PENDING_"* ]]; then
      pass "Got real address for $name: $address"
      info "Script type: $script_type"
      
      # Verify no mock data
      if [[ "$address" == *"MOCK"* || "$xpub" == *"MOCK"* || "$address" == "PENDING" ]]; then
        fail "MOCK or PENDING data detected in $name response!"
      else
        pass "Real data verified for $name"
      fi
    else
      fail "Empty or pending response for $name - need real device data!"
    fi
  elif [[ "$pubkey_status" == "503" ]]; then
    fail "Pubkey for $name returned 503 - NO DEVICE OR DATA AVAILABLE! This is a FAILURE!"
  else
    fail "Pubkey for $name returned unexpected status $pubkey_status"
  fi
done

########################################
# Final Summary
########################################
echo -e "${BOLD}${WHITE}"
echo "╔════════════════════════════════════════════════════════════════╗"
echo "║                    🎉 BITCOIN API TEST RESULTS 🎉                ║"
echo "╠════════════════════════════════════════════════════════════════╣"
echo "║                                                                ║"

if [[ $FAILED_TESTS -eq 0 ]]; then
  echo "║  ${GREEN}${CHECKMARK} ALL TESTS PASSED!${NC}${BOLD}${WHITE}                                        ║"
  echo "║  ${GREEN}Real device communication working${NC}${BOLD}${WHITE}                           ║"
  echo "║  ${GREEN}Frontloading successful${NC}${BOLD}${WHITE}                                     ║"
  echo "║  ${GREEN}Real addresses retrieved${NC}${BOLD}${WHITE}                                    ║"
else
  echo "║  ${RED}${CROSSMARK} FAILED TESTS: $FAILED_TESTS${NC}${BOLD}${WHITE}                                         ║"
  echo "║                                                                ║"
  echo "║  ${RED}Common issues:${NC}${BOLD}${WHITE}                                               ║"
  echo "║  - No KeepKey device connected                                ║"
  echo "║  - Device in bootloader mode (needs firmware)                ║"
  echo "║  - Device communication not implemented                       ║"
  echo "║  - Frontloading failed                                        ║"
fi

echo "║                                                                ║"
echo "║  ${BLUE}Requirements for passing:${NC}${BOLD}${WHITE}                                    ║"
echo "║  1. KeepKey device must be connected and unlocked            ║"
echo "║  2. Frontload must populate real addresses                   ║"
echo "║  3. Pubkey endpoints must return real xpubs/addresses        ║"
echo "║                                                                ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo -e "${NC}"

if [[ $FAILED_TESTS -gt 0 ]]; then
  error "Test suite FAILED with $FAILED_TESTS failures!"
  exit 1
else
  info "All tests passed! Real Bitcoin API working correctly."
fi