# Configuration

Everything is configured on `tauri_plugin_webview_guard::Builder`, in Rust, at
registration. There is no runtime configuration, no `tauri.conf.json` section,
and no switch: a registered guard is a live guard.

Every option has a default, so `Builder::new().build()` — or its shorthand
`tauri_plugin_webview_guard::init()` — is a complete, working guard at
Tailwind v4's floor with English copy.

## Builder options

| Option                        | Default                            | Meaning                                                                                                                                                                                  |
| ----------------------------- | ---------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `.min_chromium(u32)`          | `111`                              | Minimum Chromium **major**, Android. Inclusive: `111` lets 111 through and blocks 110. Compared against the major of the provider's `versionName`.                                       |
| `.min_ios(impl Into<String>)` | `"16.4"`                           | Minimum iOS version, `major.minor` or `major.minor.patch`. Inclusive. Parsed strictly when `.build()` runs — `"16.4-beta"` is refused.                                                   |
| `.android_copy(AndroidCopy)`  | English, [below](#the-dialog-copy) | The Android dialog's title, message and two button labels.                                                                                                                               |
| `.ios_copy(IosCopy)`          | English, [below](#the-dialog-copy) | The iOS alert's title and message.                                                                                                                                                       |
| `.build()`                    | —                                  | The plugin. **Panics** if `min_ios` does not parse — a constant in your source, wrong on every launch, so it fails the first `tauri dev`. `Builder::floor()` validates without building. |

The defaults are exported as constants, `DEFAULT_MIN_CHROMIUM` (`111`) and
`DEFAULT_MIN_IOS` (`OsVersion` 16.4.0), for an app that wants to derive its
floor from them.

### How versions are compared

- **Android:** the Chromium major is the leading run of digits of the
  provider's `versionName`, up to the first dot — `109` from
  `109.0.5414.123` (`chromium_major`). Anything else, including no provider at
  all, is `Verdict::Unknown`.
- **iOS:** `OsVersion` is one to three dot-separated non-negative integers,
  nothing else. Missing components read as zero, so `"16.4"` is `16.4.0` and
  compares equal to the `16.4.0` that `ProcessInfo.operatingSystemVersion`
  reports. The minor is compared, not only the major.
- **Unknown boots.** An unreadable version is not evidence of an old engine,
  and an "update your WebView" dialog in front of a current device is the same
  dead end as the white screen. The app boots, with a warning in the log.

### Validating in a test

`Builder::floor()` returns the validated `Floor` (or a `ParseVersionError`)
without building a plugin, so an app's own test can check its configuration
without a Tauri runtime:

```rust
let floor = tauri_plugin_webview_guard::Builder::new()
    .min_chromium(120)
    .min_ios("17.2")
    .floor()
    .unwrap();
assert_eq!(floor.min_chromium, 120);
```

## The dialog copy

The one rule every string follows: **name the component that is out of date,
and never tell the user to update the app.** The app is current; "update this
app" sends them to a listing with nothing to install.

Three placeholders are substituted in Rust, once, before the text reaches the
native side — so Kotlin and Swift show exactly the text the unit tests assert
on:

| Placeholder   | Android                                                                                                    | iOS           |
| ------------- | ---------------------------------------------------------------------------------------------------------- | ------------- |
| `{component}` | The provider's own label, as `PackageManager` reports it — "Android System WebView", "Chrome", an OEM name | `iOS`         |
| `{installed}` | The provider's `versionName`, e.g. `109.0.5414.123`                                                        | e.g. `16.3.1` |
| `{required}`  | The configured Chromium major, e.g. `111`                                                                  | e.g. `16.4`   |

If the provider reports no label, or a blank one, `{component}` falls back to
"Android System WebView", which is what the stock provider calls itself.

### Defaults

```text
Android  title    Update {component}
         message  This app needs {component} version {required} or newer to display
                  correctly. This phone has version {installed}.

                  The app itself is up to date. Update {component} from the Play Store,
                  then open the app again.
         buttons  Open Play Store · Close app

iOS      title    Update iOS to continue
         message  This app needs iOS {required} or later. This iPhone runs iOS {installed}.

                  The app itself is up to date. Update iOS in Settings › General ›
                  Software Update, then open the app again.
```

The Android title and two buttons are deliberately unlike Android 14's own
"This app was built for an older version of Android" warning, which uses the
app name as title and a single **OK**.

The iOS alert has **no buttons**: iOS has no public URL that opens Software
Update (`openSettingsURLString` opens this app's own settings page — a dead end
dressed as a way forward), and apps are not meant to quit themselves.

### `AndroidCopy` and `IosCopy`

| Struct        | Fields                                              |
| ------------- | --------------------------------------------------- |
| `AndroidCopy` | `title`, `message`, `update_button`, `close_button` |
| `IosCopy`     | `title`, `message`                                  |

All fields are `String`, and all four Android strings are required: an empty
button label would render as an untappable blank. Both implement `Default`
with the English text above.

### Translating it

A French app, for example:

```rust
use tauri_plugin_webview_guard::{AndroidCopy, Builder};

Builder::new().android_copy(AndroidCopy {
    title: "Mettez à jour {component}".into(),
    message: "Cette application a besoin de {component} {required} ou plus récent. \
              Ce téléphone a la version {installed}.\n\n\
              L'application elle-même est à jour : mettez à jour {component} \
              depuis le Play Store, puis rouvrez-la."
        .into(),
    update_button: "Ouvrir le Play Store".into(),
    close_button: "Fermer".into(),
})
```
