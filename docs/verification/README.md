# Device verification, 2026-09-23

The example app (`examples/tauri-app`), built from this branch, run on four
Android emulators that bracket the Chromium 111 floor, plus an iOS simulator.
Every screenshot below was opened and looked at before it was cited, and for
each Android capture the focused window was recorded — Android 14's
"built for an older version of Android" system dialog reads a lot like this
guard's, so a screenshot is only attributed to the guard when the focused
window is the example's own `MainActivity` and the UI tree's only package is
`app.vaam.webviewguard.example`. The example targets SDK 36 (Tauri's generated
`gen/android/app/build.gradle.kts`), and that system dialog did not appear on
any arm.

## Setup

| AVD                   | API | Image               | Profile                      |
| --------------------- | --- | ------------------- | ---------------------------- |
| `spike_api24_webview` | 24  | `google_apis` arm64 | pixel_4, 1080×2280 @ 440 dpi |
| `spike_api33_webview` | 33  | `google_apis` arm64 | same                         |
| `spike_api34_webview` | 34  | `google_apis` arm64 | same                         |
| `spike_api36_webview` | 36  | `google_apis` arm64 | same                         |

Profile read back from each device with `wm size` / `wm density`. None of the
four has a real Play Store: `com.android.vending` is present but is a 1.8 stub
that resolves no `market:` intent (`pm query-activities` for
`market://details?id=x`: "No activities found").

Build: `npm run tauri android build --debug --target aarch64 --apk`, with the
floor at the plugin default (111), after
`examples/tauri-app/android-7-jackson-workaround.zsh` — without it, stock Tauri
does not start on API 24 at all (see below). Each run: `am force-stop`,
`logcat -c`, `am start`, **15 s settle**, then focus, UI tree, `screencap`,
logcat.

## The four arms

| Arm                   | `getCurrentWebViewPackage` returned                                           | Guard verdict | What appeared                                                                                                           | Update button                                                                                                                                                         | Screenshot                                                                                                                                                    |
| --------------------- | ----------------------------------------------------------------------------- | ------------- | ----------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Chromium 53 (API 24)  | `com.google.android.webview`, `53.0.2785.124`, label "Android System WebView" | `below_floor` | **Dialog** "Update Android System WebView", over a hidden WebView                                                       | `market:` → no handler; `https://play.google.com/…` → **WebView Browser Tester** (`org.chromium.webview_shell`, the image's only browser) showing the WebView listing | [dialog](screenshots/cr53-default.png) · [after Update](screenshots/cr53-default-update.png)                                                                  |
| Chromium 109 (API 33) | `com.google.android.webview`, `109.0.5414.123`, "Android System WebView"      | `below_floor` | **Dialog** — instead of the app, which on this engine would have rendered legibly with every `color-mix()` colour wrong | `market:` → no handler; https → **Chrome** (`com.android.chrome`, first-run screen)                                                                                   | [dialog](screenshots/cr109-default.png) · [after Update](screenshots/cr109-default-update.png) · [back from Chrome](screenshots/cr109-default-after-back.png) |
| Chromium 113 (API 34) | `com.google.android.webview`, `113.0.5672.136`, "Android System WebView"      | `supported`   | **App loads**; no system dialog                                                                                         | —                                                                                                                                                                     | [app](screenshots/cr113-default.png)                                                                                                                          |
| Chromium 134 (API 36) | `com.google.android.webview`, `134.0.6998.135`, "Android System WebView"      | `supported`   | **App loads**; no system dialog                                                                                         | —                                                                                                                                                                     | [app](screenshots/cr134-default.png)                                                                                                                          |

Logs and focused windows for each: `logs/cr*-default.logcat.txt`,
`logs/cr*-default.focus.txt`, `logs/cr*-default-update.*`.

**One code path covers all four, but it is two paths inside androidx.** The
same `WebViewCompat.getCurrentWebViewPackage` call returned the provider on
API 24 (below API 26 androidx reaches it by reflection into
`WebViewFactory`) and on 33/34/36 (the public API). All four returned
`com.google.android.webview`, so these stock images never exercise the
Trichrome or OEM case — "never hardcode the package" stands on the API's
contract and a unit test (`assess::tests::the_package_and_label_are_never_hardcoded`),
not on a difference observed here.

**The 109 arm is the one that justifies the plugin.** Below Chromium 111 the
app does not go blank until Chromium 99; in between it renders subtly wrong.
The guard's dialog on 109 is shown _instead of_ that app: the logcat for the
run has `refused navigation to http://tauri.localhost/` and no
`page load started` for the app URL — only for `about:blank`.

**The dialog survives a round trip to the store.** After Update opened Chrome,
Back returned to the same dialog (`cr109-default-after-back.png`; the Update
button's listener is replaced so it does not dismiss). **Close app** removed
the task and the process (`logs/cr109-close-app.txt`: focus back on the
previously open app, 0 recents entries, no pid).

## The native verdict and the floor probe agree on every arm

The diagnostic build (`WEBVIEW_GUARD_EXAMPLE_MIN_CHROMIUM=1`, which lets every
engine through on purpose) shows what the page itself sees:

| Arm | Native `versionName` major vs 111 | Probe `color-mix()` | Probe `CSS.registerProperty` | Probe computed `color-mix`                        | Probe verdict                                           |
| --- | --------------------------------- | ------------------- | ---------------------------- | ------------------------------------------------- | ------------------------------------------------------- |
| 53  | below                             | false               | false                        | `rgb(18, 52, 86)` (fallback: declaration dropped) | below — [screenshot](screenshots/cr53-diag.png)         |
| 109 | below                             | false               | true                         | `rgb(18, 52, 86)`                                 | below — [screenshot](screenshots/cr109-diag.png)        |
| 113 | above                             | true                | true                         | `oklab(0.539974 0.0962086 -0.0928316)`            | supported — [screenshot](screenshots/cr113-default.png) |
| 134 | above                             | true                | true                         | `oklab(0.539974 0.0962086 -0.0928316)`            | supported — [screenshot](screenshots/cr134-default.png) |

No disagreement: on these providers the `versionName` tracks the feature set
exactly at the floor. The probe ran on Chromium 53, so its ES5 claim is proven
on the engine that matters, not only by the parser test.

## Does Tauri's IPC work on Chromium 53? No

From the same diagnostic run on the 53 arm ([screenshot](screenshots/cr53-diag.png)):

```text
typeof __TAURI_INTERNALS__: object
internals: invoke=function ipc=function postMessage=undefined transformCallback=function
ipc: REJECTED TypeError: window.__TAURI_INTERNALS__.postMessage is not a function
```

and in logcat: `Uncaught SyntaxError: Unexpected token ...`. The script that
defines `postMessage` is tauri 2.11.6's `scripts/ipc-protocol.js`, whose line
77 is an object spread (`...options`, Chromium 60); it fails to parse, so every
`invoke()` rejects. An HTML fallback page with an "Update" button calling
`invoke("open_store")` would be dead on exactly this engine. The same probe on
Chromium 109 shows `postMessage=function` and `ipc: OK`. This is why the
dialog is native — and it would be native regardless.

## Android 7 does not start at all on stock Tauri

Before the Jackson workaround, the API 24 arm never reached the guard:
`java.lang.NoClassDefFoundError: Failed resolution of: Ljava/lang/BootstrapMethodError;`
from `ObjectMapper.<clinit>` ← `app.tauri.plugin.PluginManager.<clinit>` ←
`TauriActivity.onCreate`, i.e. inside Tauri, before any plugin is constructed
([screenshot](screenshots/cr53-stock-tauri-crash.png): Android's own
"webview-guard example has stopped"; `logs/cr53-stock-tauri-crash.logcat.txt`).
That is [tauri-apps/tauri#8788](https://github.com/tauri-apps/tauri/issues/8788),
open. `upstream.json` records it.

## iOS

No iOS 16.3 runtime is installed on this machine, so the below-floor path was
exercised on an **iOS 26.5 simulator** (a dedicated `wg-guard-verify` device,
iPhone 16) with the floor forced through the example's launch-environment
override:
`SIMCTL_CHILD_WEBVIEW_GUARD_EXAMPLE_MIN_IOS=99.0 xcrun simctl launch …`.

| Run                  | Verdict       | What appeared                                                                                                                                                                    | Evidence                                                                     |
| -------------------- | ------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------- |
| floor 99.0           | `below_floor` | **Alert** "Update iOS to continue — This app needs iOS 99.0 or later. This iPhone runs iOS 26.5.0." No buttons. Log: `refused navigation to tauri://localhost`, no app page load | [screenshot](screenshots/ios26-forced99.png) · `logs/ios26-forced99.log.txt` |
| floor 16.4 (default) | `supported`   | App loads; probe `supported`, IPC OK                                                                                                                                             | [screenshot](screenshots/ios26-default.png) · `logs/ios26-default.log.txt`   |

A side observation on the default run: WKWebView's user agent on iOS 26.5 says
`iPhone OS 18_7`. User-agent parsing would have misreported this device by
eight majors, which is one more reason the guard reads `ProcessInfo`.

The deployment-target trap was reproduced on the example itself:
`logs/ios-trap/`. First `tauri ios init` with `minimumSystemVersion: "16.4"`
→ `IPHONEOS_DEPLOYMENT_TARGET = 16.4`. Changed to `"17.0"` and re-ran init →
**still 16.4**. Added `bundle.iOS.template` and re-ran → 17.0. And
`"template": "ios-project.yml"` (relative to `src-tauri/`) failed with
"failed to read custom Xcode project template" — the path is relative to the
directory the CLI runs in.

## Mutation checks

`logs/mutation-checks.txt`. Each mutation was applied to the committed
source, the relevant suite run, and the file restored from git:

| Mutation                                                       | Caught by                                                                |
| -------------------------------------------------------------- | ------------------------------------------------------------------------ |
| Chromium `>=` → `>`                                            | 3 tests, incl. `chromium_boundary_is_inclusive`                          |
| Chromium floor hardcoded to the default (111)                  | `chromium_floor_is_configurable`                                         |
| Chromium floor hardcoded to 110                                | `chromium_boundary_is_inclusive`, `chromium_floor_is_configurable`       |
| iOS `>=` → `>`                                                 | `ios_boundary_is_inclusive_and_minor_aware`, `ios_floor_is_configurable` |
| iOS compares the major only                                    | 2 tests                                                                  |
| Navigation hook lets `localhost` through while blocked         | `nothing_but_about_blank_navigates_once_blocked`                         |
| `Unknown` also blocks (Android) / (iOS only)                   | `unknown_never_blocks` / `ios_unknown_never_blocks`                      |
| Probe `&&` → `\|\|`                                            | 2 probe tests                                                            |
| Probe gates on `color-mix()` alone (the Safari 16.2/16.3 hole) | `passes only when color-mix() AND CSS.registerProperty are both present` |
| One `const` in the probe                                       | `is ES5 — it has to parse on Chromium 53`                                |
| Probe ignores the page's template                              | `renders the page-supplied template below the floor`                     |

The iOS-only `Unknown` mutation initially survived — only the Android half had
a test. `ios_unknown_never_blocks` was added and the mutation re-run red.

## What this does not prove

- **A real Play Store.** Neither the `market:` path nor an actual WebView
  update was exercised; both images lack the Play Store. The https fallback is
  what fired on both below-floor arms.
- **A Trichrome or OEM provider.** All four arms report
  `com.google.android.webview`.
- **iOS below 16.4 on a real OS version.** The alert was forced on 26.5.
- **Release builds.** All runs are debug APKs; `proguard-rules.pro` keeps the
  plugin's classes, but R8 was not run.
