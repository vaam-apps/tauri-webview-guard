---
layout: home
title: tauri-plugin-webview-guard
titleTemplate: false

hero:
  name: webview-guard
  text: No white screen on an old WebView
  tagline: Refuse to boot a Tauri v2 app into a below-floor WebView engine, and say how to fix it — natively, before first paint.
  image:
    src: /logo.svg
    alt: tauri-plugin-webview-guard
  actions:
    - theme: brand
      text: Get started
      link: /guide/getting-started
    - theme: alt
      text: How it works
      link: /lifecycle
    - theme: alt
      text: View on GitHub
      link: https://github.com/vaam-apps/tauri-webview-guard

features:
  - title: Native, before first paint
    details: The engine version is read natively during plugin setup, which Tauri runs before it creates the first WebView. Below the floor, the WebView's first navigation is refused — the app's bundle is never parsed.
    link: /lifecycle
    linkText: The lifecycle
  - title: A dialog that names the fix
    details: An Android AlertDialog names the WebView provider and opens its Play Store listing; on iOS an alert says to update iOS. Never "update this app" — the app is current.
    link: /guide/configuration#the-dialog-copy
    linkText: The dialog copy
  - title: Tailwind v4's floor by default
    details: Chromium 111 on Android, iOS 16.4 on iOS, both configurable and inclusive. An unreadable version is Unknown, and the app boots.
    link: /guide/configuration
    linkText: Builder options
  - title: Typed end to end
    details: status() and openUpdate() are generated from the Rust command signatures by tauri-specta, with a tagged error type a frontend can switch on.
    link: /guide/api
    linkText: The API
---

## Same APK, either side of the floor

| Android, Chromium 109                                                                                              | Android, Chromium 113                                                            |
| ------------------------------------------------------------------------------------------------------------------ | -------------------------------------------------------------------------------- |
| ![The guard's dialog on Chromium 109, naming Android System WebView](./verification/screenshots/cr109-default.png) | ![The example app on Chromium 113](./verification/screenshots/cr113-default.png) |

Both screenshots are the same debug APK, two Chromium majors either side of the
default floor. [Device verification](/verification/) has all four emulator
arms, the iOS simulator, and what each run did and did not prove.

## In one minute

Register the plugin **first** in `src-tauri/src/lib.rs`:

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

That is the whole integration. The guard is live the moment it is registered;
to guard nothing, do not register it. Continue with
[Getting started](/guide/getting-started).
