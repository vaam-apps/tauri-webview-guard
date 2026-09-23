//! What the guard reports. Exported to TypeScript through tauri-specta.

use serde::Serialize;
use specta::Type;

use crate::floor::Verdict;

/// The mobile platform the guard ran on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum Platform {
    /// Android: the engine is whichever Chromium the WebView provider ships.
    Android,
    /// iOS: the engine is WKWebView, which ships with the OS.
    Ios,
}

/// The engine family behind the app's WebView.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum EngineKind {
    /// Android System WebView, Chrome/Trichrome, or an OEM Chromium build.
    Chromium,
    /// WKWebView.
    Webkit,
}

/// What the guard found at boot, and what it decided.
///
/// Computed once, during plugin setup — before the first WebView exists — and
/// never recomputed. A process keeps the WebView implementation it loaded, so
/// a provider updated mid-session does not change what this process renders
/// with; the next launch re-probes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct GuardStatus {
    /// Which platform this is.
    pub platform: Platform,
    /// Which engine family.
    pub engine: EngineKind,
    /// The version the device reported, verbatim: the provider's `versionName`
    /// on Android (e.g. `109.0.5414.123`), the OS version on iOS (e.g. `16.3.1`).
    /// `None` when the device reported nothing.
    pub installed: Option<String>,
    /// The configured floor, as shown to the user: `111` or `16.4`.
    pub required: String,
    /// Android only: the package actually providing the WebView, as
    /// `WebViewCompat.getCurrentWebViewPackage` returned it. Never assumed.
    pub provider_package: Option<String>,
    /// Android only: that package's user-visible label, e.g.
    /// "Android System WebView". This is the name the dialog uses.
    pub provider_label: Option<String>,
    /// The decision.
    pub verdict: Verdict,
}

/// Which destination [`crate::WebviewGuard::open_update`] actually reached.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, serde::Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum StoreTarget {
    /// `market://details?id=…` resolved: the Play Store app opened.
    Market,
    /// No Play Store app resolved `market://`, so the
    /// `https://play.google.com/store/apps/details?id=…` fallback opened in a
    /// browser instead.
    Web,
}

/// The result of [`crate::WebviewGuard::open_update`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, serde::Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct UpdateOpened {
    /// Where the user was sent.
    pub target: StoreTarget,
    /// The provider package whose listing was opened.
    pub package: String,
}
