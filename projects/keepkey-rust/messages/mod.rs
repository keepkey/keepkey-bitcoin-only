mod encoding;
mod macros;
mod protos;
mod timeouts;

pub use protos::*;

use macros::kk_message;

kk_message!(
    // Core device messages
    Initialize,
    Ping,
    Success,
    Failure,
    GetFeatures,
    Features,
    
    // Device management
    ChangePin,
    WipeDevice,
    ApplySettings,
    ApplyPolicies,
    ChangeWipeCode,
    
    // Firmware and bootloader
    FirmwareErase,
    FirmwareUpload,
    SoftReset,
    
    // Entropy and random
    GetEntropy,
    Entropy,
    EntropyRequest,
    EntropyAck,
    
    // Recovery and setup
    LoadDevice,
    ResetDevice,
    RecoveryDevice,
    WordRequest,
    WordAck,
    
    // PIN and passphrase
    PinMatrixRequest,
    PinMatrixAck,
    PassphraseRequest,
    PassphraseAck,
    
    // User interaction
    ButtonRequest,
    ButtonAck,
    CharacterRequest,
    CharacterAck,
    Cancel,
    
    // Session management
    ClearSession,
    
    // Bitcoin-specific messages
    GetPublicKey,
    PublicKey,
    GetAddress,
    Address,
    SignTx,
    TxRequest,
    TxAck,
    RawTxAck,
    SignMessage,
    VerifyMessage,
    MessageSignature,
    
    // Encryption/Decryption
    CipherKeyValue,
    CipheredKeyValue,
    EncryptMessage,
    EncryptedMessage,
    DecryptMessage,
    DecryptedMessage,
    
    // Identity
    SignIdentity,
    SignedIdentity,
    
    // Device info
    GetCoinTable,
    CoinTable,
    
    // Flash operations
    FlashHash,
    FlashWrite,
    FlashHashResponse,
    
    // Debug (if needed for development)
    DebugLinkFlashDump,
    DebugLinkFlashDumpResponse,
    DebugLinkDecision,
    DebugLinkGetState,
    DebugLinkState,
    DebugLinkStop,
    DebugLinkLog,
    DebugLinkFillConfig
);
