# API

None of this is needed for the guard to work. The guard acts natively during
setup, before any JavaScript can run — on a below-floor engine no JavaScript of
yours ever runs. The two commands exist for an app that wants to _show_ the
engine (an About screen, a support dump) or to offer the same update link as a
soft nudge while still above the floor.

| On `app.webview_guard()` | TypeScript              | Returns        | Platforms    |
| ------------------------ | ----------------------- | -------------- | ------------ |
| `status()`               | `commands.status()`     | `GuardStatus`  | Android, iOS |
| `open_update()`          | `commands.openUpdate()` | `UpdateOpened` | Android      |

Every other combination returns the typed `unsupported` error — see
[the error type](#the-error-type).

## Rust

```rust
use tauri_plugin_webview_guard::{WebviewGuardExt, Verdict};

let status = app.webview_guard().status()?;          // GuardStatus
if status.verdict == Verdict::Supported { /* … */ }
app.webview_guard().open_update()?;                  // UpdateOpened, Android only
```

`WebviewGuardExt::webview_guard` is implemented for every `Manager` — an
`AppHandle`, a `Window`, the `App`.

### `GuardStatus`

Computed once, during plugin setup — before the first WebView exists — and
never recomputed. A process keeps the WebView implementation it loaded, so a
provider updated mid-session does not change what this process renders with;
the next launch re-probes.

| Field (Rust / TypeScript)              | Type             | Meaning                                                                                                                      |
| -------------------------------------- | ---------------- | ---------------------------------------------------------------------------------------------------------------------------- |
| `platform`                             | `Platform`       | `android` or `ios`                                                                                                           |
| `engine`                               | `EngineKind`     | `chromium` (Android System WebView, Chrome/Trichrome, or an OEM build) or `webkit` (WKWebView)                               |
| `installed`                            | `Option<String>` | The version the device reported, verbatim: the provider's `versionName` on Android, the OS version on iOS. `None` if nothing |
| `required`                             | `String`         | The configured floor, as shown to the user: `111` or `16.4`                                                                  |
| `provider_package` / `providerPackage` | `Option<String>` | Android only: the package actually providing the WebView, as `WebViewCompat.getCurrentWebViewPackage` returned it            |
| `provider_label` / `providerLabel`     | `Option<String>` | Android only: that package's user-visible label, e.g. "Android System WebView" — the name the dialog uses                    |
| `verdict`                              | `Verdict`        | The decision                                                                                                                 |

### `Verdict`

| Variant      | Wire          | Meaning                                                                                         |
| ------------ | ------------- | ----------------------------------------------------------------------------------------------- |
| `Supported`  | `supported`   | At or above the floor. The app boots normally.                                                  |
| `BelowFloor` | `below_floor` | Below the floor. The guard blocks the WebView and shows the native dialog.                      |
| `Unknown`    | `unknown`     | The engine's version could not be read or parsed. The guard **lets the app boot**, and logs it. |

### `UpdateOpened`

`open_update()` reports where the user was sent: `target` is `StoreTarget`
`market` (`market://details?id=…` resolved, so the Play Store app opened) or
`web` (nothing resolved `market://`, so the
`https://play.google.com/store/apps/details?id=…` fallback opened in a
browser), and `package` is the provider package whose listing was opened.

## TypeScript (tauri-specta)

`guest-js/bindings.ts` is **generated** from the Rust command signatures by
tauri-specta and committed, so the JS package builds without a Rust toolchain.
`tests/bindings.rs` regenerates it and fails if the committed file is stale;
`UPDATE_BINDINGS=1 cargo test --test bindings` rewrites it.

```ts
import { commands } from "tauri-plugin-webview-guard-api";

const r = await commands.status();
if (r.status === "ok") console.log(r.data.verdict, r.data.installed);
else if (r.error.kind === "unsupported") {
  /* desktop, or openUpdate on iOS */
}
```

Both commands resolve to `{ status: "ok", data }` or
`{ status: "error", error }` rather than throwing. The package also exports
`FLOOR_PROBE_PATH` (`"probe/floor-probe.js"`), the probe's path inside the
package — see [the floor probe](./floor-probe).

Calling either command needs the `webview-guard:default` permission (or the
individual `allow-*` ones) in a capability; see
[Capabilities](./getting-started#capabilities).

## The error type

`WebviewGuardError` (aliased as `tauri_plugin_webview_guard::Error`),
serialised as a tagged object so a frontend switches on `kind`:

| `kind`             | When                                                                                                    | Fields                             |
| ------------------ | ------------------------------------------------------------------------------------------------------- | ---------------------------------- |
| `unsupported`      | Every call on desktop; `openUpdate` on iOS; `openUpdate` on an Android device that reported no provider | `platform`, `capability`, `reason` |
| `no_store_handler` | Android: neither the Play Store nor any browser could open the listing                                  | `package`                          |
| `native`           | The native side failed in a way it did not classify                                                     | `command`, `message`               |

The distinction that matters most is `unsupported`: "this platform has no such
capability" is a fact to design around, while `native` — "the capability
exists and just failed" — is a bug to report.

It is named `WebviewGuardError` rather than `Error` because tauri-specta
exports a type under its Rust name, and a TypeScript type called `Error`
shadows the global `Error` — including in the generated bindings' own
`typedError`, which tests `e instanceof Error`.

## Why tauri-specta is pinned to a release candidate

`tauri-specta = "=2.0.0-rc.25"`, `specta = "=2.0.0-rc.25"`,
`specta-typescript = "=0.0.12"` (dev only). tauri-specta 2 has never had a
stable release; its own crate docs say to pin with `=` during the rc period
because the generated bindings change shape between rcs. The house plugin
standard requires end-to-end types, and the owner accepts the rc for that.
[Upstream dependencies](/reference/upstream#tauri-specta-rc) records what to
re-check when a stable 2.0.0 lands.

## Rust docs

The crate is not on crates.io, so there is no docs.rs page. Build the API docs
locally from a checkout:

```bash
cargo doc --open
```
