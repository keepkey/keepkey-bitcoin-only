# Download, Sign, and Prepare for Upload Script
# Downloads unsigned release from GitHub, signs with Sectigo, prepares for manual upload

param(
    [string]$GitHubRepo = "BitHighlander/keepkey-bitcoin-only",
    [string]$ReleaseTag = "",  # Leave empty for latest release
    [string]$Thumbprint = "986AEBA61CF6616393E74D8CBD3A09E836213BAA",
    [string]$TimestampUrl = "http://timestamp.sectigo.com",
    [string]$OutputDir = "signed_release",
    [switch]$DownloadOnly = $false,  # Only download, don't sign
    [switch]$CleanStart = $false  # Remove existing output directory
)

$ErrorActionPreference = "Stop"

Write-Host "=== KeepKey Vault Release Download & Sign ===" -ForegroundColor Cyan
Write-Host "Repository: $GitHubRepo" -ForegroundColor Gray
Write-Host "Output Directory: $OutputDir" -ForegroundColor Gray
Write-Host ""

# Clean start if requested
if ($CleanStart -and (Test-Path $OutputDir)) {
    Write-Host "🗑️  Cleaning existing output directory..." -ForegroundColor Yellow
    Remove-Item $OutputDir -Recurse -Force
}

# Create output directory
if (-not (Test-Path $OutputDir)) {
    New-Item -ItemType Directory -Path $OutputDir | Out-Null
}

# Get release information
if ([string]::IsNullOrEmpty($ReleaseTag)) {
    Write-Host "🔍 Getting latest release information..." -ForegroundColor Yellow
    try {
        $releaseJson = gh release view --repo $GitHubRepo --json tagName,name,assets,url
        $release = $releaseJson | ConvertFrom-Json
        $ReleaseTag = $release.tagName
        Write-Host "Found latest release: $($release.name) ($ReleaseTag)" -ForegroundColor Green
        Write-Host "Release URL: $($release.url)" -ForegroundColor Gray
    } catch {
        Write-Host "❌ Failed to get release information: $_" -ForegroundColor Red
        Write-Host "💡 Make sure GitHub CLI is installed and authenticated" -ForegroundColor Yellow
        exit 1
    }
} else {
    Write-Host "🔍 Getting release information for $ReleaseTag..." -ForegroundColor Yellow
    try {
        $releaseJson = gh release view $ReleaseTag --repo $GitHubRepo --json tagName,name,assets,url
        $release = $releaseJson | ConvertFrom-Json
        Write-Host "Found release: $($release.name)" -ForegroundColor Green
        Write-Host "Release URL: $($release.url)" -ForegroundColor Gray
    } catch {
        Write-Host "❌ Release $ReleaseTag not found" -ForegroundColor Red
        exit 1
    }
}

Write-Host ""

# Set working directory
Push-Location $OutputDir

try {
    # Download all assets
    Write-Host "📥 Downloading release assets..." -ForegroundColor Yellow
    try {
        gh release download $ReleaseTag --repo $GitHubRepo
        if ($LASTEXITCODE -ne 0) { throw "GitHub CLI download failed" }
        Write-Host "✅ Assets downloaded successfully" -ForegroundColor Green
    } catch {
        Write-Host "❌ Download failed: $_" -ForegroundColor Red
        Pop-Location
        exit 1
    }

    Write-Host ""

    # List downloaded files
    Write-Host "📋 Downloaded assets:" -ForegroundColor Cyan
    $allFiles = Get-ChildItem -File | Sort-Object Name
    foreach ($file in $allFiles) {
        $size = [math]::Round($file.Length / 1MB, 2)
        $icon = switch -Wildcard ($file.Name) {
            "*.exe" { "🖥️" }
            "*.msi" { "🖥️" }
            "*.dmg" { "🍎" }
            "*.deb" { "🐧" }
            "*.AppImage" { "🐧" }
            "*.tar.gz" { "📦" }
            "*.zip" { "📦" }
            Default { "📄" }
        }
        Write-Host "   $icon $($file.Name) ($size MB)" -ForegroundColor Gray
    }
    Write-Host ""

    if ($DownloadOnly) {
        Write-Host "📁 Download complete! Files are in: $((Get-Location).Path)" -ForegroundColor Green
        Pop-Location
        return
    }

    # Find Windows executables to sign
    $windowsFiles = Get-ChildItem -File | Where-Object { 
        $_.Name -like "*.exe" -or $_.Name -like "*.msi" 
    }

    if ($windowsFiles.Count -eq 0) {
        Write-Host "⚠️  No Windows executables found to sign" -ForegroundColor Yellow
        Write-Host "📁 Files are in: $((Get-Location).Path)" -ForegroundColor Gray
        Pop-Location
        return
    }

    Write-Host "✍️  Signing Windows executables..." -ForegroundColor Yellow
    $signTool = "C:\Program Files (x86)\Windows Kits\10\bin\10.0.26100.0\x64\signtool.exe"
    
    if (-not (Test-Path $signTool)) {
        Write-Host "❌ SignTool not found at: $signTool" -ForegroundColor Red
        Write-Host "💡 Install Windows SDK to get SignTool" -ForegroundColor Yellow
        Pop-Location
        exit 1
    }

    $signedCount = 0
    foreach ($file in $windowsFiles) {
        Write-Host ""
        Write-Host "📝 Signing: $($file.Name)" -ForegroundColor Cyan
        
        # Check current signature
        try {
            $currentSig = Get-AuthenticodeSignature $file.FullName
            $isAlreadySigned = $currentSig.Status -eq 'Valid'
            
            if ($isAlreadySigned) {
                Write-Host "   ✅ Already signed by: $($currentSig.SignerCertificate.Subject)" -ForegroundColor Green
                $signedCount++
                continue
            } else {
                Write-Host "   ⚠️  Current status: $($currentSig.Status)" -ForegroundColor Yellow
            }
        } catch {
            Write-Host "   ⚠️  Could not check current signature" -ForegroundColor Yellow
        }

        # Sign the file
        try {
            $signArgs = @(
                "sign",
                "/sha1", $Thumbprint,
                "/fd", "sha256",
                "/tr", $TimestampUrl,
                "/td", "sha256",
                "/v",
                $file.FullName
            )
            
            $result = & $signTool @signArgs 2>&1
            
            if ($LASTEXITCODE -eq 0) {
                Write-Host "   ✅ Signed successfully" -ForegroundColor Green
                $signedCount++
                
                # Verify signature
                $newSig = Get-AuthenticodeSignature $file.FullName
                Write-Host "   Status: $($newSig.Status)" -ForegroundColor Green
            } else {
                Write-Host "   ❌ Signing failed: $result" -ForegroundColor Red
            }
            
        } catch {
            Write-Host "   ❌ Signing error: $_" -ForegroundColor Red
        }
    }

    Write-Host ""

    # Create summary
    Write-Host "📊 SIGNING SUMMARY" -ForegroundColor Cyan
    Write-Host "=================" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "✅ Successfully signed: $signedCount of $($windowsFiles.Count) Windows files" -ForegroundColor Green
    Write-Host "📁 Signed files location: $((Get-Location).Path)" -ForegroundColor Gray
    Write-Host ""

    # Instructions for manual upload
    Write-Host "📋 NEXT STEPS FOR MANUAL UPLOAD:" -ForegroundColor Yellow
    Write-Host "1. Go to the GitHub release: $($release.url)" -ForegroundColor Gray
    Write-Host "2. Click 'Edit release'" -ForegroundColor Gray
    Write-Host "3. Delete the unsigned Windows files" -ForegroundColor Gray
    Write-Host "4. Upload the signed files from this directory:" -ForegroundColor Gray
    
    foreach ($file in $windowsFiles) {
        Write-Host "   📎 $($file.Name)" -ForegroundColor Cyan
    }
    
    Write-Host "5. Update release notes to indicate signed status" -ForegroundColor Gray
    Write-Host "6. Save and publish the release" -ForegroundColor Gray
    Write-Host ""
    Write-Host "🎉 Ready for manual upload!" -ForegroundColor Green

} finally {
    Pop-Location
}