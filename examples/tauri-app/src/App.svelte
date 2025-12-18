<script>
  import {
    checkForUpdates,
    checkForUpdatesInBackground,
    checkForUpdateInformation,
    sessionInProgress,
    httpHeaders,
    setHttpHeaders,
    userAgentString,
    setUserAgentString,
    sendsSystemProfile,
    setSendsSystemProfile,
    clearFeedUrlFromUserDefaults,
    resetUpdateCycleAfterShortDelay,
    currentVersion,
    canCheckForUpdates,
    feedUrl,
    allowedChannels,
    setAllowedChannels,
    feedUrlOverride,
    setFeedUrlOverride,
    feedParameters,
    setFeedParameters,
    shouldDownloadReleaseNotes,
    setShouldDownloadReleaseNotes,
    shouldRelaunchApplication,
    setShouldRelaunchApplication,
    mayCheckForUpdatesConfig,
    setMayCheckForUpdatesConfig,
    shouldProceedWithUpdate,
    setShouldProceedWithUpdate,
    decryptionPassword,
    setDecryptionPassword,
    lastFoundUpdate,
    onDidFinishLoadingAppcast,
    onDidFindValidUpdate,
    onDidNotFindUpdate,
    onWillDownloadUpdate,
    onDidDownloadUpdate,
    onWillInstallUpdate,
    onDidAbortWithError,
    onDidFinishUpdateCycle,
    onFailedToDownloadUpdate,
    onUserDidCancelDownload,
    onWillExtractUpdate,
    onDidExtractUpdate,
    onWillRelaunchApplication,
    onUserDidMakeChoice,
    onWillScheduleUpdateCheck,
    onWillNotScheduleUpdateCheck,
    onShouldPromptForPermission,
    onWillInstallUpdateOnQuit
  } from 'tauri-plugin-sparkle-updater-api'

  let response = $state('')
  let version = $state('')

  function updateResponse(returnValue) {
    response += `[${new Date().toLocaleTimeString()}] ` + (typeof returnValue === 'string' ? returnValue : JSON.stringify(returnValue)) + '<br>'
  }

  async function _checkForUpdates() {
    try {
      await checkForUpdates()
      updateResponse('Check for updates initiated')
    } catch (e) {
      updateResponse(`Error: ${e}`)
    }
  }

  async function _checkBackground() {
    try {
      await checkForUpdatesInBackground()
      updateResponse('Background check initiated')
    } catch (e) {
      updateResponse(`Error: ${e}`)
    }
  }

  async function _getVersion() {
    try {
      version = await currentVersion()
      updateResponse(`Current version: ${version}`)
    } catch (e) {
      updateResponse(`Error: ${e}`)
    }
  }

  async function _canCheck() {
    try {
      const can = await canCheckForUpdates()
      updateResponse(`Can check for updates: ${can}`)
    } catch (e) {
      updateResponse(`Error: ${e}`)
    }
  }

  async function _getFeedUrl() {
    try {
      const url = await feedUrl()
      updateResponse(`Feed URL: ${url}`)
    } catch (e) {
      updateResponse(`Error: ${e}`)
    }
  }

  async function _checkForUpdateInformation() {
    try {
      await checkForUpdateInformation()
      updateResponse('Probing check initiated (no UI)')
    } catch (e) {
      updateResponse(`Error: ${e}`)
    }
  }

  async function _sessionInProgress() {
    try {
      const inProgress = await sessionInProgress()
      updateResponse(`Session in progress: ${inProgress}`)
    } catch (e) {
      updateResponse(`Error: ${e}`)
    }
  }

  async function _getHttpHeaders() {
    try {
      const headers = await httpHeaders()
      updateResponse(`HTTP headers: ${JSON.stringify(headers)}`)
    } catch (e) {
      updateResponse(`Error: ${e}`)
    }
  }

  async function _setTestHeaders() {
    try {
      await setHttpHeaders({
        'X-Test-Header': 'test-value',
        'Authorization': 'Bearer test-token'
      })
      updateResponse('HTTP headers set')
    } catch (e) {
      updateResponse(`Error: ${e}`)
    }
  }

  async function _clearHttpHeaders() {
    try {
      await setHttpHeaders(null)
      updateResponse('HTTP headers cleared')
    } catch (e) {
      updateResponse(`Error: ${e}`)
    }
  }

  async function _getUserAgent() {
    try {
      const ua = await userAgentString()
      updateResponse(`User-Agent: ${ua}`)
    } catch (e) {
      updateResponse(`Error: ${e}`)
    }
  }

  async function _setCustomUserAgent() {
    try {
      await setUserAgentString('CustomApp/1.0 (TestBuild)')
      updateResponse('Custom User-Agent set')
    } catch (e) {
      updateResponse(`Error: ${e}`)
    }
  }

  async function _getSendsProfile() {
    try {
      const sends = await sendsSystemProfile()
      updateResponse(`Sends system profile: ${sends}`)
    } catch (e) {
      updateResponse(`Error: ${e}`)
    }
  }

  async function _toggleSendsProfile() {
    try {
      const current = await sendsSystemProfile()
      await setSendsSystemProfile(!current)
      updateResponse(`Sends system profile toggled to: ${!current}`)
    } catch (e) {
      updateResponse(`Error: ${e}`)
    }
  }

  async function _clearFeedUrl() {
    try {
      const cleared = await clearFeedUrlFromUserDefaults()
      updateResponse(`Cleared feed URL: ${cleared ?? 'none'}`)
    } catch (e) {
      updateResponse(`Error: ${e}`)
    }
  }

  async function _resetCycleDelayed() {
    try {
      await resetUpdateCycleAfterShortDelay()
      updateResponse('Reset update cycle scheduled')
    } catch (e) {
      updateResponse(`Error: ${e}`)
    }
  }

  async function _getAllowedChannels() {
    try {
      const channels = await allowedChannels()
      updateResponse(`Allowed channels: ${JSON.stringify(channels)}`)
    } catch (e) {
      updateResponse(`Error: ${e}`)
    }
  }

  async function _setTestChannels() {
    try {
      await setAllowedChannels(['stable', 'beta'])
      updateResponse('Allowed channels set to [stable, beta]')
    } catch (e) {
      updateResponse(`Error: ${e}`)
    }
  }

  async function _getFeedUrlOverride() {
    try {
      const url = await feedUrlOverride()
      updateResponse(`Feed URL override: ${url ?? 'none'}`)
    } catch (e) {
      updateResponse(`Error: ${e}`)
    }
  }

  async function _getFeedParams() {
    try {
      const params = await feedParameters()
      updateResponse(`Feed parameters: ${JSON.stringify(params)}`)
    } catch (e) {
      updateResponse(`Error: ${e}`)
    }
  }

  async function _setTestFeedParams() {
    try {
      await setFeedParameters({ channel: 'beta', source: 'test' })
      updateResponse('Feed parameters set')
    } catch (e) {
      updateResponse(`Error: ${e}`)
    }
  }

  async function _getDownloadNotes() {
    try {
      const download = await shouldDownloadReleaseNotes()
      updateResponse(`Download release notes: ${download}`)
    } catch (e) {
      updateResponse(`Error: ${e}`)
    }
  }

  async function _toggleDownloadNotes() {
    try {
      const current = await shouldDownloadReleaseNotes()
      await setShouldDownloadReleaseNotes(!current)
      updateResponse(`Download release notes toggled to: ${!current}`)
    } catch (e) {
      updateResponse(`Error: ${e}`)
    }
  }

  async function _getRelaunch() {
    try {
      const relaunch = await shouldRelaunchApplication()
      updateResponse(`Should relaunch: ${relaunch}`)
    } catch (e) {
      updateResponse(`Error: ${e}`)
    }
  }

  async function _toggleRelaunch() {
    try {
      const current = await shouldRelaunchApplication()
      await setShouldRelaunchApplication(!current)
      updateResponse(`Should relaunch toggled to: ${!current}`)
    } catch (e) {
      updateResponse(`Error: ${e}`)
    }
  }

  async function _getMayCheck() {
    try {
      const may = await mayCheckForUpdatesConfig()
      updateResponse(`May check for updates: ${may}`)
    } catch (e) {
      updateResponse(`Error: ${e}`)
    }
  }

  async function _toggleMayCheck() {
    try {
      const current = await mayCheckForUpdatesConfig()
      await setMayCheckForUpdatesConfig(!current)
      updateResponse(`May check for updates toggled to: ${!current}`)
    } catch (e) {
      updateResponse(`Error: ${e}`)
    }
  }

  async function _getShouldProceed() {
    try {
      const proceed = await shouldProceedWithUpdate()
      updateResponse(`Should proceed with update: ${proceed}`)
    } catch (e) {
      updateResponse(`Error: ${e}`)
    }
  }

  async function _toggleShouldProceed() {
    try {
      const current = await shouldProceedWithUpdate()
      await setShouldProceedWithUpdate(!current)
      updateResponse(`Should proceed toggled to: ${!current}`)
    } catch (e) {
      updateResponse(`Error: ${e}`)
    }
  }

  async function _getDecryptionPassword() {
    try {
      const pwd = await decryptionPassword()
      updateResponse(`Decryption password: ${pwd ? '***' : 'none'}`)
    } catch (e) {
      updateResponse(`Error: ${e}`)
    }
  }

  async function _getLastFoundUpdate() {
    try {
      const update = await lastFoundUpdate()
      if (update) {
        updateResponse(`Last found update: v${update.version}${update.isCritical ? ' [CRITICAL]' : ''}`)
      } else {
        updateResponse('No update found yet')
      }
    } catch (e) {
      updateResponse(`Error: ${e}`)
    }
  }

  $effect(() => {
    const unlisteners = []

    onDidFinishLoadingAppcast(() => {
      updateResponse('Appcast loaded')
    }).then(unlisten => unlisteners.push(unlisten))

    onDidFindValidUpdate((payload) => {
      let info = `Update available: v${payload.version}`
      if (payload.isCritical) info += ' [CRITICAL]'
      if (payload.isMajorUpgrade) info += ' [MAJOR]'
      if (payload.channel) info += ` (${payload.channel})`
      if (payload.minimumSystemVersion) info += ` (requires macOS ${payload.minimumSystemVersion})`
      if (payload.title) info += ` - ${payload.title}`
      updateResponse(info)
    }).then(unlisten => unlisteners.push(unlisten))

    onDidNotFindUpdate(() => {
      updateResponse('No update available')
    }).then(unlisten => unlisteners.push(unlisten))

    onWillDownloadUpdate((payload) => {
      updateResponse(`Will download: ${payload.version}`)
    }).then(unlisten => unlisteners.push(unlisten))

    onDidDownloadUpdate((payload) => {
      updateResponse(`Downloaded: ${payload.version}`)
    }).then(unlisten => unlisteners.push(unlisten))

    onWillInstallUpdate((payload) => {
      updateResponse(`Will install: ${payload.version}`)
    }).then(unlisten => unlisteners.push(unlisten))

    onDidAbortWithError((payload) => {
      updateResponse(`Error: ${payload.message}`)
    }).then(unlisten => unlisteners.push(unlisten))

    onDidFinishUpdateCycle((payload) => {
      updateResponse(`Update cycle finished: ${payload.updateCheck}${payload.error ? ` (error: ${payload.error.message})` : ''}`)
    }).then(unlisten => unlisteners.push(unlisten))

    onFailedToDownloadUpdate((payload) => {
      updateResponse(`Download failed: ${payload.version} - ${payload.error.message}`)
    }).then(unlisten => unlisteners.push(unlisten))

    onUserDidCancelDownload(() => {
      updateResponse('User cancelled download')
    }).then(unlisten => unlisteners.push(unlisten))

    onWillExtractUpdate((payload) => {
      updateResponse(`Will extract: ${payload.version}`)
    }).then(unlisten => unlisteners.push(unlisten))

    onDidExtractUpdate((payload) => {
      updateResponse(`Extracted: ${payload.version}`)
    }).then(unlisten => unlisteners.push(unlisten))

    onWillRelaunchApplication(() => {
      updateResponse('Will relaunch application')
    }).then(unlisten => unlisteners.push(unlisten))

    onUserDidMakeChoice((payload) => {
      updateResponse(`User choice: ${payload.choice} for ${payload.version} (${payload.stage})`)
    }).then(unlisten => unlisteners.push(unlisten))

    onWillScheduleUpdateCheck((payload) => {
      updateResponse(`Will schedule check in ${payload.delay}s`)
    }).then(unlisten => unlisteners.push(unlisten))

    onWillNotScheduleUpdateCheck(() => {
      updateResponse('Will not schedule update check')
    }).then(unlisten => unlisteners.push(unlisten))

    onShouldPromptForPermission(() => {
      updateResponse('Should prompt for permission')
    }).then(unlisten => unlisteners.push(unlisten))

    onWillInstallUpdateOnQuit((payload) => {
      updateResponse(`Will install ${payload.version} on quit`)
    }).then(unlisten => unlisteners.push(unlisten))

    return () => {
      unlisteners.forEach(unlisten => unlisten())
    }
  })
</script>

<h1>Sparkle Updater Demo</h1>

<section>
  <h4>Update Actions</h4>
  <div class="buttons">
    <button onclick={_checkForUpdates}>Check for Updates</button>
    <button onclick={_checkBackground}>Background Check</button>
    <button onclick={_checkForUpdateInformation}>Probe Update</button>
  </div>
</section>

<section>
  <h4>Status & Info</h4>
  <div class="buttons">
    <button onclick={_sessionInProgress}>Session In Progress?</button>
    <button onclick={_getVersion}>Get Version</button>
    <button onclick={_canCheck}>Can Check?</button>
    <button onclick={_getLastFoundUpdate}>Last Update</button>
  </div>
</section>

<section>
  <h4>Feed Configuration</h4>
  <div class="buttons">
    <button onclick={_getFeedUrl}>Get Feed URL</button>
    <button onclick={_clearFeedUrl}>Clear Feed URL</button>
    <button onclick={_getFeedUrlOverride}>Feed Override</button>
    <button onclick={_getAllowedChannels}>Get Channels</button>
    <button onclick={_setTestChannels}>Set Channels</button>
    <button onclick={_getFeedParams}>Get Params</button>
    <button onclick={_setTestFeedParams}>Set Params</button>
  </div>
</section>

<section>
  <h4>HTTP Settings</h4>
  <div class="buttons">
    <button onclick={_getHttpHeaders}>Get Headers</button>
    <button onclick={_setTestHeaders}>Set Headers</button>
    <button onclick={_clearHttpHeaders}>Clear Headers</button>
    <button onclick={_getUserAgent}>Get UA</button>
    <button onclick={_setCustomUserAgent}>Set UA</button>
    <button onclick={_getSendsProfile}>Sends Profile?</button>
    <button onclick={_toggleSendsProfile}>Toggle Profile</button>
  </div>
</section>

<section>
  <h4>Update Behavior</h4>
  <div class="buttons">
    <button onclick={_getDownloadNotes}>Download Notes?</button>
    <button onclick={_toggleDownloadNotes}>Toggle Notes</button>
    <button onclick={_getRelaunch}>Relaunch?</button>
    <button onclick={_toggleRelaunch}>Toggle Relaunch</button>
    <button onclick={_getMayCheck}>May Check?</button>
    <button onclick={_toggleMayCheck}>Toggle May Check</button>
    <button onclick={_getShouldProceed}>Proceed?</button>
    <button onclick={_toggleShouldProceed}>Toggle Proceed</button>
  </div>
</section>

<section>
  <h4>Other</h4>
  <div class="buttons">
    <button onclick={_resetCycleDelayed}>Reset Cycle</button>
    <button onclick={_getDecryptionPassword}>Get Password</button>
  </div>
</section>

<h3>Log</h3>
<div class="log">{@html response}</div>
