# KeepKey Vault v0.2.0 Release Notes

**Release Date**: December 2024

## 🚀 What's New in v0.2.0

### 🎨 Enhanced User Interface

#### Improved Layout Design
- Transaction review and receive screens now use **horizontal layouts** that better utilize screen space
- Advanced details appear alongside main content instead of stacking vertically
- More efficient use of available screen real estate

#### Dark Mode Enforcement
- The app now **enforces dark mode** regardless of system settings
- Consistent visual experience across all devices and operating systems
- No more light mode flashes on startup

#### Version Display
- Version number now appears in the **window title bar** for easy identification
- Updated from generic "KeepKey Vault v2" to specific "KeepKey Vault v0.2.0"

### 💡 Better User Experience

#### Click-to-Copy Functionality
- Added convenient **copy buttons** for:
  - Bitcoin receive addresses
  - Extended public keys (xPubs)
  - Transaction recipient addresses in send flow
- Visual feedback when copying (button changes to "Copied!" with checkmark)

#### Signing Protection
- Fixed critical issue where signing dialog could disappear during device disconnection
- Dialog now **persists** even if device disconnects
- Users can **retry** without losing transaction context
- Clear messaging about device state during signing process

#### Compact Advanced Views
- Technical details displayed more efficiently
- **Receive screen**: Advanced info appears to the right of QR code
- **Send review**: Two-column layout for transaction details and fees
- Reduced vertical scrolling requirements

### 🔧 Technical Improvements

#### Embedded Firmware Module
- Integrated embedded firmware module for offline firmware updates
- Firmware and bootloader binaries bundled directly in the application
- Supports offline device updates without internet connectivity

#### Dialog State Management
- Improved dialog persistence during device operations
- Better handling of device disconnection scenarios
- More reliable user experience during critical operations

#### Build Configuration
- Updated all version references across the project
- Consistent versioning in package.json, Cargo.toml, and tauri.conf.json

## 📥 Download

### Windows
- **MSI Installer**: `vault-v2_0.2.0_x64_en-US.msi`
- **NSIS Setup**: `vault-v2_0.2.0_x64-setup.exe`

### System Requirements
- Windows 10/11 (64-bit)
- KeepKey hardware wallet
- USB connection

## 🐛 Bug Fixes

- Fixed dialog dismissal during device disconnection
- Resolved layout issues with vertical stacking
- Corrected color mode detection on light-themed systems

## 📝 Known Issues

- Some build warnings in Rust code (non-critical)
- Large bundle size for index.js (optimization planned for future release)

## 🔮 Coming Next

We're continuing to improve the KeepKey Vault experience:
- Performance optimizations
- Additional wallet features
- Enhanced device communication
- Multi-platform support

---

**Thank you for using KeepKey Vault!** Your feedback helps us build a better product.

For support or questions, please visit our [GitHub repository](https://github.com/BitHighlander/keepkey-bitcoin-only). 