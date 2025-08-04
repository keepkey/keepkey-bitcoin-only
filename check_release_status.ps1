# Check Release Status and Download When Ready
# Monitors GitHub release status and downloads unsigned builds for signing

param(
    [string]$GitHubRepo = "keepkey/keepkey-bitcoin-only",  # Main repo
    [string]$ReleaseTag = "v2.2.7-test",
    [int]$CheckIntervalSeconds = 60,
    [int]$MaxWaitMinutes = 30,
    [switch]$DownloadWhenReady = $true,
    [switch]$UseFork = $false  # Use BitHighlander fork for testing
)

# Handle fork/main repo selection
if ($UseFork) {
    $GitHubRepo = "BitHighlander/keepkey-bitcoin-only"
    Write-Host "🔄 Monitoring FORK for testing: $GitHubRepo" -ForegroundColor Yellow
} else {
    Write-Host "🎯 Monitoring MAIN repository: $GitHubRepo" -ForegroundColor Green
}

Write-Host "=== KeepKey Vault Release Monitor ===" -ForegroundColor Cyan
Write-Host "Repository: $GitHubRepo" -ForegroundColor Gray
Write-Host "Release Tag: $ReleaseTag" -ForegroundColor Gray
Write-Host "Check Interval: $CheckIntervalSeconds seconds" -ForegroundColor Gray
Write-Host ""

$repoUrl = "https://github.com/$GitHubRepo"
$releaseUrl = "$repoUrl/releases/tag/$ReleaseTag"
$actionsUrl = "$repoUrl/actions"

Write-Host "📋 Manual Monitoring Links:" -ForegroundColor Yellow
Write-Host "🔗 GitHub Actions: $actionsUrl" -ForegroundColor Cyan
Write-Host "🔗 Release Page: $releaseUrl" -ForegroundColor Cyan
Write-Host ""

# Check if GitHub CLI is available
try {
    $null = Get-Command gh -ErrorAction Stop
    $hasGhCli = $true
    Write-Host "✅ GitHub CLI found - will auto-check release status" -ForegroundColor Green
} catch {
    $hasGhCli = $false
    Write-Host "⚠️  GitHub CLI not found - manual monitoring only" -ForegroundColor Yellow
    Write-Host "💡 Install GitHub CLI for automated checking: https://cli.github.com/" -ForegroundColor Gray
}

Write-Host ""

if (-not $hasGhCli) {
    Write-Host "🔍 MANUAL STEPS TO MONITOR RELEASE:" -ForegroundColor Yellow
    Write-Host "1. Open GitHub Actions: $actionsUrl" -ForegroundColor Gray
    Write-Host "2. Look for 'Release KeepKey Vault v5' workflow running" -ForegroundColor Gray
    Write-Host "3. Wait for all builds (Windows, macOS, Linux) to complete" -ForegroundColor Gray
    Write-Host "4. Check the release page: $releaseUrl" -ForegroundColor Gray
    Write-Host "5. When ready, run the download script:" -ForegroundColor Gray
    Write-Host "   .\download_sign_upload.ps1 -ReleaseTag $ReleaseTag" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "🎯 Expected Artifacts:" -ForegroundColor Yellow
    Write-Host "   🖥️  vault-v2_2.2.7_x64-setup.exe (Windows NSIS installer)" -ForegroundColor Gray
    Write-Host "   🍎 vault-v2_2.2.7.dmg (macOS universal)" -ForegroundColor Gray  
    Write-Host "   🐧 vault-v2_2.2.7.AppImage (Linux)" -ForegroundColor Gray
    Write-Host ""
    return
}

# Automated monitoring with GitHub CLI
$maxChecks = [math]::Ceiling($MaxWaitMinutes * 60 / $CheckIntervalSeconds)
$checkCount = 0

Write-Host "🤖 Starting automated monitoring..." -ForegroundColor Green
Write-Host "Will check every $CheckIntervalSeconds seconds for up to $MaxWaitMinutes minutes" -ForegroundColor Gray
Write-Host ""

while ($checkCount -lt $maxChecks) {
    $checkCount++
    $timeRemaining = [math]::Max(0, $MaxWaitMinutes - ($checkCount * $CheckIntervalSeconds / 60))
    
    Write-Host "[$($checkCount.ToString().PadLeft(2))/$maxChecks] Checking release status... (${timeRemaining:F1} min remaining)" -ForegroundColor Yellow
    
    try {
        # Check if release exists
        $releaseJson = gh release view $ReleaseTag --repo $GitHubRepo --json assets,draft,prerelease 2>$null
        
        if ($LASTEXITCODE -eq 0) {
            $release = $releaseJson | ConvertFrom-Json
            $assetCount = $release.assets.Count
            $isDraft = $release.draft
            $isPrerelease = $release.prerelease
            
            Write-Host "   ✅ Release found with $assetCount assets" -ForegroundColor Green
            
            if ($assetCount -gt 0) {
                Write-Host "   📦 Available assets:" -ForegroundColor Cyan
                foreach ($asset in $release.assets) {
                    $sizeMB = [math]::Round($asset.size / 1MB, 1)
                    Write-Host "      📄 $($asset.name) (${sizeMB} MB)" -ForegroundColor Gray
                }
                
                # Check for expected Windows asset
                $windowsAsset = $release.assets | Where-Object { $_.name -like "*setup.exe" -or $_.name -like "*.msi" }
                $macosAsset = $release.assets | Where-Object { $_.name -like "*.dmg" }
                $linuxAsset = $release.assets | Where-Object { $_.name -like "*.AppImage" }
                
                if ($windowsAsset -and $macosAsset -and $linuxAsset) {
                    Write-Host "   🎉 All expected platforms found!" -ForegroundColor Green
                    
                    if ($DownloadWhenReady) {
                        Write-Host ""
                        Write-Host "⬇️  Starting download and signing process..." -ForegroundColor Cyan
                        .\download_sign_upload.ps1 -ReleaseTag $ReleaseTag
                        return
                    } else {
                        Write-Host ""
                        Write-Host "✅ Release is ready! Run this command to download and sign:" -ForegroundColor Green
                        Write-Host ".\download_sign_upload.ps1 -ReleaseTag $ReleaseTag" -ForegroundColor Cyan
                        return
                    }
                } else {
                    Write-Host "   ⏳ Waiting for all platforms..." -ForegroundColor Yellow
                    if (-not $windowsAsset) { Write-Host "      ⏳ Windows build pending" -ForegroundColor Gray }
                    if (-not $macosAsset) { Write-Host "      ⏳ macOS build pending" -ForegroundColor Gray }
                    if (-not $linuxAsset) { Write-Host "      ⏳ Linux build pending" -ForegroundColor Gray }
                }
            } else {
                Write-Host "   ⏳ Release exists but no assets yet..." -ForegroundColor Yellow
            }
        } else {
            Write-Host "   ⏳ Release not found yet..." -ForegroundColor Yellow
        }
        
    } catch {
        Write-Host "   ❌ Error checking release: $_" -ForegroundColor Red
    }
    
    if ($checkCount -lt $maxChecks) {
        Write-Host "   Waiting $CheckIntervalSeconds seconds..." -ForegroundColor Gray
        Start-Sleep -Seconds $CheckIntervalSeconds
    }
    
    Write-Host ""
}

Write-Host "⏰ Timeout reached after $MaxWaitMinutes minutes" -ForegroundColor Yellow
Write-Host "📋 Manual check needed:" -ForegroundColor Gray
Write-Host "   🔗 Actions: $actionsUrl" -ForegroundColor Cyan
Write-Host "   🔗 Release: $releaseUrl" -ForegroundColor Cyan