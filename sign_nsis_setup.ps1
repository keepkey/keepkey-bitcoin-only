# Sign NSIS Setup .exe File with Sectigo EV Certificate
param(
    [string]$BuildPath = "projects\vault-v2\src-tauri\target\release\bundle\nsis",
    [string]$Thumbprint = "986AEBA61CF6616393E74D8CBD3A09E836213BAA",
    [string]$TimestampUrl = "http://timestamp.sectigo.com",
    [string]$SignTool = "C:\Program Files (x86)\Windows Kits\10\bin\10.0.26100.0\x64\signtool.exe"
)

Write-Host "=== KeepKey Vault NSIS Setup Signing ===" -ForegroundColor Cyan
Write-Host ""

# Verify signtool exists
if (-not (Test-Path $SignTool)) {
    Write-Host "❌ SignTool not found at: $SignTool" -ForegroundColor Red
    Write-Host "Please install Windows SDK or update the path" -ForegroundColor Yellow
    exit 1
}

# Find the NSIS setup executable
if (-not (Test-Path $BuildPath)) {
    Write-Host "❌ Build path not found: $BuildPath" -ForegroundColor Red
    Write-Host "💡 Run 'bun run tauri build' first to create the installer" -ForegroundColor Yellow
    exit 1
}

$setupFiles = Get-ChildItem -Path $BuildPath -Filter "*setup.exe" | Sort-Object LastWriteTime -Descending
if ($setupFiles.Count -eq 0) {
    Write-Host "❌ No NSIS setup.exe files found in $BuildPath" -ForegroundColor Red
    Write-Host "💡 Make sure Tauri is configured to build NSIS installers" -ForegroundColor Yellow
    exit 1
}

$setupFile = $setupFiles[0]
Write-Host "📦 Found NSIS Setup: $($setupFile.Name)" -ForegroundColor Green
Write-Host "📁 Size: $([math]::Round($setupFile.Length / 1MB, 2)) MB" -ForegroundColor Gray
Write-Host "🕐 Modified: $($setupFile.LastWriteTime)" -ForegroundColor Gray
Write-Host ""

# Check current signature status
Write-Host "🔍 Checking current signature status..." -ForegroundColor Yellow
try {
    $currentSig = Get-AuthenticodeSignature $setupFile.FullName
    $statusColor = if ($currentSig.Status -eq 'Valid') { 'Green' } else { 'Red' }
    Write-Host "Current status: $($currentSig.Status)" -ForegroundColor $statusColor
    
    if ($currentSig.SignerCertificate) {
        Write-Host "Current signer: $($currentSig.SignerCertificate.Subject)" -ForegroundColor Gray
    }
} catch {
    Write-Host "Current status: Not signed" -ForegroundColor Red
}
Write-Host ""

# Verify certificate is available
Write-Host "🔑 Verifying certificate..." -ForegroundColor Yellow
try {
    $cert = Get-ChildItem -Path "Cert:\CurrentUser\My" | Where-Object { $_.Thumbprint -eq $Thumbprint }
    if (-not $cert) {
        $cert = Get-ChildItem -Path "Cert:\LocalMachine\My" | Where-Object { $_.Thumbprint -eq $Thumbprint }
    }
    
    if ($cert) {
        Write-Host "✅ Certificate found: $($cert.Subject)" -ForegroundColor Green
        Write-Host "Valid until: $($cert.NotAfter)" -ForegroundColor Gray
        Write-Host "Issuer: $($cert.Issuer)" -ForegroundColor Gray
    } else {
        Write-Host "❌ Certificate not found with thumbprint: $Thumbprint" -ForegroundColor Red
        Write-Host "💡 Install your Sectigo EV certificate first" -ForegroundColor Yellow
        exit 1
    }
} catch {
    Write-Host "❌ Error checking certificate: $_" -ForegroundColor Red
    exit 1
}
Write-Host ""

# Sign the NSIS setup executable
Write-Host "✍️  Signing NSIS setup with Sectigo EV certificate..." -ForegroundColor Yellow
Write-Host "Certificate: $Thumbprint" -ForegroundColor Gray
Write-Host "Timestamp: $TimestampUrl" -ForegroundColor Gray
Write-Host "File: $($setupFile.FullName)" -ForegroundColor Gray
Write-Host ""

try {
    $signArgs = @(
        "sign",
        "/sha1", $Thumbprint,
        "/fd", "sha256",
        "/tr", $TimestampUrl,
        "/td", "sha256",
        "/v",
        $setupFile.FullName
    )
    
    Write-Host "Running: $SignTool $($signArgs -join ' ')" -ForegroundColor Gray
    $result = & $SignTool @signArgs 2>&1
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ NSIS setup signed successfully!" -ForegroundColor Green
        Write-Host ""
        
        # Verify the signature
        Write-Host "🔍 Verifying signature..." -ForegroundColor Cyan
        $signature = Get-AuthenticodeSignature $setupFile.FullName
        
        $statusColor = if ($signature.Status -eq 'Valid') { 'Green' } else { 'Red' }
        Write-Host "Status: $($signature.Status)" -ForegroundColor $statusColor
        
        if ($signature.SignerCertificate) {
            Write-Host "Signer: $($signature.SignerCertificate.Subject)" -ForegroundColor Gray
            Write-Host "Issuer: $($signature.SignerCertificate.Issuer)" -ForegroundColor Gray
            Write-Host "Valid Until: $($signature.SignerCertificate.NotAfter)" -ForegroundColor Gray
            Write-Host "Thumbprint: $($signature.SignerCertificate.Thumbprint)" -ForegroundColor Gray
        }
        
        if ($signature.TimeStamperCertificate) {
            Write-Host "Timestamp: $($signature.TimeStamperCertificate.Subject)" -ForegroundColor Gray
        }
        
        Write-Host ""
        Write-Host "🎉 Installer is ready for distribution!" -ForegroundColor Green
        Write-Host "📁 Signed file: $($setupFile.FullName)" -ForegroundColor Cyan
        Write-Host ""
        Write-Host "✨ Benefits of NSIS over MSI:" -ForegroundColor Yellow
        Write-Host "   • Creates desktop shortcut automatically" -ForegroundColor Gray
        Write-Host "   • Adds to Start Menu" -ForegroundColor Gray
        Write-Host "   • Proper install/uninstall via Control Panel" -ForegroundColor Gray
        Write-Host "   • No firmware bundling issues during signing" -ForegroundColor Gray
        Write-Host "   • Better user experience on Windows" -ForegroundColor Gray
        
    } else {
        Write-Host "❌ Signing failed with exit code: $LASTEXITCODE" -ForegroundColor Red
        Write-Host "Output:" -ForegroundColor Gray
        $result | ForEach-Object { Write-Host "  $_" -ForegroundColor Gray }
        exit 1
    }
} catch {
    Write-Host "❌ Error during signing: $_" -ForegroundColor Red
    exit 1
}