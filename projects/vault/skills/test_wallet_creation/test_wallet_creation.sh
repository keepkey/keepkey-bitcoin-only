#!/bin/bash
# test_wallet_creation.sh - Test wallet creation wizard in KeepKey Desktop v5
set -euo pipefail

# Pioneer style logging
LOG_FILE="/tmp/keepkey_wallet_test.log"

log_message() {
    local level="$1"
    local message="$2"
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] [${level}] ${message}" | tee -a "$LOG_FILE"
}

log_message "INFO" "=== Starting KeepKey Wallet Creation Test ==="

# Change to project directory
PROJECT_DIR="/Users/highlander/WebstormProjects/keepkey-stack/projects/keepkey-desktop-v5"
cd "$PROJECT_DIR" || {
    log_message "ERROR" "Failed to change to project directory: $PROJECT_DIR"
    exit 1
}

log_message "INFO" "Project directory: $(pwd)"

# Check if dependencies are installed
if [[ ! -d "node_modules" ]]; then
    log_message "INFO" "Installing dependencies..."
    bun install || {
        log_message "ERROR" "Failed to install dependencies"
        exit 1
    }
fi

# Build the project first
log_message "INFO" "Building project..."
bun run build || {
    log_message "WARN" "Build failed, continuing with dev mode..."
}

# Start the development server
log_message "INFO" "Starting KeepKey Desktop v5 in development mode..."
log_message "INFO" "This will start the Tauri application with wallet creation wizard"
log_message "INFO" "To test the wallet creation flow:"
log_message "INFO" "1. When the app starts, it may show onboarding first"
log_message "INFO" "2. Complete onboarding to reach the main app"
log_message "INFO" "3. Look for device connection or wallet creation options"
log_message "INFO" "4. The wallet creation wizard should appear when triggered"

# Export environment for debugging
export RUST_LOG=debug
export TAURI_DEBUG=1

# Start the application
bun run tauri dev 