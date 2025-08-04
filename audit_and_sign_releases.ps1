# KeepKey Vault Release Audit and Signing Script
# Downloads releases, signs them locally with Sectigo EV cert, and re-uploads
param(
    [string]$GitHubRepo = "BitHighlander/keepkey-bitcoin-only",
    [string]$ReleaseTag = "",  # Leave empty to process latest release
    [string]$GitHubToken = "",  # GitHub Personal Access Token for uploads
    [string]$Thumbprint = "986AEBA61CF6616393E74D8CBD3A09E836213BAA",
    [string]$TimestampUrl = "http://timestamp.sectigo.com",
    [switch]$AuditOnly = $false,  # Only audit, don't sign or upload
    [switch]$Force = $false  # Force re-signing even if already signed
)

$ErrorActionPreference = "Stop"

Write-Host "=== KeepKey Vault Release Audit & Signing ===" -ForegroundColor Cyan
Write-Host "Repository: $GitHubRepo" -ForegroundColor Gray
Write-Host "Mode: $(if($AuditOnly) {'Audit Only'} else {'Audit + Sign + Upload'})" -ForegroundColor Gray
Write-Host ""

# Ensure required tools are available
$requiredTools = @(
    @{ Name = "git"; Command = "git --version" },
    @{ Name = "gh"; Command = "gh --version" },
    @{ Name = "curl"; Command = "curl --version" }
)

foreach ($tool in $requiredTools) {
    try {
        $null = Invoke-Expression $tool.Command 2>&1
        Write-Host "✅ $($tool.Name) is available" -ForegroundColor Green
    } catch {
        Write-Host "❌ $($tool.Name) not found. Please install GitHub CLI and ensure it's in PATH" -ForegroundColor Red
        exit 1
    }
}
Write-Host ""

# Get release information
if ([string]::IsNullOrEmpty($ReleaseTag)) {
    Write-Host "🔍 Getting latest release information..." -ForegroundColor Yellow
    try {
        $releaseJson = gh release view --repo $GitHubRepo --json tagName,name,assets,draft
        $release = $releaseJson | ConvertFrom-Json
        $ReleaseTag = $release.tagName
        Write-Host "Found latest release: $($release.name) ($ReleaseTag)" -ForegroundColor Green
    } catch {
        Write-Host "❌ Failed to get release information: $_" -ForegroundColor Red
        exit 1
    }
} else {
    Write-Host "🔍 Getting release information for $ReleaseTag..." -ForegroundColor Yellow
    try {
        $releaseJson = gh release view $ReleaseTag --repo $GitHubRepo --json tagName,name,assets,draft
        $release = $releaseJson | ConvertFrom-Json
        Write-Host "Found release: $($release.name)" -ForegroundColor Green
    } catch {
        Write-Host "❌ Release $ReleaseTag not found" -ForegroundColor Red
        exit 1
    }
}

# Create working directory
$workDir = "release_audit_$($ReleaseTag.Replace('v', '').Replace('.', '_'))"
if (Test-Path $workDir) {
    Write-Host "🗂️  Cleaning existing work directory..." -ForegroundColor Yellow
    Remove-Item $workDir -Recurse -Force
}
New-Item -ItemType Directory -Path $workDir | Out-Null
Set-Location $workDir

Write-Host "📁 Working in: $((Get-Location).Path)" -ForegroundColor Gray
Write-Host ""

# Audit and process assets
$windowsAssets = $release.assets | Where-Object { $_.name -like "*.exe" -or $_.name -like "*.msi" }
$signatureResults = @()

if ($windowsAssets.Count -eq 0) {
    Write-Host "⚠️  No Windows executables found in this release" -ForegroundColor Yellow
    exit 0
}

foreach ($asset in $windowsAssets) {
    Write-Host "📦 Processing: $($asset.name)" -ForegroundColor Cyan
    
    # Download asset
    Write-Host "⬇️  Downloading..." -ForegroundColor Yellow
    try {
        gh release download $ReleaseTag --repo $GitHubRepo --pattern $asset.name
        if (-not (Test-Path $asset.name)) {
            throw "Downloaded file not found"
        }
        $fileSize = (Get-Item $asset.name).Length
        Write-Host "   Downloaded: $([math]::Round($fileSize / 1MB, 2)) MB" -ForegroundColor Gray
    } catch {
        Write-Host "   ❌ Download failed: $_" -ForegroundColor Red
        continue
    }
    
    # Check current signature
    Write-Host "🔍 Checking signature..." -ForegroundColor Yellow
    try {
        $signature = Get-AuthenticodeSignature $asset.name
        $isValidSig = $signature.Status -eq 'Valid'
        $statusColor = if ($isValidSig) { 'Green' } else { 'Red' }
        
        Write-Host "   Status: $($signature.Status)" -ForegroundColor $statusColor
        
        if ($signature.SignerCertificate) {
            Write-Host "   Signer: $($signature.SignerCertificate.Subject)" -ForegroundColor Gray
            $isSectigo = $signature.SignerCertificate.Issuer -like "*Sectigo*"
            Write-Host "   Issuer: $($signature.SignerCertificate.Issuer)" -ForegroundColor Gray
            Write-Host "   Sectigo Cert: $(if($isSectigo) {'✅ Yes'} else {'❌ No'})" -ForegroundColor $(if($isSectigo) {'Green'} else {'Red'})
        } else {
            Write-Host "   ❌ No signature found" -ForegroundColor Red
            $isSectigo = $false
        }
        
        $signatureResults += @{
            Asset = $asset.name
            Status = $signature.Status
            IsSectigo = $isSectigo
            Signer = if($signature.SignerCertificate) { $signature.SignerCertificate.Subject } else { "None" }
            NeedsResigning = -not $isValidSig -or (-not $isSectigo -and -not $AuditOnly)
        }
        
    } catch {
        Write-Host "   ❌ Error checking signature: $_" -ForegroundColor Red
        $signatureResults += @{
            Asset = $asset.name
            Status = "Error"
            IsSectigo = $false
            Signer = "Error"
            NeedsResigning = $true
        }
    }
    
    # Sign if needed and not audit-only
    $needsSigning = ($signatureResults[-1].NeedsResigning -or $Force) -and -not $AuditOnly
    
    if ($needsSigning) {
        Write-Host "✍️  Signing with Sectigo certificate..." -ForegroundColor Yellow
        
        try {
            $signArgs = @(
                "sign",
                "/sha1", $Thumbprint,
                "/t", $TimestampUrl,
                "/fd", "sha256",
                "/tr", $TimestampUrl,
                "/td", "sha256",
                "/v",
                $asset.name
            )
            
            $signTool = "C:\Program Files (x86)\Windows Kits\10\bin\10.0.26100.0\x64\signtool.exe"
            if (-not (Test-Path $signTool)) {
                throw "SignTool not found at $signTool"
            }
            
            $result = & $signTool @signArgs 2>&1
            
            if ($LASTEXITCODE -eq 0) {
                Write-Host "   ✅ Signed successfully" -ForegroundColor Green
                
                # Verify new signature
                $newSig = Get-AuthenticodeSignature $asset.name
                Write-Host "   New status: $($newSig.Status)" -ForegroundColor Green
                
                # TODO: Re-upload to GitHub Release (requires GitHub token)
                if (-not [string]::IsNullOrEmpty($GitHubToken)) {
                    Write-Host "   ⬆️  Re-uploading to GitHub..." -ForegroundColor Yellow
                    # Implementation would go here using GitHub API
                    Write-Host "   ⚠️  Re-upload not implemented yet - manual upload required" -ForegroundColor Yellow
                }
                
            } else {
                Write-Host "   ❌ Signing failed: $result" -ForegroundColor Red
            }
            
        } catch {
            Write-Host "   ❌ Signing error: $_" -ForegroundColor Red
        }
    } elseif ($AuditOnly) {
        Write-Host "   ℹ️  Audit mode - skipping signing" -ForegroundColor Blue
    } else {
        Write-Host "   ✅ Already properly signed" -ForegroundColor Green
    }
    
    Write-Host ""
}

# Summary Report
Write-Host "📊 AUDIT SUMMARY FOR $ReleaseTag" -ForegroundColor Cyan
Write-Host "================================" -ForegroundColor Cyan
Write-Host ""

foreach ($result in $signatureResults) {
    $statusEmoji = switch ($result.Status) {
        "Valid" { "✅" }
        "NotSigned" { "❌" }
        "Error" { "⚠️" }
        Default { "❓" }
    }
    
    Write-Host "$statusEmoji $($result.Asset)" -ForegroundColor White
    Write-Host "   Status: $($result.Status)" -ForegroundColor $(if($result.Status -eq 'Valid') {'Green'} else {'Red'})
    Write-Host "   Sectigo: $(if($result.IsSectigo) {'✅ Yes'} else {'❌ No'})" -ForegroundColor $(if($result.IsSectigo) {'Green'} else {'Red'})
    Write-Host "   Signer: $($result.Signer)" -ForegroundColor Gray
    Write-Host ""
}

# Recommendations
$needsAction = $signatureResults | Where-Object { $_.NeedsResigning }
if ($needsAction.Count -gt 0) {
    Write-Host "🚨 ACTIONS NEEDED:" -ForegroundColor Red
    Write-Host "• $($needsAction.Count) file(s) need proper Sectigo signing" -ForegroundColor Yellow
    if ($AuditOnly) {
        Write-Host "• Run again without -AuditOnly to sign files" -ForegroundColor Yellow
    }
    Write-Host "• Consider switching to NSIS (.exe) installers for easier signing" -ForegroundColor Yellow
} else {
    Write-Host "✅ All Windows executables are properly signed with Sectigo certificates!" -ForegroundColor Green
}

Write-Host ""
Write-Host "📁 Files available in: $((Get-Location).Path)" -ForegroundColor Gray

# Return to original directory
Set-Location ".."