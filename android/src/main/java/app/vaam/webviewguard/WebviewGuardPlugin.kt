package app.vaam.webviewguard

import android.app.Activity
import android.content.DialogInterface
import android.os.SystemClock
import android.util.Log
import android.view.View
import android.webkit.WebView
import androidx.appcompat.app.AlertDialog
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin

internal const val TAG = "WebviewGuard"

/** The rendered dialog, exactly as Rust composed it. */
@InvokeArg
internal class BlockArgs {
    lateinit var title: String
    lateinit var message: String
    lateinit var updateButton: String
    lateinit var closeButton: String

    /** `null` when the device reported no provider: the dialog then has no Update button. */
    var providerPackage: String? = null
}

@InvokeArg
internal class OpenStoreArgs {
    lateinit var providerPackage: String
}

/**
 * The native half of the guard.
 *
 * The order of events on Android, all on the main thread (each step is logged
 * under the `WebviewGuard` tag with `SystemClock.uptimeMillis`, so a device run
 * shows it rather than asserting it):
 *
 * 1. Tauri constructs this class during plugin setup, inside `Builder::build`,
 *    before `App::run` creates any window.
 * 2. Rust calls [engine], then — only below the floor — [block], which records
 *    the verdict and raises the dialog. No WebView exists yet.
 * 3. Tauri creates the WebView, attaches it with `setContentView`, and calls
 *    [load] — all inside one main-thread message (wry `main_pipe.rs`), so no
 *    frame is drawn in between. The app's first `loadUrl` is *not* in that
 *    message: wry's `loadUrlMainThread` posts it to the main looper, so it runs
 *    after [load] returns. (Measured, not assumed: a first version that only
 *    blanked the WebView inside [load] watched `about:blank` load and then the
 *    app's own `http://tauri.localhost/` load right after it.)
 * 4. So [load] makes the WebView unable to show or run the app *before* that
 *    posted load arrives — JavaScript off, view hidden — and the Rust side's
 *    `on_navigation` hook refuses the posted load itself when it reaches
 *    `RustWebView.loadUrl`'s `shouldOverride` check. A second [neutralize],
 *    posted behind the pending load, stops it in case that hook ever lets it
 *    through (it does when the WebView is not yet registered with Tauri's
 *    manager, which is a race Tauri does not order).
 *
 * What the user sees before step 2 is the activity's window background (the
 * launch theme); the WebView itself never paints.
 */
@TauriPlugin
class WebviewGuardPlugin(private val activity: Activity) : Plugin(activity) {

    /** Set by [block]. Read by [load], which may run before or after it. */
    private var block: BlockArgs? = null
    private var webView: WebView? = null
    private var dialog: AlertDialog? = null

    init {
        mark("constructed")
    }

    @Command
    fun engine(invoke: Invoke) {
        val provider = WebViewProvider.current(activity)
        mark(
            "engine: package=${provider.packageName} versionName=${provider.versionName} " +
                "label=${provider.label} sdk=${android.os.Build.VERSION.SDK_INT}"
        )
        invoke.resolve(
            JSObject().apply {
                put("package", provider.packageName)
                put("versionName", provider.versionName)
                put("label", provider.label)
            }
        )
    }

    @Command
    fun block(invoke: Invoke) {
        val args = invoke.parseArgs(BlockArgs::class.java)
        activity.runOnUiThread {
            block = args
            mark("block: webViewAlreadyCreated=${webView != null}")
            webView?.let(::neutralize)
            showDialog(args)
            invoke.resolve()
        }
    }

    @Command
    fun openStore(invoke: Invoke) {
        val args = invoke.parseArgs(OpenStoreArgs::class.java)
        activity.runOnUiThread {
            when (val target = StoreLink.open(activity, args.providerPackage)) {
                null -> invoke.reject(args.providerPackage, "no_store_handler")
                else -> invoke.resolve(
                    JSObject().apply {
                        put("target", target.wire)
                        put("package", args.providerPackage)
                    }
                )
            }
        }
    }

    override fun load(webView: WebView) {
        this.webView = webView
        mark("load: webview created, blocked=${block != null}, pendingUrl=${webView.url}")
        if (block != null) {
            neutralize(webView)
            // FIFO behind the load wry has already posted: runs after it.
            webView.post { neutralize(webView) }
        }
    }

    /**
     * Stop the WebView from ever running the app.
     *
     * JavaScript off first, so that even a page that does commit cannot run a
     * line of the app's bundle — on a below-floor engine that bundle is exactly
     * what cannot be trusted. Then hidden, so not even a blank page paints
     * under the dialog. Then any navigation in flight is stopped and replaced
     * with `about:blank`, the one URL the Rust navigation hook still allows.
     */
    private fun neutralize(webView: WebView) {
        webView.settings.javaScriptEnabled = false
        webView.visibility = View.INVISIBLE
        webView.stopLoading()
        webView.loadUrl("about:blank")
        mark("neutralized: js off, hidden, stopped, url=${webView.url}")
    }

    private fun showDialog(args: BlockArgs) {
        if (dialog?.isShowing == true) return
        val builder = AlertDialog.Builder(activity)
            .setTitle(args.title)
            .setMessage(args.message)
            .setCancelable(false)
            .setNegativeButton(args.closeButton) { _, _ -> activity.finishAndRemoveTask() }
        val pkg = args.providerPackage
        if (pkg != null) builder.setPositiveButton(args.updateButton, null)

        val d = builder.create()
        d.setCanceledOnTouchOutside(false)
        if (pkg != null) {
            // Replace the positive button's listener after show(): the default
            // one dismisses the dialog, and a user who comes back from the Play
            // Store without updating must find the same dialog, not the blank
            // app behind it.
            d.setOnShowListener {
                d.getButton(DialogInterface.BUTTON_POSITIVE).setOnClickListener {
                    val target = StoreLink.open(activity, pkg)
                    mark("update tapped: opened=${target?.wire ?: "nothing"}")
                }
            }
        }
        d.show()
        dialog = d
        mark("dialog shown")
    }

    private fun mark(event: String) {
        Log.i(
            TAG,
            "t=${SystemClock.uptimeMillis()} thread=${Thread.currentThread().name} $event"
        )
    }
}
