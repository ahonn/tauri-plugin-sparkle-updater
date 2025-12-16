import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';

// ============ Types ============

/** Update information returned when an update is available */
export interface UpdateInfo {
  version: string;
  releaseNotes?: string;
}

/** Version information for downloading/downloaded/installing events */
export interface VersionInfo {
  version: string;
}

/** Error information */
export interface UpdateError {
  message: string;
  code?: number;
  domain?: string;
}

/** Event payload types */
export type CheckingPayload = Record<string, never>;
export type UpdateAvailablePayload = UpdateInfo;
export type UpdateNotAvailablePayload = Record<string, never>;
export type DownloadingPayload = VersionInfo;
export type DownloadedPayload = VersionInfo;
export type InstallingPayload = VersionInfo;
export type ErrorPayload = UpdateError;

// ============ Commands ============

/** Check for updates with UI dialog */
export async function checkForUpdates(): Promise<void> {
  return invoke('plugin:sparkle-updater|check_for_updates');
}

/** Check for updates silently in background */
export async function checkForUpdatesInBackground(): Promise<void> {
  return invoke('plugin:sparkle-updater|check_for_updates_in_background');
}

/** Returns whether updates can be checked */
export async function canCheckForUpdates(): Promise<boolean> {
  return invoke('plugin:sparkle-updater|can_check_for_updates');
}

/** Returns the current application version */
export async function currentVersion(): Promise<string> {
  return invoke('plugin:sparkle-updater|current_version');
}

/** Returns the current feed URL */
export async function feedUrl(): Promise<string | null> {
  return invoke('plugin:sparkle-updater|feed_url');
}

/** Sets the feed URL */
export async function setFeedUrl(url: string): Promise<void> {
  return invoke('plugin:sparkle-updater|set_feed_url', { url });
}

/** Returns whether automatic update checks are enabled */
export async function automaticallyChecksForUpdates(): Promise<boolean> {
  return invoke('plugin:sparkle-updater|automatically_checks_for_updates');
}

/** Sets whether automatic update checks are enabled */
export async function setAutomaticallyChecksForUpdates(enabled: boolean): Promise<void> {
  return invoke('plugin:sparkle-updater|set_automatically_checks_for_updates', { enabled });
}

/** Returns whether updates are automatically downloaded */
export async function automaticallyDownloadsUpdates(): Promise<boolean> {
  return invoke('plugin:sparkle-updater|automatically_downloads_updates');
}

/** Sets whether updates are automatically downloaded */
export async function setAutomaticallyDownloadsUpdates(enabled: boolean): Promise<void> {
  return invoke('plugin:sparkle-updater|set_automatically_downloads_updates', { enabled });
}

/** Returns the last update check date as Unix timestamp (ms) */
export async function lastUpdateCheckDate(): Promise<number | null> {
  return invoke('plugin:sparkle-updater|last_update_check_date');
}

/** Resets the update cycle */
export async function resetUpdateCycle(): Promise<void> {
  return invoke('plugin:sparkle-updater|reset_update_cycle');
}

// ============ Events ============

/** Event names emitted by the updater */
export const Events = {
  CHECKING: 'sparkle://checking',
  UPDATE_AVAILABLE: 'sparkle://update-available',
  UPDATE_NOT_AVAILABLE: 'sparkle://update-not-available',
  DOWNLOADING: 'sparkle://downloading',
  DOWNLOADED: 'sparkle://downloaded',
  INSTALLING: 'sparkle://installing',
  ERROR: 'sparkle://error',
} as const;

/** Listen for the checking event (appcast loaded) */
export function onChecking(handler: (payload: CheckingPayload) => void): Promise<UnlistenFn> {
  return listen<CheckingPayload>(Events.CHECKING, (event) => handler(event.payload));
}

/** Listen for update available event */
export function onUpdateAvailable(handler: (payload: UpdateAvailablePayload) => void): Promise<UnlistenFn> {
  return listen<UpdateAvailablePayload>(Events.UPDATE_AVAILABLE, (event) => handler(event.payload));
}

/** Listen for update not available event */
export function onUpdateNotAvailable(handler: (payload: UpdateNotAvailablePayload) => void): Promise<UnlistenFn> {
  return listen<UpdateNotAvailablePayload>(Events.UPDATE_NOT_AVAILABLE, (event) => handler(event.payload));
}

/** Listen for downloading event */
export function onDownloading(handler: (payload: DownloadingPayload) => void): Promise<UnlistenFn> {
  return listen<DownloadingPayload>(Events.DOWNLOADING, (event) => handler(event.payload));
}

/** Listen for downloaded event */
export function onDownloaded(handler: (payload: DownloadedPayload) => void): Promise<UnlistenFn> {
  return listen<DownloadedPayload>(Events.DOWNLOADED, (event) => handler(event.payload));
}

/** Listen for installing event */
export function onInstalling(handler: (payload: InstallingPayload) => void): Promise<UnlistenFn> {
  return listen<InstallingPayload>(Events.INSTALLING, (event) => handler(event.payload));
}

/** Listen for error event */
export function onError(handler: (payload: ErrorPayload) => void): Promise<UnlistenFn> {
  return listen<ErrorPayload>(Events.ERROR, (event) => handler(event.payload));
}
