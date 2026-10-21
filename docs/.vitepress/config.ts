import { cpSync } from "node:fs";
import { resolve } from "node:path";
import { defineMd3Config } from "@vaam-apps/vitepress-md3/config";

// Served from GitHub Pages at /tauri-webview-guard/. The docs workflow passes
// the path Pages reports through DOCS_BASE (empty on pull-request builds,
// hence `||`, not `??`).
const base = process.env.DOCS_BASE || "/tauri-webview-guard/";
const repo = "https://github.com/vaam-apps/tauri-webview-guard";

export default defineMd3Config({
  base,
  lang: "en",
  title: "webview-guard",
  titleTemplate: ":title · tauri-plugin-webview-guard",
  description:
    "Refuse to boot a Tauri v2 app into a below-floor WebView engine, and say how to fix it — natively, before first paint.",
  cleanUrls: true,
  lastUpdated: true,
  // A link to a page that does not exist fails the build.
  ignoreDeadLinks: false,
  head: [["link", { rel: "icon", type: "image/svg+xml", href: `${base}logo.svg` }]],

  // The device-verification notes live at docs/verification/README.md so
  // GitHub shows them when browsing the folder; serve them at /verification/.
  rewrites: {
    "verification/README.md": "verification/index.md",
  },

  // verification/README.md links its screenshots as plain links
  // (`[dialog](screenshots/cr53-default.png)`), which Vite does not bundle as
  // it does images. Ship the folder next to the page so those links resolve.
  buildEnd(site) {
    cpSync(
      resolve(site.srcDir, "verification/screenshots"),
      resolve(site.outDir, "verification/screenshots"),
      { recursive: true },
    );
  },

  markdown: {
    // lifecycle.md draws its sequences as ```mermaid fences. Hand each one to
    // the <Mermaid> component (theme/Mermaid.vue), which renders it in the
    // browser; every other fence renders as code, as usual.
    config(md) {
      const fence = md.renderer.rules.fence;
      md.renderer.rules.fence = (tokens, idx, options, env, self) => {
        const token = tokens[idx];
        if (token?.info.trim() === "mermaid") {
          return `<Mermaid code="${encodeURIComponent(token.content)}" />`;
        }
        return fence ? fence(tokens, idx, options, env, self) : self.renderToken(tokens, idx, options);
      };
    },
  },

  themeConfig: {
    md3: {
      style: "expressive",
      seed: "#c2185b",
    },

    logo: "/logo.svg",

    nav: [
      { text: "Guide", link: "/guide/getting-started", activeMatch: "^/guide/" },
      { text: "Platforms", link: "/platforms/", activeMatch: "^/platforms/" },
      { text: "How it works", link: "/lifecycle" },
      { text: "Verification", link: "/verification/" },
      {
        text: "More",
        items: [
          { text: "Upstream dependencies", link: "/reference/upstream" },
          { text: "Changelog", link: "/reference/changelog" },
          { text: "All vaam-apps docs", link: "https://vaam-apps.github.io/vitepress-md3/projects/" },
        ],
      },
    ],

    sidebar: [
      {
        text: "Introduction",
        items: [
          { text: "Overview", link: "/" },
          { text: "Why it exists", link: "/guide/why" },
        ],
      },
      {
        text: "Guide",
        items: [
          { text: "Getting started", link: "/guide/getting-started" },
          { text: "Configuration", link: "/guide/configuration" },
          { text: "API", link: "/guide/api" },
          { text: "Browser builds: the floor probe", link: "/guide/floor-probe" },
        ],
      },
      {
        text: "Platforms",
        items: [
          { text: "Support matrix", link: "/platforms/" },
          { text: "Android", link: "/platforms/android" },
          { text: "iOS", link: "/platforms/ios" },
          { text: "Desktop", link: "/platforms/desktop" },
        ],
      },
      {
        text: "How it works",
        items: [
          { text: "Lifecycle", link: "/lifecycle" },
          { text: "Device verification", link: "/verification/" },
        ],
      },
      {
        text: "Reference",
        items: [
          { text: "Upstream dependencies", link: "/reference/upstream" },
          { text: "Changelog", link: "/reference/changelog" },
        ],
      },
    ],

    outline: { level: [2, 3] },
    search: { provider: "local" },
    socialLinks: [{ icon: "github", link: repo }],
    editLink: {
      pattern: `${repo}/edit/main/docs/:path`,
      text: "Edit this page on GitHub",
    },
    footer: {
      message: "Released under the MIT License.",
    },
  },
});
