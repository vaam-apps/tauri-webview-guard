# Getting started

## Requirements

- Tauri v2. The plugin is verified against **tauri 2.11.6** and **wry 0.55.1**
  ([the lifecycle](/lifecycle) cites their source lines).
- Rust 1.85 or newer (`rust-version` in `Cargo.toml`: tauri-specta
  2.0.0-rc.25 is an edition-2024 crate).
- The guard acts on **Android** and **iOS**. On desktop it compiles, so
  `tauri dev` runs, but guards nothing — see [Desktop](/platforms/desktop).

## Install

Pin the tag. Like its sibling
[`tauri-sign-keypair`](https://github.com/vaam-apps/tauri-sign-keypair), this
plugin is consumed by git ref, not from crates.io or npm: Rust, Kotlin, Swift
and TypeScript ship from one tag, and publishing only two of them would let
the others drift.

```bash
cargo add tauri-plugin-webview-guard --git https://github.com/vaam-apps/tauri-webview-guard --tag vX.Y.Z
```

The JavaScript package, `tauri-plugin-webview-guard-api`, is needed **only**
for the typed bindings or the browser floor probe:

```bash
npm install github:vaam-apps/tauri-webview-guard#vX.Y.Z
```

The tags and their notes are on the
[releases page](https://github.com/vaam-apps/tauri-webview-guard/releases);
the [changelog](/reference/changelog) lists what changed.

## Register it — first

Register it in `src-tauri/src/lib.rs`, **first**, so its setup runs before any
plugin that might create a window:

```rust
tauri::Builder::default()
    .plugin(
        tauri_plugin_webview_guard::Builder::new()
            .min_chromium(111)
            .min_ios("16.4")
            .build(),
    )
    // ...
```

That is the whole integration. The guard is live the moment it is registered —
there is no switch to turn it on; to guard nothing, do not register it.

`tauri_plugin_webview_guard::init()` is the same as `Builder::new().build()`:
the default floor (Chromium 111, iOS 16.4) with the default English copy. Every
option is on the [Configuration](./configuration) page.

## Capabilities

The guard itself needs **no** permission — it acts before any page exists, and
a capability that denies both commands leaves it fully in force.

The frontend needs a permission only to call `status()` / `openUpdate()`:

```json
{ "permissions": ["webview-guard:default"] }
```

`webview-guard:default` allows both commands. Each can also be granted or
denied on its own:

| Identifier                        | What it does                        |
| --------------------------------- | ----------------------------------- |
| `webview-guard:default`           | `allow-status`, `allow-open-update` |
| `webview-guard:allow-status`      | Enables the `status` command        |
| `webview-guard:deny-status`       | Denies the `status` command         |
| `webview-guard:allow-open-update` | Enables the `open_update` command   |
| `webview-guard:deny-open-update`  | Denies the `open_update` command    |

Neither command can change anything a page could not already see or do:
`status` reports the engine version and the verdict the guard already acted
on at boot; `open_update` opens a public store listing.

## Before you ship

- **iOS:** set the deployment target to your floor, and check it actually
  reached the Xcode project — it often does not.
  [iOS: the deployment target is the real defence](/platforms/ios#the-deployment-target-is-the-real-defence).
- **Android 7.x (API 24/25):** stock Tauri crashes there before any plugin,
  this one included, is constructed.
  [Android 7 does not reach the guard](/platforms/android#android-7-x-does-not-reach-the-guard).
- **Browser build:** with no native layer, inline the
  [floor probe](./floor-probe) instead.

## Try the example

[`examples/tauri-app`](https://github.com/vaam-apps/tauri-webview-guard/tree/main/examples/tauri-app)
registers the guard exactly as an app would. On an engine at or above the
floor its page prints the user agent, the floor probe's verdict, the state of
Tauri's IPC, and the plugin's `status()`; below the floor the page never loads
and the native dialog is all there is.

```bash
# From the repository root, once: the example depends on the built JS package.
npm install && npm run build

cd examples/tauri-app
npm install
npm run tauri android init
./android-7-jackson-workaround.zsh     # only if you will run it on API 24/25
npm run tauri android build -- --debug --target aarch64 --apk
```

Its diagnostic build (`WEBVIEW_GUARD_EXAMPLE_MIN_CHROMIUM=1`) lowers the floor
on purpose, to show what a below-floor engine does to the page and to Tauri's
IPC; on iOS, `SIMCTL_CHILD_WEBVIEW_GUARD_EXAMPLE_MIN_IOS=99.0` forces the
alert on a current simulator. See the example's
[README](https://github.com/vaam-apps/tauri-webview-guard/blob/main/examples/tauri-app/README.md).
