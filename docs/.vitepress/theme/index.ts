// The vaam-apps house theme: Material 3, configured in ../config.ts.
// Extended only to render the ```mermaid blocks in lifecycle.md as diagrams.
import type { Theme } from "vitepress";
import Md3Theme from "@vaam-apps/vitepress-md3";
import Mermaid from "./Mermaid.vue";

export default {
  extends: Md3Theme,
  enhanceApp({ app }) {
    app.component("Mermaid", Mermaid);
  },
} satisfies Theme;
