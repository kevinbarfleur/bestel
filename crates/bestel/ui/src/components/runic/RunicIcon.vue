<script setup lang="ts">
/**
 * Unified inline-SVG icon set used by buttons, modals, search, theme toggle.
 * Stroke-only, currentColor-driven, 24×24 viewBox so any pixel size scales
 * crisply. Match the v5 design's `Icon` component byte-for-byte.
 */

type IconName =
  | 'plus'
  | 'check'
  | 'arrow'
  | 'open'
  | 'trash'
  | 'search'
  | 'close'
  | 'eye'
  | 'save'
  | 'sun'
  | 'moon'
  | 'gear'
  | 'reload'
  | 'back'
  | 'forward';

withDefaults(
  defineProps<{
    name: IconName;
    /** Pixel size (square). Default 16. */
    size?: number;
    /** Stroke width. Default 1.6. */
    strokeWidth?: number;
  }>(),
  { size: 16, strokeWidth: 1.6 },
);
</script>

<template>
  <svg
    :width="size"
    :height="size"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    :stroke-width="strokeWidth"
    stroke-linecap="round"
    stroke-linejoin="round"
    class="runic-icon"
    aria-hidden="true"
  >
    <template v-if="name === 'plus'">
      <path d="M12 5v14" />
      <path d="M5 12h14" />
    </template>
    <template v-else-if="name === 'check'">
      <path d="M5 12.5l4.5 4.5L19 7" />
    </template>
    <template v-else-if="name === 'arrow'">
      <path d="M5 12h14" />
      <path d="M13 6l6 6-6 6" />
    </template>
    <template v-else-if="name === 'open'">
      <path d="M14 5h5v5" />
      <path d="M19 5l-9 9" />
      <path d="M5 9v10h10" />
    </template>
    <template v-else-if="name === 'trash'">
      <path d="M4 7h16" />
      <path d="M9 7V4h6v3" />
      <path d="M6 7l1 13h10l1-13" />
    </template>
    <template v-else-if="name === 'search'">
      <circle cx="11" cy="11" r="6" />
      <path d="M20 20l-4.5-4.5" />
    </template>
    <template v-else-if="name === 'close'">
      <path d="M6 6l12 12" />
      <path d="M18 6L6 18" />
    </template>
    <template v-else-if="name === 'eye'">
      <path d="M2 12s3.5-7 10-7 10 7 10 7-3.5 7-10 7S2 12 2 12z" />
      <circle cx="12" cy="12" r="2.6" />
    </template>
    <template v-else-if="name === 'save'">
      <path d="M5 5h11l3 3v11H5z" />
      <path d="M8 5v5h7V5" />
      <path d="M8 19v-6h8v6" />
    </template>
    <template v-else-if="name === 'sun'">
      <circle cx="12" cy="12" r="3.6" />
      <path
        d="M12 3v2M12 19v2M3 12h2M19 12h2M5.6 5.6l1.4 1.4M17 17l1.4 1.4M5.6 18.4L7 17M17 7l1.4-1.4"
      />
    </template>
    <template v-else-if="name === 'moon'">
      <path d="M20 14.5A8 8 0 0 1 9.5 4a8 8 0 1 0 10.5 10.5z" />
    </template>
    <template v-else-if="name === 'gear'">
      <circle cx="12" cy="12" r="3" />
      <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09a1.65 1.65 0 0 0-1-1.51 1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09a1.65 1.65 0 0 0 1.51-1 1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" />
    </template>
    <template v-else-if="name === 'reload'">
      <path d="M3 12a9 9 0 0 1 15.5-6.3L21 8" />
      <path d="M21 3v5h-5" />
      <path d="M21 12a9 9 0 0 1-15.5 6.3L3 16" />
      <path d="M3 21v-5h5" />
    </template>
    <template v-else-if="name === 'back'">
      <path d="M15 18l-6-6 6-6" />
    </template>
    <template v-else-if="name === 'forward'">
      <path d="M9 18l6-6-6-6" />
    </template>
  </svg>
</template>

<style scoped>
.runic-icon {
  flex: none;
  display: block;
}
</style>
