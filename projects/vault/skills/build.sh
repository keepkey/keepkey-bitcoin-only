#!/bin/bash

# KeepKey Desktop Build & Run Script
# This script manages the build and execution of both frontend and backend components
# of the KeepKey Desktop Tauri application.

# Set up error handling
set -e

# Define colors for output
GREEN="\033[0;32m"
YELLOW="\033[1;33m"
RED="\033[0;31m"
BLUE="\033[0;34m"
NC="\033[0m" # No Color

# Define paths
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && cd .. && pwd)"
FRONTEND_DIR="$ROOT_DIR"
BACKEND_DIR="$ROOT_DIR/src-tauri"

# Define log function
log() {
    local type=$1
    local message=$2
    local color=$NC
    
    case $type in
        "info") color=$BLUE ;;
        "success") color=$GREEN ;;
        "warning") color=$YELLOW ;;
        "error") color=$RED ;;
    esac
    
    echo -e "${color}[$(date +"%Y-%m-%d %H:%M:%S")] $message${NC}"
}

# Function to check if a process is running
process_running() {
    local search_term=$1
    pgrep -f "$search_term" > /dev/null
    return $?
}

# Function to kill processes
kill_process() {
    local search_term=$1
    local process_name=$2
    
    if process_running "$search_term"; then
        log "warning" "Stopping $process_name..."
        pkill -f "$search_term" || true
        sleep 2
        
        # Force kill if still running
        if process_running "$search_term"; then
            log "warning" "Force stopping $process_name..."
            pkill -9 -f "$search_term" || true
        fi
        
        log "success" "$process_name stopped successfully"
    else
        log "info" "No running $process_name found"
    fi
}

# Function to clean up all processes
cleanup() {
    log "info" "Cleaning up processes..."
    
    # Kill frontend dev server
    kill_process "vite" "Frontend server"
    
    # Kill any running Tauri processes
    kill_process "tauri" "Tauri process"
    kill_process "keepkey-desktop" "KeepKey Desktop app"
    
    # Kill any cargo processes related to our app
    kill_process "cargo.*keepkey" "Cargo build process"
    
    log "success" "Cleanup complete"
}

# Function to install dependencies
install_dependencies() {
    log "info" "Installing dependencies..."
    
    # Install npm dependencies
    if [ -f "$FRONTEND_DIR/package.json" ]; then
        log "info" "Installing frontend dependencies..."
        cd "$FRONTEND_DIR" && npm install
    else
        log "error" "package.json not found in $FRONTEND_DIR"
        exit 1
    fi
    
    # Check Rust and cargo
    if ! command -v cargo &> /dev/null; then
        log "error" "Rust/Cargo not found. Please install Rust: https://rustup.rs/"
        exit 1
    fi
    
    log "success" "Dependencies installed successfully"
}

# Function to build the application
build_app() {
    log "info" "Building KeepKey Desktop application..."
    
    # Clean up any previous processes
    cleanup
    
    # Install dependencies
    install_dependencies
    
    # Build the app in development mode
    log "info" "Starting Tauri development build..."
    cd "$FRONTEND_DIR" && npm run tauri dev
}

# Function to build for production
build_production() {
    log "info" "Building KeepKey Desktop for production..."
    
    # Clean up any previous processes
    cleanup
    
    # Install dependencies
    install_dependencies
    
    # Build the frontend first
    log "info" "Building frontend..."
    cd "$FRONTEND_DIR" && npm run build
    
    # Build the Tauri app
    log "info" "Building Tauri application..."
    cd "$FRONTEND_DIR" && npm run tauri build
    
    log "success" "Production build completed"
    log "info" "Application bundle can be found in src-tauri/target/release/bundle/"
}

# Function to run the app in development mode
run_dev() {
    log "info" "Running KeepKey Desktop in development mode..."
    
    # Clean up any previous processes
    cleanup
    
    # Check if the Vite config exists
    if [ ! -f "$FRONTEND_DIR/vite.config.ts" ] && [ ! -f "$FRONTEND_DIR/vite.config.js" ]; then
        log "error" "Vite configuration not found"
        exit 1
    fi
    
    # Start the Vite server in the background
    log "info" "Starting Vite development server..."
    cd "$FRONTEND_DIR" && npm run dev &
    VITE_PID=$!
    
    # Wait for Vite server to initialize
    log "info" "Waiting for Vite server to initialize..."
    sleep 5
    
    # Check if Vite server started successfully
    if ! process_running "vite"; then
        log "error" "Vite server failed to start"
        exit 1
    fi
    
    log "success" "Vite server started successfully"
    
    # Now start Tauri
    log "info" "Starting Tauri development process..."
    cd "$FRONTEND_DIR" && npm run tauri dev
    
    # When Tauri exits, also kill the Vite server
    log "info" "Tauri exited, cleaning up..."
    if process_running "vite"; then
        kill $VITE_PID 2>/dev/null || true
    fi
}

# Function to show help
show_help() {
    echo -e "\n${GREEN}KeepKey Desktop Build Script${NC}"
    echo -e "Usage: ./skills/build.sh [OPTION]"
    echo -e "\nOptions:"
    echo -e "  ${BLUE}dev${NC}         Run in development mode (default)"
    echo -e "  ${BLUE}build${NC}       Build for production"
    echo -e "  ${BLUE}clean${NC}       Clean up all running processes"
    echo -e "  ${BLUE}help${NC}        Display this help message\n"
}

# Main script execution
main() {
    local command=${1:-"dev"}
    
    case $command in
        "dev")
            run_dev
            ;;
        "build")
            build_production
            ;;
        "clean")
            cleanup
            ;;
        "help")
            show_help
            ;;
        *)
            log "error" "Unknown command: $command"
            show_help
            exit 1
            ;;
    esac
}

# Execute main function with all arguments
main "$@"
