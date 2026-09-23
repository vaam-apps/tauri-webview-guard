//! The IPC surface: two read-mostly commands, both typed end to end through
//! tauri-specta.
//!
//! Neither is needed for the guard to work — the guard acts natively during
//! setup, before any JavaScript can run. They exist for an app that wants to
//! *show* the engine (an About screen, a support dump) or to offer the same
//! update link as a soft nudge while still above the floor.
//!
//! Both are `async` so they run on Tauri's async runtime rather than the main
//! thread: `open_update` starts an Activity on Android, which must be posted to
//! the main thread, and a synchronous command would be holding it.

use tauri::{command, AppHandle, Runtime};

use crate::models::{GuardStatus, UpdateOpened};
use crate::WebviewGuardExt;

/// What the guard found at boot, and what it decided.
#[command]
#[specta::specta]
pub(crate) async fn status<R: Runtime>(app: AppHandle<R>) -> crate::Result<GuardStatus> {
    app.webview_guard().status()
}

/// Open the WebView provider's store listing. Android only.
#[command]
#[specta::specta]
pub(crate) async fn open_update<R: Runtime>(app: AppHandle<R>) -> crate::Result<UpdateOpened> {
    app.webview_guard().open_update()
}
