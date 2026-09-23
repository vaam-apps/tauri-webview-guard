package app.vaam.webviewguard

import org.junit.Assert.assertEquals
import org.junit.Test

class StoreLinkTest {
    @Test
    fun marketLinkTargetsTheReportedPackage() {
        assertEquals("market://details?id=com.android.chrome", StoreLink.marketUri("com.android.chrome"))
    }

    @Test
    fun webFallbackTargetsTheSamePackage() {
        assertEquals(
            "https://play.google.com/store/apps/details?id=com.google.android.webview",
            StoreLink.webUri("com.google.android.webview"),
        )
    }

    /** The wire tags must match Rust's `StoreTarget`, which serializes snake_case. */
    @Test
    fun wireTagsMatchTheRustEnum() {
        assertEquals(listOf("market", "web"), StoreTarget.values().map { it.wire })
    }
}
