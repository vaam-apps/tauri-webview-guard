/**
 * The frontend surface of `tauri-plugin-webview-guard`.
 *
 * You do not need any of this for the guard to work. The guard acts natively,
 * during plugin setup, before any page exists — on a below-floor engine no
 * JavaScript of yours ever runs. What this package offers is:
 *
 * - `commands.status()` / `commands.openUpdate()`: typed end to end from the
 *   Rust signatures by tauri-specta (`bindings.ts` is generated; never edit
 *   it). Both resolve to `{ status: "ok", data }` or
 *   `{ status: "error", error }`, where `error.kind` is `"unsupported"` on
 *   every desktop target and, for `openUpdate`, on iOS.
 * - `FLOOR_PROBE_PATH`: the path of the ES5 floor probe inside this package,
 *   for inlining into `index.html` in a browser build. See the README.
 */

export * from './bindings'

/**
 * The floor probe's path relative to this package's root. Resolve it with
 * `require.resolve('tauri-plugin-webview-guard-api/probe/floor-probe.js')` in
 * a build script and inline the file's text in `<head>`. It must be inlined,
 * not imported: an import is part of the bundle, and the bundle is exactly
 * what a below-floor engine may fail to parse.
 */
export const FLOOR_PROBE_PATH = 'probe/floor-probe.js'
