use aes_gcm::{
    aead::{Aead, KeyInit, OsRng, rand_core::RngCore},
    Aes256Gcm, Nonce,
};
use argon2::{
    password_hash::{PasswordHasher, SaltString},
    Argon2,
};
use base64::{Engine as _, engine::general_purpose};
use rusqlite::{Connection, OpenFlags};
use std::path::Path;
use std::sync::Mutex;
use zeroize::Zeroizing;

pub struct Vault {
    conn: Connection,
    cipher: Aes256Gcm,
}

#[derive(Default)]
pub struct VaultState(pub Mutex<Option<Vault>>);

impl Vault {
    /// Create a new encrypted vault database
    pub fn create(path: &Path, master_pw: &str, kk_sig: &[u8]) -> anyhow::Result<Self> {
        let key = derive_key(master_pw, kk_sig)?;
        let cipher = Aes256Gcm::new_from_slice(&key)?;
        
        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let conn = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_CREATE | OpenFlags::SQLITE_OPEN_READ_WRITE,
        )?;
        
        // Create tables - store encrypted values as TEXT
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS settings (
                 key TEXT PRIMARY KEY,
                 value TEXT NOT NULL,
                 nonce TEXT NOT NULL
             );
             CREATE TABLE IF NOT EXISTS accounts (
                 id INTEGER PRIMARY KEY AUTOINCREMENT,
                 label TEXT NOT NULL,
                 public_key TEXT NOT NULL,
                 chain TEXT NOT NULL,
                 created_at INTEGER NOT NULL,
                 nonce TEXT NOT NULL
             );
             CREATE TABLE IF NOT EXISTS vault_metadata (
                 key TEXT PRIMARY KEY,
                 value TEXT NOT NULL
             );
             INSERT OR REPLACE INTO vault_metadata (key, value) VALUES ('version', '1.0');"
        )?;
        
        Ok(Self { conn, cipher })
    }

    /// Unlock an existing vault database
    pub fn unlock(path: &Path, master_pw: &str, kk_sig: &[u8]) -> anyhow::Result<Self> {
        let key = derive_key(master_pw, kk_sig)?;
        let cipher = Aes256Gcm::new_from_slice(&key)?;
        
        let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_WRITE)?;
        
        // Quick integrity check
        conn.execute_batch("SELECT count(*) FROM sqlite_master;")?;
        
        // Verify we can decrypt something (check version)
        let version: String = conn.query_row(
            "SELECT value FROM vault_metadata WHERE key = 'version'",
            [],
            |row| row.get(0),
        )?;
        
        if version != "1.0" {
            return Err(anyhow::anyhow!("Unsupported vault version: {}", version));
        }
        
        Ok(Self { conn, cipher })
    }
    
    /// Check if vault exists at the default location
    pub fn exists() -> bool {
        let path = dirs::home_dir()
            .map(|home| home.join(".keepkey/vault.db"))
            .unwrap_or_default();
        path.exists()
    }
    
    /// Encrypt and store a setting
    pub fn set_setting(&self, key: &str, value: &str) -> anyhow::Result<()> {
        let (encrypted_value, nonce) = self.encrypt_value(value)?;
        
        self.conn.execute(
            "INSERT OR REPLACE INTO settings (key, value, nonce) VALUES (?1, ?2, ?3)",
            (key, encrypted_value, nonce),
        )?;
        
        Ok(())
    }
    
    /// Retrieve and decrypt a setting
    pub fn get_setting(&self, key: &str) -> anyhow::Result<Option<String>> {
        let result = self.conn.query_row(
            "SELECT value, nonce FROM settings WHERE key = ?1",
            [key],
            |row| {
                let encrypted_value: String = row.get(0)?;
                let nonce: String = row.get(1)?;
                Ok((encrypted_value, nonce))
            },
        );
        
        match result {
            Ok((encrypted_value, nonce)) => {
                let decrypted = self.decrypt_value(&encrypted_value, &nonce)?;
                Ok(Some(decrypted))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
    
    /// Encrypt a value and return base64-encoded ciphertext and nonce
    fn encrypt_value(&self, plaintext: &str) -> anyhow::Result<(String, String)> {
        // Generate a random 12-byte nonce
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        let ciphertext = self.cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;
        
        let encrypted_b64 = general_purpose::STANDARD.encode(&ciphertext);
        let nonce_b64 = general_purpose::STANDARD.encode(nonce.as_slice());
        
        Ok((encrypted_b64, nonce_b64))
    }
    
    /// Decrypt a base64-encoded value using the provided nonce
    fn decrypt_value(&self, encrypted_b64: &str, nonce_b64: &str) -> anyhow::Result<String> {
        let ciphertext = general_purpose::STANDARD.decode(encrypted_b64)?;
        let nonce_bytes = general_purpose::STANDARD.decode(nonce_b64)?;
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        let plaintext = self.cipher
            .decrypt(nonce, ciphertext.as_ref())
            .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))?;
        
        Ok(String::from_utf8(plaintext)?)
    }
}

/// Derive encryption key from master password and KeepKey signature
fn derive_key(pw: &str, sig: &[u8]) -> anyhow::Result<Zeroizing<Vec<u8>>> {
    // Use the KeepKey signature as salt (65-byte schnorr sig is perfect salt)
    let salt = SaltString::encode_b64(sig)
        .map_err(|e| anyhow::anyhow!("Failed to encode salt: {}", e))?;
    
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(pw.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!("Failed to hash password: {}", e))?;
    
    // Extract the hash bytes and ensure we have exactly 32 bytes for AES-256
    let hash = password_hash
        .hash
        .ok_or_else(|| anyhow::anyhow!("No hash produced"))?;
    
    let hash_bytes = hash.as_bytes();
    
    // Take the first 32 bytes for AES-256
    let mut key = vec![0u8; 32];
    key.copy_from_slice(&hash_bytes[..32]);
    
    Ok(Zeroizing::new(key))
} 