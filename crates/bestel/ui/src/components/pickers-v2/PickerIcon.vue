<!--
  Inline SVG icon set for the v2 picker family. Mirrors the glyphs from
  the upstream design's `picker-primitives.jsx` Icon function (line 48-73)
  so every modal speaks the same vocabulary. Strokes scale to text size
  via the `:size` prop, default 16 to sit cleanly next to 14-15 px body.

  Why these glyphs and not lucide / heroicons:
  - The 1.6 stroke-width and rounded line caps match EB Garamond's stems.
  - We needed `external` and `pin` glyphs that aren't in either library.
  - Inline SVG ships as part of the page's HTML stream — no extra fetch,
    no font face fallback flicker on first paint of a modal.
-->
<script setup lang="ts">
import { computed } from 'vue';

type IconName =
  | 'plus' | 'check' | 'arrow' | 'open' | 'trash' | 'search' | 'close'
  | 'eye' | 'save' | 'sun' | 'moon' | 'external' | 'pin';

const props = withDefaults(
  defineProps<{
    name: IconName;
    size?: number;
    strokeWidth?: number;
    color?: string;
  }>(),
  { size: 16, strokeWidth: 1.6, color: 'currentColor' },
);

const viewBox = '0 0 24 24';

const path = computed(() => {
  switch (props.name) {
    case 'plus':
      return '<path d="M12 5v14"/><path d="M5 12h14"/>';
    case 'check':
      return '<path d="M5 12.5l4.5 4.5L19 7"/>';
    case 'arrow':
      return '<path d="M5 12h14"/><path d="M13 6l6 6-6 6"/>';
    case 'open':
      return '<path d="M14 5h5v5"/><path d="M19 5l-9 9"/><path d="M5 9v10h10"/>';
    case 'trash':
      return '<path d="M4 7h16"/><path d="M9 7V4h6v3"/><path d="M6 7l1 13h10l1-13"/>';
    case 'search':
      return '<circle cx="11" cy="11" r="6"/><path d="M20 20l-4.5-4.5"/>';
    case 'close':
      return '<path d="M6 6l12 12"/><path d="M18 6L6 18"/>';
    case 'eye':
      return '<path d="M2 12s3.5-7 10-7 10 7 10 7-3.5 7-10 7S2 12 2 12z"/><circle cx="12" cy="12" r="2.6"/>';
    case 'save':
      return '<path d="M5 5h11l3 3v11H5z"/><path d="M8 5v5h7V5"/><path d="M8 19v-6h8v6"/>';
    case 'sun':
      return '<circle cx="12" cy="12" r="3.6"/><path d="M12 3v2M12 19v2M3 12h2M19 12h2M5.6 5.6l1.4 1.4M17 17l1.4 1.4M5.6 18.4L7 17M17 7l1.4-1.4"/>';
    case 'moon':
      return '<path d="M20 14.5A8 8 0 0 1 9.5 4a8 8 0 1 0 10.5 10.5z"/>';
    case 'external':
      return '<path d="M14 4h6v6"/><path d="M20 4l-9 9"/><path d="M19 14v5a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1V6a1 1 0 0 1 1-1h5"/>';
    case 'pin':
      return '<path d="M14 3l7 7-3 1-3 5-3-3-5 5v-5l3-3-3-3 5-3 1-3z"/>';
    default:
      return '';
  }
});
</script>

<template>
  <svg
    :width="size"
    :height="size"
    :viewBox="viewBox"
    fill="none"
    :stroke="color"
    :stroke-width="strokeWidth"
    stroke-linecap="round"
    stroke-linejoin="round"
    aria-hidden="true"
    class="picker-icon"
    v-html="path"
  />
</template>

<style scoped>
.picker-icon {
  flex: none;
  display: block;
}
</style>
