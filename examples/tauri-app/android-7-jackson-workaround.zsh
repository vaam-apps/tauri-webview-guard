#!/bin/zsh
# Makes the generated Android project start on Android 7.x (API 24/25).
#
# WHY: stock Tauri (checked on tauri 2.11.6) crashes in TauriActivity.onCreate on
# API 24/25, before ANY plugin — this guard included — is constructed:
#   java.lang.NoClassDefFoundError: Failed resolution of: Ljava/lang/BootstrapMethodError;
#     at com.fasterxml.jackson.databind.ObjectMapper.<clinit>
#     at app.tauri.plugin.PluginManager.<clinit>
# tauri-android depends on jackson-databind 2.15.3, whose Android baseline is
# API 26. Upstream: https://github.com/tauri-apps/tauri/issues/8788 (open), where
# this exact workaround — force jackson-databind 2.12.7.1, enable core library
# desugaring — is reported working. Provenance and re-check notes: upstream.json.
#
# The example applies it only so the guard can be shown on the Chromium 53 arm
# (an API 24 image). It is NOT something the plugin does to your app: forcing a
# different version of a Tauri dependency is your decision, and on minSdk >= 26
# it is unnecessary.
#
# gen/android is generated (and gitignored), so this re-applies after every
# `tauri android init`. Idempotent.
set -eu
gradle=${0:A:h}/src-tauri/gen/android/app/build.gradle.kts
marker='// webview-guard example: android-7-jackson-workaround'
if grep -qF "$marker" "$gradle"; then
  echo "already applied: $gradle"
  exit 0
fi
cat >> "$gradle" <<GRADLE

$marker (see upstream.json, tauri-apps/tauri#8788)
android {
    compileOptions {
        isCoreLibraryDesugaringEnabled = true
    }
}
dependencies {
    coreLibraryDesugaring("com.android.tools:desugar_jdk_libs:2.0.4")
}
configurations.all {
    resolutionStrategy {
        force("com.fasterxml.jackson.core:jackson-databind:2.12.7.1")
        force("com.fasterxml.jackson.core:jackson-core:2.12.7")
        force("com.fasterxml.jackson.core:jackson-annotations:2.12.7")
    }
}
GRADLE
echo "applied: $gradle"
