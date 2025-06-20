# KeepKey Vault-v2 PowerShell Build Script for Windows
# This script provides Windows-specific build functionality for vault-v2

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
    Write-Host "KeepKey Vault-v2 PowerShell Build Script"
    Write-Host ""
    Write-Host "Usage: .\build.ps1 [OPTIONS]"
    Write-Host ""
    Write-Host "Options:"
    Write-Host "  -Debug         Build in debug mode (runs tauri dev)"
    Write-Host "  -Clean         Clean build artifacts before building"
    Write-Host "  -Help          Show this help message"
    Write-Host ""
    Write-Host "Examples:"
    Write-Host "  .\build.ps1                 # Build vault-v2 for production"
    Write-Host "  .\build.ps1 -Debug          # Run vault-v2 in development mode"
    Write-Host "  .\build.ps1 -Clean          # Clean and build"
    Write-Host "  .\build.ps1 -Clean -Debug   # Clean and run in dev mode"
}

# Show help if requested
if ($Help) {
    Show-Help
    exit 0
}

# Change to vault-v2 directory if not already there
if (-not (Test-Path "package.json") -or -not (Test-Path "src-tauri")) {
    if (Test-Path "projects\vault-v2") {
        Write-Status "Changing to vault-v2 directory..."
        Set-Location "projects\vault-v2"
    } else {
        Write-Error "vault-v2 directory not found. Please run from project root or vault-v2 directory."
        exit 1
    }
}

$BuildType = if ($Debug) { "debug" } else { "release" }
Write-Status "Building KeepKey Vault-v2 for Windows in $BuildType mode"

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

Write-Success "Prerequisites check completed"

# Check if keepkey-rust is built
Write-Status "Checking keepkey-rust dependency..."
if ((Test-Path "..\..\projects\keepkey-rust\target\release\deps") -or (Test-Path "..\..\projects\keepkey-rust\Cargo.toml")) {
    Write-Status "Building keepkey-rust dependency..."
    $originalLocation = Get-Location
    Set-Location "..\..\projects\keepkey-rust"
    try {
        cargo check --all-features
        if ($LASTEXITCODE -ne 0) {
            throw "keepkey-rust build failed"
        }
        Write-Success "keepkey-rust built successfully"
    } catch {
        Write-Error "Failed to build keepkey-rust: $_"
        Set-Location $originalLocation
        exit 1
    }
    Set-Location $originalLocation
} else {
    Write-Warning "keepkey-rust not found, continuing without dependency check"
}

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

try {
    if ($PackageManager -eq "bun") {
        bun install
    } else {
        npm install
    }
    
    if ($LASTEXITCODE -ne 0) {
        throw "Package installation failed"
    }
} catch {
    Write-Error "Failed to install dependencies: $_"
    exit 1
}

Write-Success "Dependencies installed"

# Add Windows Rust target
Write-Status "Adding Windows Rust target..."
rustup target add x86_64-pc-windows-msvc

# Build/Run the application
Write-Status "Building application..."

try {
    if ($Debug) {
        Write-Status "Starting vault-v2 in development mode..."
        if ($PackageManager -eq "bun") {
            bun run tauri:dev
        } else {
            npm run tauri:dev
        }
    } else {
        Write-Status "Building vault-v2 for production..."
        if ($PackageManager -eq "bun") {
            bun run tauri:build
        } else {
            npm run tauri:build
        }
        
        Write-Success "Build completed successfully!"
        
        # Show build artifacts location
        Write-Status "Build artifacts can be found in:"
        Write-Host "  📁 src-tauri\target\x86_64-pc-windows-msvc\release\"
        Write-Host "  📦 src-tauri\target\x86_64-pc-windows-msvc\release\bundle\"
        
        Write-Status "Windows-specific files:"
        Write-Host "  🔧 vault-v2.exe - Main executable"
        Write-Host "  📦 *.msi - Windows installer (if configured)"
    }
    
    Write-Success "KeepKey Vault-v2 build process completed! 🎉"
    
} catch {
    Write-Error "Build failed: $($_.Exception.Message)"
    exit 1
} 