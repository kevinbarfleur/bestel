<script setup lang="ts">
/**
 * 2-pane picker layout shared by ModelPicker / BuildPicker / ChatPicker.
 *
 * Slots:
 *   - sidebar: list pane on the left (340px)
 *   - main:    detail pane on the right (flex)
 *   - actionBar: optional sticky action bar at bottom of main pane
 *                (Cancel + primary CTA)
 *   - footer:  optional shortcut hint row at the bottom of the modal
 *              (rendered with kbd chips like "↑↓ navigate · ⏎ select · esc close")
 *
 * The layout never scrolls itself — the sidebar and main panes scroll
 * independently. The parent `RunicModal` must use `max-width="panes"` so the
 * modal content has a fixed height for the panes to fill.
 */
defineProps<{
  /** Caption shown above the sidebar list (small caps, optional). */
  sidebarCaption?: string;
  /** Caption shown above the main pane (small caps, optional). */
  mainCaption?: string;
}>();
</script>

<template>
  <div class="picker-layout">
    <aside class="picker-layout__sidebar runic-scrollbar">
      <header v-if="sidebarCaption" class="picker-layout__pane-head">
        <span class="picker-layout__pane-caption">{{ sidebarCaption }}</span>
        <span class="picker-layout__pane-hairline" />
      </header>
      <div class="picker-layout__sidebar-body">
        <slot name="sidebar" />
      </div>
    </aside>

    <div class="picker-layout__divider" aria-hidden="true" />

    <section class="picker-layout__main">
      <div class="picker-layout__main-body runic-scrollbar">
        <header v-if="mainCaption" class="picker-layout__pane-head picker-layout__pane-head--main">
          <span class="picker-layout__pane-caption">{{ mainCaption }}</span>
          <span class="picker-layout__pane-hairline" />
        </header>
        <slot name="main" />
      </div>
      <footer v-if="$slots.actionBar" class="picker-layout__action-bar">
        <slot name="actionBar" />
      </footer>
    </section>

    <footer v-if="$slots.footer" class="picker-layout__footer">
      <slot name="footer" />
    </footer>
  </div>
</template>

<style scoped>
.picker-layout {
  display: grid;
  grid-template-columns: 340px 1px 1fr;
  grid-template-rows: 1fr auto;
  height: 100%;
  background: var(--paper);
  font-family: var(--hand);
  color: var(--ink);
}

.picker-layout__sidebar {
  grid-column: 1;
  grid-row: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
  overflow-y: auto;
  background: var(--paper);
}
.picker-layout__sidebar-body {
  display: flex;
  flex-direction: column;
}

.picker-layout__divider {
  grid-column: 2;
  grid-row: 1;
  background: var(--paper-line);
}

.picker-layout__main {
  grid-column: 3;
  grid-row: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  background: var(--paper);
}
.picker-layout__main-body {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding: 24px 32px;
}

.picker-layout__pane-head {
  display: flex;
  align-items: baseline;
  gap: 0.5rem;
  padding: 14px 16px 8px;
}
.picker-layout__pane-head--main {
  padding: 0 0 12px;
  margin-bottom: 16px;
  border-bottom: 1px dotted var(--paper-line);
}
.picker-layout__pane-caption {
  font-family: var(--label);
  font-size: var(--fs-caps);
  letter-spacing: 0.18em;
  text-transform: uppercase;
  color: var(--ink-faint);
  font-weight: var(--fw-semibold);
}
.picker-layout__pane-hairline {
  flex: 1;
  height: 1px;
  background: var(--paper-line);
  align-self: center;
}

.picker-layout__action-bar {
  flex: none;
  padding: 16px 32px;
  border-top: 1px solid var(--paper-line);
  background: var(--paper-shade);
  display: flex;
  align-items: center;
  gap: 16px;
}

.picker-layout__footer {
  grid-column: 1 / -1;
  grid-row: 2;
  padding: 10px 24px;
  border-top: 1px solid var(--paper-line);
  background: var(--paper-shade);
  font-family: var(--hand);
  font-size: var(--fs-meta);
  color: var(--ink-soft);
  display: flex;
  align-items: center;
  gap: 18px;
}

@media (max-width: 760px) {
  .picker-layout {
    grid-template-columns: 1fr;
    grid-template-rows: auto auto 1fr auto;
  }
  .picker-layout__sidebar { grid-column: 1; grid-row: 1; max-height: 36vh; }
  .picker-layout__divider {
    grid-column: 1;
    grid-row: 2;
    height: 1px;
    width: 100%;
  }
  .picker-layout__main { grid-column: 1; grid-row: 3; }
  .picker-layout__footer { grid-row: 4; }
}
</style>
