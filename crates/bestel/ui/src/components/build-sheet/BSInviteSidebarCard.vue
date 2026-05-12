<script setup lang="ts">
import { computed } from 'vue';
import { storeToRefs } from 'pinia';

import { useChatStore } from '../../stores/chat';
import { useSheetStore } from '../../stores/sheet';

/**
 * Sidebar invite card — shown when a PoB is attached but the chat has
 * neither a validated sheet nor an active interview. Mirrors the design
 * `SheetPanel state="invite"` block (`build-sheet.jsx:185-203`): badge
 * "no sheet yet", a short lede, and a `Start interview` outline button.
 *
 * Click on Start sends a predefined message via `chat.send(...)` in the
 * current chat. The agent reads the runtime tag `Build sheet: absent`
 * and runs the deep-analysis + `sheet_open_interview` flow as usual.
 */

const sheet = useSheetStore();
const chat = useChatStore();

const { activeBuild } = storeToRefs(chat);
const { activeSheet, activeInterview } = storeToRefs(sheet);

const visible = computed(
  () =>
    activeBuild.value !== null &&
    activeSheet.value === null &&
    activeInterview.value === null,
);

const START_MESSAGE =
  'Please run a thorough analysis of my build and start the Build Sheet interview. Look at my PoB carefully — defining uniques, main skill, defense layers — then prepare a comprehensive sheet so every future chat about this character has full context.';

async function onStart() {
  await chat.send(START_MESSAGE);
}
</script>

<template>
  <div v-if="visible" class="bs-invite">
    <div class="bs-invite__head">
      <span class="bs-invite__badge">
        <span class="bs-invite__glyph" aria-hidden="true">·</span>
        No Build Sheet yet
      </span>
    </div>
    <p class="bs-invite__lede">
      Bestel hasn't read this build in depth yet. A short interview now gives Bestel a Build Sheet —
      its notes on this character — that every future chat will reuse.
    </p>
    <button type="button" class="bs-invite__cta" @click="onStart">
      <span class="bs-invite__cta-glyph" aria-hidden="true">+</span>
      Start interview
    </button>
  </div>
</template>

<style scoped>
.bs-invite {
  flex: 0 0 auto;
  width: 100%;
  padding: 14px 16px;
  border: 1px dashed var(--paper-line);
  border-radius: 5px;
  background: var(--paper-shade);
  display: flex;
  flex-direction: column;
  gap: 10px;
  box-sizing: border-box;
}

.bs-invite__head {
  display: flex;
  align-items: center;
  gap: 8px;
}
.bs-invite__badge {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 2px 8px;
  border: 1px solid var(--ink-faint);
  border-radius: 3px;
  background: rgba(138, 134, 130, 0.12);
  font-family: var(--label);
  font-size: 11px;
  letter-spacing: 0.14em;
  text-transform: uppercase;
  color: var(--ink-soft);
  font-weight: 700;
  white-space: nowrap;
}
.bs-invite__glyph {
  font-family: var(--hand);
  font-size: 14px;
  line-height: 1;
  color: var(--ink-faint);
}

.bs-invite__lede {
  margin: 0;
  font-family: var(--hand);
  font-size: 14px;
  color: var(--ink-soft);
  line-height: 1.5;
}

.bs-invite__cta {
  align-self: flex-start;
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  border: 1px solid var(--ink-soft);
  border-radius: 4px;
  background: var(--paper);
  font-family: var(--hand);
  font-size: 13px;
  font-weight: 500;
  color: var(--ink);
  cursor: pointer;
  transition: border-color 0.15s ease, background 0.15s ease;
}
.bs-invite__cta:hover {
  border-color: var(--amber);
  background: var(--amber-glow);
}
.bs-invite__cta-glyph {
  font-family: var(--hand);
  font-size: 14px;
  line-height: 1;
  color: var(--amber);
}
</style>
