# Desktop

macOS, Windows and Linux are compiled so the app runs in `tauri dev` — and
nothing more. The desktop engines are not probed:

| Engine    | Platform | What it is                                                                     |
| --------- | -------- | ------------------------------------------------------------------------------ |
| WebView2  | Windows  | Chromium, evergreen                                                            |
| WKWebView | macOS    | Versioned with Safari, which updates independently of the OS on older releases |
| WebKitGTK | Linux    | Versioned by the distribution                                                  |

## What happens

- At setup the guard logs one warning, through the `log` facade, naming the
  OS and the floor it _would_ have applied:
  `webview-guard: macos is not guarded (floor would be Chromium 111 / iOS 16.4); …`
- The app's navigation is never refused: with no verdict, nothing blocks.
- Every call — `status()` and `open_update()`, from Rust or TypeScript —
  returns the typed `unsupported` error, rather than a status that would claim
  a verdict nobody computed:

```json
{
  "kind": "unsupported",
  "platform": "macos",
  "capability": "status",
  "reason": "the webview guard checks the Android WebView provider and the iOS version; desktop engines are not probed"
}
```

A frontend shared between desktop and mobile should treat
`error.kind === "unsupported"` as "nothing to show", not as a failure. See
[the error type](/guide/api#the-error-type).
