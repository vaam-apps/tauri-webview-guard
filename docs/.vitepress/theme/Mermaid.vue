<script setup lang="ts">
// Renders one ```mermaid fence (see the markdown config in ../config.ts).
// Client-only: mermaid needs the DOM, so the server render is the source
// text, which is also what a reader without JavaScript sees. Colours come
// from the page's Material 3 tokens and follow the light/dark toggle.
import { onBeforeUnmount, onMounted, ref } from "vue";

const props = defineProps<{ code: string }>();
const source = decodeURIComponent(props.code);
const svg = ref("");
let observer: MutationObserver | undefined;
let seq = 0;

function token(name: string): string {
  return getComputedStyle(document.documentElement).getPropertyValue(`--md-sys-color-${name}`).trim();
}

async function draw() {
  const { default: mermaid } = await import("mermaid");
  mermaid.initialize({
    startOnLoad: false,
    securityLevel: "strict",
    theme: "base",
    fontFamily: "inherit",
    // The sequences are wide; draw them at full size and let the box scroll
    // rather than shrinking them below legibility.
    sequence: { useMaxWidth: false },
    themeVariables: {
      darkMode: document.documentElement.classList.contains("dark"),
      background: token("surface"),
      primaryColor: token("primary-container"),
      primaryTextColor: token("on-primary-container"),
      primaryBorderColor: token("primary"),
      secondaryColor: token("secondary-container"),
      tertiaryColor: token("tertiary-container"),
      lineColor: token("outline"),
      textColor: token("on-surface"),
      noteBkgColor: token("tertiary-container"),
      noteTextColor: token("on-tertiary-container"),
      noteBorderColor: token("tertiary"),
      actorBkg: token("primary-container"),
      actorTextColor: token("on-primary-container"),
      actorBorder: token("primary"),
      actorLineColor: token("outline"),
      signalColor: token("on-surface"),
      signalTextColor: token("on-surface"),
      labelBoxBkgColor: token("secondary-container"),
      labelTextColor: token("on-secondary-container"),
      loopTextColor: token("on-surface"),
    },
  });
  const id = `mermaid-${Math.random().toString(36).slice(2)}-${seq++}`;
  svg.value = (await mermaid.render(id, source)).svg;
}

onMounted(() => {
  void draw();
  observer = new MutationObserver(() => void draw());
  observer.observe(document.documentElement, { attributes: true, attributeFilter: ["class"] });
});
onBeforeUnmount(() => observer?.disconnect());
</script>

<template>
  <div class="mermaid-diagram">
    <div v-if="svg" v-html="svg" />
    <pre v-else><code>{{ source }}</code></pre>
  </div>
</template>

<style scoped>
.mermaid-diagram {
  margin: 16px 0;
  padding: 16px;
  overflow-x: auto;
  border-radius: var(--md-sys-shape-corner-large, 16px);
  background-color: var(--md-sys-color-surface-container-low);
}
.mermaid-diagram :deep(svg) {
  display: block;
  margin: 0 auto;
}
.mermaid-diagram pre {
  margin: 0;
  white-space: pre-wrap;
}
</style>
