# Why it exists

Tailwind v4's browser floor is **Chromium 111 / Safari 16.4 / Firefox 128** —
the union of `color-mix()` (Chromium 111, Safari 16.2) and `@property`
(Chromium 85, Safari 16.4). Below it an app does not crash. It does something
worse:

- **Chromium 99–110** lays out perfectly and gets every derived colour wrong.
  Each `color-mix(in oklab, var(--x) 30%, transparent)` is invalid, so the token
  falls back to full strength: a 30 %-opacity skeleton becomes a near-white
  blob, a subtle border becomes a hard line. The app looks _subtly wrong_, and
  the report reads as a bug, not as an old engine.
- **Below Chromium 99** there are no cascade layers, and Tailwind v4 emits its
  entire stylesheet inside `@layer`. A parser that does not know the at-rule
  discards the whole block: a white screen.

The owner of the apps this was built for has ruled the floor final — no
normalizer, no polyfill, no transpilation. So the job is not to make an old
engine work; it is to **refuse to boot into one and say how to fix it**,
instead of showing a screen the user will read as "this app is broken".

On Android the fix is usually two taps: the WebView is a Play-updatable
component, independent of the OS. On iOS WKWebView ships with the OS, so the
real defence is the App Store refusing to install at all (see
[iOS: the deployment target is the real defence](/platforms/ios#the-deployment-target-is-the-real-defence));
the in-app check is the second line.

## Why the dialog is native, never HTML

On Chromium 53, Tauri's own injected IPC script does not parse (object spread,
Chromium 60), so `window.__TAURI_INTERNALS__.postMessage` is never defined and
**every `invoke()` rejects** — measured, see
[the verification note](/verification/#does-tauri-s-ipc-work-on-chromium-53-no).
An HTML fallback page whose "Update" button calls a Tauri command is dead on
exactly the devices it exists for. An `AlertDialog` / `UIAlertController` has
no dependency on the engine it is judging.

## Why the check is native, never a JS probe in the bundle

A bundle that cannot parse cannot run the check that explains why it cannot
parse. The guard reads the engine version natively, during plugin setup —
which Tauri runs **before it creates the first WebView** — and refuses the
WebView's first navigation. [The lifecycle](/lifecycle) has the sequence, with
the Tauri and wry source lines behind each step.

For a build with no native layer at all — the plain browser build — the
package ships an ES5 [floor probe](./floor-probe) instead.
