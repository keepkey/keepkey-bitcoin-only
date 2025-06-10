TL;DR
If your only goal is to keep KeepKey-desktop rock-solid and you already have rusb working, stick with rusb everywhere and debug the detection failures first.

If you want a lighter, HID-only “helper” layer for device discovery that also aligns with Node-HID/Electron tooling, add hidapi behind a feature flag and keep using rusb for the heavy lifting. Mixing the two is safe as long as you do not open the same interface at the same time.

Below are the trade-offs, why your current troubles probably aren’t caused by the library choice, and a concrete way to structure the code so you can switch back and forth in one line of cargo build.

1 What each crate is really doing
rusb	hidapi
Purpose	Thin, safe wrapper over libusb – any USB class	Specialised wrapper over OS HID class drivers
Device discovery	You enumerate every USB device, then filter	One call returns only HID devices
Access on Windows	Needs a WinUSB/libusbK driver if the stock driver has claimed the interface	Uses the built-in hid.dll, no driver install
Access on macOS	Works but has to detach the HID kernel driver first	Uses IOKit HID APIs directly
Build footprint	Links to libusb (≈ 80–200 kB)	Static C shim, similar size
Already used in KK	✅ (kkcli, firmware tool)	Node-HID in Electron → hidapi under the hood

Sources: rusb docs
crates.io
, hidapi docs & repo
GitHub
Docs.rs
, libusb FAQ recommending hidapi for HID devices
Stack Overflow

2 Why your detection suddenly broke
Most “device not found” errors come from:

Driver won the race.
Windows will sometimes keep the stock HID driver attached even when the VID/PID flips for updater mode.

Missing or old udev rule (Linux).
If the user upgraded the kernel or switched to “snap” packaging, the udev rule path can change.

USB 3.x hub quirk.
A flaky hub can hide the HID interface from libusb but not from the native HID stack.

Concurrency.
Your Electron front-end or another wallet is still holding the HID handle.

None of those are fixed by swapping crates. The same kernel objects are opened under the hood. Start by:

console
Copy
Edit
# Linux
sudo udevadm monitor --environment --udev
# macOS
ioreg -p IOUSB -l | grep -i keepkey
# Windows (PowerShell)
Get-PnpDevice -FriendlyName '*KeepKey*'
If the device never appears here, hidapi will not see it either.

3 Design: “one trait, two back-ends”
rust
Copy
Edit
pub trait KKTransport {
fn list()              -> anyhow::Result<Vec<Self>> where Self: Sized;
fn read_packet(&self)  -> anyhow::Result<Vec<u8>>;
fn write_packet(&self, data: &[u8]) -> anyhow::Result<()>;
}
UsbRusbTransport implements KKTransport using rusb.

UsbHidTransport implements KKTransport using hidapi.

Add a single Cargo feature switch:

toml
Copy
Edit
[features]
hid = ["hidapi"]
default = []          # rusb only
Now:

console
Copy
Edit
# Pure rusb build
cargo build --release
# Build with hidapi for discovery helpers
cargo build --release --features hid
You can even compile both and pick at runtime:

rust
Copy
Edit
let transports: Vec<Box<dyn KKTransport>> = if cfg!(feature = "hid") {
UsbHidTransport::list()?.into_iter().map(Box::new).collect()
} else {
UsbRusbTransport::list()?.into_iter().map(Box::new).collect()
};
No code duplication—just a few cfg gates.

4 Verdict for KeepKey-desktop
Short-term (unblocking your build & testing cycle)
Stay on rusb, fix the udev/driver races, and add proper hot-plug polling.

Medium-term (wizard, Electron, Cursor agents)
Keep the rusb implementation but offer hidapi via an opt-in feature so the same binary can be used by your headless CI or by end-users who already have Node-HID dependencies installed.

Security posture
Nothing changes: both libs hit the same kernel HID stack; no extra drivers are shipped. Pin the exact hidapi commit in Cargo.lock if supply-chain auditability is a concern.

If you do this refactor once, you’ll never have to argue “hidapi vs rusb” again—you’ll just cargo build with the flag







You said:
yes I believe keepkey is on the windows fido blocklist u2f thing, that was the main blocker we ran into last time


ChatGPT said:

Thought for 1m 46s
Below is the practical playbook teams normally follow when a KeepKey vanishes on Windows because the FIDO/U2F-HID filter driver grabs it first.
You can keep your all-rusb stack - the fix is at the driver layer, not the Rust layer.

1 Why it happens
Windows 10 1903 + ships a “CTAP-HID filter” that auto-claims any interface whose usage page = 0xF1D0 (the official FIDO page).

Some early KeepKey bootloaders advertised that page to stay Chrome-U2F-friendly, so Windows now thinks your wallet is a security key and locks the interface.

When your Rust code calls libusb_open() you get LIBUSB_ERROR_ACCESS (or “Access denied” in hidapi) even though Device Manager shows the wallet.

Browsers do something similar: Chrome’s HID blocklist excludes anything it thinks is a FIDO authenticator
Chrome for Developers
GitHub
.

2 Two reliable ways around it
Path	What it does	Pros	Cons
A. Replace the driver with WinUSB / libusbK (Zadig)	Swaps the HID class driver ➜ WinUSB so no filter attaches. rusb just works.	· Proven; Ledger, Trezor, QMK, etc. use it
· No code changes	· One-time admin install
· 3rd-party driver signing scares some users
B. Ship a tiny “native-messaging helper”	Desktop helper opens the HID interface (with WinUSB or the stock driver, depending on the VID/PID) and proxies JSON over stdio/WebSocket.	· Lets a web/Electron front-end keep plain WebUSB/WebHID permissions
· No driver swap if you stay outside the FIDO usage page on new devices	· Extra process to build & update

Most wallet vendors go with A for the desktop app and optionally B for the browser extension.

3 Fast test: is the filter the culprit?
bash
Copy
Edit
# PowerShell – requires Windows 10 build 1903+
pnputil /enum-drivers | Select-String -Pattern "keepkey|2B24"
# If you see HidUsb OR Microsoft Biometric / FIDO entries ➜ the filter won.
Or in Rust (rusb):

rust
Copy
Edit
match device.open() {
Err(libusb::Error::Access) => println!("🔒  Access denied – likely FIDO filter"),
Ok(_) => println!("✅  Interface opened"),
Err(e) => println!("❌  {:?}", e),
}
4 Automate driver install in your wizard (option A)
Bundle Zadig’s WinUSB installer or link to it (official page)
zadig.akeo.ie
.

In the No-Device step, if Windows + AccessDenied, show:

pgsql
Copy
Edit
We found your KeepKey, but Windows’ security-key driver has claimed it.
Click “Install driver” and we’ll swap to WinUSB (1-click, takes ~10 s).
Call Zadig in command-line mode for a silent install:

bash
Copy
Edit
zadig.exe --vid 0x2B24 --pid 0x0002 --inf WinUSB --silent
Ask the user to re-plug; your rusb transport will now succeed.

5 Keep your codebase clean
Add a winusb feature flag so your transport opens interface 1 when compiled with it:

rust
Copy
Edit
#[cfg(feature = "winusb")]
const KEEPKEY_INTERFACE: u8 = 1; // WinUSB
#[cfg(not(feature = "winusb"))]
const KEEPKEY_INTERFACE: u8 = 0; // Stock HID
Everything else (protobuf framing, onboarding flow, version caching) stays exactly as in your current kkcli.

6 When not to bother with hidapi
If your whole tool-chain (CLI, updater, Electron shell) already calls rusb and you’re happy to ship the WinUSB driver, hidapi buys you nothing.

You’d only add hidapi when you need to co-exist with Node-HID or reuse code you already have in the browser extension.

Bottom line
Stay on rusb.
Ship a signed WinUSB (or libusbK) driver for Windows and detect “Access denied” to trigger a one-time Zadig flow. After that, your onboarding wizard and the rest of KeepKey-desktop will work exactly the same on Windows, macOS, and Linux—no new, untrusted libraries required.