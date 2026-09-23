//! The example's Tauri setup.
//!
//! Registers the guard exactly as an app would, plus two things only an
//! example wants: a page-load trace (so a device run shows whether the app's
//! page ever started loading behind the guard), and a compile-time override of
//! the Chromium floor, used to build the *diagnostic* variant that lets a
//! below-floor engine through on purpose — the only way to observe what Tauri's
//! IPC and the ES5 floor probe do on such an engine.

use tauri::webview::PageLoadEvent;

/// `WEBVIEW_GUARD_EXAMPLE_MIN_CHROMIUM=1 npm run tauri android build …` builds
/// the diagnostic variant. Unset, it is the plugin default (111).
fn min_chromium() -> u32 {
    option_env!("WEBVIEW_GUARD_EXAMPLE_MIN_CHROMIUM")
        .and_then(|v| v.parse().ok())
        .unwrap_or(tauri_plugin_webview_guard::DEFAULT_MIN_CHROMIUM)
}

/// Forces the iOS block on a current simulator — no iOS 16.3 runtime needed to
/// see the alert. Read from the *launch* environment, because a compile-time
/// variable does not survive the Xcode build phase that invokes cargo:
///
/// ```sh
/// SIMCTL_CHILD_WEBVIEW_GUARD_EXAMPLE_MIN_IOS=99.0 xcrun simctl launch <udid> app.vaam.webviewguard.example
/// ```
///
/// Unset, it is the plugin default (16.4).
fn min_ios() -> String {
    std::env::var("WEBVIEW_GUARD_EXAMPLE_MIN_IOS")
        .unwrap_or_else(|_| tauri_plugin_webview_guard::DEFAULT_MIN_IOS.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(target_os = "android")]
    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Info)
            .with_tag("WebviewGuardExample"),
    );
    #[cfg(target_os = "ios")]
    let _ = oslog::OsLogger::new("app.vaam.webviewguard.example")
        .level_filter(log::LevelFilter::Info)
        .init();

    log::info!(
        "example: registering the guard with min_chromium={} min_ios={}",
        min_chromium(),
        min_ios()
    );

    tauri::Builder::default()
        .plugin(
            tauri_plugin_webview_guard::Builder::new()
                .min_chromium(min_chromium())
                .min_ios(min_ios())
                .build(),
        )
        .on_page_load(|webview, payload| {
            let phase = match payload.event() {
                PageLoadEvent::Started => "started",
                PageLoadEvent::Finished => "finished",
            };
            log::info!(
                "example: page load {phase}: {} ({})",
                payload.url(),
                webview.label()
            );
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
