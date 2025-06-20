# keepkey-rest: Bitcoin-Only Server Planning and Migration

## Objective
Create a robust, testable, Bitcoin-only REST API library (`keepkey-rest`) to be shared by both Vault and kkcli. Eliminate all multichain logic and hardware/USB dependencies from this crate.

## Key Endpoints to Support
- **Networks**: List supported networks (mainnet, testnet, etc).
- **Paths**: Validate and parse BIP32 derivation paths.
- **Pubkeys**: Return xpubs or public keys for given paths/networks.
- **Balances**: (Stub/mock) Return fake balances for addresses.
- **Signing**: Stub/mock signing for txs and messages (return deterministic mock sigs).

## Requirements
- All endpoints must have clear request/response models.
- All business logic must be covered by robust tests (unit and integration).
- No USB or hardware code—pure logic and mocks.
- Vault should consume `keepkey-rest` as a library and call these endpoints in-process (not HTTP).

## Implementation Plan

### 1. Refactor/Expand `keepkey-rest`
- Organize endpoints in `src/routes/bitcoin.rs` (or submodules).
- Each endpoint gets:
  - Clear request/response models.
  - Mock logic (e.g., signing returns `"MOCK_SIG_<input>"`).
  - Unit tests for all edge cases.

#### Endpoints Sketch
- `GET /networks` → `["bitcoin-mainnet", "bitcoin-testnet"]`
- `POST /parse-path` → Validates/normalizes BIP32 path
- `POST /pubkey` → `{path, network} => {xpub, pubkey}`
- `POST /balance` → `{address, network} => {balance}`
- `POST /sign-tx` → `{inputs, outputs, path, network} => {mock_signature}`
- `POST /sign-message` → `{message, path, network} => {mock_signature}`

### 2. Testing
- Write robust unit tests for all endpoint logic (in `tests/` or as mod tests).
- Include both happy-path and error-path cases.

### 3. Vault Integration
- Add `keepkey-rest` as a dependency in Vault.
- Write a minimal Vault test that calls the exported Rust functions directly (no HTTP needed).

---

## Immediate Actions
- [ ] Stub and document all required endpoints in `keepkey-rest` (`bitcoin.rs` or split files).
- [ ] Add/expand tests for each endpoint.
- [ ] Show example Vault-side usage (Rust test or main).
- [ ] After Vault integration is proven, revisit kkcli integration.

---

*This document will be updated as the migration and implementation progress.*
