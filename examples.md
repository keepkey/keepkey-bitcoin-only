# KeepKey Vault Signing Workflow Examples

## Overview

This repository supports building and signing releases for both:
- **Main Repository**: `keepkey/keepkey-bitcoin-only` (production releases)
- **Fork Repository**: `BitHighlander/keepkey-bitcoin-only` (testing)

## Quick Examples

### Testing on Fork
```powershell
# Test build and sign on the fork
.\download_sign_upload.ps1 -UseFork -ReleaseTag "v2.2.7-test"

# Monitor fork releases
.\check_release_status.ps1 -UseFork -ReleaseTag "v2.2.7-test"

# Audit fork releases
.\audit_and_sign_releases.ps1 -UseFork -AuditOnly
```

### Production on Main Repo
```powershell
# Build and sign main repo release
.\download_sign_upload.ps1 -ReleaseTag "v2.2.7"

# Monitor main repo releases
.\check_release_status.ps1 -ReleaseTag "v2.2.7"

# Audit main repo releases
.\audit_and_sign_releases.ps1 -AuditOnly
```

## Typical Workflow

### 1. Test on Fork First
```powershell
# 1. Push test release to fork
git push origin release-2.2.7
git tag v2.2.7-test
git push origin v2.2.7-test

# 2. Monitor and sign fork release
.\check_release_status.ps1 -UseFork -ReleaseTag "v2.2.7-test"
```

### 2. Deploy to Main Repo
```powershell
# 1. Push to main repository  
git push upstream release-2.2.7
git tag v2.2.7
git push upstream v2.2.7

# 2. Monitor and sign main release
.\check_release_status.ps1 -ReleaseTag "v2.2.7"
```

## All Script Options

### download_sign_upload.ps1
- `-UseFork`: Use BitHighlander fork instead of main repo
- `-ReleaseTag`: Specific release (default: latest)
- `-DownloadOnly`: Only download, don't sign
- `-CleanStart`: Remove existing output directory

### check_release_status.ps1  
- `-UseFork`: Monitor fork instead of main repo
- `-ReleaseTag`: Release to monitor
- `-CheckIntervalSeconds`: How often to check (default: 60)
- `-MaxWaitMinutes`: Maximum wait time (default: 30)

### audit_and_sign_releases.ps1
- `-UseFork`: Audit fork instead of main repo
- `-AuditOnly`: Only audit, don't sign
- `-Force`: Force re-signing of already signed files

## Repository Structure

```
origin    -> BitHighlander/keepkey-bitcoin-only (fork)
upstream  -> keepkey/keepkey-bitcoin-only (main)
```

Push test releases to `origin`, production releases to `upstream`.