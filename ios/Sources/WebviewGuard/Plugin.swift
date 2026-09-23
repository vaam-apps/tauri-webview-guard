import Foundation
import SwiftRs
import Tauri
import UIKit
import WebKit

/// The rendered alert, exactly as Rust composed it.
struct BlockArgs: Decodable {
  let title: String
  let message: String
}

struct EngineReport: Encodable {
  /// `major.minor.patch`, from `ProcessInfo` — which, unlike `UIDevice`, is
  /// safe to read off the main thread, where Tauri dispatches plugin commands.
  let osVersion: String
}

/// The native half of the guard on iOS.
///
/// Order of events (logged with `NSLog` under `[WebviewGuard]`):
///
/// 1. Tauri constructs this class during plugin setup, inside `Builder::build`,
///    before `App::run` creates any window — so no WKWebView exists yet.
/// 2. Rust calls `engine`, then — only below the floor — `block`. `block`
///    records the verdict **synchronously**, before resolving, because Rust's
///    setup is blocked on that resolve: by the time Tauri goes on to create the
///    WebView, the verdict is already visible to `load(webview:)`.
/// 3. Tauri builds the WKWebView — which issues its first `load` inside the
///    builder — and then calls `load(webview:)` through `with_webview`, a
///    message posted to the main run loop. That gap is the one place the page
///    could start before the guard sees the view; `load(webview:)` stops it,
///    replaces it with an empty document and hides the view.
///
/// There are no buttons on the alert. WKWebView updates only with iOS, iOS has
/// no public URL that opens Software Update, and an app is not supposed to
/// quit itself — so any button would be a dead end.
class WebviewGuardPlugin: Plugin {
  private let lock = NSLock()
  private var blockArgs: BlockArgs?
  private weak var webview: WKWebView?
  private var presented = false

  override init() {
    super.init()
    Self.mark("constructed")
  }

  @objc public func engine(_ invoke: Invoke) {
    let v = ProcessInfo.processInfo.operatingSystemVersion
    let report = EngineReport(osVersion: "\(v.majorVersion).\(v.minorVersion).\(v.patchVersion)")
    Self.mark("engine: osVersion=\(report.osVersion)")
    invoke.resolve(report)
  }

  @objc public func block(_ invoke: Invoke) throws {
    let args = try invoke.parseArgs(BlockArgs.self)
    lock.lock()
    blockArgs = args
    lock.unlock()
    Self.mark("block: recorded")
    DispatchQueue.main.async {
      if let webview = self.webview { self.neutralize(webview) }
      self.presentWhenReady(attempt: 0)
    }
    invoke.resolve()
  }

  override func load(webview: WKWebView) {
    self.webview = webview
    lock.lock()
    let blocked = blockArgs != nil
    lock.unlock()
    Self.mark("load: webview created, blocked=\(blocked), url=\(webview.url?.absoluteString ?? "nil")")
    if blocked {
      neutralize(webview)
      presentWhenReady(attempt: 0)
    }
  }

  private func neutralize(_ webview: WKWebView) {
    webview.stopLoading()
    webview.loadHTMLString("", baseURL: nil)
    webview.isHidden = true
    Self.mark("neutralized: webview stopped, blanked and hidden")
  }

  /// Present once the root view controller is actually in a window.
  ///
  /// `load(webview:)` can run before the controller's view is attached, and
  /// `present` on a detached controller logs a warning and does nothing — the
  /// guard would have blocked the WebView and then shown the user nothing at
  /// all, which is worse than the white screen it replaces. So poll the run
  /// loop briefly until there is a window to present in.
  private func presentWhenReady(attempt: Int) {
    lock.lock()
    let args = blockArgs
    lock.unlock()
    guard let args = args, !presented else { return }
    guard let controller = manager.viewController, controller.view.window != nil else {
      if attempt < 200 {
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.025) {
          self.presentWhenReady(attempt: attempt + 1)
        }
      } else {
        Self.mark("gave up waiting for a window to present in")
      }
      return
    }
    presented = true
    let alert = UIAlertController(title: args.title, message: args.message, preferredStyle: .alert)
    controller.present(alert, animated: false)
    Self.mark("alert presented after \(attempt) wait(s)")
  }

  private static func mark(_ event: String) {
    NSLog("[WebviewGuard] t=%.3f %@", ProcessInfo.processInfo.systemUptime, event)
  }
}

@_cdecl("init_plugin_webview_guard")
func initPlugin() -> Plugin {
  return WebviewGuardPlugin()
}
