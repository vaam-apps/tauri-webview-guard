package app.vaam.webviewguard

import android.content.ActivityNotFoundException
import android.content.Context
import android.content.Intent
import android.net.Uri

/** Where a store link actually landed. The wire tags match Rust's `StoreTarget`. */
internal enum class StoreTarget(val wire: String) {
    MARKET("market"),
    WEB("web"),
}

internal object StoreLink {
    fun marketUri(pkg: String): String = "market://details?id=$pkg"

    fun webUri(pkg: String): String = "https://play.google.com/store/apps/details?id=$pkg"

    /**
     * Open [pkg]'s store listing: the Play Store app if anything resolves
     * `market:`, otherwise the Play website in a browser.
     *
     * Fired and caught rather than resolved in advance, so no `<queries>`
     * manifest entry is needed on Android 11+. Returns `null` when neither
     * intent has a handler — a device with no store and no browser.
     */
    fun open(context: Context, pkg: String): StoreTarget? {
        for ((target, uri) in listOf(StoreTarget.MARKET to marketUri(pkg), StoreTarget.WEB to webUri(pkg))) {
            try {
                context.startActivity(
                    Intent(Intent.ACTION_VIEW, Uri.parse(uri)).addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
                )
                return target
            } catch (e: ActivityNotFoundException) {
                android.util.Log.i(TAG, "no handler for $uri")
            }
        }
        return null
    }
}
