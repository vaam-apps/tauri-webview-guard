# webview-guard example

A minimal Tauri v2 app that registers the guard exactly as an app would, and
shows what the page sees when it is let through.

```bash
# From the repository root, once: the example depends on the built JS package.
npm install && npm run build

cd examples/tauri-app
npm install
npm run tauri android init
./android-7-jackson-workaround.zsh     # only if you will run it on API 24/25; see below
npm run tauri android build -- --debug --target aarch64 --apk
```

On an engine at or above the floor the page loads and prints the user agent,
the floor probe's verdict, the state of Tauri's IPC, and the plugin's
`status()` through the generated bindings. Below the floor the page never
loads; the native dialog is all there is.

## The diagnostic build

To see what a below-floor engine does to the page and to Tauri's IPC — which
the guard normally never lets you see — lower the floor at compile time:

```bash
WEBVIEW_GUARD_EXAMPLE_MIN_CHROMIUM=1 npm run tauri android build -- --debug --target aarch64 --apk
```

The page then loads on every engine, the inline ES5 floor probe renders its
below-floor template, and the diagnostics say whether `invoke()` works. On
Chromium 53 it does not: `docs/verification/README.md`.

## iOS

`src-tauri/ios-project.yml` is Tauri 2.11.6's own iOS project template,
verbatim, set as `bundle.iOS.template` so that `bundle.iOS.minimumSystemVersion`
(16.4 here) actually reaches the Xcode project on every `tauri ios init` — see
the root README's iOS section for why that matters.

To force the iOS alert on a current simulator, override the floor through the
launch environment (a compile-time variable does not survive the Xcode build
phase that invokes cargo):

```bash
npm run tauri ios init
npm run tauri ios build -- --debug --target aarch64-sim
xcrun simctl install <udid> "src-tauri/gen/apple/build/arm64-sim/webview-guard example.app"
SIMCTL_CHILD_WEBVIEW_GUARD_EXAMPLE_MIN_IOS=99.0 xcrun simctl launch <udid> app.vaam.webviewguard.example
```

## Android 7 (API 24/25)

Stock Tauri 2.11.6 crashes on launch there, before any plugin runs
([tauri-apps/tauri#8788](https://github.com/tauri-apps/tauri/issues/8788)).
`android-7-jackson-workaround.zsh` appends the workaround reported in that
issue to the generated Gradle project, which is gitignored and so has to be
patched after every `tauri android init`. It is the example's choice, not the
plugin's; `upstream.json` tracks when it can go.
