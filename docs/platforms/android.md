# Android

On Android the WebView is a Play-updatable component, independent of the OS,
so the fix for a below-floor engine is usually two taps. The guard's job is to
say which component, and to open its listing.

## What is checked

The provider is `WebViewCompat.getCurrentWebViewPackage(context)` from
`androidx.webkit` — the public WebView API on 26+, reflection into
`WebViewFactory` below. The same call returned the provider on API 24, 33, 34
and 36.

The package is never a constant. On Android 7–9 it is typically Android System
WebView (`com.google.android.webview`); on 10+ many devices use Chrome's
Trichrome library; some OEMs ship their own. Whatever `WebViewCompat` reports
is what the dialog names and what the Update button deep-links to.

The Chromium major is read from the start of the provider's `versionName`
(`109` from `109.0.5414.123`) and compared, inclusively, with
[`min_chromium`](/guide/configuration#builder-options) — `111` by default.

## Below the floor

- The WebView is **neutralized**: JavaScript off first, so even a page that
  does commit cannot run a line of the app's bundle; then hidden
  (`INVISIBLE`), so not even a blank page paints under the dialog; then any
  navigation in flight is stopped and replaced with `about:blank` — the one
  URL the Rust navigation hook still allows.
- An `AlertDialog` names the provider by its own label ("Android System
  WebView", "Chrome", an OEM name), with **Open Play Store** and **Close app**.
- **Open Play Store** fires `market://details?id=<provider>`; if nothing
  handles it, `https://play.google.com/store/apps/details?id=<provider>`.
  Fired and caught, not resolved in advance, so no `<queries>` entry is needed.
  The dialog stays up — a user who comes back without updating finds the same
  dialog, not a blank app.
- **Close app** calls `finishAndRemoveTask()`.
- The dialog is not cancellable by Back or by tapping outside.
- If the device reported no provider package at all, the dialog has no Update
  button — there is nothing to link to.

The plugin's manifest is empty on purpose: no permissions and no `<queries>`.
`getCurrentWebViewPackage` needs neither — the provider's `PackageInfo` is
handed to the app by the WebView factory, not looked up through
`PackageManager`, so Android 11's package-visibility filtering does not apply.

## An unreadable version boots

An unreadable version (no provider, an OEM `versionName` that does not start
with a Chromium major) is `Verdict::Unknown`: **the app boots**, with a
warning in the log. An unreadable version is not evidence of an old engine,
and an "update your WebView" dialog in front of a current device is the same
dead end as the white screen.

## Release builds and R8

The plugin's classes are kept by `proguard-rules.pro`. Tauri's
`PluginManager` reaches the plugin class and its commands by reflection, so R8
cannot see any reference to them; a stripped plugin would mean "No command
engine found", only in release, only on a minified app — and the guard
silently never running.

::: warning Not measured
Every device run so far used a debug APK. The keep rule is in place, but R8
has not been run against it — see
[what verification does not prove](/verification/#what-this-does-not-prove).
:::

## Android 7.x does not reach the guard

**Android 7.x (API 24/25) does not reach the guard on stock Tauri.** Tauri
2.11.6 crashes in `TauriActivity.onCreate` on API 24/25 — jackson-databind
2.15.3's static initialiser needs `java.lang.BootstrapMethodError` (API 26) —
before any plugin is constructed. Measured on the API 24 emulator;
[tauri-apps/tauri#8788](https://github.com/tauri-apps/tauri/issues/8788), open.

Either:

- raise `minSdk` to 26, or
- force an older Jackson in your generated Gradle project, as
  [`examples/tauri-app/android-7-jackson-workaround.zsh`](https://github.com/vaam-apps/tauri-webview-guard/blob/main/examples/tauri-app/android-7-jackson-workaround.zsh)
  does (reported working in that issue, and how the Chromium 53 arm here was
  verified). The generated Gradle project is gitignored, so it has to be
  patched after every `tauri android init`.

The plugin does not do this to your app: overriding a Tauri dependency is your
call. [Upstream dependencies](/reference/upstream#tauri-android-jackson-api24)
tracks when the workaround can go.

## Kotlin tests

`android/` is a Gradle subproject with no wrapper of its own, so its JVM tests
run from the example's generated Android project:

```bash
cd examples/tauri-app && npm install && npx tauri android init
npx tauri android build --debug --target aarch64 --apk
cd src-tauri/gen/android && ./gradlew :tauri-plugin-webview-guard:testDebugUnitTest
```
