//! Refuse to boot a Tauri v2 app into a WebView engine below its CSS floor,
//! and tell the user — in a **native** dialog — how to fix it.
//!
//! Tailwind v4's floor is Chromium 111 / Safari 16.4. Below it an app does not
//! crash; it renders subtly wrong (Chromium 99–110: every `color-mix()` colour
//! at full strength) or not at all (below 99: every `@layer` block discarded,
//! a white screen). Either way the user concludes the app is broken, when the
//! fix on Android is two taps in the Play Store.
//!
//! The check runs natively during plugin setup, which Tauri performs before it
//! creates the first WebView. Below the floor the WebView is stopped before it
//! can paint, and a platform dialog — never HTML, which is exactly what a
//! below-floor engine cannot be trusted to render — names the out-of-date
//! component and links to it.
//!
//! ```no_run
//! # // A real app passes `tauri::generate_context!()`, which needs a
//! # // tauri.conf.json this crate does not have.
//! # let context = tauri::test::mock_context(tauri::test::noop_assets());
//! tauri::Builder::default()
//!     .plugin(
//!         tauri_plugin_webview_guard::Builder::new()
//!             .min_chromium(111)
//!             .min_ios("16.4")
//!             .build(),
//!     )
//!     .run(context)
//!     .expect("error while running tauri application");
//! ```
//!
//! The guard is live the moment it is registered. There is no switch to turn
//! it on; to guard nothing, do not register it.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use tauri::{
    plugin::{Builder as PluginBuilder, TauriPlugin},
    Manager, Runtime,
};

mod assess;
mod commands;
mod copy;
mod error;
mod floor;
mod models;

#[cfg(desktop)]
mod desktop;
#[cfg(mobile)]
mod mobile;

pub use copy::{AndroidCopy, IosCopy};
pub use error::{Error, Result, WebviewGuardError};
pub use floor::{
    chromium_major, Floor, OsVersion, ParseVersionError, Verdict, DEFAULT_MIN_CHROMIUM,
    DEFAULT_MIN_IOS,
};
pub use models::{EngineKind, GuardStatus, Platform, StoreTarget, UpdateOpened};

/// The platform's guard, as returned by [`WebviewGuardExt::webview_guard`].
#[cfg(desktop)]
pub use desktop::WebviewGuard;
#[cfg(mobile)]
pub use mobile::WebviewGuard;

/// The plugin's name: the `webview-guard` in `plugin:webview-guard|status` and
/// in the `webview-guard:default` capability.
pub const PLUGIN_NAME: &str = "webview-guard";

/// Reach the guard from any [`Manager`] — an `AppHandle`, a `Window`, the `App`.
pub trait WebviewGuardExt<R: Runtime> {
    /// The guard's backend for this platform.
    fn webview_guard(&self) -> &WebviewGuard<R>;
}

impl<R: Runtime, T: Manager<R>> WebviewGuardExt<R> for T {
    fn webview_guard(&self) -> &WebviewGuard<R> {
        self.state::<WebviewGuard<R>>().inner()
    }
}

/// Everything the guard needs at setup, moved into the setup closure.
#[derive(Debug, Clone)]
pub(crate) struct GuardConfig {
    pub floor: Floor,
    // Read only by the backend for the platform being compiled.
    #[cfg_attr(not(target_os = "android"), allow(dead_code))]
    pub android: AndroidCopy,
    #[cfg_attr(not(target_os = "ios"), allow(dead_code))]
    pub ios: IosCopy,
}

/// Configures and builds the plugin.
///
/// Every option has a default, so `Builder::new().build()` is a complete,
/// working guard at Tailwind v4's floor with English copy.
#[derive(Debug, Clone)]
#[must_use]
pub struct Builder {
    min_chromium: u32,
    min_ios: String,
    android: AndroidCopy,
    ios: IosCopy,
}

impl Default for Builder {
    fn default() -> Self {
        Self {
            min_chromium: DEFAULT_MIN_CHROMIUM,
            min_ios: DEFAULT_MIN_IOS.to_string(),
            android: AndroidCopy::default(),
            ios: IosCopy::default(),
        }
    }
}

impl Builder {
    /// A guard at the default floor: Chromium 111, iOS 16.4.
    pub fn new() -> Self {
        Self::default()
    }

    /// The minimum Chromium **major** version on Android. Default `111`.
    ///
    /// Compared against the major of the WebView provider's `versionName`,
    /// inclusively: `min_chromium(111)` lets 111 through and blocks 110.
    pub fn min_chromium(mut self, major: u32) -> Self {
        self.min_chromium = major;
        self
    }

    /// The minimum iOS version, as `"major.minor"` or `"major.minor.patch"`.
    /// Default `"16.4"`. Inclusive.
    ///
    /// Parsed when [`Self::build`] runs; see its panics section.
    pub fn min_ios(mut self, version: impl Into<String>) -> Self {
        self.min_ios = version.into();
        self
    }

    /// Replace the Android dialog's text. See [`AndroidCopy`] for placeholders.
    pub fn android_copy(mut self, copy: AndroidCopy) -> Self {
        self.android = copy;
        self
    }

    /// Replace the iOS alert's text. See [`IosCopy`] for placeholders.
    pub fn ios_copy(mut self, copy: IosCopy) -> Self {
        self.ios = copy;
        self
    }

    /// Validate the configuration into a [`Floor`] and the copy, without
    /// building a plugin. [`Self::build`] calls this; it is public so a test
    /// can check an app's configuration without a Tauri runtime.
    pub fn floor(&self) -> std::result::Result<Floor, ParseVersionError> {
        Ok(Floor {
            min_chromium: self.min_chromium,
            min_ios: self.min_ios.parse()?,
        })
    }

    /// Build the plugin.
    ///
    /// # Panics
    ///
    /// If the [`Self::min_ios`] string is not a dotted numeric version. That is
    /// a constant in the app's own source, wrong on every launch or on none,
    /// so it fails the first `tauri dev` rather than shipping a guard that
    /// silently compares against a floor nobody chose.
    pub fn build<R: Runtime>(self) -> TauriPlugin<R> {
        let floor = self
            .floor()
            .unwrap_or_else(|e| panic!("tauri-plugin-webview-guard: invalid min_ios: {e}"));
        let config = GuardConfig {
            floor,
            android: self.android,
            ios: self.ios,
        };
        let specta = specta_builder::<R>();
        // Set during setup, read on every navigation. Setup runs before any
        // WebView exists, so no navigation can observe the initial `false`
        // on a device that is going to be blocked.
        let blocked = Arc::new(AtomicBool::new(false));
        let blocked_at_setup = Arc::clone(&blocked);
        PluginBuilder::new(PLUGIN_NAME)
            .invoke_handler(specta.invoke_handler())
            .setup(move |app, api| {
                #[cfg(mobile)]
                let guard = mobile::init(app, api, &config)?;
                #[cfg(desktop)]
                let guard = desktop::init(app, api, &config);
                if let Ok(status) = guard.status() {
                    blocked_at_setup.store(status.verdict == Verdict::BelowFloor, Ordering::SeqCst);
                }
                app.manage(guard);
                Ok(())
            })
            // The Rust half of the block, and the one that holds on both
            // platforms regardless of native timing: Tauri consults every
            // plugin's navigation hook before the WebView loads a URL — on
            // Android through `RustWebView.loadUrl`'s `shouldOverride`, which
            // is how the app's first `loadUrl` reaches the page.
            .on_navigation(move |_webview, url| {
                let allowed = navigation_allowed(blocked.load(Ordering::SeqCst), url.as_str());
                if !allowed {
                    log::info!("webview-guard: refused navigation to {url}");
                }
                allowed
            })
            .build()
    }
}

/// Whether the WebView may navigate to `url`.
///
/// Everything is allowed until the guard has blocked. After that only
/// `about:blank` is — the empty document the native side swaps in so that
/// nothing of the app's bundle is ever fetched or parsed by an engine that
/// cannot be trusted with it.
fn navigation_allowed(blocked: bool, url: &str) -> bool {
    !blocked || url == "about:blank"
}

/// A guard at the default floor with the default copy.
/// Equivalent to `Builder::new().build()`.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new().build()
}

/// The tauri-specta builder for this plugin's commands.
///
/// The same builder produces the invoke handler registered in
/// [`Builder::build`] and the TypeScript bindings in `guest-js/bindings.ts`
/// (exported by `tests/bindings.rs`), so the two cannot describe different
/// commands.
///
/// The `::<tauri::Wry>` turbofish is for *type collection only*: the macro
/// strips it from the invoke handler it builds, which stays generic over `R`,
/// and a command's TypeScript signature does not depend on the runtime.
pub fn specta_builder<R: Runtime>() -> tauri_specta::Builder<R> {
    tauri_specta::Builder::<R>::new()
        .plugin_name(PLUGIN_NAME)
        .commands(tauri_specta::collect_commands![
            commands::status::<tauri::Wry>,
            commands::open_update::<tauri::Wry>,
        ])
        .typ::<Verdict>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nothing_but_about_blank_navigates_once_blocked() {
        assert!(!navigation_allowed(true, "http://tauri.localhost/"));
        assert!(!navigation_allowed(true, "tauri://localhost"));
        assert!(!navigation_allowed(true, "https://example.com/"));
        assert!(navigation_allowed(true, "about:blank"));
    }

    #[test]
    fn everything_navigates_when_not_blocked() {
        assert!(navigation_allowed(false, "http://tauri.localhost/"));
        assert!(navigation_allowed(false, "about:blank"));
    }

    #[test]
    fn builder_defaults_are_tailwind_v4s_floor() {
        assert_eq!(Builder::new().floor().unwrap(), Floor::default());
    }

    #[test]
    fn builder_options_reach_the_floor() {
        let floor = Builder::new()
            .min_chromium(120)
            .min_ios("17.2")
            .floor()
            .unwrap();
        assert_eq!(floor.min_chromium, 120);
        assert_eq!(floor.min_ios, OsVersion::new(17, 2, 0));
    }

    #[test]
    fn a_malformed_ios_floor_is_refused() {
        assert!(Builder::new().min_ios("16.4-beta").floor().is_err());
    }

    #[test]
    #[should_panic(expected = "invalid min_ios")]
    fn build_panics_on_a_malformed_ios_floor() {
        let _ = Builder::new()
            .min_ios("sixteen")
            .build::<tauri::test::MockRuntime>();
    }
}
