import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';

export interface UpdateInfo {
  version: string;
  releaseNotes?: string;
}

export interface VersionInfo {
  version: string;
}

export interface UpdateError {
  message: string;
  code?: number;
  domain?: string;
}

export type CheckingPayload = Record<string, never>;
export type UpdateAvailablePayload = UpdateInfo;
export type UpdateNotAvailablePayload = Record<string, never>;
export type DownloadingPayload = VersionInfo;
export type DownloadedPayload = VersionInfo;
export type InstallingPayload = VersionInfo;
export type ErrorPayload = UpdateError;

export async function checkForUpdates(): Promise<void> {
  return invoke('plugin:sparkle-updater|check_for_updates');
}

export async function checkForUpdatesInBackground(): Promise<void> {
  return invoke('plugin:sparkle-updater|check_for_updates_in_background');
}

export async function canCheckForUpdates(): Promise<boolean> {
  return invoke('plugin:sparkle-updater|can_check_for_updates');
}

export async function currentVersion(): Promise<string> {
  return invoke('plugin:sparkle-updater|current_version');
}

export async function feedUrl(): Promise<string | null> {
  return invoke('plugin:sparkle-updater|feed_url');
}

export async function setFeedUrl(url: string): Promise<void> {
  return invoke('plugin:sparkle-updater|set_feed_url', { url });
}

export async function automaticallyChecksForUpdates(): Promise<boolean> {
  return invoke('plugin:sparkle-updater|automatically_checks_for_updates');
}

export async function setAutomaticallyChecksForUpdates(enabled: boolean): Promise<void> {
  return invoke('plugin:sparkle-updater|set_automatically_checks_for_updates', { enabled });
}

export async function automaticallyDownloadsUpdates(): Promise<boolean> {
  return invoke('plugin:sparkle-updater|automatically_downloads_updates');
}

export async function setAutomaticallyDownloadsUpdates(enabled: boolean): Promise<void> {
  return invoke('plugin:sparkle-updater|set_automatically_downloads_updates', { enabled });
}

/** Returns Unix timestamp in milliseconds. */
export async function lastUpdateCheckDate(): Promise<number | null> {
  return invoke('plugin:sparkle-updater|last_update_check_date');
}

export async function resetUpdateCycle(): Promise<void> {
  return invoke('plugin:sparkle-updater|reset_update_cycle');
}

export const Events = {
  CHECKING: 'sparkle://checking',
  UPDATE_AVAILABLE: 'sparkle://update-available',
  UPDATE_NOT_AVAILABLE: 'sparkle://update-not-available',
  DOWNLOADING: 'sparkle://downloading',
  DOWNLOADED: 'sparkle://downloaded',
  INSTALLING: 'sparkle://installing',
  ERROR: 'sparkle://error',
} as const;

export function onChecking(handler: (payload: CheckingPayload) => void): Promise<UnlistenFn> {
  return listen<CheckingPayload>(Events.CHECKING, (event) => handler(event.payload));
}

export function onUpdateAvailable(handler: (payload: UpdateAvailablePayload) => void): Promise<UnlistenFn> {
  return listen<UpdateAvailablePayload>(Events.UPDATE_AVAILABLE, (event) => handler(event.payload));
}

export function onUpdateNotAvailable(handler: (payload: UpdateNotAvailablePayload) => void): Promise<UnlistenFn> {
  return listen<UpdateNotAvailablePayload>(Events.UPDATE_NOT_AVAILABLE, (event) => handler(event.payload));
}

export function onDownloading(handler: (payload: DownloadingPayload) => void): Promise<UnlistenFn> {
  return listen<DownloadingPayload>(Events.DOWNLOADING, (event) => handler(event.payload));
}

export function onDownloaded(handler: (payload: DownloadedPayload) => void): Promise<UnlistenFn> {
  return listen<DownloadedPayload>(Events.DOWNLOADED, (event) => handler(event.payload));
}

export function onInstalling(handler: (payload: InstallingPayload) => void): Promise<UnlistenFn> {
  return listen<InstallingPayload>(Events.INSTALLING, (event) => handler(event.payload));
}

export function onError(handler: (payload: ErrorPayload) => void): Promise<UnlistenFn> {
  return listen<ErrorPayload>(Events.ERROR, (event) => handler(event.payload));
}
