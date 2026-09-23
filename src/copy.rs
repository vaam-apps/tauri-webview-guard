//! What the native dialog says.
//!
//! Configuration, not code: an app ships its own language by passing its own
//! strings to [`crate::Builder::android_copy`] / [`crate::Builder::ios_copy`].
//! The defaults are English.
//!
//! The one rule every string here follows, and every replacement should too:
//! **name the component that is actually out of date, and never tell the user
//! to update the app.** The app is current. "Update this app" sends a user to a
//! store listing with nothing to install, and they conclude the app is broken —
//! the exact outcome the guard exists to prevent.
//!
//! Three placeholders are substituted before the strings reach the native side:
//!
//! | placeholder | Android | iOS |
//! |---|---|---|
//! | `{component}` | the provider's own label, e.g. "Android System WebView" | `iOS` |
//! | `{installed}` | the provider's `versionName`, e.g. `109.0.5414.123` | e.g. `16.3.1` |
//! | `{required}` | the configured Chromium major, e.g. `111` | e.g. `16.4` |
//!
//! Rendering happens in Rust, once, so Kotlin and Swift show exactly the text
//! the unit tests below assert on — there is no second template engine.

/// The Android dialog's text. All four strings are required; an empty button
/// label would render as an untappable blank.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AndroidCopy {
    /// Dialog title.
    pub title: String,
    /// Dialog body.
    pub message: String,
    /// The positive button. Opens the provider's Play Store listing.
    pub update_button: String,
    /// The negative button. Closes the app.
    pub close_button: String,
}

impl Default for AndroidCopy {
    fn default() -> Self {
        Self {
            title: "Update {component}".into(),
            message: "This app needs {component} version {required} or newer to display \
                      correctly. This phone has version {installed}.\n\n\
                      The app itself is up to date. Update {component} from the Play Store, \
                      then open the app again."
                .into(),
            update_button: "Open Play Store".into(),
            close_button: "Close app".into(),
        }
    }
}

/// The iOS alert's text.
///
/// There are no buttons. WKWebView ships with the OS, so the only fix is an iOS
/// update, and iOS offers no public URL that opens Software Update — the
/// `openSettingsURLString` destination is this app's own settings page, which
/// would be a dead end dressed up as a way forward. Apps are also not meant to
/// quit themselves on iOS, so there is no "Close" either.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IosCopy {
    /// Alert title.
    pub title: String,
    /// Alert body.
    pub message: String,
}

impl Default for IosCopy {
    fn default() -> Self {
        Self {
            title: "Update iOS to continue".into(),
            message:
                "This app needs iOS {required} or later. This iPhone runs iOS {installed}.\n\n\
                      The app itself is up to date. Update iOS in Settings › General › \
                      Software Update, then open the app again."
                    .into(),
        }
    }
}

/// The values substituted into the placeholders.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Placeholders<'a> {
    pub component: &'a str,
    pub installed: &'a str,
    pub required: &'a str,
}

pub(crate) fn render(template: &str, p: Placeholders<'_>) -> String {
    template
        .replace("{component}", p.component)
        .replace("{installed}", p.installed)
        .replace("{required}", p.required)
}

impl AndroidCopy {
    #[cfg_attr(not(target_os = "android"), allow(dead_code))]
    pub(crate) fn render(&self, p: Placeholders<'_>) -> AndroidCopy {
        AndroidCopy {
            title: render(&self.title, p),
            message: render(&self.message, p),
            update_button: render(&self.update_button, p),
            close_button: render(&self.close_button, p),
        }
    }
}

impl IosCopy {
    #[cfg_attr(not(target_os = "ios"), allow(dead_code))]
    pub(crate) fn render(&self, p: Placeholders<'_>) -> IosCopy {
        IosCopy {
            title: render(&self.title, p),
            message: render(&self.message, p),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const WEBVIEW: Placeholders<'static> = Placeholders {
        component: "Android System WebView",
        installed: "109.0.5414.123",
        required: "111",
    };

    #[test]
    fn android_default_names_the_component_and_never_the_app() {
        let c = AndroidCopy::default().render(WEBVIEW);
        assert_eq!(c.title, "Update Android System WebView");
        assert!(c
            .message
            .contains("Android System WebView version 111 or newer"));
        assert!(c.message.contains("This phone has version 109.0.5414.123."));
        assert!(c.message.contains("The app itself is up to date."));
        assert!(!c.message.to_lowercase().contains("update this app"));
        assert!(!c.message.to_lowercase().contains("update the app"));
        assert!(
            !c.message.contains('{'),
            "unrendered placeholder: {}",
            c.message
        );
    }

    #[test]
    fn android_default_follows_whatever_provider_the_device_reports() {
        let c = AndroidCopy::default().render(Placeholders {
            component: "Chrome",
            ..WEBVIEW
        });
        assert_eq!(c.title, "Update Chrome");
        assert!(c.message.contains("Update Chrome from the Play Store"));
    }

    /// The default buttons must not look like Android's own
    /// "built for an older version of Android" warning, which has a single OK.
    #[test]
    fn android_default_buttons_are_distinctive() {
        let c = AndroidCopy::default();
        assert_eq!(c.update_button, "Open Play Store");
        assert_eq!(c.close_button, "Close app");
        assert_ne!(c.update_button, "OK");
    }

    #[test]
    fn ios_default_says_update_ios() {
        let c = IosCopy::default().render(Placeholders {
            component: "iOS",
            installed: "16.3.1",
            required: "16.4",
        });
        assert_eq!(c.title, "Update iOS to continue");
        assert!(c
            .message
            .starts_with("This app needs iOS 16.4 or later. This iPhone runs iOS 16.3.1."));
        assert!(c.message.contains("Settings › General › Software Update"));
        assert!(!c.message.to_lowercase().contains("update the app"));
    }

    #[test]
    fn custom_copy_is_rendered_with_the_same_placeholders() {
        let fr = AndroidCopy {
            title: "Mettre à jour {component}".into(),
            message: "{component} {installed} < {required}".into(),
            update_button: "Ouvrir le Play Store".into(),
            close_button: "Fermer".into(),
        }
        .render(WEBVIEW);
        assert_eq!(fr.title, "Mettre à jour Android System WebView");
        assert_eq!(fr.message, "Android System WebView 109.0.5414.123 < 111");
    }
}
