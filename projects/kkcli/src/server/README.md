# Server Module Organization

This directory contains the KeepKey CLI server implementation, organized into logical modules for maintainability.

## Module Structure

- **`mod.rs`** - Main server setup, router configuration, and device utilities
- **`routes/`** - HTTP route handlers for all endpoints
- **`cache/`** - Device data caching implementation
- **`impl_device.rs`** - Device management implementations
- **`impl_addresses.rs`** - Address generation implementations for all chains
- **`impl_bitcoin.rs`** - Bitcoin-specific transaction implementations  
- **`impl_ethereum.rs`** - Ethereum transaction signing implementations
- **`impl_cosmos.rs`** - Cosmos-based chains (Cosmos, Mayachain, Thorchain, etc.)
- **`impl_system.rs`** - System management, debug, and manufacturing implementations

## Key Design Principles

1. **Separation of Concerns** - Each module handles a specific domain
2. **No Mock Data** - All implementations work with real device or return errors
3. **Timeout Protection** - All device operations have 30-second timeouts
4. **Caching** - Address and device data is cached to improve performance
5. **Error Handling** - Explicit error messages for unimplemented features

## Bitcoin Transaction Signing Status

The Bitcoin signing protocol is fully implemented with proper SignTx/TxRequest/TxAck flow. However, it requires previous transaction data to verify input amounts (security requirement). 

Current limitations:
- Previous transaction hex must be provided in the request
- Without blockchain access, cannot fetch previous transactions automatically

The `/utxo/sign-transaction` endpoint is SDK-compatible and accepts amounts as either strings or numbers.

## Adding New Features

When implementing new endpoints:
1. Add route handler in `routes/` module
2. Add implementation in appropriate `impl_*.rs` file
3. Connect route to implementation in route handler
4. Test with real device before marking as complete 