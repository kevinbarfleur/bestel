<script setup lang="ts">
import { useUiStore } from '../../stores/ui';

/**
 * Hero block shown over the chat canvas when the user has no build in
 * this chat AND no messages yet. The first time a user launches Bestel,
 * the composer used to render over a blank page with no signposting —
 * this card replaces that void with a single clear CTA. The library
 * modal is the sole entry point for build attachment, so we surface
 * it directly here.
 */
const ui = useUiStore();
</script>

<template>
  <div class="empty-hero">
    <div class="empty-hero__card">
      <div class="empty-hero__eyebrow">how Bestel works</div>
      <h2 class="empty-hero__title">Talk to Bestel about your character.</h2>
      <p class="empty-hero__lede">
        Bestel works from a Path of Building file. Save one to your library to
        get started — every chat is grounded in real numbers from your build.
      </p>
      <button
        type="button"
        class="empty-hero__cta"
        @click="ui.openRegistryModal()"
      >
        Open your build library
      </button>
      <p class="empty-hero__footer">
        Watcher folders live in Settings → Watcher folders.
      </p>
    </div>
  </div>
</template>

<style scoped>
.empty-hero {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 32px;
}

.empty-hero__card {
  max-width: 520px;
  text-align: center;
  padding: 28px 32px 24px;
  background: var(--paper);
  border: 1px solid var(--paper-line);
  border-radius: 6px;
  box-shadow: 0 8px 24px rgba(60, 40, 20, 0.08);
}
.theme-dark .empty-hero__card {
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.35);
}

.empty-hero__eyebrow {
  font-family: var(--label);
  font-size: 11px;
  letter-spacing: 0.20em;
  text-transform: uppercase;
  color: var(--ink-soft);
  font-weight: 600;
  margin-bottom: 12px;
}

.empty-hero__title {
  margin: 0 0 12px;
  font-family: var(--hand);
  font-size: 26px;
  font-weight: 700;
  color: var(--ink);
  line-height: 1.2;
}

.empty-hero__lede {
  margin: 0 0 22px;
  font-family: var(--hand);
  font-size: 15px;
  line-height: 1.55;
  color: var(--ink-soft);
}

.empty-hero__cta {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 10px 24px;
  background: var(--ink);
  color: var(--paper);
  border: 1px solid var(--ink);
  border-radius: 4px;
  font-family: var(--hand);
  font-size: 15px;
  font-weight: 600;
  cursor: pointer;
  transition: opacity 0.15s ease;
}
.empty-hero__cta:hover {
  opacity: 0.92;
}

.empty-hero__footer {
  margin: 18px 0 0;
  font-family: var(--hand);
  font-size: 12.5px;
  color: var(--ink-faint);
}
</style>
