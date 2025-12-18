import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';

export interface UpdateInfo {
  version: string;
  releaseNotes?: string;
  title?: string;
  releaseNotesUrl?: string;
  infoUrl?: string;
  minimumSystemVersion?: string;
  channel?: string;
  date?: number;
  isCritical: boolean;
  isMajorUpgrade: boolean;
  isInformationOnly: boolean;
  maximumSystemVersion?: string;
  minimumOsVersionOk: boolean;
  maximumOsVersionOk: boolean;
  installationType: string;
  phasedRolloutInterval?: number;
  fullReleaseNotesUrl?: string;
  minimumAutoupdateVersion?: string;
  ignoreSkippedUpgradesBelowVersion?: string;
  dateString?: string;
  itemDescriptionFormat?: string;
}

export interface VersionInfo {
  version: string;
}

export interface UpdateError {
  message: string;
  code: number;
  domain: string;
}

export interface UpdateCycleInfo {
  updateCheck: 'userInitiated' | 'background' | 'information';
  error?: UpdateError;
}

export interface DownloadFailedInfo {
  version: string;
  error: UpdateError;
}

export interface UserChoiceInfo {
  choice: 'skip' | 'install' | 'dismiss';
  version: string;
  stage: 'notDownloaded' | 'downloaded' | 'installing';
}

export interface ScheduleInfo {
  delay: number;
}

export type DidFinishLoadingAppcastPayload = Record<string, never>;
export type DidFindValidUpdatePayload = UpdateInfo;
export type DidNotFindUpdatePayload = Record<string, never>;
export type WillDownloadUpdatePayload = VersionInfo;
export type DidDownloadUpdatePayload = VersionInfo;
export type WillInstallUpdatePayload = VersionInfo;
export type DidAbortWithErrorPayload = UpdateError;
export type DidFinishUpdateCyclePayload = UpdateCycleInfo;
export type FailedToDownloadUpdatePayload = DownloadFailedInfo;
export type UserDidCancelDownloadPayload = Record<string, never>;
export type WillExtractUpdatePayload = VersionInfo;
export type DidExtractUpdatePayload = VersionInfo;
export type WillRelaunchApplicationPayload = Record<string, never>;
export type UserDidMakeChoicePayload = UserChoiceInfo;
export type WillScheduleUpdateCheckPayload = ScheduleInfo;
export type WillNotScheduleUpdateCheckPayload = Record<string, never>;
export type WillInstallUpdateOnQuitPayload = VersionInfo;

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

/** Returns the update check interval in seconds. */
export async function updateCheckInterval(): Promise<number> {
  return invoke('plugin:sparkle-updater|update_check_interval');
}

/** Sets the update check interval in seconds. */
export async function setUpdateCheckInterval(interval: number): Promise<void> {
  return invoke('plugin:sparkle-updater|set_update_check_interval', { interval });
}

/**
 * Begins a "probing" check for updates which will not actually offer to update.
 * The delegate methods `onDidFindValidUpdate` and `onDidNotFindUpdate` will be called.
 * Useful for showing update availability badges in your UI.
 */
export async function checkForUpdateInformation(): Promise<void> {
  return invoke('plugin:sparkle-updater|check_for_update_information');
}

/**
 * Returns whether an update session is in progress.
 * An update session is in progress when the appcast is being downloaded,
 * an update is being downloaded, an update is being shown, update permission
 * is being requested, or the installer is being started.
 */
export async function sessionInProgress(): Promise<boolean> {
  return invoke('plugin:sparkle-updater|session_in_progress');
}

/**
 * Returns the custom HTTP headers used for update requests.
 */
export async function httpHeaders(): Promise<Record<string, string> | null> {
  return invoke('plugin:sparkle-updater|http_headers');
}

/**
 * Sets custom HTTP headers for update requests.
 * Useful for authentication (API keys, Bearer tokens, etc.)
 *
 * @param headers - Key-value pairs of HTTP headers, or null to clear
 */
export async function setHttpHeaders(headers: Record<string, string> | null): Promise<void> {
  return invoke('plugin:sparkle-updater|set_http_headers', { headers });
}

/**
 * Returns the User-Agent string used for update requests.
 * Default format: "AppName/1.0 Sparkle/2.x"
 */
export async function userAgentString(): Promise<string> {
  return invoke('plugin:sparkle-updater|user_agent_string');
}

/**
 * Sets a custom User-Agent string for update requests.
 * Useful for analytics or debugging purposes.
 *
 * @param userAgent - The User-Agent string to use
 */
export async function setUserAgentString(userAgent: string): Promise<void> {
  return invoke('plugin:sparkle-updater|set_user_agent_string', { userAgent });
}

/**
 * Returns whether the updater sends system profile information.
 */
export async function sendsSystemProfile(): Promise<boolean> {
  return invoke('plugin:sparkle-updater|sends_system_profile');
}

/**
 * Sets whether the updater should send system profile information.
 * System profile includes basic hardware and OS information for analytics.
 *
 * @param sends - Whether to send system profile
 */
export async function setSendsSystemProfile(sends: boolean): Promise<void> {
  return invoke('plugin:sparkle-updater|set_sends_system_profile', { sends });
}

/**
 * Clears the feed URL stored in user defaults.
 * Returns the URL that was cleared, or null if no URL was stored.
 */
export async function clearFeedUrlFromUserDefaults(): Promise<string | null> {
  return invoke('plugin:sparkle-updater|clear_feed_url_from_user_defaults');
}

/**
 * Resets the update cycle after a short delay.
 * Useful when settings change and you want to allow the user to undo
 * before the next check happens.
 */
export async function resetUpdateCycleAfterShortDelay(): Promise<void> {
  return invoke('plugin:sparkle-updater|reset_update_cycle_after_short_delay');
}

export async function allowedChannels(): Promise<string[] | null> {
  return invoke('plugin:sparkle-updater|allowed_channels');
}

export async function setAllowedChannels(channels: string[] | null): Promise<void> {
  return invoke('plugin:sparkle-updater|set_allowed_channels', { channels });
}

export async function feedUrlOverride(): Promise<string | null> {
  return invoke('plugin:sparkle-updater|feed_url_override');
}

export async function setFeedUrlOverride(url: string | null): Promise<void> {
  return invoke('plugin:sparkle-updater|set_feed_url_override', { url });
}

export async function feedParameters(): Promise<Record<string, string> | null> {
  return invoke('plugin:sparkle-updater|feed_parameters');
}

export async function setFeedParameters(params: Record<string, string> | null): Promise<void> {
  return invoke('plugin:sparkle-updater|set_feed_parameters', { params });
}

export async function shouldDownloadReleaseNotes(): Promise<boolean> {
  return invoke('plugin:sparkle-updater|should_download_release_notes');
}

export async function setShouldDownloadReleaseNotes(enabled: boolean): Promise<void> {
  return invoke('plugin:sparkle-updater|set_should_download_release_notes', { enabled });
}

export async function shouldRelaunchApplication(): Promise<boolean> {
  return invoke('plugin:sparkle-updater|should_relaunch_application');
}

export async function setShouldRelaunchApplication(enabled: boolean): Promise<void> {
  return invoke('plugin:sparkle-updater|set_should_relaunch_application', { enabled });
}

export async function mayCheckForUpdatesConfig(): Promise<boolean> {
  return invoke('plugin:sparkle-updater|may_check_for_updates_config');
}

export async function setMayCheckForUpdatesConfig(enabled: boolean): Promise<void> {
  return invoke('plugin:sparkle-updater|set_may_check_for_updates_config', { enabled });
}

export async function shouldProceedWithUpdate(): Promise<boolean> {
  return invoke('plugin:sparkle-updater|should_proceed_with_update');
}

export async function setShouldProceedWithUpdate(enabled: boolean): Promise<void> {
  return invoke('plugin:sparkle-updater|set_should_proceed_with_update', { enabled });
}

export async function decryptionPassword(): Promise<string | null> {
  return invoke('plugin:sparkle-updater|decryption_password');
}

export async function setDecryptionPassword(password: string | null): Promise<void> {
  return invoke('plugin:sparkle-updater|set_decryption_password', { password });
}

export async function lastFoundUpdate(): Promise<UpdateInfo | null> {
  return invoke('plugin:sparkle-updater|last_found_update');
}

export const Events = {
  DID_FINISH_LOADING_APPCAST: 'sparkle://did-finish-loading-appcast',
  DID_FIND_VALID_UPDATE: 'sparkle://did-find-valid-update',
  DID_NOT_FIND_UPDATE: 'sparkle://did-not-find-update',
  WILL_DOWNLOAD_UPDATE: 'sparkle://will-download-update',
  DID_DOWNLOAD_UPDATE: 'sparkle://did-download-update',
  WILL_INSTALL_UPDATE: 'sparkle://will-install-update',
  DID_ABORT_WITH_ERROR: 'sparkle://did-abort-with-error',
  DID_FINISH_UPDATE_CYCLE: 'sparkle://did-finish-update-cycle',
  FAILED_TO_DOWNLOAD_UPDATE: 'sparkle://failed-to-download-update',
  USER_DID_CANCEL_DOWNLOAD: 'sparkle://user-did-cancel-download',
  WILL_EXTRACT_UPDATE: 'sparkle://will-extract-update',
  DID_EXTRACT_UPDATE: 'sparkle://did-extract-update',
  WILL_RELAUNCH_APPLICATION: 'sparkle://will-relaunch-application',
  USER_DID_MAKE_CHOICE: 'sparkle://user-did-make-choice',
  WILL_SCHEDULE_UPDATE_CHECK: 'sparkle://will-schedule-update-check',
  WILL_NOT_SCHEDULE_UPDATE_CHECK: 'sparkle://will-not-schedule-update-check',
  WILL_INSTALL_UPDATE_ON_QUIT: 'sparkle://will-install-update-on-quit',
} as const;

function createListener<T>(event: string) {
  return (handler: (payload: T) => void): Promise<UnlistenFn> =>
    listen<T>(event, (e) => handler(e.payload));
}

export const onDidFinishLoadingAppcast = createListener<DidFinishLoadingAppcastPayload>(Events.DID_FINISH_LOADING_APPCAST);
export const onDidFindValidUpdate = createListener<DidFindValidUpdatePayload>(Events.DID_FIND_VALID_UPDATE);
export const onDidNotFindUpdate = createListener<DidNotFindUpdatePayload>(Events.DID_NOT_FIND_UPDATE);
export const onWillDownloadUpdate = createListener<WillDownloadUpdatePayload>(Events.WILL_DOWNLOAD_UPDATE);
export const onDidDownloadUpdate = createListener<DidDownloadUpdatePayload>(Events.DID_DOWNLOAD_UPDATE);
export const onWillInstallUpdate = createListener<WillInstallUpdatePayload>(Events.WILL_INSTALL_UPDATE);
export const onDidAbortWithError = createListener<DidAbortWithErrorPayload>(Events.DID_ABORT_WITH_ERROR);
export const onDidFinishUpdateCycle = createListener<DidFinishUpdateCyclePayload>(Events.DID_FINISH_UPDATE_CYCLE);
export const onFailedToDownloadUpdate = createListener<FailedToDownloadUpdatePayload>(Events.FAILED_TO_DOWNLOAD_UPDATE);
export const onUserDidCancelDownload = createListener<UserDidCancelDownloadPayload>(Events.USER_DID_CANCEL_DOWNLOAD);
export const onWillExtractUpdate = createListener<WillExtractUpdatePayload>(Events.WILL_EXTRACT_UPDATE);
export const onDidExtractUpdate = createListener<DidExtractUpdatePayload>(Events.DID_EXTRACT_UPDATE);
export const onWillRelaunchApplication = createListener<WillRelaunchApplicationPayload>(Events.WILL_RELAUNCH_APPLICATION);
export const onUserDidMakeChoice = createListener<UserDidMakeChoicePayload>(Events.USER_DID_MAKE_CHOICE);
export const onWillScheduleUpdateCheck = createListener<WillScheduleUpdateCheckPayload>(Events.WILL_SCHEDULE_UPDATE_CHECK);
export const onWillNotScheduleUpdateCheck = createListener<WillNotScheduleUpdateCheckPayload>(Events.WILL_NOT_SCHEDULE_UPDATE_CHECK);
export const onWillInstallUpdateOnQuit = createListener<WillInstallUpdateOnQuitPayload>(Events.WILL_INSTALL_UPDATE_ON_QUIT);
