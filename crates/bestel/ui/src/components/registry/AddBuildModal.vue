<script setup lang="ts">
import { ref } from 'vue';
import RunicButton from '../runic/RunicButton.vue';
import RunicIcon from '../runic/RunicIcon.vue';
import FreshPill from '../sheet/FreshPill.vue';
import type { RegistryEntryDto } from '../../api/types';

defineProps<{
  /** When set, the modal renders a preview of an already-parsed entry. */
  preview?: RegistryEntryDto | null;
  /** Error message to display below the file picker (parse failed, etc.). */
  error?: string | null;
  busy?: boolean;
}>();
const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'pick-file'): void;
  (e: 'save', followUp: boolean): void;
}>();

const openInChat = ref(true);
</script>

<template>
  <div class="add-build-modal" role="dialog" aria-modal="true" aria-label="Add a build to the registry">
    <div class="add-build-modal__backdrop" @click="emit('close')" />
    <div class="add-build-modal__panel">
      <header class="add-build-modal__header">
        <div class="add-build-modal__heading">
          <div class="add-build-modal__title">Add a build to registry</div>
          <div class="add-build-modal__subtitle">
            Pick a PoB XML. Bestel parses it, computes signatures, and links any existing sheet.
          </div>
        </div>
        <button class="add-build-modal__close" type="button" aria-label="Close" @click="emit('close')">
          <RunicIcon name="close" :size="14" />
        </button>
      </header>

      <div class="add-build-modal__body">
        <div class="add-build-modal__drop">
          <div class="add-build-modal__xml-icon">XML</div>
          <div class="add-build-modal__drop-meta">
            <div class="add-build-modal__drop-title">
              {{ preview ? preview.pob_path.split(/[\\/]/).pop() : 'Pick a PoB XML…' }}
            </div>
            <div v-if="preview" class="add-build-modal__drop-path">
              {{ preview.pob_path }}
            </div>
          </div>
          <RunicButton variant="secondary" @click="emit('pick-file')">Browse…</RunicButton>
        </div>

        <div v-if="error" class="add-build-modal__error">{{ error }}</div>

        <section v-if="preview" class="add-build-modal__section">
          <div class="add-build-modal__section-head">
            <span>Identity preview</span>
            <span class="add-build-modal__section-rule" />
            <span class="add-build-modal__section-status">parsed</span>
          </div>
          <div class="add-build-modal__preview">
            <div class="add-build-modal__preview-title">
              <span class="add-build-modal__game">{{ preview.game }}</span>
              <span class="add-build-modal__name">{{ preview.display_name }}</span>
            </div>
            <div class="add-build-modal__preview-sub">
              {{ preview.summary.class }} · {{ preview.summary.ascendancy ?? '—' }}
              <template v-if="preview.summary.level"> · lvl {{ preview.summary.level }}</template>
              <template v-if="preview.summary.main_skill">
                · main skill <strong>{{ preview.summary.main_skill }}</strong>
              </template>
            </div>
            <dl class="add-build-modal__preview-grid">
              <div>
                <dt>defining uniques</dt>
                <dd>{{ preview.summary.defining_uniques.join(' · ') || '—' }}</dd>
              </div>
              <div>
                <dt>pob_hash</dt>
                <dd class="add-build-modal__mono">{{ preview.pob_hash.slice(0, 12) }}…</dd>
              </div>
            </dl>
          </div>
        </section>

        <section v-if="preview" class="add-build-modal__section">
          <div class="add-build-modal__section-head">
            <span>Signatures computed</span>
            <span class="add-build-modal__section-rule" />
          </div>
          <FreshPill />
          <div v-if="preview.linked_sheet_id" class="add-build-modal__linked">
            A matching sheet was found in your library (id={{ preview.linked_sheet_id }}) and will be linked automatically.
          </div>
        </section>
      </div>

      <footer class="add-build-modal__footer">
        <label class="add-build-modal__follow">
          <input v-model="openInChat" type="checkbox" />
          Open a new chat with this build attached after saving
        </label>
        <RunicButton variant="secondary" @click="emit('close')">Cancel</RunicButton>
        <RunicButton
          variant="primary"
          icon="check"
          :disabled="!preview || busy"
          @click="emit('save', openInChat)"
        >
          {{ busy ? 'Saving…' : 'Save to registry' }}
        </RunicButton>
      </footer>
    </div>
  </div>
</template>

<style scoped>
.add-build-modal {
  position: fixed;
  inset: 0;
  z-index: 100;
  display: flex;
  align-items: center;
  justify-content: center;
  font-family: var(--hand);
  color: var(--ink);
}
.add-build-modal__backdrop {
  position: absolute;
  inset: 0;
  background: rgba(60, 55, 50, 0.35);
}
.theme-dark .add-build-modal__backdrop {
  background: rgba(8, 6, 4, 0.55);
}
.add-build-modal__panel {
  position: relative;
  width: 820px;
  max-width: calc(100% - 32px);
  max-height: calc(100% - 48px);
  background: var(--paper);
  border-radius: 6px;
  display: grid;
  grid-template-rows: auto 1fr auto;
  overflow: hidden;
  box-shadow: 0 14px 30px rgba(60, 40, 20, 0.18);
}
.theme-dark .add-build-modal__panel {
  box-shadow: 0 18px 40px rgba(0, 0, 0, 0.5);
}
.add-build-modal__header {
  padding: 16px 24px 14px;
  border-bottom: 1px solid var(--paper-line);
  display: flex;
  align-items: center;
  gap: 16px;
}
.add-build-modal__heading {
  flex: 1;
}
.add-build-modal__title {
  font-size: 19px;
  font-weight: 600;
}
.add-build-modal__subtitle {
  font-size: 14px;
  color: var(--ink-soft);
  margin-top: 2px;
}
.add-build-modal__close {
  width: 32px;
  height: 32px;
  padding: 0;
  background: transparent;
  color: var(--ink-soft);
  border: 1px solid transparent;
  border-radius: 4px;
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  justify-content: center;
}
.add-build-modal__close:hover {
  background: var(--paper-shade);
}
.add-build-modal__body {
  padding: 22px 28px;
  overflow: auto;
  display: flex;
  flex-direction: column;
  gap: 20px;
}
.add-build-modal__drop {
  border: 1.4px dashed var(--ink-faint);
  border-radius: 6px;
  padding: 20px 24px;
  background: var(--paper-shade);
  display: flex;
  align-items: center;
  gap: 18px;
}
.add-build-modal__xml-icon {
  width: 44px;
  height: 44px;
  border-radius: 4px;
  background: var(--paper);
  border: 1px solid var(--paper-line);
  display: flex;
  align-items: center;
  justify-content: center;
  font-family: 'JetBrains Mono', monospace;
  font-size: 11px;
  color: var(--ink-faint);
}
.add-build-modal__drop-meta {
  flex: 1;
  min-width: 0;
}
.add-build-modal__drop-title {
  font-size: 16px;
  font-weight: 600;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.add-build-modal__drop-path {
  font-family: 'JetBrains Mono', monospace;
  font-size: 12px;
  color: var(--ink-faint);
  margin-top: 2px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.add-build-modal__error {
  font-size: 13px;
  color: var(--bad);
  background: rgba(164, 72, 72, 0.08);
  border: 1px solid var(--bad);
  border-radius: 4px;
  padding: 10px 14px;
}
.add-build-modal__section-head {
  display: flex;
  align-items: baseline;
  gap: 8px;
  margin-bottom: 8px;
  font-family: var(--label);
  font-size: 11px;
  letter-spacing: 0.18em;
  text-transform: uppercase;
  color: var(--ink-soft);
  font-weight: 600;
}
.add-build-modal__section-rule {
  flex: 1;
  height: 1px;
  background: var(--paper-line);
}
.add-build-modal__section-status {
  color: var(--good);
}
.add-build-modal__preview {
  padding: 14px 18px;
  border: 1px solid var(--paper-line);
  border-radius: 5px;
  background: var(--paper);
}
.add-build-modal__preview-title {
  display: flex;
  align-items: baseline;
  gap: 10px;
}
.add-build-modal__game {
  font-family: 'JetBrains Mono', monospace;
  font-size: 12px;
  color: var(--ink-faint);
  padding: 2px 6px;
  border: 1px solid var(--paper-line);
  border-radius: 3px;
}
.add-build-modal__name {
  font-size: 22px;
  font-weight: 700;
}
.add-build-modal__preview-sub {
  font-size: 14px;
  color: var(--ink-soft);
  margin-top: 4px;
}
.add-build-modal__preview-sub strong {
  color: var(--ink);
  font-weight: 600;
}
.add-build-modal__preview-grid {
  margin-top: 12px;
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 6px 24px;
  font-size: 13px;
  color: var(--ink-soft);
}
.add-build-modal__preview-grid dt {
  font-family: var(--label);
  font-size: 11px;
  letter-spacing: 0.16em;
  text-transform: uppercase;
  font-weight: 600;
}
.add-build-modal__preview-grid dd {
  margin: 2px 0 0;
  color: var(--ink);
  font-size: 14px;
}
.add-build-modal__mono {
  font-family: 'JetBrains Mono', monospace;
  font-size: 13px;
}
.add-build-modal__linked {
  font-size: 13px;
  color: var(--ink-soft);
  margin-top: 10px;
  line-height: 1.55;
}
.add-build-modal__footer {
  padding: 16px 28px;
  border-top: 1px solid var(--paper-line);
  background: var(--paper-shade);
  display: flex;
  align-items: center;
  gap: 12px;
}
.add-build-modal__follow {
  flex: 1;
  display: inline-flex;
  align-items: center;
  gap: 8px;
  font-size: 14px;
  color: var(--ink-soft);
  cursor: pointer;
}
.add-build-modal__follow input {
  cursor: pointer;
}
</style>
