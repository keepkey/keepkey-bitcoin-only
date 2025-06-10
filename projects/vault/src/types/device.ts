// Device update workflow types

export type VersionComparison = 'current' | 'patchBehind' | 'minorBehind' | 'majorBehind'

export interface BootloaderCheck {
  currentVersion: string
  latestVersion: string
  targetVersion: string
  isOutdated: boolean
  isCritical: boolean
  comparison: VersionComparison
}

export interface FirmwareCheck {
  currentVersion: string
  latestVersion: string
  targetVersion: string
  isOutdated: boolean
  releaseNotes: string[]
  securityUpdate: boolean
  comparison: VersionComparison
}

export interface InitializationCheck {
  initialized: boolean
  hasBackup: boolean
  imported: boolean
  needsSetup: boolean
}

export interface DeviceStatus {
  deviceId: string
  bootloaderCheck?: BootloaderCheck
  firmwareCheck?: FirmwareCheck
  initializationCheck: InitializationCheck
  needsBootloaderUpdate: boolean
  needsFirmwareUpdate: boolean
  needsInitialization: boolean
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