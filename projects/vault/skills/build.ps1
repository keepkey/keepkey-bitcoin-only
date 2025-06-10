# KeepKey Desktop v3 PowerShell Build Script for Windows
# This script provides the same functionality as build.sh but for Windows PowerShell

param(
    [switch]$Debug,
    [switch]$Clean,
    [switch]$Help
)

# Colors for output
$ErrorColor = "Red"
$SuccessColor = "Green"
$WarningColor = "Yellow"
$InfoColor = "Cyan"

function Write-Status {
    param([string]$Message)
    Write-Host "[INFO] $Message" -ForegroundColor $InfoColor
}

function Write-Success {
    param([string]$Message)
    Write-Host "[SUCCESS] $Message" -ForegroundColor $SuccessColor
}

function Write-Warning {
    param([string]$Message)
    Write-Host "[WARNING] $Message" -ForegroundColor $WarningColor
}

function Write-Error {
    param([string]$Message)
    Write-Host "[ERROR] $Message" -ForegroundColor $ErrorColor
}

function Show-Help {
    Write-Host "KeepKey Desktop v3 PowerShell Build Script"
    Write-Host ""
    Write-Host "Usage: .\build.ps1 [OPTIONS]"
    Write-Host ""
    Write-Host "Options:"
    Write-Host "  -Debug         Build in debug mode (default: release)"
    Write-Host "  -Clean         Clean build artifacts before building"
    Write-Host "  -Help          Show this help message"
    Write-Host ""
    Write-Host "Examples:"
    Write-Host "  .\build.ps1                 # Build for Windows in release mode"
    Write-Host "  .\build.ps1 -Debug          # Build in debug mode"
    Write-Host "  .\build.ps1 -Clean          # Clean and build"
    Write-Host "  .\build.ps1 -Clean -Debug   # Clean and build in debug mode"
}

# Show help if requested
if ($Help) {
    Show-Help
    exit 0
}

# Check if we're in the right directory
if (-not (Test-Path "package.json") -or -not (Test-Path "src-tauri")) {
    Write-Error "This script must be run from the project root directory"
    exit 1
}

$BuildType = if ($Debug) { "debug" } else { "release" }
Write-Status "Building KeepKey Desktop v3 for Windows in $BuildType mode"

# Check prerequisites
Write-Status "Checking prerequisites..."

# Check for Rust
try {
    $null = Get-Command rustc -ErrorAction Stop
    $rustVersion = rustc --version
    Write-Status "Found Rust: $rustVersion"
} catch {
    Write-Error "Rust is not installed. Please install Rust from https://rustup.rs/"
    exit 1
}

# Check for package manager (Bun preferred, npm fallback)
$PackageManager = $null
try {
    $null = Get-Command bun -ErrorAction Stop
    $bunVersion = bun --version
    Write-Status "Found Bun: $bunVersion"
    $PackageManager = "bun"
} catch {
    Write-Warning "Bun not found, checking for npm..."
    try {
        $null = Get-Command npm -ErrorAction Stop
        $npmVersion = npm --version
        Write-Status "Found npm: $npmVersion"
        $PackageManager = "npm"
    } catch {
        Write-Error "Neither Bun nor npm is installed. Please install Node.js from https://nodejs.org/"
        exit 1
    }
}

# Check for Tauri CLI
try {
    $null = Get-Command tauri -ErrorAction Stop
    $tauriVersion = tauri --version
    Write-Status "Found Tauri CLI: $tauriVersion"
} catch {
    Write-Status "Installing Tauri CLI..."
    if ($PackageManager -eq "bun") {
        bun add -g @tauri-apps/cli
    } else {
        npm install -g @tauri-apps/cli
    }
}

Write-Success "Prerequisites check completed"

# Clean if requested
if ($Clean) {
    Write-Status "Cleaning build artifacts..."
    
    # Clean Rust artifacts
    if (Test-Path "src-tauri\target") {
        Remove-Item -Recurse -Force "src-tauri\target"
        Write-Status "Cleaned Rust target directory"
    }
    
    # Clean frontend artifacts
    if (Test-Path "dist") {
        Remove-Item -Recurse -Force "dist"
        Write-Status "Cleaned frontend dist directory"
    }
    
    if (Test-Path "node_modules") {
        Remove-Item -Recurse -Force "node_modules"
        Write-Status "Cleaned node_modules"
    }
    
    Write-Success "Clean completed"
}

# Install dependencies
Write-Status "Installing dependencies..."

if ($PackageManager -eq "bun") {
    bun install
} else {
    npm install
}

Write-Success "Dependencies installed"

# Add Windows Rust target
Write-Status "Adding Windows Rust target..."
rustup target add x86_64-pc-windows-msvc

# Build the application
Write-Status "Building application..."

try {
    if ($Debug) {
        if ($PackageManager -eq "bun") {
            bun run tauri dev
        } else {
            npm run tauri dev
        }
    } else {
        if ($PackageManager -eq "bun") {
            bun run tauri build --target x86_64-pc-windows-msvc
        } else {
            npm run tauri build -- --target x86_64-pc-windows-msvc
        }
    }
    
    Write-Success "Build completed successfully!"
    
    # Show build artifacts location
    Write-Status "Build artifacts can be found in:"
    Write-Host "  📁 src-tauri\target\x86_64-pc-windows-msvc\$BuildType\"
    Write-Host "  📦 src-tauri\target\x86_64-pc-windows-msvc\$BuildType\bundle\"
    
    Write-Status "Windows-specific files:"
    Write-Host "  🔧 keepkey-desktop-v3.exe - Main executable"
    Write-Host "  📦 keepkey-desktop-v3_0.1.0_x64_en-US.msi - Windows installer"
    
    Write-Success "KeepKey Desktop v3 build process completed! 🎉"
    
} catch {
    Write-Error "Build failed: $($_.Exception.Message)"
    exit 1
} 