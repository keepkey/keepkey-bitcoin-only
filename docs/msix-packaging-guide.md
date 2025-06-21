# KeepKey Vault MSIX Packaging Guide

This guide documents the complete process for creating MSIX packages for Microsoft Store submission from the KeepKey Vault Tauri application.

## 📋 Overview

MSIX is Microsoft's modern packaging format for Windows applications, designed for Microsoft Store distribution. This guide covers converting a Tauri-built executable into a proper MSIX package.

## 🛠️ Prerequisites

- **Windows 10/11** with Windows SDK installed
- **KeepKey Vault** built in release mode
- **makeappx.exe** (part of Windows SDK)
- **PowerShell** with execution policy allowing scripts

## 📁 Directory Structure

```
keepkey-bitcoin-only/
├── projects/vault-v2/target/release/
│   ├── vault-v2.exe                    # Main executable (20MB)
│   └── bundle/
│       ├── msi/                        # MSI installers
│       └── nsis/                       # NSIS installers
├── msix-package/                       # MSIX package structure
│   ├── AppxManifest.xml                # MSIX manifest
│   ├── vault-v2.exe                    # Application executable
│   └── Assets/                         # Application icons
│       ├── StoreLogo.png
│       ├── Square44x44Logo.png
│       ├── Square150x150Logo.png
│       └── Wide310x150Logo.png
└── KeepKeyVault.msix                   # Final MSIX package
```

## 🔧 Build Process

### Step 1: Build Production Release

```bash
# Build the Tauri application in release mode
.\vault.bat build
# OR
powershell -ExecutionPolicy Bypass -File "skills/build.ps1"
```

**Output**: `projects/vault-v2/target/release/vault-v2.exe` (20MB with embedded firmware)

### Step 2: Prepare MSIX Package Structure

The MSIX package requires specific components:

#### A. Application Executable
- **Source**: `projects/vault-v2/target/release/vault-v2.exe`
- **Destination**: `msix-package/vault-v2.exe`

#### B. Application Icons
Required icon sizes for Microsoft Store:
- **StoreLogo.png**: 50x50 pixels
- **Square44x44Logo.png**: 44x44 pixels  
- **Square150x150Logo.png**: 150x150 pixels
- **Wide310x150Logo.png**: 310x150 pixels

#### C. AppxManifest.xml
The manifest defines package identity, capabilities, and metadata.

### Step 3: Create AppxManifest.xml

```xml
<?xml version="1.0" encoding="utf-8"?>
<Package xmlns="http://schemas.microsoft.com/appx/manifest/foundation/windows10"
         xmlns:uap="http://schemas.microsoft.com/appx/manifest/uap/windows10"
         xmlns:desktop="http://schemas.microsoft.com/appx/manifest/desktop/windows10"
         xmlns:rescap="http://schemas.microsoft.com/appx/manifest/foundation/windows10/restrictedcapabilities">
  <Identity Name="KeepKey.KeepKeyVault"
            Version="0.1.0.0"
            Publisher="CN=04F121F6-2675-4AE6-86AE-5A8EFF7C38E3"
            ProcessorArchitecture="x64" />
  <Properties>
    <DisplayName>KeepKey Vault</DisplayName>
    <PublisherDisplayName>KEY HODLERS LLC</PublisherDisplayName>
    <Logo>Assets\StoreLogo.png</Logo>
    <Description>KeepKey Bitcoin-only hardware wallet vault application</Description>
  </Properties>
  <Dependencies>
    <TargetDeviceFamily Name="Windows.Desktop" MinVersion="10.0.17763.0" MaxVersionTested="10.0.22000.0" />
  </Dependencies>
  <Resources>
    <Resource Language="en-US" />
  </Resources>
  <Capabilities>
    <rescap:Capability Name="runFullTrust" />
    <DeviceCapability Name="humaninterfacedevice">
      <Device Id="any">
        <Function Type="usage:f1d0 *" />
      </Device>
    </DeviceCapability>
  </Capabilities>
  <Applications>
    <Application Id="KeepKeyVault" Executable="vault-v2.exe" EntryPoint="Windows.FullTrustApplication">
      <uap:VisualElements DisplayName="KeepKey Vault"
                          Description="KeepKey Bitcoin-only hardware wallet vault"
                          BackgroundColor="transparent"
                          Square150x150Logo="Assets\Square150x150Logo.png"
                          Square44x44Logo="Assets\Square44x44Logo.png">
        <uap:DefaultTile Wide310x150Logo="Assets\Wide310x150Logo.png" />
      </uap:VisualElements>
    </Application>
  </Applications>
</Package>
```

### Step 4: Build MSIX Package

```powershell
# Using makeappx.exe from Windows SDK
& "C:\Program Files (x86)\Windows Kits\10\bin\10.0.26100.0\x64\makeappx.exe" pack /d "msix-package" /p "KeepKeyVault.msix" /o
```

## 🔐 Capabilities Explained

### runFullTrust
**Required for**: Tauri desktop applications
**Reason**: Tauri apps are packaged desktop applications that need full system access
**Microsoft Store**: Requires approval with justification

### humaninterfacedevice
**Required for**: KeepKey hardware wallet communication
**Reason**: Direct USB/HID communication with KeepKey devices
**Microsoft Store**: Standard capability, no special approval needed

## 📝 Microsoft Store Submission

### Identity Requirements
- **Package Name**: Must match Partner Center reservation: `KeepKey.KeepKeyVault`
- **Publisher**: Must match Partner Center ID: `CN=04F121F6-2675-4AE6-86AE-5A8EFF7C38E3`
- **Publisher Display Name**: Must match Partner Center: `KEY HODLERS LLC`
- **Version**: Semantic versioning: `0.1.0.0`

### runFullTrust Justification
When submitting to Microsoft Store, use this justification:

```
KeepKey Vault requires runFullTrust capability as a Tauri-based desktop application that:

1. Communicates directly with KeepKey hardware wallets via USB/HID protocols
2. Performs secure firmware updates requiring low-level device access
3. Manages cryptographic operations for Bitcoin transactions
4. Operates as a full desktop application packaged for Microsoft Store distribution

This capability is essential for hardware wallet functionality and cannot be achieved through standard UWP APIs.
```

## 🧪 Validation Steps

### 1. Package Structure Validation
```powershell
# Verify all required files exist
Test-Path "msix-package/vault-v2.exe"
Test-Path "msix-package/AppxManifest.xml"
Test-Path "msix-package/Assets/StoreLogo.png"
```

### 2. Manifest Validation
```powershell
# Check manifest syntax and publisher information
[xml]$manifest = Get-Content "msix-package/AppxManifest.xml"
$manifest.Package.Identity.Name
$manifest.Package.Properties.PublisherDisplayName
```

### 3. Package Creation
```powershell
# Build and verify MSIX package
& makeappx pack /d "msix-package" /p "KeepKeyVault.msix" /o
Test-Path "KeepKeyVault.msix"
```

## 📊 Package Information

- **Final Package Size**: ~43MB
- **Architecture**: x64
- **Target OS**: Windows 10 version 1809+ (10.0.17763.0)
- **Language**: English (US)
- **Package Type**: Desktop application (full trust)
- **Publisher**: KEY HODLERS LLC

## 🔄 Automation Script

For automated MSIX creation, use this PowerShell script:

```powershell
# build-msix.ps1
param(
    [string]$Version = "0.1.0.0"
)

Write-Host "Building MSIX package for KeepKey Vault v$Version" -ForegroundColor Green

# Verify prerequisites
$makeappx = "C:\Program Files (x86)\Windows Kits\10\bin\10.0.26100.0\x64\makeappx.exe"
if (!(Test-Path $makeappx)) {
    Write-Error "makeappx.exe not found"
    exit 1
}

# Update version in manifest
$manifestPath = "msix-package\AppxManifest.xml"
[xml]$manifest = Get-Content $manifestPath
$manifest.Package.Identity.Version = $Version
$manifest.Save($manifestPath)

# Verify publisher display name
$publisherDisplayName = $manifest.Package.Properties.PublisherDisplayName
if ($publisherDisplayName -ne "KEY HODLERS LLC") {
    Write-Error "Publisher Display Name must be 'KEY HODLERS LLC', found: $publisherDisplayName"
    exit 1
}

# Build package
& $makeappx pack /d "msix-package" /p "KeepKeyVault.msix" /o

if ($LASTEXITCODE -eq 0) {
    $size = [math]::Round((Get-Item "KeepKeyVault.msix").Length / 1MB, 2)
    Write-Host "✅ MSIX package created: KeepKeyVault.msix ($size MB)" -ForegroundColor Green
} else {
    Write-Error "❌ MSIX package creation failed"
}
```

## ⚠️ Common Validation Errors

### Publisher Display Name Mismatch
**Error**: "PublisherDisplayName element doesn't match your publisher display name"
**Solution**: Ensure `<PublisherDisplayName>KEY HODLERS LLC</PublisherDisplayName>` in manifest

### Missing runFullTrust Justification  
**Warning**: "runFullTrust requires approval"
**Solution**: Provide detailed justification explaining hardware wallet requirements

## 🎯 Success Criteria

- ✅ Package builds without errors
- ✅ All required assets included
- ✅ Manifest validates correctly
- ✅ Package size reasonable (~43MB)
- ✅ Identity matches Microsoft Partner Center
- ✅ Publisher Display Name matches: "KEY HODLERS LLC"
- ✅ Capabilities properly declared

## 📚 References

- [MSIX Packaging Documentation](https://docs.microsoft.com/en-us/windows/msix/)
- [Microsoft Store Submission Process](https://docs.microsoft.com/en-us/windows/uwp/publish/)
- [Tauri MSIX Guide](https://tauri.app/v1/guides/distribution/windows/)
- [Windows App Certification Kit](https://docs.microsoft.com/en-us/windows/uwp/debug-test-perf/windows-app-certification-kit) 