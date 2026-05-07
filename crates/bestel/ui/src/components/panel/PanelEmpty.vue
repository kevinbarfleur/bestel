<script setup lang="ts">
// Reachable when the side panel slot is forced visible with no current
// artifact. Today the layout hides the panel entirely on `panelArtifact ==
// null` (PanelView.vue's outer `v-if`), so this component is shipped as
// ready code for the future "always-show panel side" toggle. The empty
// copy is from the v9 design (.design-bundle-v9/bestel/project/side-panel.jsx
// EmptyPanel).
</script>

<template>
  <div class="panel-empty">
    <p class="panel-empty__line">
      When Bestel highlights an item, gem or mechanic worth a closer look, it lands here.
    </p>
    <p class="panel-empty__line panel-empty__line--soft">
      You can also pin any open webview from its toolbar and recall it later from the bandeau under the topbar.
    </p>
  </div>
</template>

<style scoped>
.panel-empty {
  display: flex;
  flex-direction: column;
  gap: 14px;
  padding: 32px 4px;
  font-family: var(--hand);
  font-size: 14.5px;
  line-height: 1.6;
  color: var(--ink-soft);
  text-align: left;
}

.panel-empty__line {
  margin: 0;
}
.panel-empty__line--soft {
  color: var(--ink-faint);
}
</style>
