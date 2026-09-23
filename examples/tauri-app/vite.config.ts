import { readFileSync } from 'node:fs'
import { createRequire } from 'node:module'

import { defineConfig } from 'vite'

// The floor probe is inlined, never imported: an import would put it inside
// the bundle, and the bundle is what a below-floor engine may not parse.
const require = createRequire(import.meta.url)
const probe = readFileSync(
  require.resolve('tauri-plugin-webview-guard-api/probe/floor-probe.js'),
  'utf8',
)

export default defineConfig({
  clearScreen: false,
  // Fixed port: the Tauri CLI points the WebView at `devUrl`, so the two must agree.
  server: { port: 5173, strictPort: true },
  // The bundle targets the floor itself: it only ever runs on an engine the
  // guard let through, so there is nothing to gain from transpiling lower.
  build: { target: ['chrome111', 'safari16.4'] },
  plugins: [
    {
      name: 'inline-webview-guard-floor-probe',
      transformIndexHtml: {
        order: 'pre',
        handler: (html) =>
          html.replace('<!-- webview-guard:floor-probe -->', () => `<script>${probe}</script>`),
      },
    },
  ],
})
