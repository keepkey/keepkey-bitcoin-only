use anyhow::Result;
use tracing::{debug, error, info, warn};
use std::sync::Arc;
use tokio::time::timeout;

use crate::server::{DEVICE_OPERATION_TIMEOUT, routes, ServerState};
use crate::messages::{self, Message as KkMessage, ApplySettings, ChangePin, WipeDevice, RecoveryDevice, ResetDevice, LoadDevice, FirmwareErase, FirmwareUpload, Failure as ProtosFailure, MessageType as ProtosMessageType, PolicyType as ProtosPolicyType, ApplyPolicies as ProtosApplyPolicies};
use crate::transport::{ProtocolAdapter, UsbTransport}; // UsbTransport for type, ProtocolAdapter for .call()

// System management implementations
pub(crate) async fn system_apply_settings_impl(
    server_state: Arc<ServerState>,
    request: routes::ApplySettingsRequest,
) -> Result<()> {
    info!("Applying settings: label={:?}, language={:?}", request.label, request.language);

    let result = timeout(DEVICE_OPERATION_TIMEOUT, async {
        let mut transport_guard = server_state.active_transport.lock().await;
        if let Some(transport) = transport_guard.as_mut() {
            let apply_settings_msg = ApplySettings {
                u2f_counter: Some(0), // Default value, was missing
                language: request.language,
                label: request.label,
                use_passphrase: request.use_passphrase,
                auto_lock_delay_ms: request.auto_lock_delay_ms,
                // deprecated_homescreen: None, // Deprecated, not used
            };

            let response = transport.with_standard_handler().handle(apply_settings_msg.into()).map_err(|e| {
                error!("Error sending ApplySettings: {:?}", e);
                anyhow::anyhow!("Failed to send ApplySettings: {}", e)
            })?;

            match response {
                KkMessage::Success(success_msg) => {
                    info!("Successfully applied settings: {:?}", success_msg.message);
                    Ok(())
                }
                KkMessage::Failure(failure_msg) => {
                    error!("Failed to apply settings: {:?}", failure_msg.message);
                    Err(anyhow::anyhow!("Device returned failure: {:?}", failure_msg.message))
                }
                unexpected_msg => {
                    error!("Unexpected response to ApplySettings: {:?}", unexpected_msg);
                    Err(anyhow::anyhow!("Unexpected response type from device: {:?}", unexpected_msg.message_type()))
                }
            }
        } else {
            error!("Device transport not available for ApplySettings.");
            Err(anyhow::anyhow!("Device not connected or transport not initialized"))
        }
    }).await;

    match result {
        Ok(Ok(_)) => Ok(()),
        Ok(Err(e)) => Err(e),
        Err(_) => {
            error!("Apply settings timed out.");
            Err(anyhow::anyhow!("Device operation timed out"))
        }
    }
}

pub(crate) async fn system_apply_policy_impl(
    server_state: Arc<ServerState>,
    request: routes::ApplyPolicyRequest,
) -> Result<()> {
    info!("Applying policy: name={}, enabled={}", request.policy_name, request.enabled);

    let result = timeout(DEVICE_OPERATION_TIMEOUT, async {
        let mut transport_guard = server_state.active_transport.lock().await;
        if let Some(transport) = transport_guard.as_mut() {
            let policy_type_messages: Vec<ProtosPolicyType> = vec![
                ProtosPolicyType {
                    policy_name: Some(request.policy_name),
                    enabled: Some(request.enabled),
                }
            ];

            let apply_policies_msg = ProtosApplyPolicies {
                policy: policy_type_messages,
            };

            let kk_apply_policies_msg = KkMessage::ApplyPolicies(apply_policies_msg);

            match transport.handle(kk_apply_policies_msg) {
                Ok(KkMessage::Success(s)) => {
                    info!("Policies applied successfully: {:?}", s.message);
                    Ok(s) // Return the ProtosSuccess message
                }
                Ok(KkMessage::Failure(f)) => {
                    let error_message = f.message.unwrap_or_else(|| "Unknown device failure".to_string());
                    error!("ApplyPolicies failed on device: {}", error_message);
                    Err(anyhow::anyhow!("Device error: {}", error_message))
                }
                Ok(other_msg) => {
                    let msg_type = other_msg.message_type(); // Get MessageType before moving other_msg into error
                    error!("ApplyPolicies received unexpected response: {:?}", msg_type);
                    Err(anyhow::anyhow!("Unexpected response from device: {:?}", msg_type))
                }
                Err(e) => {
                    error!("Transport error during ApplyPolicies: {}", e);
                    Err(e) // Assumes Error is anyhow::Error
                }
            }
        } else {
            error!("Device transport not available for ApplyPolicy.");
            Err(anyhow::anyhow!("Device not connected or transport not initialized"))
        }
    }).await;

    match result {
        Ok(Ok(_)) => Ok(()),
        Ok(Err(e)) => Err(e),
        Err(_) => {
            error!("Apply policy timed out.");
            Err(anyhow::anyhow!("Device operation timed out"))
        }
    }
}

pub(crate) async fn system_change_pin_impl(
    server_state: Arc<ServerState>,
    request: routes::ChangePinRequest,
) -> Result<()> {
    info!("Changing PIN: remove={:?}", request.remove);

    let result = timeout(DEVICE_OPERATION_TIMEOUT, async {
        let mut transport_guard = server_state.active_transport.lock().await;
        if let Some(transport) = transport_guard.as_mut() {
            let change_pin_msg = ChangePin {
                remove: request.remove,
            };

            // ChangePin is interactive, the standard handler will manage PIN/Button prompts
            let response = transport.with_standard_handler().handle(change_pin_msg.into()).map_err(|e| {
                error!("Error sending ChangePin: {:?}", e);
                anyhow::anyhow!("Failed to send ChangePin: {}", e)
            })?;

            match response {
                KkMessage::Success(success_msg) => {
                    info!("Successfully changed PIN: {:?}", success_msg.message);
                    Ok(())
                }
                KkMessage::Failure(failure_msg) => {
                    error!("Failed to change PIN: {:?}", failure_msg.message);
                    Err(anyhow::anyhow!("Device returned failure: {:?}", failure_msg.message))
                }
                // Intermediate messages like PinMatrixRequest or ButtonRequest should be handled by with_standard_handler.
                // If they are returned here, it's unexpected.
                unexpected_msg => {
                    error!("Unexpected response to ChangePin: {:?}", unexpected_msg);
                    Err(anyhow::anyhow!("Unexpected response type from device: {:?}", unexpected_msg.message_type()))
                }
            }
        } else {
            error!("Device transport not available for ChangePin.");
            Err(anyhow::anyhow!("Device not connected or transport not initialized"))
        }
    }).await;

    match result {
        Ok(Ok(_)) => Ok(()),
        Ok(Err(e)) => Err(e),
        Err(_) => {
            error!("Change PIN timed out.");
            Err(anyhow::anyhow!("Device operation timed out"))
        }
    }
}

pub(crate) async fn system_wipe_device_impl(server_state: Arc<ServerState>) -> Result<()> {
    info!("Wiping device");

    let result = timeout(DEVICE_OPERATION_TIMEOUT, async {
        let mut transport_guard = server_state.active_transport.lock().await;
        if let Some(transport) = transport_guard.as_mut() {
            let wipe_device_msg = WipeDevice {};

            // WipeDevice is interactive and will require button confirmation
            let response = transport.with_standard_handler().handle(wipe_device_msg.into()).map_err(|e| {
                error!("Error sending WipeDevice: {:?}", e);
                anyhow::anyhow!("Failed to send WipeDevice: {}", e)
            })?;

            match response {
                KkMessage::Success(success_msg) => {
                    info!("Successfully wiped device: {:?}", success_msg.message);
                    Ok(())
                }
                KkMessage::Failure(failure_msg) => {
                    error!("Failed to wipe device: {:?}", failure_msg.message);
                    Err(anyhow::anyhow!("Device returned failure: {:?}", failure_msg.message))
                }
                unexpected_msg => {
                    error!("Unexpected response to WipeDevice: {:?}", unexpected_msg);
                    Err(anyhow::anyhow!("Unexpected response type from device: {:?}", unexpected_msg.message_type()))
                }
            }
        } else {
            error!("Device transport not available for WipeDevice.");
            Err(anyhow::anyhow!("Device not connected or transport not initialized"))
        }
    }).await;

    match result {
        Ok(Ok(_)) => Ok(()),
        Ok(Err(e)) => Err(e),
        Err(_) => {
            error!("Wipe device timed out.");
            Err(anyhow::anyhow!("Device operation timed out"))
        }
    }
}

pub(crate) async fn system_recovery_device_impl(
    server_state: Arc<ServerState>,
    request: routes::RecoveryDeviceRequest,
) -> Result<()> {
    info!("Recovering device: word_count={}", request.word_count);

    let result = timeout(DEVICE_OPERATION_TIMEOUT, async {
        let mut transport_guard = server_state.active_transport.lock().await;
        if let Some(transport) = transport_guard.as_mut() {
            let recovery_device_msg = RecoveryDevice {
                auto_lock_delay_ms: Some(0), // Default value
                u2f_counter: Some(0),      // Default value
                use_character_cipher: Some(false), // Default value
                word_count: Some(request.word_count),
                passphrase_protection: request.passphrase_protection,
                pin_protection: request.pin_protection,
                language: request.language,
                label: request.label,
                enforce_wordlist: request.enforce_wordlist,
                // use_character_cipher: None, // Not typically set by client
                dry_run: request.dry_run,
            };

            // RecoveryDevice is interactive
            let response = transport.with_standard_handler().handle(recovery_device_msg.into()).map_err(|e| {
                error!("Error sending RecoveryDevice: {:?}", e);
                anyhow::anyhow!("Failed to send RecoveryDevice: {}", e)
            })?;

            match response {
                KkMessage::Success(success_msg) => {
                    info!("Successfully initiated device recovery: {:?}", success_msg.message);
                    Ok(())
                }
                KkMessage::Failure(failure_msg) => {
                    error!("Failed to initiate device recovery: {:?}", failure_msg.message);
                    Err(anyhow::anyhow!("Device returned failure: {:?}", failure_msg.message))
                }
                unexpected_msg => {
                    error!("Unexpected response to RecoveryDevice: {:?}", unexpected_msg);
                    Err(anyhow::anyhow!("Unexpected response type from device: {:?}", unexpected_msg.message_type()))
                }
            }
        } else {
            error!("Device transport not available for RecoveryDevice.");
            Err(anyhow::anyhow!("Device not connected or transport not initialized"))
        }
    }).await;

    match result {
        Ok(Ok(_)) => Ok(()),
        Ok(Err(e)) => Err(e),
        Err(_) => {
            error!("Device recovery timed out.");
            Err(anyhow::anyhow!("Device operation timed out"))
        }
    }
}

pub(crate) async fn system_reset_device_impl(
    server_state: Arc<ServerState>,
    request: routes::ResetDeviceRequest,
) -> Result<()> {
    info!("Resetting device: label={:?}, strength={:?}", request.label, request.strength);

    let result = timeout(DEVICE_OPERATION_TIMEOUT, async {
        let mut transport_guard = server_state.active_transport.lock().await;
        if let Some(transport) = transport_guard.as_mut() {
            let reset_device_msg = ResetDevice {
                u2f_counter: Some(0),      // Default value
                display_random: Some(request.display_random),
                strength: request.strength,
                passphrase_protection: request.passphrase_protection,
                pin_protection: request.pin_protection,
                language: request.language,
                label: request.label,
                // u2f_counter: None, // Not typically set by client
                // skip_backup: None, // Deprecated, use no_backup
                no_backup: request.no_backup,
                auto_lock_delay_ms: request.auto_lock_delay_ms,
            };

            // ResetDevice is interactive
            let response = transport.with_standard_handler().handle(reset_device_msg.into()).map_err(|e| {
                error!("Error sending ResetDevice: {:?}", e);
                anyhow::anyhow!("Failed to send ResetDevice: {}", e)
            })?;

            match response {
                KkMessage::Success(success_msg) => {
                    info!("Successfully initiated device reset: {:?}", success_msg.message);
                    Ok(())
                }
                KkMessage::Failure(failure_msg) => {
                    error!("Failed to initiate device reset: {:?}", failure_msg.message);
                    Err(anyhow::anyhow!("Device returned failure: {:?}", failure_msg.message))
                }
                unexpected_msg => {
                    error!("Unexpected response to ResetDevice: {:?}", unexpected_msg);
                    Err(anyhow::anyhow!("Unexpected response type from device: {:?}", unexpected_msg.message_type()))
                }
            }
        } else {
            error!("Device transport not available for ResetDevice.");
            Err(anyhow::anyhow!("Device not connected or transport not initialized"))
        }
    }).await;

    match result {
        Ok(Ok(_)) => Ok(()),
        Ok(Err(e)) => Err(e),
        Err(_) => {
            error!("Device reset timed out.");
            Err(anyhow::anyhow!("Device operation timed out"))
        }
    }
}

pub(crate) async fn system_load_device_impl(
    server_state: Arc<ServerState>,
    request: routes::LoadDeviceRequest,
) -> Result<()> {
    info!("Loading device with new seed: label={:?}", request.label);

    let result = timeout(DEVICE_OPERATION_TIMEOUT, async {
        let mut transport_guard = server_state.active_transport.lock().await;
        if let Some(transport) = transport_guard.as_mut() {
            let load_device_msg = LoadDevice {
                passphrase_protection: Some(request.passphrase.is_some()), // Was missing; true if passphrase provided in request
                mnemonic: Some(request.mnemonic), // request.mnemonic is String
                pin: request.pin,                 // request.pin is Option<String>
                // passphrase_protection: request.passphrase_protection, // Field does not exist on LoadDevice protobuf message as per compiler error
                label: request.label,             // request.label is Option<String>, matches proto field type
                language: request.language,       // request.language is Option<String>, matches proto field type
                skip_checksum: Some(false),       // Default value; request does not have skip_checksum. LoadDevice protobuf has this field.
                u2f_counter: Some(0),             // Default value
                node: None,                       // Default value
            };

            // LoadDevice is interactive (PIN, potentially passphrase if enabled on device but not provided in request)
            let response = transport.with_standard_handler().handle(load_device_msg.into()).map_err(|e| {
                error!("Error sending LoadDevice: {:?}", e);
                anyhow::anyhow!("Failed to send LoadDevice: {}", e)
            })?;

            // If passphrase was provided in the request, and device expects it, send it now.
            // The standard handler might have already prompted if it was a ButtonRequest for passphrase.
            // This explicit send handles cases where the device directly asks for PassphraseAck after LoadDevice.
            if let Some(passphrase_str) = request.passphrase {
                if let KkMessage::PassphraseRequest(_) = response { // Or if some other logic determines passphrase is now needed
                    info!("Passphrase provided, sending PassphraseAck");
                    let passphrase_ack_msg = messages::PassphraseAck { passphrase: passphrase_str };
                    let _ = transport.with_standard_handler().handle(passphrase_ack_msg.into()).map_err(|e| {
                        error!("Error sending PassphraseAck: {:?}", e);
                        anyhow::anyhow!("Failed to send PassphraseAck: {}", e)
                    })?;
                    // Expect Success/Failure after PassphraseAck
                    // This part might need further refinement based on exact device flow for LoadDevice + Passphrase
                }
            }

            match response {
                KkMessage::Success(success_msg) => {
                    info!("Successfully loaded device: {:?}", success_msg.message);
                    Ok(())
                }
                KkMessage::Failure(failure_msg) => {
                    error!("Failed to load device: {:?}", failure_msg.message);
                    Err(anyhow::anyhow!("Device returned failure: {:?}", failure_msg.message))
                }
                // PassphraseRequest might be handled above or by standard_handler
                KkMessage::PassphraseRequest(_) => {
                    // This case implies passphrase was not provided or standard_handler didn't fully resolve it.
                    error!("Device requested passphrase, but it was not handled.");
                    Err(anyhow::anyhow!("Passphrase required by device but not fully handled"))
                }
                unexpected_msg => {
                    error!("Unexpected response to LoadDevice: {:?}", unexpected_msg);
                    Err(anyhow::anyhow!("Unexpected response type from device: {:?}", unexpected_msg.message_type()))
                }
            }
        } else {
            error!("Device transport not available for LoadDevice.");
            Err(anyhow::anyhow!("Device not connected or transport not initialized"))
        }
    }).await;

    match result {
        Ok(Ok(_)) => Ok(()),
        Ok(Err(e)) => Err(e),
        Err(_) => {
            error!("Load device timed out.");
            Err(anyhow::anyhow!("Device operation timed out"))
        }
    }
}

pub(crate) async fn system_backup_device_impl(server_state: Arc<ServerState>) -> Result<()> {
    info!("Initiating device backup");

    let result = timeout(DEVICE_OPERATION_TIMEOUT, async {
        let mut transport_guard = server_state.active_transport.lock().await;
        if let Some(_transport) = transport_guard.as_mut() {
            // let backup_device_msg = BackupDevice {};
            // info!("backup_device_msg: {:?}", backup_device_msg);
            // TODO send backup_device_msg
            // let response = transport.with_standard_handler().handle(backup_device_msg.into()).map_err(|e| {
                // error!("Error sending BackupDevice: {:?}", e);
                // anyhow::anyhow!("Failed to send BackupDevice: {}", e)
            // })?;

            // match response {
            //     KkMessage::Success(success_msg) => {
            //         info!("Successfully initiated device backup: {:?}", success_msg.message);
            //         Ok(())
            //     }
            //     KkMessage::Failure(failure_msg) => {
            //         error!("Failed to initiate device backup: {:?}", failure_msg.message);
            //         Err(anyhow::anyhow!("Device returned failure: {:?}", failure_msg.message))
            //     }
            //     unexpected_msg => {
            //         error!("Unexpected response to BackupDevice: {:?}", unexpected_msg);
            //         Err(anyhow::anyhow!("Unexpected response type from device: {:?}", unexpected_msg.message_type()))
            //     }
            // }
            Err::<(), anyhow::Error>(anyhow::anyhow!("BackupDevice not implemented"))
        } else {
            error!("Device transport not available for BackupDevice.");
            Err(anyhow::anyhow!("Device not connected or transport not initialized"))
        }
    }).await;

    match result {
        Ok(Ok(_)) => Ok(()),
        Ok(Err(e)) => Err(e),
        Err(_) => {
            error!("Device backup timed out.");
            Err(anyhow::anyhow!("Device operation timed out"))
        }
    }
}

pub(crate) async fn system_firmware_erase_impl(server_state: Arc<ServerState>) -> Result<()> {
    info!("Initiating firmware erase");

    let result = timeout(DEVICE_OPERATION_TIMEOUT, async {
        let mut transport_guard = server_state.active_transport.lock().await;
        if let Some(transport) = transport_guard.as_mut() {
            let firmware_erase_msg = FirmwareErase {
                // length: None, // Field does not exist
            };

            // FirmwareErase is interactive and requires confirmation
            let response = transport.with_standard_handler().handle(firmware_erase_msg.into()).map_err(|e| {
                error!("Error sending FirmwareErase: {:?}", e);
                anyhow::anyhow!("Failed to send FirmwareErase: {}", e)
            })?;

            match response {
                KkMessage::Success(success_msg) => {
                    info!("Successfully initiated firmware erase: {:?}", success_msg.message);
                    Ok(())
                }
                KkMessage::Failure(failure_msg) => {
                    error!("Failed to initiate firmware erase: {:?}", failure_msg.message);
                    Err(anyhow::anyhow!("Device returned failure: {:?}", failure_msg.message))
                }
                unexpected_msg => {
                    error!("Unexpected response to FirmwareErase: {:?}", unexpected_msg);
                    Err(anyhow::anyhow!("Unexpected response type from device: {:?}", unexpected_msg.message_type()))
                }
            }
        } else {
            error!("Device transport not available for FirmwareErase.");
            Err(anyhow::anyhow!("Device not connected or transport not initialized"))
        }
    }).await;

    match result {
        Ok(Ok(_)) => Ok(()),
        Ok(Err(e)) => Err(e),
        Err(_) => {
            error!("Firmware erase timed out.");
            Err(anyhow::anyhow!("Device operation timed out"))
        }
    }
}

const FIRMWARE_CHUNK_SIZE: usize = 1024; // Define a reasonable chunk size

pub(crate) async fn system_firmware_upload_impl(
    server_state: Arc<ServerState>,
    request: routes::FirmwareUploadRequest,
) -> Result<()> {
    info!("Initiating firmware upload: {} bytes", request.firmware.len());

    let result = timeout(DEVICE_OPERATION_TIMEOUT * 5, async { // Firmware upload can take longer
        let mut transport_guard = server_state.active_transport.lock().await;
        if let Some(transport) = transport_guard.as_mut() {
            // First, optionally send FirmwareErase if not done separately.
            // For this example, assuming erase is a separate preceding step or handled by bootloader.

            // Send FirmwareUpload messages in chunks
            for (i, chunk) in request.firmware.chunks(FIRMWARE_CHUNK_SIZE).enumerate() {
                info!("Uploading firmware chunk {}/{}", i + 1, (request.firmware.len() + FIRMWARE_CHUNK_SIZE - 1) / FIRMWARE_CHUNK_SIZE);
                let firmware_upload_msg = FirmwareUpload {
                    payload: chunk.to_vec().into(),
                    payload_hash: Vec::new(), // Field expects Vec<u8>, not Option<Vec<u8>>
                };

                // FirmwareUpload itself might not be interactive until the very end or if an error occurs.
                // The device usually just accepts data chunks.
                let response = transport.with_standard_handler().handle(firmware_upload_msg.into()).map_err(|e| {
                    error!("Error sending FirmwareUpload chunk {}: {:?}", i + 1, e);
                    anyhow::anyhow!("Failed to send FirmwareUpload chunk {}: {}", i + 1, e)
                })?;

                // Check response after each chunk. Some devices might send Success per chunk, others only at the end.
                // For robust implementation, one might need to handle intermediate responses or specific device behavior.
                match response {
                    KkMessage::Success(success_msg) => {
                        info!("Successfully sent firmware chunk {}: {:?}", i + 1, success_msg.message);
                        // Continue to next chunk
                    }
                    KkMessage::Failure(failure_msg) => {
                        error!("Firmware upload failed at chunk {}: {:?}", i + 1, failure_msg.message);
                        return Err(anyhow::anyhow!("Device returned failure during firmware upload: {:?}", failure_msg.message));
                    }
                    // Some devices might send a specific message like FirmwareRequest after a chunk, signaling readiness for the next.
                    // This example assumes Success means proceed or the final Success after all chunks.
                    unexpected_msg => {
                        error!("Unexpected response during firmware upload at chunk {}: {:?}", i + 1, unexpected_msg);
                        return Err(anyhow::anyhow!("Unexpected response type from device during firmware upload: {:?}", unexpected_msg.message_type()));
                    }
                }
            }

            // After all chunks are sent, the device might perform verification and then reboot.
            // The final success might be implicit if no errors, or a final Success message.
            info!("All firmware chunks sent successfully.");
            Ok(())
        } else {
            error!("Device transport not available for FirmwareUpload.");
            Err(anyhow::anyhow!("Device not connected or transport not initialized"))
        }
    }).await;

    match result {
        Ok(Ok(_)) => Ok(()),
        Ok(Err(e)) => Err(e),
        Err(_) => {
            error!("Firmware upload timed out.");
            Err(anyhow::anyhow!("Device operation timed out"))
        }
    }
}

// Debug implementations
pub(crate) async fn debug_link_state_impl() -> Result<routes::DebugLinkState> {
    error!("Debug link state not implemented");
    Err(anyhow::anyhow!("Not implemented"))
}

pub(crate) async fn debug_fill_config_impl(_request: routes::DebugFillConfig) -> Result<()> {
    error!("Debug fill config not implemented");
    Err(anyhow::anyhow!("Not implemented"))
}

// Manufacturing implementations
pub(crate) async fn manufacturing_get_hash_impl() -> Result<String> {
    error!("Manufacturing get hash not implemented");
    Err(anyhow::anyhow!("Not implemented"))
}

pub(crate) async fn manufacturing_model_prefix_impl() -> Result<String> {
    error!("Manufacturing model prefix not implemented");
    Err(anyhow::anyhow!("Not implemented"))
}

// Raw message implementation
pub(crate) async fn raw_message_impl(_body: axum::body::Bytes) -> Result<axum::body::Bytes> {
    error!("Raw message not implemented");
    Err(anyhow::anyhow!("Not implemented"))
} 