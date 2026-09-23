//! From "what the device reported" to "what the guard does about it".
//!
//! Compiled on every target, although only the mobile backends call it, so the
//! whole decision — verdict, status, and the exact dialog text — is unit-tested
//! by a plain `cargo test` on a laptop. The native sides then do only two
//! things they cannot avoid doing natively: report the engine, and draw the
//! dialog they are handed. Each platform's half is dead code on the other
//! targets, hence the per-item `allow`s — narrower than a module-wide one, so
//! a genuinely unused item on its own platform still fails clippy.

use serde::{Deserialize, Serialize};

use crate::copy::{AndroidCopy, IosCopy, Placeholders};
use crate::floor::{Floor, Verdict};
use crate::models::{EngineKind, GuardStatus, Platform};

/// The name used in the dialog when the provider reports a package but no
/// label. It is what the stock provider calls itself.
#[cfg_attr(not(target_os = "android"), allow(dead_code))]
pub(crate) const FALLBACK_ANDROID_COMPONENT: &str = "Android System WebView";

/// What the Android side reports from `WebViewCompat.getCurrentWebViewPackage`.
#[cfg_attr(not(target_os = "android"), allow(dead_code))]
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AndroidEngineReport {
    pub package: Option<String>,
    pub version_name: Option<String>,
    pub label: Option<String>,
}

/// What the iOS side reports from `ProcessInfo.operatingSystemVersion`.
#[cfg_attr(not(target_os = "ios"), allow(dead_code))]
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct IosEngineReport {
    pub os_version: Option<String>,
}

/// The arguments of the Android `block` command: the rendered dialog, and the
/// package the Update button deep-links to.
#[cfg_attr(not(target_os = "android"), allow(dead_code))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AndroidBlock {
    pub title: String,
    pub message: String,
    pub update_button: String,
    pub close_button: String,
    /// `None` only if the device reported no provider package at all, in which
    /// case the dialog has no Update button — there is nothing to link to.
    /// Named `providerPackage` on the wire because `package` is a Kotlin
    /// keyword, and a backticked field is one more thing to get wrong.
    pub provider_package: Option<String>,
}

/// The arguments of the iOS `block` command.
#[cfg_attr(not(target_os = "ios"), allow(dead_code))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct IosBlock {
    pub title: String,
    pub message: String,
}

#[cfg_attr(not(target_os = "android"), allow(dead_code))]
pub(crate) fn assess_android(
    floor: &Floor,
    copy: &AndroidCopy,
    report: &AndroidEngineReport,
) -> (GuardStatus, Option<AndroidBlock>) {
    let verdict = floor.judge_chromium(report.version_name.as_deref());
    let required = floor.min_chromium.to_string();
    let status = GuardStatus {
        platform: Platform::Android,
        engine: EngineKind::Chromium,
        installed: report.version_name.clone(),
        required: required.clone(),
        provider_package: report.package.clone(),
        provider_label: report.label.clone(),
        verdict,
    };
    let block = (verdict == Verdict::BelowFloor).then(|| {
        let component = report
            .label
            .as_deref()
            .filter(|l| !l.trim().is_empty())
            .unwrap_or(FALLBACK_ANDROID_COMPONENT);
        let rendered = copy.render(Placeholders {
            component,
            installed: report.version_name.as_deref().unwrap_or("?"),
            required: &required,
        });
        AndroidBlock {
            title: rendered.title,
            message: rendered.message,
            update_button: rendered.update_button,
            close_button: rendered.close_button,
            provider_package: report.package.clone(),
        }
    });
    (status, block)
}

#[cfg_attr(not(target_os = "ios"), allow(dead_code))]
pub(crate) fn assess_ios(
    floor: &Floor,
    copy: &IosCopy,
    report: &IosEngineReport,
) -> (GuardStatus, Option<IosBlock>) {
    let verdict = floor.judge_ios(report.os_version.as_deref());
    let required = floor.min_ios.to_string();
    let status = GuardStatus {
        platform: Platform::Ios,
        engine: EngineKind::Webkit,
        installed: report.os_version.clone(),
        required: required.clone(),
        provider_package: None,
        provider_label: None,
        verdict,
    };
    let block = (verdict == Verdict::BelowFloor).then(|| {
        let rendered = copy.render(Placeholders {
            component: "iOS",
            installed: report.os_version.as_deref().unwrap_or("?"),
            required: &required,
        });
        IosBlock {
            title: rendered.title,
            message: rendered.message,
        }
    });
    (status, block)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::floor::OsVersion;

    fn android(version: &str) -> AndroidEngineReport {
        AndroidEngineReport {
            package: Some("com.google.android.webview".into()),
            version_name: Some(version.into()),
            label: Some("Android System WebView".into()),
        }
    }

    #[test]
    fn below_floor_blocks_with_the_reported_provider() {
        let (status, block) = assess_android(
            &Floor::default(),
            &AndroidCopy::default(),
            &android("109.0.5414.123"),
        );
        assert_eq!(status.verdict, Verdict::BelowFloor);
        assert_eq!(status.required, "111");
        let block = block.expect("a below-floor engine must be blocked");
        assert_eq!(
            block.provider_package.as_deref(),
            Some("com.google.android.webview")
        );
        assert_eq!(block.title, "Update Android System WebView");
        assert!(block.message.contains("version 111 or newer"));
        assert!(block.message.contains("109.0.5414.123"));
    }

    #[test]
    fn at_or_above_floor_never_blocks() {
        for v in ["111.0.5563.116", "113.0.5672.136", "134.0.6998.135"] {
            let (status, block) =
                assess_android(&Floor::default(), &AndroidCopy::default(), &android(v));
            assert_eq!(status.verdict, Verdict::Supported, "{v}");
            assert!(block.is_none(), "{v} was blocked");
        }
    }

    #[test]
    fn unknown_never_blocks() {
        let report = AndroidEngineReport::default();
        let (status, block) = assess_android(&Floor::default(), &AndroidCopy::default(), &report);
        assert_eq!(status.verdict, Verdict::Unknown);
        assert!(block.is_none());
    }

    /// The package is whatever the device reported — a Trichrome device gets
    /// sent to Chrome's listing, not to a hardcoded WebView one.
    #[test]
    fn the_package_and_label_are_never_hardcoded() {
        let report = AndroidEngineReport {
            package: Some("com.android.chrome".into()),
            version_name: Some("100.0.4896.127".into()),
            label: Some("Chrome".into()),
        };
        let (status, block) = assess_android(&Floor::default(), &AndroidCopy::default(), &report);
        let block = block.unwrap();
        assert_eq!(
            block.provider_package.as_deref(),
            Some("com.android.chrome")
        );
        assert_eq!(block.title, "Update Chrome");
        assert_eq!(
            status.provider_package.as_deref(),
            Some("com.android.chrome")
        );
    }

    #[test]
    fn a_blank_label_falls_back_to_the_stock_name() {
        let report = AndroidEngineReport {
            label: Some("  ".into()),
            ..android("90.0.4430.91")
        };
        let (_, block) = assess_android(&Floor::default(), &AndroidCopy::default(), &report);
        assert_eq!(block.unwrap().title, "Update Android System WebView");
    }

    #[test]
    fn ios_blocks_below_and_says_update_ios() {
        let (status, block) = assess_ios(
            &Floor::default(),
            &IosCopy::default(),
            &IosEngineReport {
                os_version: Some("16.3.1".into()),
            },
        );
        assert_eq!(status.verdict, Verdict::BelowFloor);
        assert_eq!(status.engine, EngineKind::Webkit);
        let block = block.unwrap();
        assert_eq!(block.title, "Update iOS to continue");
        assert!(block.message.contains("iOS 16.4 or later"));
        assert!(block.message.contains("iOS 16.3.1"));
    }

    #[test]
    fn ios_uses_the_configured_floor() {
        let floor = Floor {
            min_ios: OsVersion::new(99, 0, 0),
            ..Floor::default()
        };
        let (status, block) = assess_ios(
            &floor,
            &IosCopy::default(),
            &IosEngineReport {
                os_version: Some("26.4".into()),
            },
        );
        assert_eq!(status.verdict, Verdict::BelowFloor);
        assert_eq!(status.required, "99.0");
        assert!(block.is_some());
    }

    #[test]
    fn block_arguments_are_camel_case_on_the_wire() {
        let (_, block) = assess_android(
            &Floor::default(),
            &AndroidCopy::default(),
            &android("53.0.2785.124"),
        );
        let wire = serde_json::to_value(block.unwrap()).unwrap();
        for key in [
            "title",
            "message",
            "updateButton",
            "closeButton",
            "providerPackage",
        ] {
            assert!(wire.get(key).is_some(), "missing {key}: {wire}");
        }
    }
}
