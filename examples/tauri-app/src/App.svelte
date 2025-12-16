<script>
  import Greet from './lib/Greet.svelte'
  import {
    checkForUpdates,
    checkForUpdatesInBackground,
    currentVersion,
    canCheckForUpdates,
    feedUrl,
    onUpdateAvailable,
    onUpdateNotAvailable,
    onError
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

  // Set up event listeners
  $effect(() => {
    const unlisteners = []

    onUpdateAvailable((payload) => {
      updateResponse(`Update available: ${JSON.stringify(payload)}`)
    }).then(unlisten => unlisteners.push(unlisten))

    onUpdateNotAvailable(() => {
      updateResponse('No update available')
    }).then(unlisten => unlisteners.push(unlisten))

    onError((payload) => {
      updateResponse(`Update error: ${JSON.stringify(payload)}`)
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
