# Browser builds: the floor probe

With no native layer there is nothing to stop the bundle loading, so the
package ships an **ES5** probe to inline in `<head>`, before every other
script:
[`probe/floor-probe.js`](https://github.com/vaam-apps/tauri-webview-guard/blob/main/probe/floor-probe.js).

::: tip In a Tauri app on Android or iOS you do not need this
The plugin's native guard refuses to let the WebView load at all. The probe is
for the plain browser build, and as a cross-check.
:::

## What it tests

The two features the floor is actually made of, not a version string:

| Feature       | Chromium | Safari | Firefox |
| ------------- | -------- | ------ | ------- |
| `color-mix()` | 111      | 16.2   | 113     |
| `@property`   | 85       | 16.4   | 128     |

The floor (Chromium 111 / Safari 16.4 / Firefox 128) is the union of those two
rows, so the gate is their conjunction: `color-mix()` **and**
`CSS.registerProperty` as the `@property` sentinel. A `color-mix()`-only check
would wave iOS 16.2 and 16.3 through. `CSS.registerProperty` stands in for
`@property` because `CSS.supports` cannot test an at-rule, and the JS API and
the at-rule shipped together in every engine.

## What it does

- **Always:** sets `<html data-webview-floor="supported|below">` and
  `window.__WEBVIEW_GUARD_FLOOR__ = { supported, colorMix, registerProperty, computedColorMix }`.
  `computedColorMix` is diagnostic only, never part of the verdict: an engine
  that drops the declaration reports the `#123456` fallback, `rgb(18, 52, 86)`.
- **Supported:** promotes every
  `<script type="text/plain" data-webview-guard-src="…">` to a module script
  (keeping its `crossorigin`) — so a bundle parked that way is never handed to
  an engine that cannot parse it.
- **Below:** on `DOMContentLoaded`, replaces `<body>` with the content of your
  `<template id="webview-guard-below-floor">`, or a plain English "Update your
  browser" message if the page has no such template.

## Inlining it

It must be inlined, not imported: an import is part of the bundle, and the
bundle is exactly what a below-floor engine may fail to parse. With Vite, as
the example app does:

```ts
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";

import { defineConfig } from "vite";

const require = createRequire(import.meta.url);
const probe = readFileSync(
  require.resolve("tauri-plugin-webview-guard-api/probe/floor-probe.js"),
  "utf8",
);

export default defineConfig({
  plugins: [
    {
      name: "inline-webview-guard-floor-probe",
      transformIndexHtml: {
        order: "pre",
        handler: (html) =>
          html.replace("<!-- webview-guard:floor-probe -->", () => `<script>${probe}</script>`),
      },
    },
  ],
});
```

with the marker comment in `index.html`'s `<head>`, before every other script:

```html
<head>
  <meta charset="UTF-8" />
  <!-- webview-guard:floor-probe -->
  <script type="module" src="/src/main.ts"></script>
</head>
```

::: warning Park the bundle, or check the verdict
A Vite module script is _not_ parked by default, and runs on Chromium 61–110
anyway — park it, or have your entry return early when
`document.documentElement.dataset.webviewFloor === 'below'`.
:::

## How it is checked

Verified on Chromium 53, 109, 113 and 134, where its verdict agreed with the
native one on every arm ([the table](/verification/#the-native-verdict-and-the-floor-probe-agree-on-every-arm));
the parser test (`acorn`, `ecmaVersion: 5`) keeps it ES5.
