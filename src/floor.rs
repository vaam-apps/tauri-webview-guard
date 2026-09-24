//! The floor, and the one comparison that decides whether an engine clears it.
//!
//! Everything here is pure: no Tauri, no JNI, no UIKit. It is the part of the
//! guard that can be wrong in a way no device test would notice until a real
//! user on exactly the boundary version hit it, so it is the part that is
//! unit-tested hardest — including the boundary itself on both sides.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use specta::Type;

/// Tailwind v4's Chromium floor. `color-mix()` shipped in Chromium 111; below
/// it every alpha-reduced colour token collapses to full strength, and below
/// Chromium 99 (`@layer`) the whole stylesheet is discarded.
pub const DEFAULT_MIN_CHROMIUM: u32 = 111;

/// Tailwind v4's Safari floor. `@property` shipped in Safari 16.4, and WKWebView
/// ships with the OS, so on iOS the engine floor *is* an OS floor.
pub const DEFAULT_MIN_IOS: OsVersion = OsVersion::new(16, 4, 0);

/// Whether the running engine clears the configured floor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    /// At or above the floor. The app boots normally.
    Supported,
    /// Below the floor. The guard blocks the WebView and shows the native dialog.
    BelowFloor,
    /// The engine's version could not be read or parsed.
    ///
    /// The guard **lets the app boot** in this case and says so in the log and
    /// in [`crate::GuardStatus`]. Blocking would put an "update your WebView"
    /// dialog in front of a device that may be perfectly current — telling a
    /// user to fix something that is not broken is the same dead end as the
    /// white screen this plugin exists to replace.
    Unknown,
}

/// A dotted `major.minor.patch` operating-system version.
///
/// Missing components read as zero, so `"17"` is `17.0.0` and `"16.4"` is
/// `16.4.0` — which is what makes `16.4` compare equal to the `16.4.0` that
/// `ProcessInfo.operatingSystemVersion` reports.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OsVersion {
    /// Major component.
    pub major: u32,
    /// Minor component.
    pub minor: u32,
    /// Patch component.
    pub patch: u32,
}

impl OsVersion {
    /// Build a version from its three components.
    pub const fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }
}

impl fmt::Display for OsVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.patch == 0 {
            write!(f, "{}.{}", self.major, self.minor)
        } else {
            write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
        }
    }
}

/// Why a version string was refused.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("\"{0}\" is not a dotted numeric version such as 16.4 or 16.4.1")]
pub struct ParseVersionError(pub String);

impl FromStr for OsVersion {
    type Err = ParseVersionError;

    /// Strict: one to three dot-separated non-negative integers, nothing else.
    ///
    /// Strict because this parses *configuration* (`min_ios("16.4")`) as well
    /// as the device's own report, and a floor of `"16.4-beta"` silently read
    /// as `16.4` would be a typo nobody finds.
    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let err = || ParseVersionError(raw.to_string());
        let mut parts = raw.trim().split('.');
        let mut next = |required: bool| -> Result<u32, ParseVersionError> {
            match parts.next() {
                Some(p) if !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()) => {
                    p.parse().map_err(|_| err())
                }
                None if !required => Ok(0),
                _ => Err(err()),
            }
        };
        let version = OsVersion::new(next(true)?, next(false)?, next(false)?);
        if parts.next().is_some() {
            return Err(err());
        }
        Ok(version)
    }
}

/// The Chromium major version out of an Android WebView provider's
/// `versionName`, e.g. `109` from `"109.0.5414.123"`.
///
/// Returns `None` for anything that does not start with a run of digits
/// followed by a dot or the end of the string. Provider version names are
/// Chromium version strings on every provider observed (Android System WebView,
/// Chrome/Trichrome), but an OEM build is free to put anything there, and
/// `None` routes it to [`Verdict::Unknown`] rather than to a guess.
pub fn chromium_major(version_name: &str) -> Option<u32> {
    let trimmed = version_name.trim();
    let digits_end = trimmed
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(trimmed.len());
    if digits_end == 0 {
        return None;
    }
    match trimmed[digits_end..].chars().next() {
        None | Some('.') => trimmed[..digits_end].parse().ok(),
        Some(_) => None,
    }
}

/// The configured floor for both engines.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Floor {
    /// Minimum Chromium major, checked on Android.
    pub min_chromium: u32,
    /// Minimum iOS version, checked on iOS.
    pub min_ios: OsVersion,
}

impl Default for Floor {
    fn default() -> Self {
        Self {
            min_chromium: DEFAULT_MIN_CHROMIUM,
            min_ios: DEFAULT_MIN_IOS,
        }
    }
}

impl Floor {
    /// Judge an Android WebView provider by its `versionName`.
    ///
    /// `None` — no provider, or a provider with no version name — is
    /// [`Verdict::Unknown`].
    pub fn judge_chromium(&self, version_name: Option<&str>) -> Verdict {
        match version_name.and_then(chromium_major) {
            Some(major) if major >= self.min_chromium => Verdict::Supported,
            Some(_) => Verdict::BelowFloor,
            None => Verdict::Unknown,
        }
    }

    /// Judge iOS by the operating-system version string the device reports.
    pub fn judge_ios(&self, os_version: Option<&str>) -> Verdict {
        match os_version.map(str::parse::<OsVersion>) {
            Some(Ok(v)) if v >= self.min_ios => Verdict::Supported,
            Some(Ok(_)) => Verdict::BelowFloor,
            Some(Err(_)) | None => Verdict::Unknown,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn floor() -> Floor {
        Floor::default()
    }

    #[test]
    fn the_default_floor_is_tailwind_v4s() {
        assert_eq!(DEFAULT_MIN_CHROMIUM, 111);
        assert_eq!(DEFAULT_MIN_IOS, OsVersion::new(16, 4, 0));
    }

    /// The four emulator arms this plugin was verified on, by the exact
    /// `versionName` each one's provider reported.
    #[test]
    fn the_four_verified_arms() {
        assert_eq!(
            floor().judge_chromium(Some("53.0.2785.124")),
            Verdict::BelowFloor
        );
        assert_eq!(
            floor().judge_chromium(Some("109.0.5414.123")),
            Verdict::BelowFloor
        );
        assert_eq!(
            floor().judge_chromium(Some("113.0.5672.136")),
            Verdict::Supported
        );
        assert_eq!(
            floor().judge_chromium(Some("134.0.6998.135")),
            Verdict::Supported
        );
    }

    /// The boundary itself, on both sides. This is the test a `>=` → `>`
    /// mutation, or a floor hardcoded one off, turns red.
    #[test]
    fn chromium_boundary_is_inclusive() {
        assert_eq!(
            floor().judge_chromium(Some("110.0.5481.153")),
            Verdict::BelowFloor
        );
        assert_eq!(
            floor().judge_chromium(Some("111.0.5563.116")),
            Verdict::Supported
        );
        assert_eq!(
            floor().judge_chromium(Some("112.0.5615.136")),
            Verdict::Supported
        );
    }

    /// The configured floor is the one used — not the default.
    #[test]
    fn chromium_floor_is_configurable() {
        let strict = Floor {
            min_chromium: 120,
            ..Floor::default()
        };
        assert_eq!(
            strict.judge_chromium(Some("119.0.6045.194")),
            Verdict::BelowFloor
        );
        assert_eq!(
            strict.judge_chromium(Some("120.0.6099.230")),
            Verdict::Supported
        );

        let lax = Floor {
            min_chromium: 50,
            ..Floor::default()
        };
        assert_eq!(
            lax.judge_chromium(Some("53.0.2785.124")),
            Verdict::Supported
        );
    }

    #[test]
    fn unreadable_chromium_versions_are_unknown_not_guessed() {
        for raw in [
            None,
            Some(""),
            Some("   "),
            Some("beta"),
            Some("v113.0"),
            Some("113a.0"),
        ] {
            assert_eq!(floor().judge_chromium(raw), Verdict::Unknown, "{raw:?}");
        }
    }

    #[test]
    fn chromium_major_parses_what_providers_report() {
        assert_eq!(chromium_major("109.0.5414.123"), Some(109));
        assert_eq!(chromium_major("134"), Some(134));
        assert_eq!(chromium_major(" 120.0.6099.230 "), Some(120));
        assert_eq!(
            chromium_major("99999999999.0"),
            None,
            "overflow is not a version"
        );
    }

    #[test]
    fn ios_boundary_is_inclusive_and_minor_aware() {
        assert_eq!(floor().judge_ios(Some("16.3.1")), Verdict::BelowFloor);
        assert_eq!(floor().judge_ios(Some("16.3")), Verdict::BelowFloor);
        assert_eq!(floor().judge_ios(Some("16.4")), Verdict::Supported);
        assert_eq!(floor().judge_ios(Some("16.4.0")), Verdict::Supported);
        assert_eq!(floor().judge_ios(Some("16.4.1")), Verdict::Supported);
        assert_eq!(floor().judge_ios(Some("17")), Verdict::Supported);
        assert_eq!(floor().judge_ios(Some("15.8")), Verdict::BelowFloor);
        // Lexical comparison would call this below 16.4.
        assert_eq!(floor().judge_ios(Some("16.10")), Verdict::Supported);
    }

    #[test]
    fn ios_floor_is_configurable() {
        let strict = Floor {
            min_ios: OsVersion::new(17, 0, 0),
            ..Floor::default()
        };
        assert_eq!(strict.judge_ios(Some("16.7.10")), Verdict::BelowFloor);
        assert_eq!(strict.judge_ios(Some("17.0")), Verdict::Supported);
    }

    #[test]
    fn unreadable_ios_versions_are_unknown() {
        for raw in [
            None,
            Some(""),
            Some("sixteen"),
            Some("16.4-beta"),
            Some("16..4"),
        ] {
            assert_eq!(floor().judge_ios(raw), Verdict::Unknown, "{raw:?}");
        }
    }

    #[test]
    fn os_version_parsing_is_strict() {
        assert_eq!("16.4".parse(), Ok(OsVersion::new(16, 4, 0)));
        assert_eq!("16.4.1".parse(), Ok(OsVersion::new(16, 4, 1)));
        assert_eq!("17".parse(), Ok(OsVersion::new(17, 0, 0)));
        for bad in ["", "16.", ".4", "16.4.1.2", "16,4", "-16.4", "16.4 beta"] {
            assert!(bad.parse::<OsVersion>().is_err(), "{bad:?} parsed");
        }
    }

    #[test]
    fn os_version_displays_without_a_zero_patch() {
        assert_eq!(OsVersion::new(16, 4, 0).to_string(), "16.4");
        assert_eq!(OsVersion::new(16, 4, 1).to_string(), "16.4.1");
    }
}
