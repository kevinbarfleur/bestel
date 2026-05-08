<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core';
import { useToastsStore } from '../../stores/toasts';

const toasts = useToastsStore();

async function openDevPanel() {
  try {
    await invoke('dev_panel_open');
  } catch (e) {
    toasts.push({
      variant: 'error',
      title: 'Failed to open dev panel',
      body: String(e),
    });
  }
}
</script>

<template>
  <div class="pane">
    <header class="pane__header">
      <h1 class="pane__title">Developer</h1>
      <p class="pane__sub">
        The dev panel is a separate window that lets you browse past chat
        runs, drive scenarios + real-prompt batteries, and compare rendered
        markdown against the raw stream events. Useful for diagnosing UI
        rendering bugs and comparing models.
      </p>
    </header>

    <section class="dev-section">
      <button class="dev-cta" @click="openDevPanel">
        Open dev panel
      </button>
      <p class="dev-shortcut">
        or press <kbd>Ctrl</kbd> + <kbd>Shift</kbd> + <kbd>D</kbd> from
        any window.
      </p>
    </section>

    <section class="dev-section">
      <h2 class="dev-h2">What's inside</h2>
      <ul class="dev-list">
        <li><strong>Runs</strong> — every past chat (UI + smoke) with full
          transcript, tool details, usage and elapsed time.</li>
        <li><strong>Scenarios</strong> — the assertion-driven TOML files
          under <code>tests/scenarios/*.toml</code>.</li>
        <li><strong>Real prompts</strong> — voice-preserved prompts mined
          from Reddit / forums / Maxroll under
          <code>docs/test_prompts/real_user_prompts.toml</code>.</li>
        <li><strong>Live test</strong> — pick a model, load a PoB fixture,
          send a free-form prompt, and inspect both the rendered markdown
          and the raw <code>LlmDelta</code> stream side by side.</li>
      </ul>
    </section>

    <section class="dev-section">
      <h2 class="dev-h2">Headless CLI</h2>
      <p class="dev-text">
        For batch runs without the UI, use:
      </p>
      <pre class="dev-cli">bestel run-battery tests/scenarios --out target/test-runs/</pre>
      <p class="dev-text">
        Persists one <code>PersistedRun</code> JSON per scenario and exits
        non-zero if any scenario fails its assertions.
      </p>
    </section>
  </div>
</template>

<style scoped>
.pane {
  display: flex;
  flex-direction: column;
  gap: 24px;
  padding: 0 4px;
}
.pane__header {
  border-bottom: 1px solid var(--c-rule, #2a2a30);
  padding-bottom: 12px;
}
.pane__title {
  margin: 0;
  font-family: var(--font-display, 'Cinzel', serif);
  font-size: 22px;
  letter-spacing: 0.04em;
}
.pane__sub {
  color: var(--c-fg-mute, #75757f);
  margin: 6px 0 0 0;
  font-size: 13px;
  line-height: 1.5;
}
.dev-section {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.dev-h2 {
  font-family: var(--font-display, 'Cinzel', serif);
  font-size: 14px;
  letter-spacing: 0.04em;
  margin: 0 0 4px 0;
  color: var(--c-fg, #e8e6df);
}
.dev-cta {
  align-self: flex-start;
  background: var(--c-accent, #c9a86a);
  color: #141417;
  border: none;
  padding: 10px 18px;
  border-radius: 4px;
  font: inherit;
  font-weight: 500;
  cursor: pointer;
  font-size: 13px;
}
.dev-cta:hover { filter: brightness(1.08); }
.dev-shortcut {
  font-size: 12px;
  color: var(--c-fg-mute, #75757f);
  margin: 0;
}
kbd {
  display: inline-block;
  padding: 1px 6px;
  background: var(--c-bg-deep, #0a0a0c);
  color: var(--c-fg, #e8e6df);
  border: 1px solid var(--c-rule, #2a2a30);
  border-radius: 3px;
  font-family: var(--font-mono, monospace);
  font-size: 11px;
}
.dev-list {
  margin: 0;
  padding-left: 18px;
  display: flex;
  flex-direction: column;
  gap: 6px;
  font-size: 13px;
  color: var(--c-fg, #e8e6df);
  line-height: 1.5;
}
.dev-list strong {
  color: var(--c-accent, #c9a86a);
  font-weight: 500;
}
code {
  background: var(--c-bg-deep, #0a0a0c);
  color: var(--c-accent, #c9a86a);
  padding: 1px 4px;
  border-radius: 2px;
  font-size: 11px;
  font-family: var(--font-mono, monospace);
}
.dev-cli {
  background: var(--c-bg-deep, #0a0a0c);
  border: 1px solid var(--c-rule, #2a2a30);
  color: var(--c-accent, #c9a86a);
  padding: 10px 12px;
  border-radius: 4px;
  font-family: var(--font-mono, monospace);
  font-size: 12px;
  margin: 0;
}
.dev-text {
  font-size: 13px;
  color: var(--c-fg-mute, #75757f);
  margin: 0;
  line-height: 1.5;
}
</style>
