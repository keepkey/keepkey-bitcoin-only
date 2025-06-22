#!/bin/bash

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}🔍 Testing Vault-v2 Logging System${NC}"
echo ""

# Check if logs directory exists
LOG_DIR="$HOME/.keepkey/logs"
echo -e "${BLUE}📂 Checking logs directory...${NC}"

if [ -d "$LOG_DIR" ]; then
    echo -e "${GREEN}✓ Logs directory exists: $LOG_DIR${NC}"
    
    # List log files
    echo -e "${BLUE}📄 Log files:${NC}"
    ls -la "$LOG_DIR"/*.log 2>/dev/null || echo "No log files found yet"
    
    # Check today's log file
    TODAY=$(date +%Y-%m-%d)
    TODAY_LOG="$LOG_DIR/device-communications-$TODAY.log"
    
    echo ""
    echo -e "${BLUE}📝 Today's log file: $TODAY_LOG${NC}"
    
    if [ -f "$TODAY_LOG" ]; then
        echo -e "${GREEN}✓ Today's log file exists${NC}"
        echo -e "${BLUE}📊 File size: $(stat -f%z "$TODAY_LOG" 2>/dev/null || stat -c%s "$TODAY_LOG" 2>/dev/null || echo "unknown") bytes${NC}"
        echo ""
        echo -e "${BLUE}🔍 Last 10 log entries:${NC}"
        tail -n 10 "$TODAY_LOG" | jq '.' 2>/dev/null || tail -n 10 "$TODAY_LOG"
    else
        echo -e "${RED}✗ Today's log file doesn't exist yet${NC}"
        echo "  It will be created when the app starts or when a device operation occurs"
    fi
else
    echo -e "${RED}✗ Logs directory doesn't exist: $LOG_DIR${NC}"
    echo "  It will be created when the app starts"
fi

echo ""
echo -e "${BLUE}💡 Tips:${NC}"
echo "1. Connect a KeepKey device to generate log entries"
echo "2. Use the 'Save Logs' button in Settings > Logs to copy the log path"
echo "3. Logs are automatically cleaned up after 30 days"
echo "4. Log format is JSON for easy parsing and analysis"

# Check if jq is installed for pretty printing
if ! command -v jq &> /dev/null; then
    echo ""
    echo -e "${BLUE}💡 Install jq for better log viewing:${NC}"
    echo "  brew install jq"
fi 