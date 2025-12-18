<script>
  import Greet from './lib/Greet.svelte'
  import {
    checkForUpdates,
    checkForUpdatesInBackground,
    checkForUpdateInformation,
    sessionInProgress,
    currentVersion,
    canCheckForUpdates,
    feedUrl,
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

<main class="container">
  <h1>Sparkle Updater Demo</h1>

  <div class="row">
    <a href="https://vite.dev" target="_blank">
      <img src="/vite.svg" class="logo vite" alt="Vite Logo" />
    </a>
    <a href="https://tauri.app" target="_blank">
      <img src="/tauri.svg" class="logo tauri" alt="Tauri Logo" />
    </a>
    <a href="https://svelte.dev" target="_blank">
      <img src="/svelte.svg" class="logo svelte" alt="Svelte Logo" />
    </a>
  </div>

  <p>Test the Sparkle Updater plugin:</p>

  <div class="row">
    <Greet />
  </div>

  <div class="buttons">
    <button onclick={_checkForUpdates}>Check for Updates</button>
    <button onclick={_checkBackground}>Background Check</button>
    <button onclick={_checkForUpdateInformation}>Probe Update (No UI)</button>
    <button onclick={_sessionInProgress}>Session In Progress?</button>
    <button onclick={_getVersion}>Get Version</button>
    <button onclick={_canCheck}>Can Check?</button>
    <button onclick={_getFeedUrl}>Get Feed URL</button>
  </div>

  <div class="response">
    <h3>Response:</h3>
    <div class="log">{@html response}</div>
  </div>

</main>

<style>
  .logo.vite:hover {
    filter: drop-shadow(0 0 2em #747bff);
  }

  .logo.svelte:hover {
    filter: drop-shadow(0 0 2em #ff3e00);
  }

  .buttons {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
    justify-content: center;
    margin: 16px 0;
  }

  .response {
    margin-top: 16px;
    text-align: left;
  }

  .log {
    background: #1a1a1a;
    color: #f6f6f6;
    padding: 12px;
    border-radius: 8px;
    font-family: monospace;
    font-size: 12px;
    max-height: 200px;
    overflow-y: auto;
  }
</style>
