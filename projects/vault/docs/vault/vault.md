Kick off Phase 0 by wiring rusqlcipher into your existing .keepkey home-dir path, guarded behind the new vault feature flag. Then expose vault::create_vault() and vault::unlock() as Tauri commands and wire a minimal Svelte/Chakra prompt to prove the flow.

Once that compiles on all three OS’s, the rest is incremental.

Happy hacking!










Sources
You said:
ok, first step for all this is a generic sqlite with cypher like you recomend, write a quick plan for that in tauri in rust.


ChatGPT said:

Thought for 29 seconds
“Phase 0” mini-roadmap – dropping an encrypted SQLite (SQLCipher) vault into KeepKey Desktop
What you’ll have when done	One file at ~/.keepkey/vault.db (or %APPDATA%\KeepKey\vault.db on Windows) that: <br>• is AES-256-encrypted by SQLCipher <br>• unlocks with Argon2(master-pw) ⊕ bitcoinSignMessage() <br>• is opened through a safe Rust wrapper that every Tauri window can reach over #[tauri::command]

1 Pick the crates
Need	Crate / feature flag	Notes
SQLCipher itself	rusqlcipher = { version = "0.14", features = ["bundled"] }	“bundled” compiles a full SQLCipher + OpenSSL stack for every OS so users don’t need system libs
Docs.rs
KDF	argon2 = "0.5"	argon2id, 256 kB mem, 4 iters is a good default.
Secrets in RAM	zeroize = "1"	Auto-zeros key material on drop.
Path helpers	dirs = "5"	Cross-platform vault location.

Why not tauri-plugin-sql?
Today it ships plain SQLite. Enabling PRAGMA key involves patching its build; easier to talk to SQLCipher directly for now
GitHub
.

2 Cargo/build setup
toml
Copy
Edit
# Cargo.toml (workspace root)
[dependencies]
rusqlcipher  = { version = "0.14", features = ["bundled"] }
argon2       = "0.5"
zeroize      = "1"
dirs         = "5"

[build-dependencies]
sqlcipher-sys = { version = "0.4", features = ["bundled"] }   # pulls the C lib
build.rs (sits in src-tauri/):

rust
Copy
Edit
fn main() {
// Tell cargo to link the bundled SQLCipher we just built
println!("cargo:rustc-link-lib=static=sqlcipher");
}
On Windows you’ll also drop the produced libcrypto-3.dll next to the shipped EXE and list it under tauri.conf.json > tauri > bundle > resources.

3 Vault bootstrap & open
rust
Copy
Edit
// vault.rs
use argon2::{Argon2, PasswordHasher};
use rusqlcipher::{Connection, OpenFlags};
use zeroize::Zeroizing;

pub struct Vault { conn: Connection }

impl Vault {
pub fn create(path: &Path, master_pw: &str, kk_sig: &[u8]) -> anyhow::Result<Self> {
let key = derive(master_pw, kk_sig)?;
let conn = Connection::open_with_flags(
path,
OpenFlags::SQLITE_OPEN_CREATE | OpenFlags::SQLITE_OPEN_READ_WRITE,
)?;
conn.pragma_update(None, "key", &hex::encode(key.as_ref()))?;
conn.execute_batch("PRAGMA cipher_page_size = 4096; PRAGMA kdf_iter = 64000;")?;
Ok(Self { conn })
}

    pub fn unlock(path: &Path, master_pw: &str, kk_sig: &[u8]) -> anyhow::Result<Self> {
        let key = derive(master_pw, kk_sig)?;
        let conn =
            Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_WRITE)?;
        conn.pragma_update(None, "key", &hex::encode(key.as_ref()))?;
        // quick integrity ping
        conn.execute_batch("SELECT count(*) FROM sqlite_master;")?;
        Ok(Self { conn })
    }
}

fn derive(pw: &str, sig: &[u8]) -> anyhow::Result<Zeroizing<Vec<u8>>> {
let salt = sig;                                // 65-byte schnorr sig is perfect salt
let argon = Argon2::default();
let hash = argon.hash_password(pw.as_bytes(), salt)?;
Ok(Zeroizing::new(hash.hash.unwrap().as_bytes().to_vec()))
}
4 Wire it into Tauri
rust
Copy
Edit
// src-tauri/api.rs
#[derive(Default)]
pub struct VaultState(Mutex<Option<Vault>>);

#[tauri::command]
fn create_vault(state: tauri::State<VaultState>, pw: String, kk_sig: String) -> Result<(), String> {
let home = dirs::home_dir().unwrap().join(".keepkey/vault.db");
let kk_sig = hex::decode(kk_sig).map_err(|e| e.to_string())?;
let vault = Vault::create(&home, &pw, &kk_sig).map_err(|e| e.to_string())?;
*state.0.lock().unwrap() = Some(vault);
Ok(())
}

#[tauri::command]
fn unlock_vault(state: tauri::State<VaultState>, pw: String, kk_sig: String) -> Result<(), String> {
let home = dirs::home_dir().unwrap().join(".keepkey/vault.db");
let kk_sig = hex::decode(kk_sig).map_err(|e| e.to_string())?;
let vault = Vault::unlock(&home, &pw, &kk_sig).map_err(|e| e.to_string())?;
*state.0.lock().unwrap() = Some(vault);
Ok(())
}
Add the state:

rust
Copy
Edit
tauri::Builder::default()
.manage(VaultState::default())
.invoke_handler(tauri::generate_handler![create_vault, unlock_vault])
.run(context)?;
5 Frontend smoke-test
ts
Copy
Edit
// invoke from React
await invoke("create_vault", { pw, kkSig });
Enter a password.

Tauri asks the plugged-in KeepKey to Sign Message “Unlock KeepKey Vault <timestamp>”.

Pass that hex sig + password to the command above.

DB file appears, encrypted; reopening without the same pw + sig fails.