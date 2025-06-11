[2025-06-11][23:45:43][keepkey_gui_lib][INFO]  | lib |  set‑up phase
[2025-06-11][23:45:43][keepkey_gui_lib::usb_manager][INFO] Starting rusb hotplug listener
[2025-06-11][23:45:43][keepkey_gui_lib::device_controller][INFO] DeviceController starting...
[2025-06-11][23:45:43][keepkey_gui_lib::device_controller][INFO] 0 device(s) connected
[2025-06-11][23:45:43][keepkey_gui_lib][INFO]  | lib |  Starting server...
[2025-06-11][23:45:43][keepkey_gui_lib::server][INFO] Starting simple Bitcoin-focused server on port 1646
[2025-06-11][23:45:43][keepkey_gui_lib::index_db][INFO] Opening database at: "/Users/highlander/.keepkey/index.db"
[2025-06-11][23:45:43][keepkey_gui_lib::usb_manager][INFO] Device arrived: KeyHodlers, LLC - KeepKey - KeepKey3 (2b24:0002) ID: 343737340F4736331F003B00
[2025-06-11][23:45:43][keepkey_gui_lib::device_controller][INFO] Device connected: KeyHodlers, LLC - KeyHodlers, LLC - KeepKey - KeepKey3 (343737340F4736331F003B00)
[2025-06-11][23:45:43][keepkey_gui_lib::usb_manager][INFO] Device arrived: Razer - Razer Huntsman V3 Pro Mini (1532:02b0) ID: bus0_addr4
[2025-06-11][23:45:43][keepkey_gui_lib::usb_manager][INFO] Device arrived: Razer Basilisk V3 Pro 35K (1532:00cc) ID: 000000000000
[2025-06-11][23:45:43][keepkey_gui_lib::usb_manager][INFO] Device arrived: GenesysLogic - USB2.1 Hub (05e3:0610) ID: bus0_addr2
[2025-06-11][23:45:43][keepkey_gui_lib::usb_manager][INFO] Device arrived: GenesysLogic - USB3.1 Hub (05e3:0626) ID: bus0_addr1
[2025-06-11][23:45:43][keepkey_gui_lib::usb_manager][INFO] Device arrived: Yubico - Yubikey 4 OTP+U2F+CCID (1050:0407) ID: bus2_addr1
[2025-06-11][23:45:43][keepkey_gui_lib::usb_manager][INFO] rusb hotplug registration successful
[2025-06-11][23:45:43][keepkey_gui_lib::server::bitcoin][INFO] Loaded 4 default paths
[2025-06-11][23:45:43][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: cache_dir resolved to: /Users/highlander/.keepkey/kkcli
[2025-06-11][23:45:43][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: db_path resolved to: /Users/highlander/.keepkey/kkcli/device_cache.db
[2025-06-11][23:45:43][keepkey_gui_lib::cache::device_cache][INFO] Opening device cache at: /Users/highlander/.keepkey/kkcli/device_cache.db
[2025-06-11][23:45:43][keepkey_gui_lib::server][INFO] Setting initial device context to first available device: 343737340F4736331F003B00
[2025-06-11][23:45:43][keepkey_gui_lib::index_db][INFO] Recording device connection: 343737340F4736331F003B00
[2025-06-11][23:45:43][keepkey_gui_lib::index_db][INFO] Device 343737340F4736331F003B00 connected and recorded
[2025-06-11][23:45:43][keepkey_gui_lib::device_controller][INFO] Device connected: Razer - Razer - Razer Huntsman V3 Pro Mini (bus0_addr4)
[2025-06-11][23:45:43][keepkey_gui_lib::device_controller][INFO] Device connected: Unknown - Razer Basilisk V3 Pro 35K (000000000000)
[2025-06-11][23:45:43][keepkey_gui_lib::device_controller][INFO] Device connected: GenesysLogic - GenesysLogic - USB2.1 Hub (bus0_addr2)
[2025-06-11][23:45:43][keepkey_gui_lib::device_controller][INFO] Device connected: GenesysLogic - GenesysLogic - USB3.1 Hub (bus0_addr1)
[2025-06-11][23:45:43][keepkey_gui_lib::device_controller][INFO] Device connected: Yubico - Yubico - Yubikey 4 OTP+U2F+CCID (bus2_addr1)
[2025-06-11][23:45:43][keepkey_gui_lib::server::context][INFO] Getting device features to save to database for device 343737340F4736331F003B00
[2025-06-11][23:45:43][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Starting with message type: GetFeatures
[2025-06-11][23:45:43][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Calling parent adapter...
[2025-06-11][23:45:43][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Processing message type: GetFeatures
[2025-06-11][23:45:43][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::send: Sending message type: GetFeatures
[2025-06-11][23:45:43][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Waiting for response (timeout: 5s)...
[2025-06-11][23:45:43][keepkey_gui_lib::commands][INFO] === Checking if first time install ===
[2025-06-11][23:45:43][keepkey_gui_lib::commands][INFO] Database file exists: true
[2025-06-11][23:45:43][keepkey_gui_lib::commands][INFO] === First time install result: false ===
[2025-06-11][23:45:43][keepkey_gui_lib::commands][INFO] === Checking if user is onboarded ===
[2025-06-11][23:45:43][keepkey_gui_lib::commands][INFO] === Onboarded status: true ===
[2025-06-11][23:45:44][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Received 289 bytes response
[2025-06-11][23:45:44][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Decoded response type: Features
[2025-06-11][23:45:44][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Parent returned message type: Features
[2025-06-11][23:45:44][keepkey_gui_lib::transport][INFO] StandardHandler: Processing message type: Features
[2025-06-11][23:45:44][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Handler returned None, done
[2025-06-11][23:45:44][keepkey_gui_lib::server::context][INFO] ✅ Got device features for 343737340F4736331F003B00
[2025-06-11][23:45:44][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: cache_dir resolved to: /Users/highlander/.keepkey/kkcli
[2025-06-11][23:45:44][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: db_path resolved to: /Users/highlander/.keepkey/kkcli/device_cache.db
[2025-06-11][23:45:44][keepkey_gui_lib::cache::device_cache][INFO] Opening device cache at: /Users/highlander/.keepkey/kkcli/device_cache.db
[2025-06-11][23:45:44][keepkey_gui_lib::cache::device_cache][INFO] Saved features for device 343737340F4736331F003B00
[2025-06-11][23:45:44][keepkey_gui_lib::server::context][INFO] ✅ Saved device features to database for 343737340F4736331F003B00
[2025-06-11][23:45:44][keepkey_gui_lib::server::context][INFO] Getting real Ethereum address from device 343737340F4736331F003B00
[2025-06-11][23:45:44][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Starting with message type: EthereumGetAddress
[2025-06-11][23:45:44][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Calling parent adapter...
[2025-06-11][23:45:44][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Processing message type: EthereumGetAddress
[2025-06-11][23:45:44][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::send: Sending message type: EthereumGetAddress
[2025-06-11][23:45:44][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Waiting for response (timeout: 5s)...
[2025-06-11][23:45:44][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Received 74 bytes response
[2025-06-11][23:45:44][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Decoded response type: EthereumAddress
[2025-06-11][23:45:44][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Parent returned message type: EthereumAddress
[2025-06-11][23:45:44][keepkey_gui_lib::transport][INFO] StandardHandler: Processing message type: EthereumAddress
[2025-06-11][23:45:44][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Handler returned None, done
[2025-06-11][23:45:44][keepkey_gui_lib::server::context][INFO] ✅ Got real Ethereum address from device 343737340F4736331F003B00: 0x141d9959cae3853b035000490c03991eb70fc4ac
[2025-06-11][23:45:44][keepkey_gui_lib::server::context][INFO] Device context set: device_id=343737340F4736331F003B00, eth_address=0x141d9959cae3853b035000490c03991eb70fc4ac, label=None
[2025-06-11][23:45:44][keepkey_gui_lib::server][INFO] Initial device context set successfully for device 343737340F4736331F003B00
[2025-06-11][23:45:44][keepkey_gui_lib::server][INFO] 🚀 Server started successfully:
[2025-06-11][23:45:44][keepkey_gui_lib::server][INFO]   📋 REST API: http://127.0.0.1:1646/api
[2025-06-11][23:45:44][keepkey_gui_lib::server][INFO]   📚 API Documentation: http://127.0.0.1:1646/docs
[2025-06-11][23:45:44][keepkey_gui_lib::server][INFO]   🎯 Device Context: http://127.0.0.1:1646/api/context
[2025-06-11][23:45:44][keepkey_gui_lib::server][INFO]   🔐 Authentication: http://127.0.0.1:1646/auth/pair
[2025-06-11][23:45:44][keepkey_gui_lib::server][INFO]   🔑 V1 API (Legacy): http://127.0.0.1:1646/addresses/utxo
[2025-06-11][23:45:44][keepkey_gui_lib::server][INFO]   🔑 V2 API (Modern): http://127.0.0.1:1646/api/v2
[2025-06-11][23:45:44][keepkey_gui_lib::server][INFO]   💾 Frontload: POST http://127.0.0.1:1646/api/frontload
[2025-06-11][23:45:49][keepkey_gui_lib::device_controller][INFO] Attempting to connect to device 343737340F4736331F003B00 (1/3)
[2025-06-11][23:45:49][keepkey_gui_lib::device_controller][INFO] Fetching features for device 343737340F4736331F003B00 (attempt 1)
[2025-06-11][23:45:49][keepkey_gui_lib::features][INFO]  | features |  Getting features for device with fallback: KeyHodlers, LLC - KeepKey - KeepKey3 (343737340F4736331F003B00)
[2025-06-11][23:45:49][keepkey_gui_lib::features][INFO]  | features |  Getting features for device: KeyHodlers, LLC - KeepKey - KeepKey3 (343737340F4736331F003B00)
[2025-06-11][23:45:49][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Processing message type: Initialize
[2025-06-11][23:45:49][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::send: Sending message type: Initialize
[2025-06-11][23:45:49][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Waiting for response (timeout: 5s)...
[2025-06-11][23:45:50][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Received 289 bytes response
[2025-06-11][23:45:50][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Decoded response type: Features
[2025-06-11][23:45:50][keepkey_gui_lib::device_update][INFO]  | device_update |  Successfully loaded releases.json from: ../firmware/releases.json
[2025-06-11][23:45:50][keepkey_gui_lib::device_update][INFO]  | device_update |  Found bootloader version 2.1.4 for hash fe98454e7ebd4aef4a6db5bd4c60f52cf3f58b974283a7c1e1fcc5fea02cf3eb
[2025-06-11][23:45:50][keepkey_gui_lib::features][INFO]  | features |  Successfully got features for device 343737340F4736331F003B00: firmware v7.10.0
[2025-06-11][23:45:50][keepkey_gui_lib::device_controller][INFO] Successfully fetched features for device 343737340F4736331F003B00
[2025-06-11][23:45:50][keepkey_gui_lib::index_db][INFO] Recording device connection: 343737340F4736331F003B00
[2025-06-11][23:45:50][keepkey_gui_lib::index_db][INFO] Device 343737340F4736331F003B00 connected and recorded
[2025-06-11][23:45:50][keepkey_gui_lib::device_update][INFO]  | get_latest_firmware_version |  Latest firmware version: 7.10.0
[2025-06-11][23:45:50][keepkey_gui_lib][INFO] 🔍 [FRONTLOAD DEBUG] Device features fetched for 343737340F4736331F003B00
[2025-06-11][23:45:50][keepkey_gui_lib][INFO] 🔍 [FRONTLOAD DEBUG] Device version: 7.10.0
[2025-06-11][23:45:50][keepkey_gui_lib][INFO] 🔍 [FRONTLOAD DEBUG] Device initialized: true
[2025-06-11][23:45:50][keepkey_gui_lib::device_update][INFO]  | get_latest_firmware_version |  Latest firmware version: 7.10.0
[2025-06-11][23:45:50][keepkey_gui_lib][INFO] 🔍 [FRONTLOAD DEBUG] Latest firmware result: Ok("7.10.0")
[2025-06-11][23:45:50][keepkey_gui_lib][INFO] 🔍 [FRONTLOAD DEBUG] Latest firmware: 7.10.0
[2025-06-11][23:45:50][keepkey_gui_lib][INFO] 🔍 [FRONTLOAD DEBUG] Version check result: Ok(false)
[2025-06-11][23:45:50][keepkey_gui_lib][INFO] 🔍 [FRONTLOAD DEBUG] Is outdated: false
[2025-06-11][23:45:50][keepkey_gui_lib][INFO] 🔍 [FRONTLOAD DEBUG] Device ready: true
[2025-06-11][23:45:50][keepkey_gui_lib][INFO] 🚀 [FRONTLOAD TRIGGER] Device 343737340F4736331F003B00 is ready (firmware v7.10.0, initialized), starting frontload...
[2025-06-11][23:45:50][keepkey_gui_lib][INFO] Setting device context for 343737340F4736331F003B00 before frontload
[2025-06-11][23:45:50][keepkey_gui_lib::device_update][INFO]  | get_latest_firmware_version |  Latest firmware version: 7.10.0
[2025-06-11][23:45:50][keepkey_gui_lib::device_update][INFO]  | get_latest_firmware_version |  Latest firmware version: 7.10.0
[2025-06-11][23:45:50][keepkey_gui_lib][INFO] Starting frontload process for device 343737340F4736331F003B00
[2025-06-11][23:45:50][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: cache_dir resolved to: /Users/highlander/.keepkey/kkcli
[2025-06-11][23:45:50][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: db_path resolved to: /Users/highlander/.keepkey/kkcli/device_cache.db
[2025-06-11][23:45:50][keepkey_gui_lib::cache::device_cache][INFO] Opening device cache at: /Users/highlander/.keepkey/kkcli/device_cache.db
[2025-06-11][23:45:50][keepkey_gui_lib][INFO] 🔗 Using existing device connection for frontload: 343737340F4736331F003B00
[2025-06-11][23:45:50][keepkey_gui_lib::server::context][INFO] Validating cache for device 343737340F4736331F003B00
[2025-06-11][23:45:50][keepkey_gui_lib::server::context][INFO] Getting real Ethereum address from device 343737340F4736331F003B00
[2025-06-11][23:45:50][keepkey_gui_lib::server::context][WARN] Failed to get real Ethereum address from device 343737340F4736331F003B00: Failed to create transport: Entity not found
[2025-06-11][23:45:50][keepkey_gui_lib][INFO] ✅ Device cache is valid for 343737340F4736331F003B00
[2025-06-11][23:45:50][tracing::span][INFO] frontload_all_with_progress;
[2025-06-11][23:45:50][keepkey_gui_lib::cache::frontload][INFO] 🔄 Starting device frontload process with transport factory...
[2025-06-11][23:45:50][keepkey_gui_lib][INFO] Frontload progress: Loading device features...
[2025-06-11][23:45:50][tracing::span][INFO] create_device_transport;
[2025-06-11][23:45:50][keepkey_gui_lib::cache::frontload][INFO] 🔗 Creating transport using device factory for 343737340F4736331F003B00
[2025-06-11][23:45:50][tracing::span][INFO] create_transport_from_device_info;
[2025-06-11][23:45:50][keepkey_gui_lib::cache::frontload][INFO] 🔍 Looking for device with unique_id: 343737340F4736331F003B00
[2025-06-11][23:45:50][keepkey_gui_lib::cache::frontload][INFO]    Serial number: Some("343737340F4736331F003B00")
[2025-06-11][23:45:50][keepkey_gui_lib::cache::frontload][INFO]    Found 1 USB devices to search
[2025-06-11][23:45:50][keepkey_gui_lib::server::routes::v2_endpoints][ERROR] get_portfolio_summary: No device found in cache
[2025-06-11][23:45:50][keepkey_gui_lib::server::routes::v2_endpoints][ERROR] get_balances: No device found in cache
[2025-06-11][23:45:50][keepkey_gui_lib::server::routes::v2_endpoints][ERROR] get_portfolio_summary: No device found in cache
[2025-06-11][23:45:50][keepkey_gui_lib::server::routes::v2_endpoints][ERROR] get_balances: No device found in cache
[2025-06-11][23:45:50][keepkey_gui_lib::server::routes::v2_endpoints][ERROR] get_portfolio_summary: No device found in cache
[2025-06-11][23:45:50][keepkey_gui_lib::server::routes::v2_endpoints][ERROR] get_balances: No device found in cache
[2025-06-11][23:45:50][keepkey_gui_lib::server::routes::v2_endpoints][ERROR] get_portfolio_summary: No device found in cache
[2025-06-11][23:45:50][keepkey_gui_lib::server::routes::v2_endpoints][ERROR] get_balances: No device found in cache
[2025-06-11][23:45:50][keepkey_gui_lib::cache::frontload][WARN] ⚠️  Could not match device by ID, using first available KeepKey device
[2025-06-11][23:45:50][keepkey_gui_lib::cache::frontload][WARN] ⚠️  USB transport failed for device 343737340F4736331F003B00: Entity not found, trying HID fallback
[2025-06-11][23:45:50][keepkey_gui_lib::transport::hid][INFO] Found 0 KeepKey devices
[2025-06-11][23:45:50][keepkey_gui_lib::transport::hid][INFO] Found 0 KeepKey devices
[2025-06-11][23:45:50][keepkey_gui_lib::cache::frontload][WARN] ⚠️  Trying HID without serial number filter
[2025-06-11][23:45:50][keepkey_gui_lib::transport::hid][INFO] Found 0 KeepKey devices
[2025-06-11][23:45:50][keepkey_gui_lib::cache::frontload][ERROR] ❌ Both USB and HID transports failed for device 343737340F4736331F003B00
[2025-06-11][23:45:50][keepkey_gui_lib::cache::frontload][ERROR]    USB error: Entity not found
[2025-06-11][23:45:50][keepkey_gui_lib::cache::frontload][ERROR]    HID error: No KeepKey devices found
[2025-06-11][23:45:50][keepkey_gui_lib::cache::frontload][ERROR] 🚨 FAILING FAST: No transport available, preventing device ready state
[2025-06-11][23:45:50][keepkey_gui_lib::cache::frontload][WARN] ⚠️  Failed to create transport: Failed with both USB (Entity not found) and HID (No KeepKey devices found), trying device registry fallback
[2025-06-11][23:45:50][keepkey_gui_lib::cache::frontload][INFO] 🔗 Using features from device registry fallback
[2025-06-11][23:45:50][keepkey_gui_lib::cache::frontload][INFO] 🔍 Found 6 device(s) in registry (attempt 1/5)
[2025-06-11][23:45:50][keepkey_gui_lib::cache::frontload][INFO] 🔍 Checking device 343737340F4736331F003B00 for features
[2025-06-11][23:45:50][keepkey_gui_lib::cache::frontload][INFO] ✅ Got device features from registry - ID: 343737340F4736331F003B00, Label: KeepKey3
[2025-06-11][23:45:50][keepkey_gui_lib::cache::device_cache][INFO] Saved features for device 343737340F4736331F003B00
[2025-06-11][23:45:50][keepkey_gui_lib::cache::device_cache][INFO] 📈 Database query result: Loaded 0 cached addresses for device 343737340F4736331F003B00
[2025-06-11][23:45:50][keepkey_gui_lib::cache::device_cache][INFO] Loaded 0 cached addresses into memory for device 343737340F4736331F003B00
[2025-06-11][23:45:50][keepkey_gui_lib::cache::frontload][INFO] 📚 Loaded existing device data into memory cache
[2025-06-11][23:45:50][keepkey_gui_lib][INFO] Frontload progress: Loading default paths...
[2025-06-11][23:45:50][keepkey_gui_lib::cache::frontload][INFO] 📂 Ensuring all default paths from JSON are loaded into database...
[2025-06-11][23:45:50][keepkey_gui_lib::cache::frontload][INFO] 📥 Found 4 default paths in JSON
[2025-06-11][23:45:50][keepkey_gui_lib::cache::frontload][INFO] 📊 Database currently has 4 paths
[2025-06-11][23:45:50][keepkey_gui_lib::cache::frontload][INFO] 🎯 Path loading complete: 0 new paths loaded, 4 already existed
[2025-06-11][23:45:50][keepkey_gui_lib::cache::frontload][INFO] 📍 Checking for missing addresses from database paths...
[2025-06-11][23:45:50][keepkey_gui_lib][INFO] Frontload progress: Checking for missing addresses...
[2025-06-11][23:45:50][keepkey_gui_lib][INFO] Frontload progress: Loading Bitcoin addresses...
[2025-06-11][23:45:50][tracing::span][INFO] create_device_transport;
[2025-06-11][23:45:50][keepkey_gui_lib::cache::frontload][INFO] 🔗 Creating transport using device factory for 343737340F4736331F003B00
[2025-06-11][23:45:50][tracing::span][INFO] create_transport_from_device_info;
[2025-06-11][23:45:50][keepkey_gui_lib::cache::frontload][INFO] 🔍 Looking for device with unique_id: 343737340F4736331F003B00
[2025-06-11][23:45:50][keepkey_gui_lib::cache::frontload][INFO]    Serial number: Some("343737340F4736331F003B00")
[2025-06-11][23:45:50][keepkey_gui_lib::cache::frontload][INFO]    Found 1 USB devices to search
[2025-06-11][23:45:50][keepkey_gui_lib::cache::frontload][WARN] ⚠️  Could not match device by ID, using first available KeepKey device
[2025-06-11][23:45:50][keepkey_gui_lib::cache::frontload][WARN] ⚠️  USB transport failed for device 343737340F4736331F003B00: Entity not found, trying HID fallback
[2025-06-11][23:45:50][keepkey_gui_lib::transport::hid][INFO] Found 1 KeepKey devices
[2025-06-11][23:45:50][keepkey_gui_lib::transport::hid][INFO] Attempting to find KeepKey device with serial number: 343737340F4736331F003B00
[2025-06-11][23:45:50][keepkey_gui_lib::transport::hid][INFO] Found KeepKey device with exact serial match: 343737340F4736331F003B00
[2025-06-11][23:45:50][keepkey_gui_lib::cache::frontload][INFO] ✅ Created HID transport for device 343737340F4736331F003B00
[2025-06-11][23:45:50][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Starting with message type: GetPublicKey
[2025-06-11][23:45:50][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Calling parent adapter...
[2025-06-11][23:45:50][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Processing message type: GetPublicKey
[2025-06-11][23:45:50][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::send: Sending message type: GetPublicKey
[2025-06-11][23:45:50][keepkey_gui_lib::transport::hid][INFO] HID Write: Sending message type 11 (0x000b), length: 31
[2025-06-11][23:45:50][keepkey_gui_lib::transport::hid][INFO] HID Write: Complete. Sent 39 bytes in 1 packets
[2025-06-11][23:45:50][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Waiting for response (timeout: 5s)...
[2025-06-11][23:45:50][keepkey_gui_lib::transport::hid][INFO] HID Read: Waiting for response (timeout: 5000 ms)...
[2025-06-11][23:45:50][keepkey_gui_lib::server::context][INFO] Getting device features to save to database for device 343737340F4736331F003B00
[2025-06-11][23:45:50][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Starting with message type: GetFeatures
[2025-06-11][23:45:50][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Calling parent adapter...
[2025-06-11][23:45:50][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Processing message type: GetFeatures
[2025-06-11][23:45:50][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::send: Sending message type: GetFeatures
[2025-06-11][23:45:50][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Waiting for response (timeout: 5s)...
[2025-06-11][23:45:50][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Received 289 bytes response
[2025-06-11][23:45:50][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Decoded response type: Features
[2025-06-11][23:45:50][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Parent returned message type: Features
[2025-06-11][23:45:50][keepkey_gui_lib::transport][INFO] StandardHandler: Processing message type: Features
[2025-06-11][23:45:50][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Handler returned None, done
[2025-06-11][23:45:50][keepkey_gui_lib::server::context][INFO] ✅ Got device features for 343737340F4736331F003B00
[2025-06-11][23:45:50][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: cache_dir resolved to: /Users/highlander/.keepkey/kkcli
[2025-06-11][23:45:50][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: db_path resolved to: /Users/highlander/.keepkey/kkcli/device_cache.db
[2025-06-11][23:45:50][keepkey_gui_lib::cache::device_cache][INFO] Opening device cache at: /Users/highlander/.keepkey/kkcli/device_cache.db
[2025-06-11][23:45:50][keepkey_gui_lib::cache::device_cache][INFO] Saved features for device 343737340F4736331F003B00
[2025-06-11][23:45:50][keepkey_gui_lib::server::context][INFO] ✅ Saved device features to database for 343737340F4736331F003B00
[2025-06-11][23:45:50][keepkey_gui_lib::server::context][INFO] Getting real Ethereum address from device 343737340F4736331F003B00
[2025-06-11][23:45:50][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Starting with message type: EthereumGetAddress
[2025-06-11][23:45:50][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Calling parent adapter...
[2025-06-11][23:45:50][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Processing message type: EthereumGetAddress
[2025-06-11][23:45:50][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::send: Sending message type: EthereumGetAddress
[2025-06-11][23:45:50][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Waiting for response (timeout: 5s)...
[2025-06-11][23:45:51][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Received 74 bytes response
[2025-06-11][23:45:51][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Decoded response type: EthereumAddress
[2025-06-11][23:45:51][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Parent returned message type: EthereumAddress
[2025-06-11][23:45:51][keepkey_gui_lib::transport][INFO] StandardHandler: Processing message type: EthereumAddress
[2025-06-11][23:45:51][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Handler returned None, done
[2025-06-11][23:45:51][keepkey_gui_lib::server::context][INFO] ✅ Got real Ethereum address from device 343737340F4736331F003B00: 0x141d9959cae3853b035000490c03991eb70fc4ac
[2025-06-11][23:45:51][keepkey_gui_lib::server::context][INFO] Device context set: device_id=343737340F4736331F003B00, eth_address=0x141d9959cae3853b035000490c03991eb70fc4ac, label=Some("KeepKey3")
[2025-06-11][23:45:51][keepkey_gui_lib][INFO] ✅ Device context set successfully for 343737340F4736331F003B00 before frontload
[2025-06-11][23:45:55][keepkey_gui_lib::transport::hid][ERROR] HID Read: No data received from device after 5000 ms timeout
[2025-06-11][23:45:55][keepkey_gui_lib::cache::frontload][WARN] ❌ Failed to cache Bitcoin p2pkh xpub at [2147483692, 2147483648, 2147483648] for network bip122:000000000019d6689c085ae165831e93: Other error: No data received from device
[2025-06-11][23:45:55][tracing::span][INFO] create_device_transport;
[2025-06-11][23:45:55][keepkey_gui_lib::cache::frontload][INFO] 🔗 Creating transport using device factory for 343737340F4736331F003B00
[2025-06-11][23:45:55][tracing::span][INFO] create_transport_from_device_info;
[2025-06-11][23:45:55][keepkey_gui_lib::cache::frontload][INFO] 🔍 Looking for device with unique_id: 343737340F4736331F003B00
[2025-06-11][23:45:55][keepkey_gui_lib::cache::frontload][INFO]    Serial number: Some("343737340F4736331F003B00")
[2025-06-11][23:45:55][keepkey_gui_lib::cache::frontload][INFO]    Found 1 USB devices to search
[2025-06-11][23:45:55][keepkey_gui_lib::cache::frontload][INFO] ✅ Found device by serial number match: 343737340F4736331F003B00
[2025-06-11][23:45:55][keepkey_gui_lib::cache::frontload][INFO] ✅ Created USB transport for device 343737340F4736331F003B00
[2025-06-11][23:45:55][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Starting with message type: GetAddress
[2025-06-11][23:45:55][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Calling parent adapter...
[2025-06-11][23:45:55][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Processing message type: GetAddress
[2025-06-11][23:45:55][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::send: Sending message type: GetAddress
[2025-06-11][23:45:55][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Waiting for response (timeout: 5s)...
[2025-06-11][23:45:55][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Received 44 bytes response
[2025-06-11][23:45:55][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Decoded response type: Address
[2025-06-11][23:45:55][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Parent returned message type: Address
[2025-06-11][23:45:55][keepkey_gui_lib::transport][INFO] StandardHandler: Processing message type: Address
[2025-06-11][23:45:55][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Handler returned None, done
[2025-06-11][23:45:55][keepkey_gui_lib::cache::device_cache][INFO] ✅ Verified address save for Bitcoin/p2pkh at path [2147483692, 2147483648, 2147483648, 0, 0] (address: 1JNYtQsc1pizKbn3ScbEPfQ7WcxNqeUHNB)
[2025-06-11][23:45:55][keepkey_gui_lib::cache::frontload][INFO] ✅ Cached missing Bitcoin p2pkh address at path [2147483692, 2147483648, 2147483648, 0, 0] for network bip122:000000000019d6689c085ae165831e93
[2025-06-11][23:45:55][tracing::span][INFO] create_device_transport;
[2025-06-11][23:45:55][keepkey_gui_lib::cache::frontload][INFO] 🔗 Creating transport using device factory for 343737340F4736331F003B00
[2025-06-11][23:45:55][tracing::span][INFO] create_transport_from_device_info;
[2025-06-11][23:45:55][keepkey_gui_lib::cache::frontload][INFO] 🔍 Looking for device with unique_id: 343737340F4736331F003B00
[2025-06-11][23:45:55][keepkey_gui_lib::cache::frontload][INFO]    Serial number: Some("343737340F4736331F003B00")
[2025-06-11][23:45:55][keepkey_gui_lib::cache::frontload][INFO]    Found 1 USB devices to search
[2025-06-11][23:45:55][keepkey_gui_lib::cache::frontload][INFO] ✅ Found device by serial number match: 343737340F4736331F003B00
[2025-06-11][23:45:56][keepkey_gui_lib::cache::frontload][INFO] ✅ Created USB transport for device 343737340F4736331F003B00
[2025-06-11][23:45:56][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Starting with message type: GetAddress
[2025-06-11][23:45:56][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Calling parent adapter...
[2025-06-11][23:45:56][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Processing message type: GetAddress
[2025-06-11][23:45:56][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::send: Sending message type: GetAddress
[2025-06-11][23:45:56][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Waiting for response (timeout: 5s)...
[2025-06-11][23:45:56][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Received 44 bytes response
[2025-06-11][23:45:56][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Decoded response type: Address
[2025-06-11][23:45:56][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Parent returned message type: Address
[2025-06-11][23:45:56][keepkey_gui_lib::transport][INFO] StandardHandler: Processing message type: Address
[2025-06-11][23:45:56][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Handler returned None, done
[2025-06-11][23:45:56][keepkey_gui_lib::cache::device_cache][INFO] ✅ Verified address save for Bitcoin/p2pkh at path [2147483692, 2147483648, 2147483648, 0, 1] (address: 19QvcGC3H5EXSzfTiAVsnvCyJRj9WxmMeq)
[2025-06-11][23:45:56][keepkey_gui_lib::cache::frontload][INFO] ✅ Cached missing Bitcoin p2pkh address at path [2147483692, 2147483648, 2147483648, 0, 1] for network bip122:000000000019d6689c085ae165831e93
[2025-06-11][23:45:56][tracing::span][INFO] create_device_transport;
[2025-06-11][23:45:56][keepkey_gui_lib::cache::frontload][INFO] 🔗 Creating transport using device factory for 343737340F4736331F003B00
[2025-06-11][23:45:56][tracing::span][INFO] create_transport_from_device_info;
[2025-06-11][23:45:56][keepkey_gui_lib::cache::frontload][INFO] 🔍 Looking for device with unique_id: 343737340F4736331F003B00
[2025-06-11][23:45:56][keepkey_gui_lib::cache::frontload][INFO]    Serial number: Some("343737340F4736331F003B00")
[2025-06-11][23:45:56][keepkey_gui_lib::cache::frontload][INFO]    Found 1 USB devices to search
[2025-06-11][23:45:56][keepkey_gui_lib::cache::frontload][INFO] ✅ Found device by serial number match: 343737340F4736331F003B00
[2025-06-11][23:45:56][keepkey_gui_lib::cache::frontload][INFO] ✅ Created USB transport for device 343737340F4736331F003B00
[2025-06-11][23:45:56][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Starting with message type: GetAddress
[2025-06-11][23:45:56][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Calling parent adapter...
[2025-06-11][23:45:56][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Processing message type: GetAddress
[2025-06-11][23:45:56][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::send: Sending message type: GetAddress
[2025-06-11][23:45:56][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Waiting for response (timeout: 5s)...
[2025-06-11][23:45:56][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Received 44 bytes response
[2025-06-11][23:45:56][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Decoded response type: Address
[2025-06-11][23:45:56][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Parent returned message type: Address
[2025-06-11][23:45:56][keepkey_gui_lib::transport][INFO] StandardHandler: Processing message type: Address
[2025-06-11][23:45:56][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Handler returned None, done
[2025-06-11][23:45:56][keepkey_gui_lib::cache::device_cache][INFO] ✅ Verified address save for Bitcoin/p2pkh at path [2147483692, 2147483648, 2147483648, 0, 2] (address: 1MA2a6ELwt8qgK6RSUPE6rMkKxxfHetexo)
[2025-06-11][23:45:56][keepkey_gui_lib::cache::frontload][INFO] ✅ Cached missing Bitcoin p2pkh address at path [2147483692, 2147483648, 2147483648, 0, 2] for network bip122:000000000019d6689c085ae165831e93
[2025-06-11][23:45:56][tracing::span][INFO] create_device_transport;
[2025-06-11][23:45:56][keepkey_gui_lib::cache::frontload][INFO] 🔗 Creating transport using device factory for 343737340F4736331F003B00
[2025-06-11][23:45:56][tracing::span][INFO] create_transport_from_device_info;
[2025-06-11][23:45:56][keepkey_gui_lib::cache::frontload][INFO] 🔍 Looking for device with unique_id: 343737340F4736331F003B00
[2025-06-11][23:45:56][keepkey_gui_lib::cache::frontload][INFO]    Serial number: Some("343737340F4736331F003B00")
[2025-06-11][23:45:56][keepkey_gui_lib::cache::frontload][INFO]    Found 1 USB devices to search
[2025-06-11][23:45:57][keepkey_gui_lib::cache::frontload][INFO] ✅ Found device by serial number match: 343737340F4736331F003B00
[2025-06-11][23:45:57][keepkey_gui_lib::cache::frontload][INFO] ✅ Created USB transport for device 343737340F4736331F003B00
[2025-06-11][23:45:57][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Starting with message type: GetAddress
[2025-06-11][23:45:57][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Calling parent adapter...
[2025-06-11][23:45:57][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Processing message type: GetAddress
[2025-06-11][23:45:57][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::send: Sending message type: GetAddress
[2025-06-11][23:45:57][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Waiting for response (timeout: 5s)...
[2025-06-11][23:45:57][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Received 44 bytes response
[2025-06-11][23:45:57][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Decoded response type: Address
[2025-06-11][23:45:57][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Parent returned message type: Address
[2025-06-11][23:45:57][keepkey_gui_lib::transport][INFO] StandardHandler: Processing message type: Address
[2025-06-11][23:45:57][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Handler returned None, done
[2025-06-11][23:45:57][keepkey_gui_lib::cache::device_cache][INFO] ✅ Verified address save for Bitcoin/p2pkh at path [2147483692, 2147483648, 2147483648, 0, 3] (address: 13DmjYe2exmwoULgChcQaSYU9mAmC9qPvf)
[2025-06-11][23:45:57][keepkey_gui_lib::cache::frontload][INFO] ✅ Cached missing Bitcoin p2pkh address at path [2147483692, 2147483648, 2147483648, 0, 3] for network bip122:000000000019d6689c085ae165831e93
[2025-06-11][23:45:57][tracing::span][INFO] create_device_transport;
[2025-06-11][23:45:57][keepkey_gui_lib::cache::frontload][INFO] 🔗 Creating transport using device factory for 343737340F4736331F003B00
[2025-06-11][23:45:57][tracing::span][INFO] create_transport_from_device_info;
[2025-06-11][23:45:57][keepkey_gui_lib::cache::frontload][INFO] 🔍 Looking for device with unique_id: 343737340F4736331F003B00
[2025-06-11][23:45:57][keepkey_gui_lib::cache::frontload][INFO]    Serial number: Some("343737340F4736331F003B00")
[2025-06-11][23:45:57][keepkey_gui_lib::cache::frontload][INFO]    Found 1 USB devices to search
[2025-06-11][23:45:57][keepkey_gui_lib::cache::frontload][INFO] ✅ Found device by serial number match: 343737340F4736331F003B00
[2025-06-11][23:45:57][keepkey_gui_lib::cache::frontload][INFO] ✅ Created USB transport for device 343737340F4736331F003B00
[2025-06-11][23:45:57][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Starting with message type: GetAddress
[2025-06-11][23:45:57][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Calling parent adapter...
[2025-06-11][23:45:57][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Processing message type: GetAddress
[2025-06-11][23:45:57][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::send: Sending message type: GetAddress
[2025-06-11][23:45:57][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Waiting for response (timeout: 5s)...
[2025-06-11][23:45:58][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Received 44 bytes response
[2025-06-11][23:45:58][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Decoded response type: Address
[2025-06-11][23:45:58][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Parent returned message type: Address
[2025-06-11][23:45:58][keepkey_gui_lib::transport][INFO] StandardHandler: Processing message type: Address
[2025-06-11][23:45:58][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Handler returned None, done
[2025-06-11][23:45:58][keepkey_gui_lib::cache::device_cache][INFO] ✅ Verified address save for Bitcoin/p2pkh at path [2147483692, 2147483648, 2147483648, 0, 4] (address: 1P1a4PuYNTjUAVJsE3oex4VAqgqkCSoAP6)
[2025-06-11][23:45:58][keepkey_gui_lib::cache::frontload][INFO] ✅ Cached missing Bitcoin p2pkh address at path [2147483692, 2147483648, 2147483648, 0, 4] for network bip122:000000000019d6689c085ae165831e93
[2025-06-11][23:45:58][keepkey_gui_lib][INFO] Frontload progress: Loading Bitcoin addresses...
[2025-06-11][23:45:58][tracing::span][INFO] create_device_transport;
[2025-06-11][23:45:58][keepkey_gui_lib::cache::frontload][INFO] 🔗 Creating transport using device factory for 343737340F4736331F003B00
[2025-06-11][23:45:58][tracing::span][INFO] create_transport_from_device_info;
[2025-06-11][23:45:58][keepkey_gui_lib::cache::frontload][INFO] 🔍 Looking for device with unique_id: 343737340F4736331F003B00
[2025-06-11][23:45:58][keepkey_gui_lib::cache::frontload][INFO]    Serial number: Some("343737340F4736331F003B00")
[2025-06-11][23:45:58][keepkey_gui_lib::cache::frontload][INFO]    Found 1 USB devices to search
[2025-06-11][23:45:58][keepkey_gui_lib::cache::frontload][INFO] ✅ Found device by serial number match: 343737340F4736331F003B00
[2025-06-11][23:45:58][keepkey_gui_lib::cache::frontload][INFO] ✅ Created USB transport for device 343737340F4736331F003B00
[2025-06-11][23:45:58][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Starting with message type: GetPublicKey
[2025-06-11][23:45:58][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Calling parent adapter...
[2025-06-11][23:45:58][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Processing message type: GetPublicKey
[2025-06-11][23:45:58][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::send: Sending message type: GetPublicKey
[2025-06-11][23:45:58][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Waiting for response (timeout: 5s)...
[2025-06-11][23:45:58][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Received 205 bytes response
[2025-06-11][23:45:58][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Decoded response type: PublicKey
[2025-06-11][23:45:58][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Parent returned message type: PublicKey
[2025-06-11][23:45:58][keepkey_gui_lib::transport][INFO] StandardHandler: Processing message type: PublicKey
[2025-06-11][23:45:58][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Handler returned None, done
[2025-06-11][23:45:58][keepkey_gui_lib::cache::device_cache][INFO] ✅ Verified address save for Bitcoin/p2sh-p2wpkh_xpub at path [2147483697, 2147483648, 2147483648] (address: ypub6WamSeXgTYgy7W25fVorMLDHFx5SPkuYaE7ToWCiyCUK2jdWpufQ8VqkDg83YjBtJFHDoekhf9ESdPDbL9aCPXC5NnmzXUiq3J6oycFShfS)
[2025-06-11][23:45:58][keepkey_gui_lib::cache::frontload][INFO] ✅ Cached Bitcoin p2sh-p2wpkh xpub: ypub6WamSeXgTYgy7W25fVorMLDHFx5SPkuYaE7ToWCiyCUK2jdWpufQ8VqkDg83YjBtJFHDoekhf9ESdPDbL9aCPXC5NnmzXUiq3J6oycFShfS
[2025-06-11][23:45:58][keepkey_gui_lib::cache::frontload][INFO] ✅ Cached missing Bitcoin p2sh-p2wpkh xpub: ypub6WamSeXgTYgy7W25fVorMLDHFx5SPkuYaE7ToWCiyCUK2jdWpufQ8VqkDg83YjBtJFHDoekhf9ESdPDbL9aCPXC5NnmzXUiq3J6oycFShfS at path [2147483697, 2147483648, 2147483648] for network bip122:000000000019d6689c085ae165831e93
[2025-06-11][23:45:58][tracing::span][INFO] create_device_transport;
[2025-06-11][23:45:58][keepkey_gui_lib::cache::frontload][INFO] 🔗 Creating transport using device factory for 343737340F4736331F003B00
[2025-06-11][23:45:58][tracing::span][INFO] create_transport_from_device_info;
[2025-06-11][23:45:58][keepkey_gui_lib::cache::frontload][INFO] 🔍 Looking for device with unique_id: 343737340F4736331F003B00
[2025-06-11][23:45:58][keepkey_gui_lib::cache::frontload][INFO]    Serial number: Some("343737340F4736331F003B00")
[2025-06-11][23:45:58][keepkey_gui_lib::cache::frontload][INFO]    Found 1 USB devices to search
[2025-06-11][23:45:58][keepkey_gui_lib::cache::frontload][INFO] ✅ Found device by serial number match: 343737340F4736331F003B00
[2025-06-11][23:45:58][keepkey_gui_lib::cache::frontload][INFO] ✅ Created USB transport for device 343737340F4736331F003B00
[2025-06-11][23:45:58][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Starting with message type: GetAddress
[2025-06-11][23:45:58][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Calling parent adapter...
[2025-06-11][23:45:58][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Processing message type: GetAddress
[2025-06-11][23:45:58][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::send: Sending message type: GetAddress
[2025-06-11][23:45:58][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Waiting for response (timeout: 5s)...
[2025-06-11][23:45:59][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Received 44 bytes response
[2025-06-11][23:45:59][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Decoded response type: Address
[2025-06-11][23:45:59][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Parent returned message type: Address
[2025-06-11][23:45:59][keepkey_gui_lib::transport][INFO] StandardHandler: Processing message type: Address
[2025-06-11][23:45:59][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Handler returned None, done
[2025-06-11][23:45:59][keepkey_gui_lib::cache::device_cache][INFO] ✅ Verified address save for Bitcoin/p2sh-p2wpkh at path [2147483697, 2147483648, 2147483648, 0, 0] (address: 3M9rBdu7rkVGwmt9gALjuRopAqpVEBdNRR)
[2025-06-11][23:45:59][keepkey_gui_lib::cache::frontload][INFO] ✅ Cached missing Bitcoin p2sh-p2wpkh address at path [2147483697, 2147483648, 2147483648, 0, 0] for network bip122:000000000019d6689c085ae165831e93
[2025-06-11][23:45:59][tracing::span][INFO] create_device_transport;
[2025-06-11][23:45:59][keepkey_gui_lib::cache::frontload][INFO] 🔗 Creating transport using device factory for 343737340F4736331F003B00
[2025-06-11][23:45:59][tracing::span][INFO] create_transport_from_device_info;
[2025-06-11][23:45:59][keepkey_gui_lib::cache::frontload][INFO] 🔍 Looking for device with unique_id: 343737340F4736331F003B00
[2025-06-11][23:45:59][keepkey_gui_lib::cache::frontload][INFO]    Serial number: Some("343737340F4736331F003B00")
[2025-06-11][23:45:59][keepkey_gui_lib::cache::frontload][INFO]    Found 1 USB devices to search
[2025-06-11][23:45:59][keepkey_gui_lib::cache::frontload][INFO] ✅ Found device by serial number match: 343737340F4736331F003B00
[2025-06-11][23:45:59][keepkey_gui_lib::cache::frontload][INFO] ✅ Created USB transport for device 343737340F4736331F003B00
[2025-06-11][23:45:59][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Starting with message type: GetAddress
[2025-06-11][23:45:59][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Calling parent adapter...
[2025-06-11][23:45:59][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Processing message type: GetAddress
[2025-06-11][23:45:59][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::send: Sending message type: GetAddress
[2025-06-11][23:45:59][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Waiting for response (timeout: 5s)...
[2025-06-11][23:45:59][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Received 44 bytes response
[2025-06-11][23:45:59][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Decoded response type: Address
[2025-06-11][23:45:59][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Parent returned message type: Address
[2025-06-11][23:45:59][keepkey_gui_lib::transport][INFO] StandardHandler: Processing message type: Address
[2025-06-11][23:45:59][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Handler returned None, done
[2025-06-11][23:45:59][keepkey_gui_lib::cache::device_cache][INFO] ✅ Verified address save for Bitcoin/p2sh-p2wpkh at path [2147483697, 2147483648, 2147483648, 0, 1] (address: 3PczoXGFPPTw5idJ6gzQ6DsHuWdVBGyPiB)
[2025-06-11][23:45:59][keepkey_gui_lib::cache::frontload][INFO] ✅ Cached missing Bitcoin p2sh-p2wpkh address at path [2147483697, 2147483648, 2147483648, 0, 1] for network bip122:000000000019d6689c085ae165831e93
[2025-06-11][23:45:59][tracing::span][INFO] create_device_transport;
[2025-06-11][23:45:59][keepkey_gui_lib::cache::frontload][INFO] 🔗 Creating transport using device factory for 343737340F4736331F003B00
[2025-06-11][23:45:59][tracing::span][INFO] create_transport_from_device_info;
[2025-06-11][23:45:59][keepkey_gui_lib::cache::frontload][INFO] 🔍 Looking for device with unique_id: 343737340F4736331F003B00
[2025-06-11][23:45:59][keepkey_gui_lib::cache::frontload][INFO]    Serial number: Some("343737340F4736331F003B00")
[2025-06-11][23:45:59][keepkey_gui_lib::cache::frontload][INFO]    Found 1 USB devices to search
[2025-06-11][23:45:59][keepkey_gui_lib::cache::frontload][INFO] ✅ Found device by serial number match: 343737340F4736331F003B00
[2025-06-11][23:45:59][keepkey_gui_lib::cache::frontload][INFO] ✅ Created USB transport for device 343737340F4736331F003B00
[2025-06-11][23:45:59][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Starting with message type: GetAddress
[2025-06-11][23:45:59][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Calling parent adapter...
[2025-06-11][23:45:59][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Processing message type: GetAddress
[2025-06-11][23:45:59][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::send: Sending message type: GetAddress
[2025-06-11][23:45:59][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Waiting for response (timeout: 5s)...
[2025-06-11][23:46:00][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Received 44 bytes response
[2025-06-11][23:46:00][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Decoded response type: Address
[2025-06-11][23:46:00][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Parent returned message type: Address
[2025-06-11][23:46:00][keepkey_gui_lib::transport][INFO] StandardHandler: Processing message type: Address
[2025-06-11][23:46:00][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Handler returned None, done
[2025-06-11][23:46:00][keepkey_gui_lib::cache::device_cache][INFO] ✅ Verified address save for Bitcoin/p2sh-p2wpkh at path [2147483697, 2147483648, 2147483648, 0, 2] (address: 3Ds7s2C91SoEynKWJWkBrE4nPoLV8YxryG)
[2025-06-11][23:46:00][keepkey_gui_lib::cache::frontload][INFO] ✅ Cached missing Bitcoin p2sh-p2wpkh address at path [2147483697, 2147483648, 2147483648, 0, 2] for network bip122:000000000019d6689c085ae165831e93
[2025-06-11][23:46:00][tracing::span][INFO] create_device_transport;
[2025-06-11][23:46:00][keepkey_gui_lib::cache::frontload][INFO] 🔗 Creating transport using device factory for 343737340F4736331F003B00
[2025-06-11][23:46:00][tracing::span][INFO] create_transport_from_device_info;
[2025-06-11][23:46:00][keepkey_gui_lib::cache::frontload][INFO] 🔍 Looking for device with unique_id: 343737340F4736331F003B00
[2025-06-11][23:46:00][keepkey_gui_lib::cache::frontload][INFO]    Serial number: Some("343737340F4736331F003B00")
[2025-06-11][23:46:00][keepkey_gui_lib::cache::frontload][INFO]    Found 1 USB devices to search
[2025-06-11][23:46:00][keepkey_gui_lib::cache::frontload][INFO] ✅ Found device by serial number match: 343737340F4736331F003B00
[2025-06-11][23:46:00][keepkey_gui_lib::cache::frontload][INFO] ✅ Created USB transport for device 343737340F4736331F003B00
[2025-06-11][23:46:00][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Starting with message type: GetAddress
[2025-06-11][23:46:00][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Calling parent adapter...
[2025-06-11][23:46:00][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Processing message type: GetAddress
[2025-06-11][23:46:00][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::send: Sending message type: GetAddress
[2025-06-11][23:46:00][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Waiting for response (timeout: 5s)...
[2025-06-11][23:46:00][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Received 44 bytes response
[2025-06-11][23:46:00][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Decoded response type: Address
[2025-06-11][23:46:00][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Parent returned message type: Address
[2025-06-11][23:46:00][keepkey_gui_lib::transport][INFO] StandardHandler: Processing message type: Address
[2025-06-11][23:46:00][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Handler returned None, done
[2025-06-11][23:46:00][keepkey_gui_lib::cache::device_cache][INFO] ✅ Verified address save for Bitcoin/p2sh-p2wpkh at path [2147483697, 2147483648, 2147483648, 0, 3] (address: 38iicuTPoRbqLA41EoSun39z7zSk17mkBA)
[2025-06-11][23:46:00][keepkey_gui_lib::cache::frontload][INFO] ✅ Cached missing Bitcoin p2sh-p2wpkh address at path [2147483697, 2147483648, 2147483648, 0, 3] for network bip122:000000000019d6689c085ae165831e93
[2025-06-11][23:46:00][tracing::span][INFO] create_device_transport;
[2025-06-11][23:46:00][keepkey_gui_lib::cache::frontload][INFO] 🔗 Creating transport using device factory for 343737340F4736331F003B00
[2025-06-11][23:46:00][tracing::span][INFO] create_transport_from_device_info;
[2025-06-11][23:46:00][keepkey_gui_lib::cache::frontload][INFO] 🔍 Looking for device with unique_id: 343737340F4736331F003B00
[2025-06-11][23:46:00][keepkey_gui_lib::cache::frontload][INFO]    Serial number: Some("343737340F4736331F003B00")
[2025-06-11][23:46:00][keepkey_gui_lib::cache::frontload][INFO]    Found 1 USB devices to search
[2025-06-11][23:46:00][keepkey_gui_lib::cache::frontload][INFO] ✅ Found device by serial number match: 343737340F4736331F003B00
[2025-06-11][23:46:00][keepkey_gui_lib::cache::frontload][INFO] ✅ Created USB transport for device 343737340F4736331F003B00
[2025-06-11][23:46:00][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Starting with message type: GetAddress
[2025-06-11][23:46:00][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Calling parent adapter...
[2025-06-11][23:46:00][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Processing message type: GetAddress
[2025-06-11][23:46:00][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::send: Sending message type: GetAddress
[2025-06-11][23:46:00][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Waiting for response (timeout: 5s)...
[2025-06-11][23:46:01][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Received 44 bytes response
[2025-06-11][23:46:01][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Decoded response type: Address
[2025-06-11][23:46:01][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Parent returned message type: Address
[2025-06-11][23:46:01][keepkey_gui_lib::transport][INFO] StandardHandler: Processing message type: Address
[2025-06-11][23:46:01][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Handler returned None, done
[2025-06-11][23:46:01][keepkey_gui_lib::cache::device_cache][INFO] ✅ Verified address save for Bitcoin/p2sh-p2wpkh at path [2147483697, 2147483648, 2147483648, 0, 4] (address: 34bVHWmnob1ABaVeCJY9nBxsPUxEXZMtvg)
[2025-06-11][23:46:01][keepkey_gui_lib::cache::frontload][INFO] ✅ Cached missing Bitcoin p2sh-p2wpkh address at path [2147483697, 2147483648, 2147483648, 0, 4] for network bip122:000000000019d6689c085ae165831e93
[2025-06-11][23:46:01][keepkey_gui_lib][INFO] Frontload progress: Loading Bitcoin addresses...
[2025-06-11][23:46:01][tracing::span][INFO] create_device_transport;
[2025-06-11][23:46:01][keepkey_gui_lib::cache::frontload][INFO] 🔗 Creating transport using device factory for 343737340F4736331F003B00
[2025-06-11][23:46:01][tracing::span][INFO] create_transport_from_device_info;
[2025-06-11][23:46:01][keepkey_gui_lib::cache::frontload][INFO] 🔍 Looking for device with unique_id: 343737340F4736331F003B00
[2025-06-11][23:46:01][keepkey_gui_lib::cache::frontload][INFO]    Serial number: Some("343737340F4736331F003B00")
[2025-06-11][23:46:01][keepkey_gui_lib::cache::frontload][INFO]    Found 1 USB devices to search
[2025-06-11][23:46:01][keepkey_gui_lib::cache::frontload][INFO] ✅ Found device by serial number match: 343737340F4736331F003B00
[2025-06-11][23:46:01][keepkey_gui_lib::cache::frontload][INFO] ✅ Created USB transport for device 343737340F4736331F003B00
[2025-06-11][23:46:01][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Starting with message type: GetPublicKey
[2025-06-11][23:46:01][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Calling parent adapter...
[2025-06-11][23:46:01][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Processing message type: GetPublicKey
[2025-06-11][23:46:01][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::send: Sending message type: GetPublicKey
[2025-06-11][23:46:01][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Waiting for response (timeout: 5s)...
[2025-06-11][23:46:01][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Received 206 bytes response
[2025-06-11][23:46:01][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Decoded response type: PublicKey
[2025-06-11][23:46:01][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Parent returned message type: PublicKey
[2025-06-11][23:46:01][keepkey_gui_lib::transport][INFO] StandardHandler: Processing message type: PublicKey
[2025-06-11][23:46:01][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Handler returned None, done
[2025-06-11][23:46:01][keepkey_gui_lib::cache::device_cache][INFO] ✅ Verified address save for Bitcoin/p2wpkh_xpub at path [2147483732, 2147483648, 2147483648] (address: zpub6rm1EEJg4JasiTqacdouiUVncAc5ymhKReiPZfLTGnH2GSZquRn9reJhj6sfs73PoSJNXzpERKPVLYbwwUGHNF6jkMX5R58vWaLB9FVyJuX)
[2025-06-11][23:46:01][keepkey_gui_lib::cache::frontload][INFO] ✅ Cached Bitcoin p2wpkh xpub: zpub6rm1EEJg4JasiTqacdouiUVncAc5ymhKReiPZfLTGnH2GSZquRn9reJhj6sfs73PoSJNXzpERKPVLYbwwUGHNF6jkMX5R58vWaLB9FVyJuX
[2025-06-11][23:46:01][keepkey_gui_lib::cache::frontload][INFO] ✅ Cached missing Bitcoin p2wpkh xpub: zpub6rm1EEJg4JasiTqacdouiUVncAc5ymhKReiPZfLTGnH2GSZquRn9reJhj6sfs73PoSJNXzpERKPVLYbwwUGHNF6jkMX5R58vWaLB9FVyJuX at path [2147483732, 2147483648, 2147483648] for network bip122:000000000019d6689c085ae165831e93
[2025-06-11][23:46:01][tracing::span][INFO] create_device_transport;
[2025-06-11][23:46:01][keepkey_gui_lib::cache::frontload][INFO] 🔗 Creating transport using device factory for 343737340F4736331F003B00
[2025-06-11][23:46:01][tracing::span][INFO] create_transport_from_device_info;
[2025-06-11][23:46:01][keepkey_gui_lib::cache::frontload][INFO] 🔍 Looking for device with unique_id: 343737340F4736331F003B00
[2025-06-11][23:46:01][keepkey_gui_lib::cache::frontload][INFO]    Serial number: Some("343737340F4736331F003B00")
[2025-06-11][23:46:01][keepkey_gui_lib::cache::frontload][INFO]    Found 1 USB devices to search
[2025-06-11][23:46:01][keepkey_gui_lib::cache::frontload][INFO] ✅ Found device by serial number match: 343737340F4736331F003B00
[2025-06-11][23:46:01][keepkey_gui_lib::cache::frontload][INFO] ✅ Created USB transport for device 343737340F4736331F003B00
[2025-06-11][23:46:01][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Starting with message type: GetAddress
[2025-06-11][23:46:01][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Calling parent adapter...
[2025-06-11][23:46:01][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Processing message type: GetAddress
[2025-06-11][23:46:01][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::send: Sending message type: GetAddress
[2025-06-11][23:46:01][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Waiting for response (timeout: 5s)...
[2025-06-11][23:46:02][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Received 52 bytes response
[2025-06-11][23:46:02][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Decoded response type: Address
[2025-06-11][23:46:02][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Parent returned message type: Address
[2025-06-11][23:46:02][keepkey_gui_lib::transport][INFO] StandardHandler: Processing message type: Address
[2025-06-11][23:46:02][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Handler returned None, done
[2025-06-11][23:46:02][keepkey_gui_lib::cache::device_cache][INFO] ✅ Verified address save for Bitcoin/p2wpkh at path [2147483732, 2147483648, 2147483648, 0, 0] (address: bc1q8w2ypqgx39gucxcypqv2m90wz9rvhmmrcnpdjs)
[2025-06-11][23:46:02][keepkey_gui_lib::cache::frontload][INFO] ✅ Cached missing Bitcoin p2wpkh address at path [2147483732, 2147483648, 2147483648, 0, 0] for network bip122:000000000019d6689c085ae165831e93
[2025-06-11][23:46:02][tracing::span][INFO] create_device_transport;
[2025-06-11][23:46:02][keepkey_gui_lib::cache::frontload][INFO] 🔗 Creating transport using device factory for 343737340F4736331F003B00
[2025-06-11][23:46:02][tracing::span][INFO] create_transport_from_device_info;
[2025-06-11][23:46:02][keepkey_gui_lib::cache::frontload][INFO] 🔍 Looking for device with unique_id: 343737340F4736331F003B00
[2025-06-11][23:46:02][keepkey_gui_lib::cache::frontload][INFO]    Serial number: Some("343737340F4736331F003B00")
[2025-06-11][23:46:02][keepkey_gui_lib::cache::frontload][INFO]    Found 1 USB devices to search
[2025-06-11][23:46:02][keepkey_gui_lib::cache::frontload][INFO] ✅ Found device by serial number match: 343737340F4736331F003B00
[2025-06-11][23:46:02][keepkey_gui_lib::cache::frontload][INFO] ✅ Created USB transport for device 343737340F4736331F003B00
[2025-06-11][23:46:02][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Starting with message type: GetAddress
[2025-06-11][23:46:02][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Calling parent adapter...
[2025-06-11][23:46:02][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Processing message type: GetAddress
[2025-06-11][23:46:02][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::send: Sending message type: GetAddress
[2025-06-11][23:46:02][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Waiting for response (timeout: 5s)...
[2025-06-11][23:46:03][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Received 52 bytes response
[2025-06-11][23:46:03][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Decoded response type: Address
[2025-06-11][23:46:03][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Parent returned message type: Address
[2025-06-11][23:46:03][keepkey_gui_lib::transport][INFO] StandardHandler: Processing message type: Address
[2025-06-11][23:46:03][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Handler returned None, done
[2025-06-11][23:46:03][keepkey_gui_lib::cache::device_cache][INFO] ✅ Verified address save for Bitcoin/p2wpkh at path [2147483732, 2147483648, 2147483648, 0, 1] (address: bc1qgajjp2rskq6kpcn0fnmq6zvlgkhuvlhwxlaen8)
[2025-06-11][23:46:03][keepkey_gui_lib::cache::frontload][INFO] ✅ Cached missing Bitcoin p2wpkh address at path [2147483732, 2147483648, 2147483648, 0, 1] for network bip122:000000000019d6689c085ae165831e93
[2025-06-11][23:46:03][tracing::span][INFO] create_device_transport;
[2025-06-11][23:46:03][keepkey_gui_lib::cache::frontload][INFO] 🔗 Creating transport using device factory for 343737340F4736331F003B00
[2025-06-11][23:46:03][tracing::span][INFO] create_transport_from_device_info;
[2025-06-11][23:46:03][keepkey_gui_lib::cache::frontload][INFO] 🔍 Looking for device with unique_id: 343737340F4736331F003B00
[2025-06-11][23:46:03][keepkey_gui_lib::cache::frontload][INFO]    Serial number: Some("343737340F4736331F003B00")
[2025-06-11][23:46:03][keepkey_gui_lib::cache::frontload][INFO]    Found 1 USB devices to search
[2025-06-11][23:46:03][keepkey_gui_lib::cache::frontload][INFO] ✅ Found device by serial number match: 343737340F4736331F003B00
[2025-06-11][23:46:03][keepkey_gui_lib::cache::frontload][INFO] ✅ Created USB transport for device 343737340F4736331F003B00
[2025-06-11][23:46:03][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Starting with message type: GetAddress
[2025-06-11][23:46:03][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Calling parent adapter...
[2025-06-11][23:46:03][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Processing message type: GetAddress
[2025-06-11][23:46:03][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::send: Sending message type: GetAddress
[2025-06-11][23:46:03][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Waiting for response (timeout: 5s)...
[2025-06-11][23:46:03][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Received 52 bytes response
[2025-06-11][23:46:03][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Decoded response type: Address
[2025-06-11][23:46:03][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Parent returned message type: Address
[2025-06-11][23:46:03][keepkey_gui_lib::transport][INFO] StandardHandler: Processing message type: Address
[2025-06-11][23:46:03][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Handler returned None, done
[2025-06-11][23:46:03][keepkey_gui_lib::cache::device_cache][INFO] ✅ Verified address save for Bitcoin/p2wpkh at path [2147483732, 2147483648, 2147483648, 0, 2] (address: bc1qnvyayyuuz53t2akytt990prvjw05mrav4hhlg3)
[2025-06-11][23:46:03][keepkey_gui_lib::cache::frontload][INFO] ✅ Cached missing Bitcoin p2wpkh address at path [2147483732, 2147483648, 2147483648, 0, 2] for network bip122:000000000019d6689c085ae165831e93
[2025-06-11][23:46:03][tracing::span][INFO] create_device_transport;
[2025-06-11][23:46:03][keepkey_gui_lib::cache::frontload][INFO] 🔗 Creating transport using device factory for 343737340F4736331F003B00
[2025-06-11][23:46:03][tracing::span][INFO] create_transport_from_device_info;
[2025-06-11][23:46:03][keepkey_gui_lib::cache::frontload][INFO] 🔍 Looking for device with unique_id: 343737340F4736331F003B00
[2025-06-11][23:46:03][keepkey_gui_lib::cache::frontload][INFO]    Serial number: Some("343737340F4736331F003B00")
[2025-06-11][23:46:03][keepkey_gui_lib::cache::frontload][INFO]    Found 1 USB devices to search
[2025-06-11][23:46:03][keepkey_gui_lib::cache::frontload][INFO] ✅ Found device by serial number match: 343737340F4736331F003B00
[2025-06-11][23:46:03][keepkey_gui_lib::cache::frontload][INFO] ✅ Created USB transport for device 343737340F4736331F003B00
[2025-06-11][23:46:03][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Starting with message type: GetAddress
[2025-06-11][23:46:03][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Calling parent adapter...
[2025-06-11][23:46:03][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Processing message type: GetAddress
[2025-06-11][23:46:03][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::send: Sending message type: GetAddress
[2025-06-11][23:46:03][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Waiting for response (timeout: 5s)...
[2025-06-11][23:46:04][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Received 52 bytes response
[2025-06-11][23:46:04][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Decoded response type: Address
[2025-06-11][23:46:04][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Parent returned message type: Address
[2025-06-11][23:46:04][keepkey_gui_lib::transport][INFO] StandardHandler: Processing message type: Address
[2025-06-11][23:46:04][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Handler returned None, done
[2025-06-11][23:46:04][keepkey_gui_lib::cache::device_cache][INFO] ✅ Verified address save for Bitcoin/p2wpkh at path [2147483732, 2147483648, 2147483648, 0, 3] (address: bc1qks93nydlxkqx7jrdx8c8fjtryzlkq89gg5rthm)
[2025-06-11][23:46:04][keepkey_gui_lib::cache::frontload][INFO] ✅ Cached missing Bitcoin p2wpkh address at path [2147483732, 2147483648, 2147483648, 0, 3] for network bip122:000000000019d6689c085ae165831e93
[2025-06-11][23:46:04][tracing::span][INFO] create_device_transport;
[2025-06-11][23:46:04][keepkey_gui_lib::cache::frontload][INFO] 🔗 Creating transport using device factory for 343737340F4736331F003B00
[2025-06-11][23:46:04][tracing::span][INFO] create_transport_from_device_info;
[2025-06-11][23:46:04][keepkey_gui_lib::cache::frontload][INFO] 🔍 Looking for device with unique_id: 343737340F4736331F003B00
[2025-06-11][23:46:04][keepkey_gui_lib::cache::frontload][INFO]    Serial number: Some("343737340F4736331F003B00")
[2025-06-11][23:46:04][keepkey_gui_lib::cache::frontload][INFO]    Found 1 USB devices to search
[2025-06-11][23:46:04][keepkey_gui_lib::cache::frontload][INFO] ✅ Found device by serial number match: 343737340F4736331F003B00
[2025-06-11][23:46:04][keepkey_gui_lib::cache::frontload][INFO] ✅ Created USB transport for device 343737340F4736331F003B00
[2025-06-11][23:46:04][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Starting with message type: GetAddress
[2025-06-11][23:46:04][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Calling parent adapter...
[2025-06-11][23:46:04][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Processing message type: GetAddress
[2025-06-11][23:46:04][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::send: Sending message type: GetAddress
[2025-06-11][23:46:04][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Waiting for response (timeout: 5s)...
[2025-06-11][23:46:04][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Received 52 bytes response
[2025-06-11][23:46:04][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Decoded response type: Address
[2025-06-11][23:46:04][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Parent returned message type: Address
[2025-06-11][23:46:04][keepkey_gui_lib::transport][INFO] StandardHandler: Processing message type: Address
[2025-06-11][23:46:04][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Handler returned None, done
[2025-06-11][23:46:04][keepkey_gui_lib::cache::device_cache][INFO] ✅ Verified address save for Bitcoin/p2wpkh at path [2147483732, 2147483648, 2147483648, 0, 4] (address: bc1q362scamhjsl9ggk5t9p4h5aqp6af6qmwzen828)
[2025-06-11][23:46:04][keepkey_gui_lib::cache::frontload][INFO] ✅ Cached missing Bitcoin p2wpkh address at path [2147483732, 2147483648, 2147483648, 0, 4] for network bip122:000000000019d6689c085ae165831e93
[2025-06-11][23:46:04][keepkey_gui_lib][INFO] Frontload progress: Loading Bitcoin addresses...
[2025-06-11][23:46:04][tracing::span][INFO] create_device_transport;
[2025-06-11][23:46:04][keepkey_gui_lib::cache::frontload][INFO] 🔗 Creating transport using device factory for 343737340F4736331F003B00
[2025-06-11][23:46:04][tracing::span][INFO] create_transport_from_device_info;
[2025-06-11][23:46:04][keepkey_gui_lib::cache::frontload][INFO] 🔍 Looking for device with unique_id: 343737340F4736331F003B00
[2025-06-11][23:46:04][keepkey_gui_lib::cache::frontload][INFO]    Serial number: Some("343737340F4736331F003B00")
[2025-06-11][23:46:04][keepkey_gui_lib::cache::frontload][INFO]    Found 1 USB devices to search
[2025-06-11][23:46:04][keepkey_gui_lib::cache::frontload][INFO] ✅ Found device by serial number match: 343737340F4736331F003B00
[2025-06-11][23:46:04][keepkey_gui_lib::cache::frontload][INFO] ✅ Created USB transport for device 343737340F4736331F003B00
[2025-06-11][23:46:04][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Starting with message type: GetPublicKey
[2025-06-11][23:46:04][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Calling parent adapter...
[2025-06-11][23:46:04][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Processing message type: GetPublicKey
[2025-06-11][23:46:04][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::send: Sending message type: GetPublicKey
[2025-06-11][23:46:04][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Waiting for response (timeout: 5s)...
[2025-06-11][23:46:05][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Received 206 bytes response
[2025-06-11][23:46:05][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Decoded response type: PublicKey
[2025-06-11][23:46:05][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Parent returned message type: PublicKey
[2025-06-11][23:46:05][keepkey_gui_lib::transport][INFO] StandardHandler: Processing message type: PublicKey
[2025-06-11][23:46:05][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Handler returned None, done
[2025-06-11][23:46:05][keepkey_gui_lib::cache::device_cache][INFO] ✅ Verified address save for Bitcoin/p2pkh_xpub at path [2147483649, 2147483648, 2147483648] (address: xpub6CLbypuZz2yjLeC2ujthma46NakKnvmLc2uCS6yQ6n9geQyotnB5pfiSax2WDggeiiR89StCz7fyvPQNw7MTSFLqkYB5DnyP8hxJ8Xpg5Th)
[2025-06-11][23:46:05][keepkey_gui_lib::cache::frontload][INFO] ✅ Cached Bitcoin p2pkh xpub: xpub6CLbypuZz2yjLeC2ujthma46NakKnvmLc2uCS6yQ6n9geQyotnB5pfiSax2WDggeiiR89StCz7fyvPQNw7MTSFLqkYB5DnyP8hxJ8Xpg5Th
[2025-06-11][23:46:05][keepkey_gui_lib::cache::frontload][INFO] ✅ Cached missing Bitcoin p2pkh xpub: xpub6CLbypuZz2yjLeC2ujthma46NakKnvmLc2uCS6yQ6n9geQyotnB5pfiSax2WDggeiiR89StCz7fyvPQNw7MTSFLqkYB5DnyP8hxJ8Xpg5Th at path [2147483649, 2147483648, 2147483648] for network bip122:000000000933ea01ad0ee984209779ba
[2025-06-11][23:46:05][tracing::span][INFO] create_device_transport;
[2025-06-11][23:46:05][keepkey_gui_lib::cache::frontload][INFO] 🔗 Creating transport using device factory for 343737340F4736331F003B00
[2025-06-11][23:46:05][tracing::span][INFO] create_transport_from_device_info;
[2025-06-11][23:46:05][keepkey_gui_lib::cache::frontload][INFO] 🔍 Looking for device with unique_id: 343737340F4736331F003B00
[2025-06-11][23:46:05][keepkey_gui_lib::cache::frontload][INFO]    Serial number: Some("343737340F4736331F003B00")
[2025-06-11][23:46:05][keepkey_gui_lib::cache::frontload][INFO]    Found 1 USB devices to search
[2025-06-11][23:46:05][keepkey_gui_lib::cache::frontload][INFO] ✅ Found device by serial number match: 343737340F4736331F003B00
[2025-06-11][23:46:05][keepkey_gui_lib::cache::frontload][INFO] ✅ Created USB transport for device 343737340F4736331F003B00
[2025-06-11][23:46:05][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Starting with message type: GetAddress
[2025-06-11][23:46:05][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Calling parent adapter...
[2025-06-11][23:46:05][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Processing message type: GetAddress
[2025-06-11][23:46:05][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::send: Sending message type: GetAddress
[2025-06-11][23:46:05][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Waiting for response (timeout: 5s)...
[2025-06-11][23:46:05][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Received 44 bytes response
[2025-06-11][23:46:05][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Decoded response type: Address
[2025-06-11][23:46:05][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Parent returned message type: Address
[2025-06-11][23:46:05][keepkey_gui_lib::transport][INFO] StandardHandler: Processing message type: Address
[2025-06-11][23:46:05][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Handler returned None, done
[2025-06-11][23:46:05][keepkey_gui_lib::cache::device_cache][INFO] ✅ Verified address save for Bitcoin/p2pkh at path [2147483649, 2147483648, 2147483648, 0, 0] (address: 181axw4Lw7Lm6rrBFLpfq4ZLiAYYyzJUF6)
[2025-06-11][23:46:05][keepkey_gui_lib::cache::frontload][INFO] ✅ Cached missing Bitcoin p2pkh address at path [2147483649, 2147483648, 2147483648, 0, 0] for network bip122:000000000933ea01ad0ee984209779ba
[2025-06-11][23:46:05][tracing::span][INFO] create_device_transport;
[2025-06-11][23:46:05][keepkey_gui_lib::cache::frontload][INFO] 🔗 Creating transport using device factory for 343737340F4736331F003B00
[2025-06-11][23:46:05][tracing::span][INFO] create_transport_from_device_info;
[2025-06-11][23:46:05][keepkey_gui_lib::cache::frontload][INFO] 🔍 Looking for device with unique_id: 343737340F4736331F003B00
[2025-06-11][23:46:05][keepkey_gui_lib::cache::frontload][INFO]    Serial number: Some("343737340F4736331F003B00")
[2025-06-11][23:46:05][keepkey_gui_lib::cache::frontload][INFO]    Found 1 USB devices to search
[2025-06-11][23:46:05][keepkey_gui_lib::cache::frontload][INFO] ✅ Found device by serial number match: 343737340F4736331F003B00
[2025-06-11][23:46:05][keepkey_gui_lib::cache::frontload][INFO] ✅ Created USB transport for device 343737340F4736331F003B00
[2025-06-11][23:46:05][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Starting with message type: GetAddress
[2025-06-11][23:46:05][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Calling parent adapter...
[2025-06-11][23:46:05][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Processing message type: GetAddress
[2025-06-11][23:46:05][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::send: Sending message type: GetAddress
[2025-06-11][23:46:05][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Waiting for response (timeout: 5s)...
[2025-06-11][23:46:06][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Received 44 bytes response
[2025-06-11][23:46:06][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Decoded response type: Address
[2025-06-11][23:46:06][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Parent returned message type: Address
[2025-06-11][23:46:06][keepkey_gui_lib::transport][INFO] StandardHandler: Processing message type: Address
[2025-06-11][23:46:06][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Handler returned None, done
[2025-06-11][23:46:06][keepkey_gui_lib::cache::device_cache][INFO] ✅ Verified address save for Bitcoin/p2pkh at path [2147483649, 2147483648, 2147483648, 0, 1] (address: 1GUwN13bUcpoM1HZsf1wq7fy3xCxf2y8ZA)
[2025-06-11][23:46:06][keepkey_gui_lib::cache::frontload][INFO] ✅ Cached missing Bitcoin p2pkh address at path [2147483649, 2147483648, 2147483648, 0, 1] for network bip122:000000000933ea01ad0ee984209779ba
[2025-06-11][23:46:06][tracing::span][INFO] create_device_transport;
[2025-06-11][23:46:06][keepkey_gui_lib::cache::frontload][INFO] 🔗 Creating transport using device factory for 343737340F4736331F003B00
[2025-06-11][23:46:06][tracing::span][INFO] create_transport_from_device_info;
[2025-06-11][23:46:06][keepkey_gui_lib::cache::frontload][INFO] 🔍 Looking for device with unique_id: 343737340F4736331F003B00
[2025-06-11][23:46:06][keepkey_gui_lib::cache::frontload][INFO]    Serial number: Some("343737340F4736331F003B00")
[2025-06-11][23:46:06][keepkey_gui_lib::cache::frontload][INFO]    Found 1 USB devices to search
[2025-06-11][23:46:06][keepkey_gui_lib::cache::frontload][INFO] ✅ Found device by serial number match: 343737340F4736331F003B00
[2025-06-11][23:46:06][keepkey_gui_lib::cache::frontload][INFO] ✅ Created USB transport for device 343737340F4736331F003B00
[2025-06-11][23:46:06][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Starting with message type: GetAddress
[2025-06-11][23:46:06][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Calling parent adapter...
[2025-06-11][23:46:06][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Processing message type: GetAddress
[2025-06-11][23:46:06][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::send: Sending message type: GetAddress
[2025-06-11][23:46:06][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Waiting for response (timeout: 5s)...
[2025-06-11][23:46:06][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Received 44 bytes response
[2025-06-11][23:46:06][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Decoded response type: Address
[2025-06-11][23:46:06][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Parent returned message type: Address
[2025-06-11][23:46:06][keepkey_gui_lib::transport][INFO] StandardHandler: Processing message type: Address
[2025-06-11][23:46:06][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Handler returned None, done
[2025-06-11][23:46:06][keepkey_gui_lib::cache::device_cache][INFO] ✅ Verified address save for Bitcoin/p2pkh at path [2147483649, 2147483648, 2147483648, 0, 2] (address: 1PEeFPYgJ7jrwt41ZHwTbfnLvRHTH1c6KR)
[2025-06-11][23:46:06][keepkey_gui_lib::cache::frontload][INFO] ✅ Cached missing Bitcoin p2pkh address at path [2147483649, 2147483648, 2147483648, 0, 2] for network bip122:000000000933ea01ad0ee984209779ba
[2025-06-11][23:46:06][tracing::span][INFO] create_device_transport;
[2025-06-11][23:46:06][keepkey_gui_lib::cache::frontload][INFO] 🔗 Creating transport using device factory for 343737340F4736331F003B00
[2025-06-11][23:46:06][tracing::span][INFO] create_transport_from_device_info;
[2025-06-11][23:46:06][keepkey_gui_lib::cache::frontload][INFO] 🔍 Looking for device with unique_id: 343737340F4736331F003B00
[2025-06-11][23:46:06][keepkey_gui_lib::cache::frontload][INFO]    Serial number: Some("343737340F4736331F003B00")
[2025-06-11][23:46:06][keepkey_gui_lib::cache::frontload][INFO]    Found 1 USB devices to search
[2025-06-11][23:46:06][keepkey_gui_lib::cache::frontload][INFO] ✅ Found device by serial number match: 343737340F4736331F003B00
[2025-06-11][23:46:06][keepkey_gui_lib::cache::frontload][INFO] ✅ Created USB transport for device 343737340F4736331F003B00
[2025-06-11][23:46:06][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Starting with message type: GetAddress
[2025-06-11][23:46:06][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Calling parent adapter...
[2025-06-11][23:46:06][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Processing message type: GetAddress
[2025-06-11][23:46:06][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::send: Sending message type: GetAddress
[2025-06-11][23:46:06][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Waiting for response (timeout: 5s)...
[2025-06-11][23:46:07][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Received 44 bytes response
[2025-06-11][23:46:07][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Decoded response type: Address
[2025-06-11][23:46:07][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Parent returned message type: Address
[2025-06-11][23:46:07][keepkey_gui_lib::transport][INFO] StandardHandler: Processing message type: Address
[2025-06-11][23:46:07][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Handler returned None, done
[2025-06-11][23:46:07][keepkey_gui_lib::cache::device_cache][INFO] ✅ Verified address save for Bitcoin/p2pkh at path [2147483649, 2147483648, 2147483648, 0, 3] (address: 1Gm6dSVBXihwDkmiCn2gnp1wkp3wD8EjJ3)
[2025-06-11][23:46:07][keepkey_gui_lib::cache::frontload][INFO] ✅ Cached missing Bitcoin p2pkh address at path [2147483649, 2147483648, 2147483648, 0, 3] for network bip122:000000000933ea01ad0ee984209779ba
[2025-06-11][23:46:07][tracing::span][INFO] create_device_transport;
[2025-06-11][23:46:07][keepkey_gui_lib::cache::frontload][INFO] 🔗 Creating transport using device factory for 343737340F4736331F003B00
[2025-06-11][23:46:07][tracing::span][INFO] create_transport_from_device_info;
[2025-06-11][23:46:07][keepkey_gui_lib::cache::frontload][INFO] 🔍 Looking for device with unique_id: 343737340F4736331F003B00
[2025-06-11][23:46:07][keepkey_gui_lib::cache::frontload][INFO]    Serial number: Some("343737340F4736331F003B00")
[2025-06-11][23:46:07][keepkey_gui_lib::cache::frontload][INFO]    Found 1 USB devices to search
[2025-06-11][23:46:07][keepkey_gui_lib::cache::frontload][INFO] ✅ Found device by serial number match: 343737340F4736331F003B00
[2025-06-11][23:46:07][keepkey_gui_lib::cache::frontload][INFO] ✅ Created USB transport for device 343737340F4736331F003B00
[2025-06-11][23:46:07][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Starting with message type: GetAddress
[2025-06-11][23:46:07][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Calling parent adapter...
[2025-06-11][23:46:07][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Processing message type: GetAddress
[2025-06-11][23:46:07][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::send: Sending message type: GetAddress
[2025-06-11][23:46:07][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Waiting for response (timeout: 5s)...
[2025-06-11][23:46:07][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Received 44 bytes response
[2025-06-11][23:46:07][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Decoded response type: Address
[2025-06-11][23:46:07][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Parent returned message type: Address
[2025-06-11][23:46:07][keepkey_gui_lib::transport][INFO] StandardHandler: Processing message type: Address
[2025-06-11][23:46:07][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Handler returned None, done
[2025-06-11][23:46:07][keepkey_gui_lib::cache::device_cache][INFO] ✅ Verified address save for Bitcoin/p2pkh at path [2147483649, 2147483648, 2147483648, 0, 4] (address: 1EDw1umZ68ZsqJfn1ocv3iE45xdAhm4mWd)
[2025-06-11][23:46:07][keepkey_gui_lib::cache::frontload][INFO] ✅ Cached missing Bitcoin p2pkh address at path [2147483649, 2147483648, 2147483648, 0, 4] for network bip122:000000000933ea01ad0ee984209779ba
[2025-06-11][23:46:07][keepkey_gui_lib::cache::frontload][INFO] 📍 Populated 23 missing addresses and xpubs from database paths
[2025-06-11][23:46:07][keepkey_gui_lib::cache::frontload][INFO] 💰 Trying to fetch balances from Pioneer API during frontload...
[2025-06-11][23:46:07][keepkey_gui_lib::cache::frontload][INFO] frontload_balances: Starting balance frontload for device 343737340F4736331F003B00
[2025-06-11][23:46:07][keepkey_gui_lib::cache::frontload][INFO] frontload_balances: Balances need refresh: true
[2025-06-11][23:46:07][keepkey_gui_lib::cache::frontload][INFO] frontload_balances: Refreshing balances from Pioneer API...
[2025-06-11][23:46:07][keepkey_gui_lib::cache::frontload][INFO] refresh_balances_from_pioneer: Using Pioneer server URL: https://pioneers.dev
[2025-06-11][23:46:07][tracing::span][INFO] create_device_transport;
[2025-06-11][23:46:07][keepkey_gui_lib::cache::frontload][INFO] 🔗 Creating transport using device factory for 343737340F4736331F003B00
[2025-06-11][23:46:07][tracing::span][INFO] create_transport_from_device_info;
[2025-06-11][23:46:07][keepkey_gui_lib::cache::frontload][INFO] 🔍 Looking for device with unique_id: 343737340F4736331F003B00
[2025-06-11][23:46:07][keepkey_gui_lib::cache::frontload][INFO]    Serial number: Some("343737340F4736331F003B00")
[2025-06-11][23:46:07][keepkey_gui_lib::cache::frontload][INFO]    Found 1 USB devices to search
[2025-06-11][23:46:07][keepkey_gui_lib::cache::frontload][INFO] ✅ Found device by serial number match: 343737340F4736331F003B00
[2025-06-11][23:46:07][keepkey_gui_lib::cache::frontload][INFO] ✅ Created USB transport for device 343737340F4736331F003B00
[2025-06-11][23:46:07][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Starting with message type: GetPublicKey
[2025-06-11][23:46:07][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Calling parent adapter...
[2025-06-11][23:46:07][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Processing message type: GetPublicKey
[2025-06-11][23:46:07][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::send: Sending message type: GetPublicKey
[2025-06-11][23:46:07][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Waiting for response (timeout: 5s)...
[2025-06-11][23:46:08][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Received 206 bytes response
[2025-06-11][23:46:08][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Decoded response type: PublicKey
[2025-06-11][23:46:08][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Parent returned message type: PublicKey
[2025-06-11][23:46:08][keepkey_gui_lib::transport][INFO] StandardHandler: Processing message type: PublicKey
[2025-06-11][23:46:08][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Handler returned None, done
[2025-06-11][23:46:08][keepkey_gui_lib::cache::device_cache][INFO] ✅ Verified address save for Bitcoin/p2pkh_xpub at path [2147483692, 2147483648, 2147483648] (address: xpub6BxKtd6aAuz23XqtWXeSqxShJZn8yqiUmaTdvsPWS3riKkNRcXEPmn1CXmKM1M43mrWfN5QwjdLRghZLrgwMLCeRZqZNuYhVNXr6Pp7aDsH)
[2025-06-11][23:46:08][keepkey_gui_lib::cache::frontload][INFO] ✅ Cached Bitcoin p2pkh xpub: xpub6BxKtd6aAuz23XqtWXeSqxShJZn8yqiUmaTdvsPWS3riKkNRcXEPmn1CXmKM1M43mrWfN5QwjdLRghZLrgwMLCeRZqZNuYhVNXr6Pp7aDsH
[2025-06-11][23:46:08][keepkey_gui_lib::cache::frontload][INFO] refresh_balances_from_pioneer: Adding UTXO asset query: caip=bip122:000000000019d6689c085ae165831e93/slip44:0, xpub=xpub6BxKtd6aAuz23XqtWXeSqxShJZn8yqiUmaTdvsPWS3riKkNRcXEPmn1CXmKM1M43mrWfN5QwjdLRghZLrgwMLCeRZqZNuYhVNXr6Pp7aDsH
[2025-06-11][23:46:08][keepkey_gui_lib::cache::frontload][INFO] refresh_balances_from_pioneer: Adding UTXO asset query: caip=bip122:000000000019d6689c085ae165831e93/slip44:0, xpub=ypub6WamSeXgTYgy7W25fVorMLDHFx5SPkuYaE7ToWCiyCUK2jdWpufQ8VqkDg83YjBtJFHDoekhf9ESdPDbL9aCPXC5NnmzXUiq3J6oycFShfS
[2025-06-11][23:46:08][keepkey_gui_lib::cache::frontload][INFO] refresh_balances_from_pioneer: Adding UTXO asset query: caip=bip122:000000000019d6689c085ae165831e93/slip44:0, xpub=zpub6rm1EEJg4JasiTqacdouiUVncAc5ymhKReiPZfLTGnH2GSZquRn9reJhj6sfs73PoSJNXzpERKPVLYbwwUGHNF6jkMX5R58vWaLB9FVyJuX
[2025-06-11][23:46:08][keepkey_gui_lib::cache::frontload][INFO] refresh_balances_from_pioneer: Fetching balances for 3 assets from https://pioneers.dev
[2025-06-11][23:46:08][keepkey_gui_lib::cache::frontload][INFO] refresh_balances_from_pioneer: Making POST request to: https://pioneers.dev/api/v1/portfolio
[2025-06-11][23:46:08][keepkey_gui_lib::cache::frontload][INFO] refresh_balances_from_pioneer: Request payload:
[
{
"caip": "bip122:000000000019d6689c085ae165831e93/slip44:0",
"pubkey": "xpub6BxKtd6aAuz23XqtWXeSqxShJZn8yqiUmaTdvsPWS3riKkNRcXEPmn1CXmKM1M43mrWfN5QwjdLRghZLrgwMLCeRZqZNuYhVNXr6Pp7aDsH"
},
{
"caip": "bip122:000000000019d6689c085ae165831e93/slip44:0",
"pubkey": "ypub6WamSeXgTYgy7W25fVorMLDHFx5SPkuYaE7ToWCiyCUK2jdWpufQ8VqkDg83YjBtJFHDoekhf9ESdPDbL9aCPXC5NnmzXUiq3J6oycFShfS"
},
{
"caip": "bip122:000000000019d6689c085ae165831e93/slip44:0",
"pubkey": "zpub6rm1EEJg4JasiTqacdouiUVncAc5ymhKReiPZfLTGnH2GSZquRn9reJhj6sfs73PoSJNXzpERKPVLYbwwUGHNF6jkMX5R58vWaLB9FVyJuX"
}
]
[2025-06-11][23:46:09][keepkey_gui_lib::cache::frontload][INFO] refresh_balances_from_pioneer: Response status: 200 OK
[2025-06-11][23:46:09][keepkey_gui_lib::cache::frontload][INFO] refresh_balances_from_pioneer: Response headers: {"date": "Wed, 11 Jun 2025 23:46:09 GMT", "content-type": "application/json; charset=utf-8", "content-length": "727", "connection": "keep-alive", "x-powered-by": "Express", "vary": "Origin, X-HTTP-Method-Override", "etag": "W/\"2d7-iZrUdlNvcEG3g6apXy++aQ6Tahk\"", "cf-cache-status": "DYNAMIC", "nel": "{\"report_to\":\"cf-nel\",\"success_fraction\":0.0,\"max_age\":604800}", "report-to": "{\"group\":\"cf-nel\",\"max_age\":604800,\"endpoints\":[{\"url\":\"https://a.nel.cloudflare.com/report/v4?s=5ox477KlxyA5R5cbnvgUYIZ6v%2B6%2F1blAtllqkCOdQZvjUf3poNq6GjNOnSZoDaxDD8%2BgskD%2BgE1JxT8Lkhsv3PlVHfsmJVmIw4ZtKA%3D%3D\"}]}", "server": "cloudflare", "cf-ray": "94e4fc72eece6b6d-DFW", "alt-svc": "h3=\":443\"; ma=86400"}
[2025-06-11][23:46:09][keepkey_gui_lib::cache::frontload][INFO] refresh_balances_from_pioneer: Raw response body:
[{"caip":"bip122:000000000019d6689c085ae165831e93/slip44:0","pubkey":"xpub6BxKtd6aAuz23XqtWXeSqxShJZn8yqiUmaTdvsPWS3riKkNRcXEPmn1CXmKM1M43mrWfN5QwjdLRghZLrgwMLCeRZqZNuYhVNXr6Pp7aDsH","balance":"0.00000000","priceUsd":"0.00","valueUsd":"0.00"},{"caip":"bip122:000000000019d6689c085ae165831e93/slip44:0","pubkey":"ypub6WamSeXgTYgy7W25fVorMLDHFx5SPkuYaE7ToWCiyCUK2jdWpufQ8VqkDg83YjBtJFHDoekhf9ESdPDbL9aCPXC5NnmzXUiq3J6oycFShfS","balance":"0.00000000","priceUsd":"0.00","valueUsd":"0.00"},{"caip":"bip122:000000000019d6689c085ae165831e93/slip44:0","pubkey":"zpub6rm1EEJg4JasiTqacdouiUVncAc5ymhKReiPZfLTGnH2GSZquRn9reJhj6sfs73PoSJNXzpERKPVLYbwwUGHNF6jkMX5R58vWaLB9FVyJuX","balance":"0.00000000","priceUsd":"0.00","valueUsd":"0.00"}]
[2025-06-11][23:46:09][keepkey_gui_lib::cache::frontload][INFO] refresh_balances_from_pioneer: Parsed 3 balance responses
[2025-06-11][23:46:09][keepkey_gui_lib::cache::frontload][INFO] refresh_balances_from_pioneer: 🎯 PIONEER API RESPONSE (Pretty Print):
[2025-06-11][23:46:09][keepkey_gui_lib::cache::frontload][INFO] refresh_balances_from_pioneer: Balance #1: {
"balance": "0.00000000",
"caip": "bip122:000000000019d6689c085ae165831e93/slip44:0",
"priceUsd": "0.00",
"pubkey": "xpub6BxKtd6aAuz23XqtWXeSqxShJZn8yqiUmaTdvsPWS3riKkNRcXEPmn1CXmKM1M43mrWfN5QwjdLRghZLrgwMLCeRZqZNuYhVNXr6Pp7aDsH",
"valueUsd": "0.00"
}
[2025-06-11][23:46:09][keepkey_gui_lib::cache::frontload][INFO] refresh_balances_from_pioneer: Balance #2: {
"balance": "0.00000000",
"caip": "bip122:000000000019d6689c085ae165831e93/slip44:0",
"priceUsd": "0.00",
"pubkey": "ypub6WamSeXgTYgy7W25fVorMLDHFx5SPkuYaE7ToWCiyCUK2jdWpufQ8VqkDg83YjBtJFHDoekhf9ESdPDbL9aCPXC5NnmzXUiq3J6oycFShfS",
"valueUsd": "0.00"
}
[2025-06-11][23:46:09][keepkey_gui_lib::cache::frontload][INFO] refresh_balances_from_pioneer: Balance #3: {
"balance": "0.00000000",
"caip": "bip122:000000000019d6689c085ae165831e93/slip44:0",
"priceUsd": "0.00",
"pubkey": "zpub6rm1EEJg4JasiTqacdouiUVncAc5ymhKReiPZfLTGnH2GSZquRn9reJhj6sfs73PoSJNXzpERKPVLYbwwUGHNF6jkMX5R58vWaLB9FVyJuX",
"valueUsd": "0.00"
}
[2025-06-11][23:46:09][keepkey_gui_lib::cache::frontload][INFO] refresh_balances_from_pioneer: Processing balance data: {
"balance": "0.00000000",
"caip": "bip122:000000000019d6689c085ae165831e93/slip44:0",
"priceUsd": "0.00",
"pubkey": "xpub6BxKtd6aAuz23XqtWXeSqxShJZn8yqiUmaTdvsPWS3riKkNRcXEPmn1CXmKM1M43mrWfN5QwjdLRghZLrgwMLCeRZqZNuYhVNXr6Pp7aDsH",
"valueUsd": "0.00"
}
[2025-06-11][23:46:09][keepkey_gui_lib::cache::frontload][INFO] refresh_balances_from_pioneer: ✅ Created cached balance: caip=bip122:000000000019d6689c085ae165831e93/slip44:0, pubkey=xpub6BxKtd6aAuz23XqtWXeSqxShJZn8yqiUmaTdvsPWS3riKkNRcXEPmn1CXmKM1M43mrWfN5QwjdLRghZLrgwMLCeRZqZNuYhVNXr6Pp7aDsH, balance=0.00000000, value_usd=$0.00
[2025-06-11][23:46:09][keepkey_gui_lib::cache::frontload][INFO] refresh_balances_from_pioneer: Processing balance data: {
"balance": "0.00000000",
"caip": "bip122:000000000019d6689c085ae165831e93/slip44:0",
"priceUsd": "0.00",
"pubkey": "ypub6WamSeXgTYgy7W25fVorMLDHFx5SPkuYaE7ToWCiyCUK2jdWpufQ8VqkDg83YjBtJFHDoekhf9ESdPDbL9aCPXC5NnmzXUiq3J6oycFShfS",
"valueUsd": "0.00"
}
[2025-06-11][23:46:09][keepkey_gui_lib::cache::frontload][INFO] refresh_balances_from_pioneer: ✅ Created cached balance: caip=bip122:000000000019d6689c085ae165831e93/slip44:0, pubkey=ypub6WamSeXgTYgy7W25fVorMLDHFx5SPkuYaE7ToWCiyCUK2jdWpufQ8VqkDg83YjBtJFHDoekhf9ESdPDbL9aCPXC5NnmzXUiq3J6oycFShfS, balance=0.00000000, value_usd=$0.00
[2025-06-11][23:46:09][keepkey_gui_lib::cache::frontload][INFO] refresh_balances_from_pioneer: Processing balance data: {
"balance": "0.00000000",
"caip": "bip122:000000000019d6689c085ae165831e93/slip44:0",
"priceUsd": "0.00",
"pubkey": "zpub6rm1EEJg4JasiTqacdouiUVncAc5ymhKReiPZfLTGnH2GSZquRn9reJhj6sfs73PoSJNXzpERKPVLYbwwUGHNF6jkMX5R58vWaLB9FVyJuX",
"valueUsd": "0.00"
}
[2025-06-11][23:46:09][keepkey_gui_lib::cache::frontload][INFO] refresh_balances_from_pioneer: ✅ Created cached balance: caip=bip122:000000000019d6689c085ae165831e93/slip44:0, pubkey=zpub6rm1EEJg4JasiTqacdouiUVncAc5ymhKReiPZfLTGnH2GSZquRn9reJhj6sfs73PoSJNXzpERKPVLYbwwUGHNF6jkMX5R58vWaLB9FVyJuX, balance=0.00000000, value_usd=$0.00
[2025-06-11][23:46:09][keepkey_gui_lib::cache::frontload][INFO] refresh_balances_from_pioneer: 💾 SAVING TO DATABASE (Pretty Print):
[2025-06-11][23:46:09][keepkey_gui_lib::cache::frontload][INFO] refresh_balances_from_pioneer: Cached Balance #1: {
"balance": "0.00000000",
"caip": "bip122:000000000019d6689c085ae165831e93/slip44:0",
"last_updated": 1749685569,
"network_id": "bip122:000000000019d6689c085ae165831e93",
"price_usd": "0.00",
"pubkey": "xpub6BxKtd6aAuz23XqtWXeSqxShJZn8yqiUmaTdvsPWS3riKkNRcXEPmn1CXmKM1M43mrWfN5QwjdLRghZLrgwMLCeRZqZNuYhVNXr6Pp7aDsH",
"symbol": "BTC",
"value_usd": "0.00"
}
[2025-06-11][23:46:09][keepkey_gui_lib::cache::frontload][INFO] refresh_balances_from_pioneer: Cached Balance #2: {
"balance": "0.00000000",
"caip": "bip122:000000000019d6689c085ae165831e93/slip44:0",
"last_updated": 1749685569,
"network_id": "bip122:000000000019d6689c085ae165831e93",
"price_usd": "0.00",
"pubkey": "ypub6WamSeXgTYgy7W25fVorMLDHFx5SPkuYaE7ToWCiyCUK2jdWpufQ8VqkDg83YjBtJFHDoekhf9ESdPDbL9aCPXC5NnmzXUiq3J6oycFShfS",
"symbol": "BTC",
"value_usd": "0.00"
}
[2025-06-11][23:46:09][keepkey_gui_lib::cache::frontload][INFO] refresh_balances_from_pioneer: Cached Balance #3: {
"balance": "0.00000000",
"caip": "bip122:000000000019d6689c085ae165831e93/slip44:0",
"last_updated": 1749685569,
"network_id": "bip122:000000000019d6689c085ae165831e93",
"price_usd": "0.00",
"pubkey": "zpub6rm1EEJg4JasiTqacdouiUVncAc5ymhKReiPZfLTGnH2GSZquRn9reJhj6sfs73PoSJNXzpERKPVLYbwwUGHNF6jkMX5R58vWaLB9FVyJuX",
"symbol": "BTC",
"value_usd": "0.00"
}
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] Saved 3 balances for device 343737340F4736331F003B00
[2025-06-11][23:46:09][keepkey_gui_lib::cache::frontload][INFO] refresh_balances_from_pioneer: Successfully cached 3 balances
[2025-06-11][23:46:09][keepkey_gui_lib::cache::frontload][INFO] frontload_balances: ✅ Successfully frontloaded balances from Pioneer API
[2025-06-11][23:46:09][keepkey_gui_lib::cache::frontload][INFO] frontload_balances: 📊 Cached 3 balances worth $0.00 USD
[2025-06-11][23:46:09][keepkey_gui_lib::cache::frontload][INFO] frontload_balances: 💎 bip122:000000000019d6689c085ae165831e93/slip44:0 = 0.00000000 ($0.00 USD)
[2025-06-11][23:46:09][keepkey_gui_lib::cache::frontload][INFO] frontload_balances: 💎 bip122:000000000019d6689c085ae165831e93/slip44:0 = 0.00000000 ($0.00 USD)
[2025-06-11][23:46:09][keepkey_gui_lib::cache::frontload][INFO] frontload_balances: 💎 bip122:000000000019d6689c085ae165831e93/slip44:0 = 0.00000000 ($0.00 USD)
[2025-06-11][23:46:09][keepkey_gui_lib::cache::frontload][INFO] ✅ Device frontload complete!
[2025-06-11][23:46:09][keepkey_gui_lib::cache::frontload][INFO]    📊 Populated 23 missing addresses in 19.12s
[2025-06-11][23:46:09][keepkey_gui_lib::cache::frontload][INFO]    💾 Using database cache for fast startup
[2025-06-11][23:46:09][keepkey_gui_lib::cache::frontload][INFO]    🏷️  Device: KeepKey3
[2025-06-11][23:46:09][keepkey_gui_lib::cache::frontload][INFO] 🔍 DEBUG: Testing address reading immediately after frontload...
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Starting address load for device 343737340F4736331F003B00
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Acquired database lock
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Device exists in devices table: true
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Address count from COUNT query: 24
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Prepared statement successfully
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Processing row - coin: Bitcoin, script_type: p2pkh, path_json: [2147483692,2147483648,2147483648,0,0]
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Successfully parsed path: [2147483692, 2147483648, 2147483648, 0, 0]
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Processing row 0
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Successfully processed address: Bitcoin/p2pkh/1JNYtQsc1pizKbn3ScbEPfQ7WcxNqeUHNB
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Processing row - coin: Bitcoin, script_type: p2pkh, path_json: [2147483692,2147483648,2147483648,0,1]
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Successfully parsed path: [2147483692, 2147483648, 2147483648, 0, 1]
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Processing row 1
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Successfully processed address: Bitcoin/p2pkh/19QvcGC3H5EXSzfTiAVsnvCyJRj9WxmMeq
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Processing row - coin: Bitcoin, script_type: p2pkh, path_json: [2147483692,2147483648,2147483648,0,2]
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Successfully parsed path: [2147483692, 2147483648, 2147483648, 0, 2]
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Processing row 2
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Successfully processed address: Bitcoin/p2pkh/1MA2a6ELwt8qgK6RSUPE6rMkKxxfHetexo
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Processing row - coin: Bitcoin, script_type: p2pkh, path_json: [2147483692,2147483648,2147483648,0,3]
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Successfully parsed path: [2147483692, 2147483648, 2147483648, 0, 3]
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Processing row 3
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Successfully processed address: Bitcoin/p2pkh/13DmjYe2exmwoULgChcQaSYU9mAmC9qPvf
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Processing row - coin: Bitcoin, script_type: p2pkh, path_json: [2147483692,2147483648,2147483648,0,4]
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Successfully parsed path: [2147483692, 2147483648, 2147483648, 0, 4]
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Processing row 4
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Successfully processed address: Bitcoin/p2pkh/1P1a4PuYNTjUAVJsE3oex4VAqgqkCSoAP6
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Processing row - coin: Bitcoin, script_type: p2sh-p2wpkh_xpub, path_json: [2147483697,2147483648,2147483648]
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Successfully parsed path: [2147483697, 2147483648, 2147483648]
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Processing row 5
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Successfully processed address: Bitcoin/p2sh-p2wpkh_xpub/ypub6WamSeXgTYgy7W25fVorMLDHFx5SPkuYaE7ToWCiyCUK2jdWpufQ8VqkDg83YjBtJFHDoekhf9ESdPDbL9aCPXC5NnmzXUiq3J6oycFShfS
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Processing row - coin: Bitcoin, script_type: p2sh-p2wpkh, path_json: [2147483697,2147483648,2147483648,0,0]
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Successfully parsed path: [2147483697, 2147483648, 2147483648, 0, 0]
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Processing row 6
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Successfully processed address: Bitcoin/p2sh-p2wpkh/3M9rBdu7rkVGwmt9gALjuRopAqpVEBdNRR
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Processing row - coin: Bitcoin, script_type: p2sh-p2wpkh, path_json: [2147483697,2147483648,2147483648,0,1]
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Successfully parsed path: [2147483697, 2147483648, 2147483648, 0, 1]
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Processing row 7
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Successfully processed address: Bitcoin/p2sh-p2wpkh/3PczoXGFPPTw5idJ6gzQ6DsHuWdVBGyPiB
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Processing row - coin: Bitcoin, script_type: p2sh-p2wpkh, path_json: [2147483697,2147483648,2147483648,0,2]
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Successfully parsed path: [2147483697, 2147483648, 2147483648, 0, 2]
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Processing row 8
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Successfully processed address: Bitcoin/p2sh-p2wpkh/3Ds7s2C91SoEynKWJWkBrE4nPoLV8YxryG
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Processing row - coin: Bitcoin, script_type: p2sh-p2wpkh, path_json: [2147483697,2147483648,2147483648,0,3]
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Successfully parsed path: [2147483697, 2147483648, 2147483648, 0, 3]
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Processing row 9
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Successfully processed address: Bitcoin/p2sh-p2wpkh/38iicuTPoRbqLA41EoSun39z7zSk17mkBA
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Processing row - coin: Bitcoin, script_type: p2sh-p2wpkh, path_json: [2147483697,2147483648,2147483648,0,4]
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Successfully parsed path: [2147483697, 2147483648, 2147483648, 0, 4]
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Processing row 10
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Successfully processed address: Bitcoin/p2sh-p2wpkh/34bVHWmnob1ABaVeCJY9nBxsPUxEXZMtvg
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Processing row - coin: Bitcoin, script_type: p2wpkh_xpub, path_json: [2147483732,2147483648,2147483648]
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Successfully parsed path: [2147483732, 2147483648, 2147483648]
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Processing row 11
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Successfully processed address: Bitcoin/p2wpkh_xpub/zpub6rm1EEJg4JasiTqacdouiUVncAc5ymhKReiPZfLTGnH2GSZquRn9reJhj6sfs73PoSJNXzpERKPVLYbwwUGHNF6jkMX5R58vWaLB9FVyJuX
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Processing row - coin: Bitcoin, script_type: p2wpkh, path_json: [2147483732,2147483648,2147483648,0,0]
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Successfully parsed path: [2147483732, 2147483648, 2147483648, 0, 0]
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Processing row 12
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Successfully processed address: Bitcoin/p2wpkh/bc1q8w2ypqgx39gucxcypqv2m90wz9rvhmmrcnpdjs
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Processing row - coin: Bitcoin, script_type: p2wpkh, path_json: [2147483732,2147483648,2147483648,0,1]
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Successfully parsed path: [2147483732, 2147483648, 2147483648, 0, 1]
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Processing row 13
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Successfully processed address: Bitcoin/p2wpkh/bc1qgajjp2rskq6kpcn0fnmq6zvlgkhuvlhwxlaen8
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Processing row - coin: Bitcoin, script_type: p2wpkh, path_json: [2147483732,2147483648,2147483648,0,2]
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Successfully parsed path: [2147483732, 2147483648, 2147483648, 0, 2]
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Processing row 14
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Successfully processed address: Bitcoin/p2wpkh/bc1qnvyayyuuz53t2akytt990prvjw05mrav4hhlg3
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Processing row - coin: Bitcoin, script_type: p2wpkh, path_json: [2147483732,2147483648,2147483648,0,3]
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Successfully parsed path: [2147483732, 2147483648, 2147483648, 0, 3]
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Processing row 15
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Successfully processed address: Bitcoin/p2wpkh/bc1qks93nydlxkqx7jrdx8c8fjtryzlkq89gg5rthm
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Processing row - coin: Bitcoin, script_type: p2wpkh, path_json: [2147483732,2147483648,2147483648,0,4]
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Successfully parsed path: [2147483732, 2147483648, 2147483648, 0, 4]
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Processing row 16
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Successfully processed address: Bitcoin/p2wpkh/bc1q362scamhjsl9ggk5t9p4h5aqp6af6qmwzen828
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Processing row - coin: Bitcoin, script_type: p2pkh_xpub, path_json: [2147483649,2147483648,2147483648]
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Successfully parsed path: [2147483649, 2147483648, 2147483648]
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Processing row 17
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Successfully processed address: Bitcoin/p2pkh_xpub/xpub6CLbypuZz2yjLeC2ujthma46NakKnvmLc2uCS6yQ6n9geQyotnB5pfiSax2WDggeiiR89StCz7fyvPQNw7MTSFLqkYB5DnyP8hxJ8Xpg5Th
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Processing row - coin: Bitcoin, script_type: p2pkh, path_json: [2147483649,2147483648,2147483648,0,0]
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Successfully parsed path: [2147483649, 2147483648, 2147483648, 0, 0]
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Processing row 18
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Successfully processed address: Bitcoin/p2pkh/181axw4Lw7Lm6rrBFLpfq4ZLiAYYyzJUF6
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Processing row - coin: Bitcoin, script_type: p2pkh, path_json: [2147483649,2147483648,2147483648,0,1]
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Successfully parsed path: [2147483649, 2147483648, 2147483648, 0, 1]
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Processing row 19
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Successfully processed address: Bitcoin/p2pkh/1GUwN13bUcpoM1HZsf1wq7fy3xCxf2y8ZA
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Processing row - coin: Bitcoin, script_type: p2pkh, path_json: [2147483649,2147483648,2147483648,0,2]
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Successfully parsed path: [2147483649, 2147483648, 2147483648, 0, 2]
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Processing row 20
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Successfully processed address: Bitcoin/p2pkh/1PEeFPYgJ7jrwt41ZHwTbfnLvRHTH1c6KR
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Processing row - coin: Bitcoin, script_type: p2pkh, path_json: [2147483649,2147483648,2147483648,0,3]
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Successfully parsed path: [2147483649, 2147483648, 2147483648, 0, 3]
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Processing row 21
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Successfully processed address: Bitcoin/p2pkh/1Gm6dSVBXihwDkmiCn2gnp1wkp3wD8EjJ3
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Processing row - coin: Bitcoin, script_type: p2pkh, path_json: [2147483649,2147483648,2147483648,0,4]
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Successfully parsed path: [2147483649, 2147483648, 2147483648, 0, 4]
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Processing row 22
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Successfully processed address: Bitcoin/p2pkh/1EDw1umZ68ZsqJfn1ocv3iE45xdAhm4mWd
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Processing row - coin: Bitcoin, script_type: p2pkh_xpub, path_json: [2147483692,2147483648,2147483648]
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Successfully parsed path: [2147483692, 2147483648, 2147483648]
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Processing row 23
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Successfully processed address: Bitcoin/p2pkh_xpub/xpub6BxKtd6aAuz23XqtWXeSqxShJZn8yqiUmaTdvsPWS3riKkNRcXEPmn1CXmKM1M43mrWfN5QwjdLRghZLrgwMLCeRZqZNuYhVNXr6Pp7aDsH
[2025-06-11][23:46:09][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: Final result - loaded 24 addresses
[2025-06-11][23:46:09][keepkey_gui_lib::cache::frontload][INFO] 🔍 DEBUG: Successfully read 24 addresses: ["Bitcoin/p2pkh/1JNYtQsc1pizKbn3ScbEPfQ7WcxNqeUHNB", "Bitcoin/p2pkh/19QvcGC3H5EXSzfTiAVsnvCyJRj9WxmMeq", "Bitcoin/p2pkh/1MA2a6ELwt8qgK6RSUPE6rMkKxxfHetexo", "Bitcoin/p2pkh/13DmjYe2exmwoULgChcQaSYU9mAmC9qPvf", "Bitcoin/p2pkh/1P1a4PuYNTjUAVJsE3oex4VAqgqkCSoAP6", "Bitcoin/p2sh-p2wpkh_xpub/ypub6WamSeXgTYgy7W25fVorMLDHFx5SPkuYaE7ToWCiyCUK2jdWpufQ8VqkDg83YjBtJFHDoekhf9ESdPDbL9aCPXC5NnmzXUiq3J6oycFShfS", "Bitcoin/p2sh-p2wpkh/3M9rBdu7rkVGwmt9gALjuRopAqpVEBdNRR", "Bitcoin/p2sh-p2wpkh/3PczoXGFPPTw5idJ6gzQ6DsHuWdVBGyPiB", "Bitcoin/p2sh-p2wpkh/3Ds7s2C91SoEynKWJWkBrE4nPoLV8YxryG", "Bitcoin/p2sh-p2wpkh/38iicuTPoRbqLA41EoSun39z7zSk17mkBA", "Bitcoin/p2sh-p2wpkh/34bVHWmnob1ABaVeCJY9nBxsPUxEXZMtvg", "Bitcoin/p2wpkh_xpub/zpub6rm1EEJg4JasiTqacdouiUVncAc5ymhKReiPZfLTGnH2GSZquRn9reJhj6sfs73PoSJNXzpERKPVLYbwwUGHNF6jkMX5R58vWaLB9FVyJuX", "Bitcoin/p2wpkh/bc1q8w2ypqgx39gucxcypqv2m90wz9rvhmmrcnpdjs", "Bitcoin/p2wpkh/bc1qgajjp2rskq6kpcn0fnmq6zvlgkhuvlhwxlaen8", "Bitcoin/p2wpkh/bc1qnvyayyuuz53t2akytt990prvjw05mrav4hhlg3", "Bitcoin/p2wpkh/bc1qks93nydlxkqx7jrdx8c8fjtryzlkq89gg5rthm", "Bitcoin/p2wpkh/bc1q362scamhjsl9ggk5t9p4h5aqp6af6qmwzen828", "Bitcoin/p2pkh_xpub/xpub6CLbypuZz2yjLeC2ujthma46NakKnvmLc2uCS6yQ6n9geQyotnB5pfiSax2WDggeiiR89StCz7fyvPQNw7MTSFLqkYB5DnyP8hxJ8Xpg5Th", "Bitcoin/p2pkh/181axw4Lw7Lm6rrBFLpfq4ZLiAYYyzJUF6", "Bitcoin/p2pkh/1GUwN13bUcpoM1HZsf1wq7fy3xCxf2y8ZA", "Bitcoin/p2pkh/1PEeFPYgJ7jrwt41ZHwTbfnLvRHTH1c6KR", "Bitcoin/p2pkh/1Gm6dSVBXihwDkmiCn2gnp1wkp3wD8EjJ3", "Bitcoin/p2pkh/1EDw1umZ68ZsqJfn1ocv3iE45xdAhm4mWd", "Bitcoin/p2pkh_xpub/xpub6BxKtd6aAuz23XqtWXeSqxShJZn8yqiUmaTdvsPWS3riKkNRcXEPmn1CXmKM1M43mrWfN5QwjdLRghZLrgwMLCeRZqZNuYhVNXr6Pp7aDsH"]
[2025-06-11][23:46:09][keepkey_gui_lib][INFO] Frontload completed successfully for device 343737340F4736331F003B00
[2025-06-11][23:46:09][keepkey_gui_lib][INFO] Setting device context to 343737340F4736331F003B00
[2025-06-11][23:46:09][keepkey_gui_lib::server::context][INFO] Getting device features to save to database for device 343737340F4736331F003B00
[2025-06-11][23:46:09][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Starting with message type: GetFeatures
[2025-06-11][23:46:09][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Calling parent adapter...
[2025-06-11][23:46:09][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Processing message type: GetFeatures
[2025-06-11][23:46:09][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::send: Sending message type: GetFeatures
[2025-06-11][23:46:09][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Waiting for response (timeout: 5s)...
[2025-06-11][23:46:10][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Received 289 bytes response
[2025-06-11][23:46:10][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Decoded response type: Features
[2025-06-11][23:46:10][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Parent returned message type: Features
[2025-06-11][23:46:10][keepkey_gui_lib::transport][INFO] StandardHandler: Processing message type: Features
[2025-06-11][23:46:10][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Handler returned None, done
[2025-06-11][23:46:10][keepkey_gui_lib::server::context][INFO] ✅ Got device features for 343737340F4736331F003B00
[2025-06-11][23:46:10][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: cache_dir resolved to: /Users/highlander/.keepkey/kkcli
[2025-06-11][23:46:10][keepkey_gui_lib::cache::device_cache][INFO] 🔍 DEBUG: db_path resolved to: /Users/highlander/.keepkey/kkcli/device_cache.db
[2025-06-11][23:46:10][keepkey_gui_lib::cache::device_cache][INFO] Opening device cache at: /Users/highlander/.keepkey/kkcli/device_cache.db
[2025-06-11][23:46:10][keepkey_gui_lib::cache::device_cache][INFO] Saved features for device 343737340F4736331F003B00
[2025-06-11][23:46:10][keepkey_gui_lib::server::context][INFO] ✅ Saved device features to database for 343737340F4736331F003B00
[2025-06-11][23:46:10][keepkey_gui_lib::server::context][INFO] Getting real Ethereum address from device 343737340F4736331F003B00
[2025-06-11][23:46:10][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Starting with message type: EthereumGetAddress
[2025-06-11][23:46:10][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Calling parent adapter...
[2025-06-11][23:46:10][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Processing message type: EthereumGetAddress
[2025-06-11][23:46:10][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::send: Sending message type: EthereumGetAddress
[2025-06-11][23:46:10][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Waiting for response (timeout: 5s)...
[2025-06-11][23:46:10][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Received 74 bytes response
[2025-06-11][23:46:10][keepkey_gui_lib::transport::protocol_adapter][INFO] ProtocolAdapter::handle: Decoded response type: EthereumAddress
[2025-06-11][23:46:10][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Parent returned message type: EthereumAddress
[2025-06-11][23:46:10][keepkey_gui_lib::transport][INFO] StandardHandler: Processing message type: EthereumAddress
[2025-06-11][23:46:10][keepkey_gui_lib::transport][INFO] MessageHandlerStack::handle: Handler returned None, done
[2025-06-11][23:46:10][keepkey_gui_lib::server::context][INFO] ✅ Got real Ethereum address from device 343737340F4736331F003B00: 0x141d9959cae3853b035000490c03991eb70fc4ac
[2025-06-11][23:46:10][keepkey_gui_lib::server::context][INFO] Device context set: device_id=343737340F4736331F003B00, eth_address=0x141d9959cae3853b035000490c03991eb70fc4ac, label=Some("KeepKey3")
[2025-06-11][23:46:10][keepkey_gui_lib][INFO] Device context set successfully for device 343737340F4736331F003B00 with label: Some("KeepKey3")
[2025-06-11][23:46:10][keepkey_gui_lib::server::routes::v2_endpoints][ERROR] get_portfolio_summary: No device found in cache
[2025-06-11][23:46:10][keepkey_gui_lib::server::routes::v2_endpoints][ERROR] get_balances: No device found in cache
[2025-06-11][23:46:10][keepkey_gui_lib::server::routes::v2_endpoints][ERROR] get_portfolio_summary: No device found in cache
[2025-06-11][23:46:10][keepkey_gui_lib::server::routes::v2_endpoints][ERROR] get_balances: No device found in cache
[2025-06-11][23:46:10][keepkey_gui_lib::server::routes::v2_endpoints][ERROR] get_portfolio_summary: No device found in cache
[2025-06-11][23:46:10][keepkey_gui_lib::server::routes::v2_endpoints][ERROR] get_portfolio_summary: No device found in cache
[2025-06-11][23:46:10][keepkey_gui_lib::server::routes::v2_endpoints][ERROR] get_balances: No device found in cache
[2025-06-11][23:46:10][keepkey_gui_lib::server::routes::v2_endpoints][ERROR] get_balances: No device found in cache
