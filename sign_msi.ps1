# Sign MSI File with EV Certificate
$signtool = "C:\Program Files (x86)\Windows Kits\10\bin\10.0.26100.0\x64\signtool.exe"
$msiPath = "target\release\bundle\msi\"
$thumbprint = "986AEBA61CF6616393E74D8CBD3A09E836213BAA"
$timestamp = "http://timestamp.sectigo.com"

Write-Host "=== EV Certificate MSI Signing ===" -ForegroundColor Cyan
Write-Host ""

# Find the latest MSI file
$msiFiles = Get-ChildItem -Path $msiPath -Filter "*.msi" | Sort-Object LastWriteTime -Descending
if ($msiFiles.Count -eq 0) {
    Write-Host "❌ No MSI files found in $msiPath" -ForegroundColor Red
    exit 1
}

$latestMsi = $msiFiles[0]
Write-Host "📦 Found MSI file: $($latestMsi.Name)" -ForegroundColor Green
Write-Host "📁 Size: $([math]::Round($latestMsi.Length / 1MB, 2)) MB" -ForegroundColor Gray
Write-Host "🕐 Modified: $($latestMsi.LastWriteTime)" -ForegroundColor Gray
Write-Host ""

# Check current signature status
Write-Host "🔍 Checking current signature status..." -ForegroundColor Yellow
$currentSig = Get-AuthenticodeSignature $latestMsi.FullName
Write-Host "Current status: $($currentSig.Status)" -ForegroundColor $(if ($currentSig.Status -eq 'Valid') {'Green'} else {'Red'})
Write-Host ""

# Sign the MSI
Write-Host "✍️  Signing MSI with EV certificate..." -ForegroundColor Yellow
Write-Host "Certificate: $thumbprint" -ForegroundColor Gray
Write-Host "Timestamp: $timestamp" -ForegroundColor Gray
Write-Host ""

try {
    $signArgs = @(
        "sign",
        "/sha1", $thumbprint,
        "/t", $timestamp,
        "/fd", "sha256",
        "/v",
        $latestMsi.FullName
    )
    
    $result = & $signtool @signArgs
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ MSI file signed successfully!" -ForegroundColor Green
        Write-Host ""
        
        # Verify the signature
        Write-Host "🔍 Verifying signature..." -ForegroundColor Cyan
        $signature = Get-AuthenticodeSignature $latestMsi.FullName
        
        Write-Host "Status: $($signature.Status)" -ForegroundColor $(if ($signature.Status -eq 'Valid') {'Green'} else {'Red'})
        if ($signature.SignerCertificate) {
            Write-Host "Signer: $($signature.SignerCertificate.Subject)" -ForegroundColor Gray
            Write-Host "Issuer: $($signature.SignerCertificate.Issuer)" -ForegroundColor Gray
            Write-Host "Valid Until: $($signature.SignerCertificate.NotAfter)" -ForegroundColor Gray
            Write-Host "Thumbprint: $($signature.SignerCertificate.Thumbprint)" -ForegroundColor Gray
        }
        Write-Host ""
        Write-Host "🎉 MSI is ready for distribution!" -ForegroundColor Green
        Write-Host "📁 Location: $($latestMsi.FullName)" -ForegroundColor Cyan
        
    } else {
        Write-Host "❌ Signing failed with exit code: $LASTEXITCODE" -ForegroundColor Red
        Write-Host "Output: $result" -ForegroundColor Gray
    }
} catch {
    Write-Host "❌ Error during signing: $_" -ForegroundColor Red
}