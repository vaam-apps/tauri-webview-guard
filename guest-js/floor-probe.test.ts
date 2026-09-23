/**
 * The floor probe must do its job on an engine that cannot run anything newer
 * than ES5 — so the first test is that it IS ES5, parsed by a real ES5 parser.
 * The rest drive it in jsdom with the two CSS APIs stubbed each way.
 *
 * What jsdom cannot tell you is whether a real Chromium 53 runs it: that was
 * checked on the `spike_api24_webview` emulator, see docs/verification/.
 */
import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'

import { parse } from 'acorn'
import { JSDOM } from 'jsdom'
import { describe, expect, it } from 'vitest'

const source = readFileSync(
  fileURLToPath(new URL('../probe/floor-probe.js', import.meta.url)),
  'utf8',
)

interface ProbeResult {
  supported: boolean
  colorMix: boolean
  registerProperty: boolean
  computedColorMix: string
}

function run(
  html: string,
  css: { supports?: (p: string, v: string) => boolean; registerProperty?: unknown } | undefined,
) {
  const dom = new JSDOM(html, { runScripts: 'outside-only' })
  const win = dom.window as unknown as Window & {
    CSS?: unknown
    __WEBVIEW_GUARD_FLOOR__?: ProbeResult
    eval: (s: string) => void
  }
  Object.defineProperty(win, 'CSS', { value: css, configurable: true })
  win.eval(source)
  return { dom, win, doc: dom.window.document }
}

/**
 * JSDOM parses synchronously but fires DOMContentLoaded on a later task, which
 * is the same ordering a real engine gives an inline <head> script — so the
 * below-floor rendering is only observable after it.
 */
function contentLoaded(doc: Document): Promise<void> {
  return doc.readyState === 'loading'
    ? new Promise((resolve) => doc.addEventListener('DOMContentLoaded', () => resolve()))
    : Promise.resolve()
}

const MODERN = {
  supports: (_p: string, v: string) => v.startsWith('color-mix('),
  registerProperty: () => undefined,
}

describe('floor-probe.js', () => {
  it('is ES5 — it has to parse on Chromium 53', () => {
    expect(() => parse(source, { ecmaVersion: 5 })).not.toThrow()
  })

  it('passes only when color-mix() AND CSS.registerProperty are both present', () => {
    const cases: Array<[typeof MODERN | Partial<typeof MODERN> | undefined, boolean]> = [
      [MODERN, true],
      [{ supports: MODERN.supports }, false], // Safari 16.2/16.3: color-mix, no @property
      [{ supports: () => false, registerProperty: MODERN.registerProperty }, false], // Chromium 85-110
      [{ supports: () => false }, false],
      [undefined, false], // no CSS object at all
    ]
    for (const [css, expected] of cases) {
      const { win, doc } = run('<!doctype html><html><head></head><body></body></html>', css)
      expect(win.__WEBVIEW_GUARD_FLOOR__?.supported).toBe(expected)
      expect(doc.documentElement.getAttribute('data-webview-floor')).toBe(
        expected ? 'supported' : 'below',
      )
    }
  })

  it('treats a throwing CSS.supports as below the floor, not as a crash', () => {
    const { win } = run('<!doctype html><html><body></body></html>', {
      supports: () => {
        throw new Error('boom')
      },
      registerProperty: () => undefined,
    })
    expect(win.__WEBVIEW_GUARD_FLOOR__?.supported).toBe(false)
  })

  it('promotes parked bundles when supported', () => {
    const { doc } = run(
      '<!doctype html><html><head><script type="text/plain" data-webview-guard-src="/assets/app.js" crossorigin=""></script></head><body></body></html>',
      MODERN,
    )
    const promoted = doc.querySelector('script[type="module"]')
    expect(promoted?.getAttribute('src')).toBe('/assets/app.js')
    expect(promoted?.hasAttribute('crossorigin')).toBe(true)
  })

  it('never promotes a parked bundle below the floor', async () => {
    const { doc } = run(
      '<!doctype html><html><head><script type="text/plain" data-webview-guard-src="/assets/app.js"></script></head><body><p>app</p></body></html>',
      { supports: () => false },
    )
    await contentLoaded(doc)
    expect(doc.querySelector('script[type="module"]')).toBeNull()
  })

  it('renders the page-supplied template below the floor', async () => {
    const { doc } = run(
      '<!doctype html><html><head></head><body><p id="app">app</p><template id="webview-guard-below-floor"><h1 id="msg">Mettez à jour</h1></template></body></html>',
      { supports: () => false },
    )
    await contentLoaded(doc)
    expect(doc.getElementById('app')).toBeNull()
    expect(doc.getElementById('msg')?.textContent).toBe('Mettez à jour')
  })

  it('falls back to a plain message when the page supplies no template', async () => {
    const { doc } = run('<!doctype html><html><body><p id="app">app</p></body></html>', {
      supports: () => false,
    })
    await contentLoaded(doc)
    expect(doc.getElementById('app')).toBeNull()
    expect(doc.body.textContent).toContain('Update your browser')
  })

  it('leaves the page alone when supported', async () => {
    const { doc } = run('<!doctype html><html><body><p id="app">app</p></body></html>', MODERN)
    await contentLoaded(doc)
    expect(doc.getElementById('app')?.textContent).toBe('app')
  })
})
