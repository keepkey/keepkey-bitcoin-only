// Device update workflow types

export type VersionComparison = 'current' | 'patchBehind' | 'minorBehind' | 'majorBehind'

export interface BootloaderCheck {
  current_version: string
  latest_version: string
  needs_update: boolean
}

export interface FirmwareCheck {
  current_version: string
  latest_version: string
  needs_update: boolean
}

export interface InitializationCheck {
  initialized: boolean
  has_backup: boolean
  imported: boolean
  needs_setup: boolean
}

export interface DeviceStatus {
  device_id: string
  connected: boolean
  features?: DeviceFeatures
  needs_bootloader_update: boolean
  needs_firmware_update: boolean
  needs_initialization: boolean
  bootloader_check?: BootloaderCheck
  firmware_check?: FirmwareCheck
  initialization_check?: InitializationCheck
}

export interface DeviceFeatures {
  label?: string
  vendor?: string
  model?: string
  firmwareVariant?: string
  deviceId?: string
  language?: string
  bootloaderMode: boolean
  version: string
  firmwareHash?: string
  bootloaderHash?: string
  bootloaderVersion?: string
  initialized: boolean
  imported?: boolean
  noBackup: boolean
  pinProtection: boolean
  pinCached: boolean
  passphraseProtection: boolean
  passphraseCached: boolean
  wipeCodeProtection: boolean
  autoLockDelayMs?: number
  policies: string[]
} 