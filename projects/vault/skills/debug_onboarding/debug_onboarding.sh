#!/bin/bash
# debug_onboarding.sh - Debug onboarding database state
set -euo pipefail

# Pioneer style logging
LOG_FILE="/tmp/keepkey_debug.log"

log_message() {
    local level="$1"
    local message="$2"
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] [${level}] ${message}" | tee -a "$LOG_FILE"
}

# Check database location
DB_PATH="${HOME}/.keepkey/index.db"

log_message "INFO" "=== KeepKey Onboarding Debug ==="
log_message "INFO" "Database path: ${DB_PATH}"

if [[ ! -f "${DB_PATH}" ]]; then
    log_message "WARN" "Database file does not exist"
    log_message "INFO" "This indicates a first-time install"
    exit 0
fi

log_message "INFO" "Database file exists, checking contents..."

# Check if SQLite is available
if ! command -v sqlite3 &> /dev/null; then
    log_message "ERROR" "sqlite3 command not found. Please install SQLite"
    exit 1
fi

# Query database state
log_message "INFO" "Checking onboarding_completed flag:"
ONBOARDING_STATUS=$(sqlite3 "${DB_PATH}" "SELECT val FROM meta WHERE key = 'onboarding_completed';" 2>/dev/null || echo "NOT_FOUND")
log_message "INFO" "onboarding_completed = ${ONBOARDING_STATUS}"

log_message "INFO" "Checking first_install_timestamp:"
INSTALL_TIMESTAMP=$(sqlite3 "${DB_PATH}" "SELECT val FROM meta WHERE key = 'first_install_timestamp';" 2>/dev/null || echo "NOT_FOUND")
log_message "INFO" "first_install_timestamp = ${INSTALL_TIMESTAMP}"

log_message "INFO" "All meta table entries:"
sqlite3 "${DB_PATH}" "SELECT key, val FROM meta ORDER BY key;" 2>/dev/null | while read -r line; do
    log_message "INFO" "  ${line}"
done

log_message "INFO" "Database schema version:"
DB_VERSION=$(sqlite3 "${DB_PATH}" "SELECT val FROM meta WHERE key = 'db_version';" 2>/dev/null || echo "NOT_FOUND")
log_message "INFO" "db_version = ${DB_VERSION}"

# Count preference entries
PREF_COUNT=$(sqlite3 "${DB_PATH}" "SELECT COUNT(*) FROM meta WHERE key LIKE 'pref_%';" 2>/dev/null || echo "0")
log_message "INFO" "Number of preference entries: ${PREF_COUNT}"

# Show current logic results
if [[ "${ONBOARDING_STATUS}" == "true" ]]; then
    log_message "INFO" "✅ User should be considered onboarded"
    log_message "INFO" "✅ Onboarding wizard should NOT show"
else
    log_message "WARN" "❌ User is NOT onboarded (onboarding_completed = '${ONBOARDING_STATUS}')"
    log_message "WARN" "❌ Onboarding wizard WILL show every time"
fi

log_message "INFO" "=== Debug complete ==="
echo ""
echo "Debug log saved to: ${LOG_FILE}" 