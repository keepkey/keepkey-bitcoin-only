@echo off
REM KeepKey Vault-v2 Windows Batch Script
REM Alternative to 'make vault' for Windows systems

echo KeepKey Vault-v2 Windows Build Script
echo =====================================

REM Check if PowerShell is available
powershell -Command "Write-Host 'PowerShell available'" >nul 2>&1
if %errorlevel% neq 0 (
    echo ERROR: PowerShell is not available on this system
    echo Please install PowerShell or use WSL
    pause
    exit /b 1
)

REM Run the PowerShell build script
echo Starting vault-v2 in development mode...
powershell -ExecutionPolicy Bypass -File "skills/build.ps1" -Debug

REM Check if the script failed
if %errorlevel% neq 0 (
    echo.
    echo ERROR: Build script failed
    echo Check the output above for details
    pause
    exit /b 1
)

echo.
echo Build script completed successfully!
pause 