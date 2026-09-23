package app.vaam.webviewguard

import android.content.Context
import androidx.webkit.WebViewCompat

/**
 * The package actually providing this device's WebView.
 *
 * Never a constant. On Android 7–9 it is typically Android System WebView
 * (`com.google.android.webview`); on 10+ many devices use Chrome's Trichrome
 * library; some OEMs ship their own. Whatever [WebViewCompat] reports is what
 * the dialog names and what the Update button deep-links to.
 */
internal data class WebViewProvider(
    val packageName: String?,
    val versionName: String?,
    val label: String?,
) {
    companion object {
        fun current(context: Context): WebViewProvider {
            val info = try {
                WebViewCompat.getCurrentWebViewPackage(context)
            } catch (e: Exception) {
                // Reflection into WebViewFactory below API 26 can throw on an
                // OEM build that renamed it. An unreadable provider is not an
                // old one; Rust turns this into Verdict.Unknown.
                android.util.Log.w(TAG, "getCurrentWebViewPackage threw", e)
                null
            }
            val label = info?.applicationInfo?.let {
                try {
                    context.packageManager.getApplicationLabel(it).toString()
                } catch (e: Exception) {
                    null
                }
            }
            return WebViewProvider(info?.packageName, info?.versionName, label)
        }
    }
}
