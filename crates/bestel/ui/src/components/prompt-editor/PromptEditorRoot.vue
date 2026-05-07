<script setup lang="ts">
// Placeholder — fleshed out in Commit D.
</script>

<template>
  <div class="prompt-editor-shell">
    <p class="prompt-editor-shell__loading">Loading prompts…</p>
  </div>
</template>

<style scoped>
.prompt-editor-shell {
  width: 100vw;
  height: 100vh;
  background: var(--paper);
  color: var(--ink);
  display: flex;
  align-items: center;
  justify-content: center;
  font-family: var(--hand);
}

.prompt-editor-shell__loading {
  font-size: 14px;
  color: var(--ink-faint);
  font-style: italic;
}
</style>
