// The app's bundle. It only ever runs on an engine the native guard let
// through, so it may use anything the floor allows.
import { commands } from 'tauri-plugin-webview-guard-api'

const out = document.getElementById('status')

const result = await commands.status()
if (out) {
  out.textContent =
    result.status === 'ok'
      ? JSON.stringify(result.data, null, 2)
      : `error (${result.error.kind}): ${JSON.stringify(result.error, null, 2)}`
}
