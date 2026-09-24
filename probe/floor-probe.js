/*
 * tauri-plugin-webview-guard — the floor probe, for builds with no native layer.
 *
 * ES5 ONLY, and inline in <head>, before every other script. On an engine
 * below the floor the app's own bundle may not parse (Chromium 53 has no `?.`,
 * no `??`, no `async`), so a check that lives in the bundle never runs on the
 * devices it exists for. This file is checked for ES5 by `npm test`.
 *
 * It tests the two features Tailwind v4's floor is actually made of, not a
 * version string:
 *
 *   color-mix()   Chromium 111, Safari 16.2, Firefox 113
 *   @property     Chromium 85,  Safari 16.4, Firefox 128
 *
 * The floor (Chromium 111 / Safari 16.4 / Firefox 128) is the union of those
 * two rows, so the gate is their conjunction: `color-mix()` alone would wave
 * iOS 16.2 and 16.3 through. `CSS.registerProperty` stands in for `@property`
 * because `CSS.supports` cannot test an at-rule, and the JS API and the at-rule
 * shipped together in every engine.
 *
 * What it does:
 *
 *   - always: sets <html data-webview-floor="supported|below"> and
 *     window.__WEBVIEW_GUARD_FLOOR__ = { supported, colorMix,
 *     registerProperty, computedColorMix }.
 *   - supported: promotes every <script type="text/plain"
 *     data-webview-guard-src="..."> to <script type="module" src="...">, so a
 *     bundle parked that way is never handed to an engine that cannot parse it.
 *   - below: on DOMContentLoaded, replaces <body> with the content of
 *     <template id="webview-guard-below-floor">, or with a plain English
 *     message if the page has no such template.
 *
 * In a Tauri app on Android or iOS you do not need this: the plugin's native
 * guard refuses to let the WebView load at all. This is for the plain browser
 * build, and as a cross-check.
 */
(function () {
  var doc = document;
  var root = doc.documentElement;

  var result = {
    supported: false,
    colorMix: false,
    registerProperty: false,
    computedColorMix: ''
  };

  try {
    result.colorMix = !!(window.CSS && window.CSS.supports &&
      window.CSS.supports('color', 'color-mix(in oklab, red, blue)'));
  } catch (e) { result.colorMix = false; }

  try {
    result.registerProperty = !!(window.CSS &&
      typeof window.CSS.registerProperty === 'function');
  } catch (e) { result.registerProperty = false; }

  // Diagnostic only, never part of the verdict: what the engine actually
  // computes for a colour-mix declaration. An engine that drops the
  // declaration keeps the #123456 fallback and reports rgb(18, 52, 86).
  try {
    var el = doc.createElement('div');
    el.style.cssText = 'background-color:#123456;' +
      'background-color:color-mix(in oklab, #ff0000 50%, #0000ff)';
    root.appendChild(el);
    result.computedColorMix = window.getComputedStyle(el).backgroundColor;
    root.removeChild(el);
  } catch (e) { result.computedColorMix = 'error: ' + e; }

  result.supported = result.colorMix && result.registerProperty;
  window.__WEBVIEW_GUARD_FLOOR__ = result;
  root.setAttribute('data-webview-floor', result.supported ? 'supported' : 'below');

  if (result.supported) {
    var parked = doc.querySelectorAll('script[type="text/plain"][data-webview-guard-src]');
    for (var i = 0; i < parked.length; i++) {
      var s = doc.createElement('script');
      s.type = 'module';
      s.src = parked[i].getAttribute('data-webview-guard-src');
      if (parked[i].hasAttribute('crossorigin')) {
        s.setAttribute('crossorigin', parked[i].getAttribute('crossorigin'));
      }
      parked[i].parentNode.insertBefore(s, parked[i].nextSibling);
    }
    return;
  }

  function renderBelowFloor() {
    var template = doc.getElementById('webview-guard-below-floor');
    var body = doc.body;
    if (!body) { return; }
    if (template) {
      body.innerHTML = template.innerHTML;
      return;
    }
    // Pre-2016 CSS only, inline: the design system is exactly what this
    // engine cannot render.
    body.setAttribute('style', 'margin:0;padding:32px 24px;background:#ffffff;' +
      'color:#111111;font-family:-apple-system,Roboto,Helvetica,Arial,sans-serif;' +
      'font-size:17px;line-height:1.5;text-align:center');
    body.innerHTML = '<div style="max-width:420px;margin:15vh auto 0">' +
      '<p style="font-size:21px;font-weight:bold;margin:0 0 16px">Update your browser</p>' +
      '<p style="margin:0">This browser is too old to display this app correctly. ' +
      'Update it, or open the app in a current browser.</p></div>';
  }

  if (doc.readyState === 'loading') {
    doc.addEventListener('DOMContentLoaded', renderBelowFloor);
  } else {
    renderBelowFloor();
  }
})();
