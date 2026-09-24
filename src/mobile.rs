//! The native path: Kotlin on Android, Swift on iOS.
//!
//! Runs inside plugin setup, which Tauri performs in `Builder::build` — before
//! `App::run` creates the first window and its WebView. So by the time the
//! WebView exists, the native side already knows whether to let it navigate.
//! The sequence, and the source lines behind each step, are in
//! `docs/lifecycle.md`.
//!
//! What this module owns is the order of three calls and what a failure of
//! each one means:
//!
//! 1. **register** the native plugin. Failure here is a build defect (the
//!    class was stripped, the Swift package was not linked), identical on every
//!    device, so it fails setup loudly.
//! 2. **engine** — the native side reports the engine version. Failure here is
//!    a runtime surprise on one device; the guard logs it and lets the app boot
//!    ([`Verdict::Unknown`]), because an unreadable version is not evidence of
//!    an old engine.
//! 3. **block**, only below the floor — the native side stops the WebView and
//!    shows the dialog. If *that* fails the log is all that is left; there is
//!    no second channel to the user that would be more reliable than the one
//!    that just failed. Independently of the native side, the plugin's
//!    `on_navigation` hook (see `lib.rs`) refuses every navigation but
//!    `about:blank` from then on.

use serde::de::DeserializeOwned;
use serde::Serialize;
use tauri::{
    plugin::{mobile::PluginInvokeError, PluginApi, PluginHandle},
    AppHandle, Runtime,
};

use crate::error::Error;
use crate::floor::Verdict;
use crate::models::{GuardStatus, UpdateOpened};
use crate::GuardConfig;

#[cfg(target_os = "android")]
const PLUGIN_IDENTIFIER: &str = "app.vaam.webviewguard";

#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(init_plugin_webview_guard);

/// The mobile guard, held in Tauri's managed state.
pub struct WebviewGuard<R: Runtime> {
    handle: PluginHandle<R>,
    status: GuardStatus,
}

pub(crate) fn init<R: Runtime, C: DeserializeOwned>(
    _app: &AppHandle<R>,
    api: PluginApi<R, C>,
    config: &GuardConfig,
) -> crate::Result<WebviewGuard<R>> {
    #[cfg(target_os = "android")]
    let handle = api
        .register_android_plugin(PLUGIN_IDENTIFIER, "WebviewGuardPlugin")
        .map_err(|e| native_error("register", e))?;
    #[cfg(target_os = "ios")]
    let handle = api
        .register_ios_plugin(init_plugin_webview_guard)
        .map_err(|e| native_error("register", e))?;

    #[cfg(target_os = "android")]
    let (status, block) = {
        let report = call(&handle, "engine", ()).unwrap_or_else(|e| {
            log::error!("webview-guard: could not read the WebView provider: {e}");
            Default::default()
        });
        crate::assess::assess_android(&config.floor, &config.android, &report)
    };
    #[cfg(target_os = "ios")]
    let (status, block) = {
        let report = call(&handle, "engine", ()).unwrap_or_else(|e| {
            log::error!("webview-guard: could not read the iOS version: {e}");
            Default::default()
        });
        crate::assess::assess_ios(&config.floor, &config.ios, &report)
    };

    match status.verdict {
        Verdict::Supported => log::info!(
            "webview-guard: {:?} {} clears the floor {}",
            status.engine,
            status.installed.as_deref().unwrap_or("?"),
            status.required,
        ),
        Verdict::Unknown => log::warn!(
            "webview-guard: engine version unreadable ({:?}); letting the app boot",
            status.installed,
        ),
        Verdict::BelowFloor => log::warn!(
            "webview-guard: {:?} {} is below the floor {}; blocking the WebView",
            status.engine,
            status.installed.as_deref().unwrap_or("?"),
            status.required,
        ),
    }

    if let Some(block) = block {
        if let Err(e) = call::<()>(&handle, "block", block) {
            log::error!("webview-guard: the native side failed to block: {e}");
        }
    }

    Ok(WebviewGuard { handle, status })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(not(target_os = "android"), allow(dead_code))]
struct OpenStoreArgs<'a> {
    provider_package: &'a str,
}

impl<R: Runtime> WebviewGuard<R> {
    /// What the guard found at boot, and what it decided.
    pub fn status(&self) -> crate::Result<GuardStatus> {
        Ok(self.status.clone())
    }

    /// Open the store listing of the package actually providing the WebView.
    ///
    /// Android: `market://details?id=<provider>`, falling back to
    /// `https://play.google.com/store/apps/details?id=<provider>` when nothing
    /// resolves the `market:` scheme. iOS: [`Error::Unsupported`] — WKWebView
    /// updates with the OS and has no store listing.
    pub fn open_update(&self) -> crate::Result<UpdateOpened> {
        #[cfg(target_os = "android")]
        {
            let package = self.status.provider_package.as_deref().ok_or_else(|| {
                Error::unsupported(
                    "open_update",
                    "the device reported no WebView provider package, so there is no listing to open",
                )
            })?;
            call(
                &self.handle,
                "openStore",
                OpenStoreArgs {
                    provider_package: package,
                },
            )
        }
        #[cfg(target_os = "ios")]
        {
            let _ = &self.handle;
            Err(Error::unsupported(
                "open_update",
                "WKWebView ships with iOS and has no store listing; the fix is an iOS update",
            ))
        }
    }
}

/// One native call, with a rejection turned back into a typed [`Error`].
///
/// The native sides reject `openStore` with the code `no_store_handler` when
/// neither the Play Store nor a browser can take the intent; that is the one
/// rejection with its own variant, because a caller can act on it.
fn call<T: DeserializeOwned>(
    handle: &PluginHandle<impl Runtime>,
    command: &str,
    args: impl Serialize,
) -> crate::Result<T> {
    handle
        .run_mobile_plugin::<T>(command, args)
        .map_err(|e| native_error(command, e))
}

fn native_error(command: &str, e: PluginInvokeError) -> Error {
    match e {
        PluginInvokeError::InvokeRejected(response)
            if response.code.as_deref() == Some("no_store_handler") =>
        {
            Error::NoStoreHandler {
                package: response.message.unwrap_or_default(),
            }
        }
        PluginInvokeError::InvokeRejected(response) => Error::Native {
            command: command.to_string(),
            message: response
                .message
                .unwrap_or_else(|| "rejected without a message".into()),
        },
        other => Error::Native {
            command: command.to_string(),
            message: other.to_string(),
        },
    }
}
