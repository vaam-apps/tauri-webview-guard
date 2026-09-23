//! Why a guard call failed.
//!
//! Serialized as a tagged object — `{"kind": "unsupported", ...}` — rather than
//! a string, and exported to TypeScript through tauri-specta, so a frontend can
//! `switch (e.kind)` instead of matching on message text. The distinction that
//! matters most is [`Error::Unsupported`]: a caller must be able to tell "this
//! platform has no such capability" apart from "the capability exists and just
//! failed", because the first is a fact to design around and the second is a
//! bug to report.

use serde::Serialize;
use specta::Type;

/// A failed guard call.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error, Serialize, Type)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum WebviewGuardError {
    /// This platform does not implement the capability at all.
    ///
    /// Returned on every desktop target for every call, and on iOS for
    /// [`crate::WebviewGuard::open_update`]. Never returned on a platform that
    /// does implement the call.
    #[error("`{capability}` is not supported on {platform}: {reason}")]
    Unsupported {
        /// The platform the call ran on, e.g. `ios`, `macos`, `windows`.
        platform: String,
        /// The call that was refused, e.g. `status`, `open_update`.
        capability: String,
        /// Why, in a sentence a developer can act on.
        reason: String,
    },

    /// Neither the Play Store app nor any browser could open the provider's
    /// store listing. Android only.
    #[error("no app on this device can open the store listing for {package}")]
    NoStoreHandler {
        /// The provider package whose listing was requested.
        package: String,
    },

    /// The native side failed in a way it did not classify.
    #[error("the native plugin failed on `{command}`: {message}")]
    Native {
        /// The native command that failed.
        command: String,
        /// The native side's own message.
        message: String,
    },
}

/// The crate's error type, under its conventional Rust name.
///
/// The enum itself is named [`WebviewGuardError`] because tauri-specta exports
/// a type under its Rust name, and a TypeScript type called `Error` shadows the
/// global `Error` in every module that imports it — including the generated
/// bindings' own `typedError` helper, which tests `e instanceof Error`.
pub type Error = WebviewGuardError;

impl WebviewGuardError {
    /// An [`Error::Unsupported`] for the platform this binary was compiled for.
    pub(crate) fn unsupported(capability: &str, reason: &str) -> Self {
        Self::Unsupported {
            platform: std::env::consts::OS.to_string(),
            capability: capability.to_string(),
            reason: reason.to_string(),
        }
    }
}

/// Convenience alias for the crate's fallible operations.
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unsupported_is_tagged_and_distinguishable_on_the_wire() {
        let e = Error::Unsupported {
            platform: "macos".into(),
            capability: "status".into(),
            reason: "not probed".into(),
        };
        assert_eq!(
            serde_json::to_value(&e).unwrap(),
            serde_json::json!({
                "kind": "unsupported",
                "platform": "macos",
                "capability": "status",
                "reason": "not probed",
            })
        );
    }

    #[test]
    fn every_kind_has_its_own_tag() {
        let kinds: Vec<String> = [
            Error::unsupported("status", "x"),
            Error::NoStoreHandler {
                package: "com.google.android.webview".into(),
            },
            Error::Native {
                command: "engine".into(),
                message: "boom".into(),
            },
        ]
        .iter()
        .map(|e| {
            serde_json::to_value(e).unwrap()["kind"]
                .as_str()
                .unwrap()
                .to_string()
        })
        .collect();
        assert_eq!(kinds, ["unsupported", "no_store_handler", "native"]);
    }
}
