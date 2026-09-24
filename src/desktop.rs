//! Desktop: compiled so the app runs in `tauri dev`, and refuses every call.
//!
//! The desktop engines are WebView2 (Windows, Chromium, evergreen), WKWebView
//! on macOS (versioned with Safari, which updates independently of the OS on
//! older macOS releases) and WebKitGTK (versioned by the distribution). None
//! of them is probed here, so every call returns [`Error::Unsupported`] — a
//! typed refusal the frontend can match on — rather than a status that would
//! claim a verdict nobody computed. The guard also says so once, at setup,
//! through the `log` facade.

use std::marker::PhantomData;

use serde::de::DeserializeOwned;
use tauri::{plugin::PluginApi, AppHandle, Runtime};

use crate::error::Error;
use crate::models::{GuardStatus, UpdateOpened};
use crate::GuardConfig;

const REASON: &str = "the webview guard checks the Android WebView provider and the iOS version; \
                      desktop engines are not probed";

pub(crate) fn init<R: Runtime, C: DeserializeOwned>(
    _app: &AppHandle<R>,
    _api: PluginApi<R, C>,
    config: &GuardConfig,
) -> WebviewGuard<R> {
    log::warn!(
        "webview-guard: {} is not guarded (floor would be Chromium {} / iOS {}); {REASON}",
        std::env::consts::OS,
        config.floor.min_chromium,
        config.floor.min_ios,
    );
    WebviewGuard(PhantomData)
}

/// The desktop guard. Every method returns [`Error::Unsupported`].
pub struct WebviewGuard<R: Runtime>(PhantomData<fn() -> R>);

impl<R: Runtime> WebviewGuard<R> {
    /// Always [`Error::Unsupported`] on desktop.
    pub fn status(&self) -> crate::Result<GuardStatus> {
        Err(Error::unsupported("status", REASON))
    }

    /// Always [`Error::Unsupported`] on desktop.
    pub fn open_update(&self) -> crate::Result<UpdateOpened> {
        Err(Error::unsupported("open_update", REASON))
    }
}
