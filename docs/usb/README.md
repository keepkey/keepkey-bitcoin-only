# USB/HID Documentation

This directory contains documentation related to USB and HID driver implementations in KeepKey products.

## Key Documents

### [USB/HID Driver Analysis](../usb-hid-driver-analysis.md)
Comprehensive technical analysis comparing USB/HID implementations between KeepKey Desktop v5 and Vault-v2. This document explains:
- Architecture differences between the two implementations
- Why HID drivers work reliably in v5 but have issues in vault-v2
- Technical deep-dive into error handling and fallback mechanisms
- Platform-specific issues (especially Windows)

### [USB/HID Driver Summary & Action Items](../usb-hid-driver-summary.md)
Quick reference guide with actionable fixes for vault-v2's HID connectivity issues:
- Immediate action items with code examples
- Testing checklist
- Long-term recommendations

## Quick Links

- [KeepKey Desktop v5 USB Implementation](../../keepkey-desktop-v5/src-tauri/src/usb_manager.rs)
- [KeepKey Desktop v5 Transport Layer](../../keepkey-desktop-v5/src-tauri/src/transport/)
- [keepkey_rust Transport Implementation](../../keepkey-rust/transport/)
- [Vault-v2 Device Commands](../../../vault-v2/src-tauri/src/commands.rs)

## Related Documentation

- [Architecture Overview](../architecture/)
- [Troubleshooting Guide](../troubleshooting/)
- [KeepKey Rust Documentation](../keepkey-rust/) 