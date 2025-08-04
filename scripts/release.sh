#!/bin/bash

# KeepKey Bitcoin-Only Release Script
# This script bumps the version across all projects in the repository

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Base directory
BASE_DIR=$(pwd)

# Projects to update
PROJECTS=(
    "projects/vault-v2"
    "projects/keepkey-rust"
)

# Get the current version from vault-v2 package.json (primary source of truth)
CURRENT_VERSION=$(grep '"version"' projects/vault-v2/package.json | head -1 | cut -d'"' -f4)

echo -e "${BLUE}=====================================${NC}"
echo -e "${BLUE}  KeepKey Bitcoin-Only Release Tool  ${NC}"
echo -e "${BLUE}=====================================${NC}"
echo -e "${YELLOW}Current version: ${CURRENT_VERSION}${NC}"

# Function to validate semver format
validate_version() {
    if ! [[ "$1" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
        echo -e "${RED}Error: Version must be in format X.Y.Z (e.g., 2.3.0)${NC}"
        exit 1
    fi
}

# Get the new version from command line argument or prompt
if [ "$1" ]; then
    NEW_VERSION="$1"
else
    echo -n "Enter new version (format: X.Y.Z): "
    read NEW_VERSION
fi

# Validate the new version
validate_version "$NEW_VERSION"

echo -e "\n${GREEN}Updating to version: ${NEW_VERSION}${NC}"

# Update function for different file types
update_json_version() {
    local file="$1"
    if [ -f "$file" ]; then
        echo "    📦 Updating $file..."
        if [[ "$OSTYPE" == "darwin"* ]]; then
            sed -i '' "s/\"version\": \"[^\"]*\"/\"version\": \"${NEW_VERSION}\"/" "$file"
        else
            sed -i "s/\"version\": \"[^\"]*\"/\"version\": \"${NEW_VERSION}\"/" "$file"
        fi
    fi
}

update_cargo_version() {
    local file="$1"
    if [ -f "$file" ]; then
        echo "    🦀 Updating $file..."
        if [[ "$OSTYPE" == "darwin"* ]]; then
            sed -i '' "s/^version = \"[^\"]*\"/version = \"${NEW_VERSION}\"/" "$file"
        else
            sed -i "s/^version = \"[^\"]*\"/version = \"${NEW_VERSION}\"/" "$file"
        fi
    fi
}

# Files to update
echo -e "\n${YELLOW}Updating version in all projects...${NC}"

# 1. Update vault-v2 project
echo -e "\n${BLUE}[vault-v2]${NC}"
update_json_version "projects/vault-v2/package.json"
update_json_version "projects/vault-v2/src-tauri/tauri.conf.json"
update_cargo_version "projects/vault-v2/src-tauri/Cargo.toml"

# Update package-lock.json if it exists
if [ -f "projects/vault-v2/package-lock.json" ]; then
    echo "    🔐 Updating package-lock.json..."
    cd projects/vault-v2
    npm install --package-lock-only
    cd "$BASE_DIR"
fi

# Update Cargo.lock if it exists
if [ -f "projects/vault-v2/src-tauri/Cargo.lock" ]; then
    echo "    🔒 Updating Cargo.lock..."
    cd projects/vault-v2/src-tauri
    cargo update -p vault-v2
    cd "$BASE_DIR"
fi

# 2. Update keepkey-rust if it has version files
echo -e "\n${BLUE}[keepkey-rust]${NC}"
update_cargo_version "projects/keepkey-rust/Cargo.toml"

# 3. Update root-level files if they exist
echo -e "\n${BLUE}[root]${NC}"
update_json_version "package.json"
update_cargo_version "Cargo.toml"

# Create/Update VERSION files
echo "$NEW_VERSION" > VERSION
echo "$NEW_VERSION" > projects/vault-v2/VERSION
if [ -d "projects/keepkey-rust" ]; then
    echo "$NEW_VERSION" > projects/keepkey-rust/VERSION
fi

echo -e "\n${GREEN}✅ Version bumped successfully to ${NEW_VERSION}${NC}"

echo -e "\n${YELLOW}Summary of changes:${NC}"
echo "  Projects updated:"
echo "    ✓ vault-v2"
echo "    ✓ keepkey-rust (if applicable)"
echo ""
echo "  Files updated:"
echo "    - package.json files"
echo "    - Cargo.toml files"
echo "    - tauri.conf.json"
echo "    - VERSION files"
echo "    - Lock files (if present)"

echo -e "\n${YELLOW}Next steps:${NC}"
echo "  1. Review the changes: git diff"
echo "  2. Build and test: make build-all"
echo "  3. Commit the changes: git add . && git commit -m \"chore: bump version to ${NEW_VERSION}\""
echo "  4. Create a git tag: git tag -a v${NEW_VERSION} -m \"Release v${NEW_VERSION}\""
echo "  5. Push changes and tags: git push && git push --tags"

# Optional: Show what changed
echo -e "\n${YELLOW}Would you like to see the changes? (y/n)${NC}"
read -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo -e "${GREEN}Changes made:${NC}"
    git diff --stat
fi

# Optional: Run build to verify everything works
echo -e "\n${YELLOW}Would you like to run a test build now? (y/n)${NC}"
read -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo -e "${GREEN}Running test build for vault-v2...${NC}"
    cd projects/vault-v2
    npm run build
    cd "$BASE_DIR"
    echo -e "${GREEN}✅ Build completed successfully${NC}"
fi