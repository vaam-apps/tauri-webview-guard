# Upstream dependencies

[`upstream.json`](https://github.com/vaam-apps/tauri-webview-guard/blob/main/upstream.json)
lists every pin, workaround and upstream behaviour this plugin depends on, each
with its URL, its state when recorded, what it is waiting on, and what to
re-check when it moves — in a shape a scheduled job can poll: for each entry
with an `upstream.api`, GET it and compare the `field` it names against
`state_when_recorded`.

This page is a readable copy of the entries as recorded on 2026-09-23. The
JSON file is the source of truth.

| Entry                                                         | Kind               | Waiting on                                                  |
| ------------------------------------------------------------- | ------------------ | ----------------------------------------------------------- |
| [`tauri-specta-rc`](#tauri-specta-rc)                         | pin                | A stable tauri-specta 2.0.0 (and specta 2.0.0)              |
| [`tauri-android-jackson-api24`](#tauri-android-jackson-api24) | upstream defect    | Tauri dropping jackson-databind 2.15.3, or requiring API 26 |
| [`tauri-ipc-script-es2018`](#tauri-ipc-script-es2018)         | upstream behaviour | Nothing                                                     |
| [`wry-android-loadurl-posted`](#wry-android-loadurl-posted)   | upstream behaviour | Nothing                                                     |
| [`tauri-ios-project-yml-once`](#tauri-ios-project-yml-once)   | upstream behaviour | Nothing                                                     |

## tauri-specta release candidate {#tauri-specta-rc}

- **What:** `tauri-specta = "=2.0.0-rc.25"` (`Cargo.toml`), with
  `specta = "=2.0.0-rc.25"` and, for `tests/bindings.rs`,
  `specta-typescript = "=0.0.12"`.
- **Why:** the house plugin standard requires tauri-specta for end-to-end
  types. It has never had a stable 2.0 release; its own crate docs tell users
  to pin with `=` during the rc period because the generated bindings change
  shape between rcs.
- **Upstream:** [crates.io/crates/tauri-specta](https://crates.io/crates/tauri-specta),
  polled on `crate.max_stable_version` (`null` when recorded).
- **Re-check:** when a stable version appears, move all three pins, run
  `UPDATE_BINDINGS=1 cargo test --test bindings`, review the diff of
  `guest-js/bindings.ts`, and update the API docs.

## Android 7 crash in stock Tauri {#tauri-android-jackson-api24}

- **What:** stock Tauri crashes in `TauriActivity.onCreate` on Android 7.x
  (API 24/25) with `NoClassDefFoundError: java.lang.BootstrapMethodError`, from
  jackson-databind 2.15.3's static initialiser — before any plugin, this guard
  included, is constructed. Measured on tauri 2.11.6.
- **Workaround:** `examples/tauri-app/android-7-jackson-workaround.zsh` forces
  jackson-databind 2.12.7.1 and enables core library desugaring in the
  example's generated Gradle project. The plugin does **not** impose this on
  consumers; see [Android](/platforms/android#android-7-x-does-not-reach-the-guard).
- **Upstream:** [tauri-apps/tauri#8788](https://github.com/tauri-apps/tauri/issues/8788),
  open when recorded.
- **Re-check:** when the issue closes, check the jackson-databind version in
  the new Tauri release and run the example without the workaround on an
  API 24 image; if it starts, delete the workaround.

## Tauri's IPC script on old engines {#tauri-ipc-script-es2018}

- **What:** Tauri's injected IPC script uses object spread (`...options`,
  `scripts/ipc-protocol.js` line 77 in tauri 2.11.6), a Chromium 60 feature.
  On Chromium 53 it fails to parse, so every `invoke()` rejects.
- **Why it matters here:** it is the premise for the dialog being native —
  an HTML fallback whose button calls `invoke()` is dead on exactly the
  engines it is for.
- **Upstream:** raised in a comment on
  [tauri-apps/tauri#8788](https://github.com/tauri-apps/tauri/issues/8788).
- **Re-check:** informational. If Tauri ever ships an ES5 IPC script, the
  native dialog stays: it is strictly more robust.

## wry's posted first load on Android {#wry-android-loadurl-posted}

- **What:** wry's Android `loadUrlMainThread` posts the app's first `loadUrl`
  to the main looper (wry 0.55.1,
  [`RustWebView.kt`](https://github.com/tauri-apps/wry/blob/wry-v0.55.1/src/android/kotlin/RustWebView.kt)),
  so it runs _after_ the plugin's `load(webView)` hook. Blanking the WebView
  inside `load()` alone is overtaken by that posted load.
- **Handled by:** Tauri's `on_navigation` hook, plus a second, posted
  neutralize. See [the lifecycle](/lifecycle#sequence-android).
- **Re-check:** on a wry bump, re-run the Chromium 109 arm and confirm the log
  still shows `refused navigation to http://tauri.localhost/` and no
  `page load started: http://tauri.localhost/`.

## Tauri's iOS project.yml is written once {#tauri-ios-project-yml-once}

- **What:** `tauri ios init` writes `project.yml` only if it does not exist,
  so changing `bundle.iOS.minimumSystemVersion` after the first init never
  reaches `IPHONEOS_DEPLOYMENT_TARGET`. `bundle.iOS.template` is rendered on
  every init and does. The template path is resolved against the CLI's
  working directory. Tauri 2.11.6's default target is 14.0; the `dev` branch
  says 15.0.
- **Upstream:** [`project.rs` at tauri-v2.11.6](https://github.com/tauri-apps/tauri/blob/tauri-v2.11.6/crates/tauri-cli/src/mobile/ios/project.rs).
- **Re-check:** on a Tauri bump, re-copy Tauri's `project.yml` template at
  the new tag into `examples/tauri-app/src-tauri/ios-project.yml.hbs`, and
  re-run the three-step check. See
  [iOS](/platforms/ios#the-deployment-target-is-the-real-defence).
