<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from 'vue';
import { storeToRefs } from 'pinia';

import { useChatStore } from '../../stores/chat';
import { useBuildStore } from '../../stores/build';
import { useSettingsStore } from '../../stores/settings';
import { useToastsStore } from '../../stores/toasts';
import { useUiStore } from '../../stores/ui';
import type { AttachmentDto } from '../../api/types';
import AttachmentChip from './artifacts/AttachmentChip.vue';
import RunicButton from '../runic/RunicButton.vue';

const chat = useChatStore();
const buildStore = useBuildStore();
const settings = useSettingsStore();
const toasts = useToastsStore();
const ui = useUiStore();

const { isStreaming } = storeToRefs(chat);
const { current } = storeToRefs(buildStore);
const { activeModel } = storeToRefs(settings);

const draft = ref('');
const textareaRef = ref<HTMLTextAreaElement | null>(null);
const fileInputRef = ref<HTMLInputElement | null>(null);
const attachments = ref<AttachmentDto[]>([]);

const MAX_FILES = 5;
const MAX_BYTES = 5 * 1024 * 1024;
const ACCEPT = 'image/png,image/jpeg,image/webp,image/gif,application/pdf,text/plain,text/markdown,.md,.txt,.pdf';

const canSend = computed(
  () => (draft.value.trim().length > 0 || attachments.value.length > 0) && !isStreaming.value,
);

const buildChipLabel = computed(() => {
  const c = current.value;
  if (!c) return null;
  return c.main_skill ?? c.class ?? c.file_name ?? 'build';
});

const modelName = computed(() => activeModel.value?.display_name ?? 'model');

const autosize = async () => {
  await nextTick();
  const el = textareaRef.value;
  if (!el) return;
  el.style.height = 'auto';
  const next = Math.min(Math.max(el.scrollHeight, 22), 220);
  el.style.height = `${next}px`;
};

watch(draft, () => { void autosize(); });

watch(isStreaming, async (s) => {
  if (!s) {
    await nextTick();
    textareaRef.value?.focus();
  }
});

const send = async () => {
  if (!canSend.value) return;
  if (
    activeModel.value &&
    !activeModel.value.vision_capable &&
    attachments.value.some((a) => a.mime.startsWith('image/'))
  ) {
    toasts.push({
      variant: 'error',
      title: 'Image attached on text-only model',
      body: `Remove the image or switch to a vision-capable model (Sonnet, Haiku, Opus, Claude Code, Codex).`,
    });
    return;
  }
  const text = draft.value;
  const atts = attachments.value;
  draft.value = '';
  attachments.value = [];
  await autosize();
  await chat.send(text, atts);
};

const cancel = () => { void chat.cancel(); };

const handleKeydown = (e: KeyboardEvent) => {
  const isPlainEnter = e.key === 'Enter' && !e.shiftKey;
  const isCmdEnter = e.key === 'Enter' && (e.ctrlKey || e.metaKey);
  if (isPlainEnter || isCmdEnter) {
    e.preventDefault();
    void send();
  }
};

const onClearBuild = async () => {
  const ok = await buildStore.clearActive();
  if (ok) {
    toasts.push({ variant: 'info', title: 'Build detached', body: 'Bestel will answer in generalist mode.' });
  }
};

const triggerAttach = () => {
  fileInputRef.value?.click();
};

const fileToBase64 = (file: File): Promise<string> =>
  new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => {
      const result = reader.result;
      if (typeof result !== 'string') {
        reject(new Error('Unexpected reader result'));
        return;
      }
      const idx = result.indexOf(',');
      resolve(idx >= 0 ? result.slice(idx + 1) : result);
    };
    reader.onerror = () => reject(reader.error ?? new Error('FileReader error'));
    reader.readAsDataURL(file);
  });

const guessMime = (file: File): string => {
  if (file.type) return file.type;
  const lower = file.name.toLowerCase();
  if (lower.endsWith('.md')) return 'text/markdown';
  if (lower.endsWith('.txt')) return 'text/plain';
  if (lower.endsWith('.pdf')) return 'application/pdf';
  return 'application/octet-stream';
};

const isImageMime = (mime: string) => mime.startsWith('image/');

const onFileChange = async (e: Event) => {
  const input = e.target as HTMLInputElement;
  if (!input.files || input.files.length === 0) return;
  const incoming = Array.from(input.files);
  let blockedImages = 0;
  for (const f of incoming) {
    if (attachments.value.length >= MAX_FILES) {
      toasts.push({
        variant: 'warning',
        title: 'Attachment limit reached',
        body: `Up to ${MAX_FILES} files per message.`,
      });
      break;
    }
    if (f.size > MAX_BYTES) {
      toasts.push({
        variant: 'warning',
        title: 'File too large',
        body: `${f.name} exceeds 5 MB.`,
      });
      continue;
    }
    const mime = guessMime(f);
    if (isImageMime(mime) && activeModel.value && !activeModel.value.vision_capable) {
      blockedImages += 1;
      continue;
    }
    try {
      const data_base64 = await fileToBase64(f);
      attachments.value.push({
        name: f.name,
        mime,
        data_base64,
      });
    } catch {
      toasts.push({ variant: 'error', title: 'Could not read file', body: f.name });
    }
  }
  input.value = '';
  if (blockedImages > 0 && activeModel.value) {
    toasts.push({
      variant: 'warning',
      title: 'Model is text-only',
      body: `${activeModel.value.display_name} can't read images. Switch to Sonnet, Haiku or Opus to attach screenshots.`,
    });
  }
};

const removeAttachment = (idx: number) => {
  attachments.value.splice(idx, 1);
};

onMounted(() => {
  void autosize();
  textareaRef.value?.focus();
});
</script>

<template>
  <div class="composer-wrap">
    <div class="composer">
      <!-- Row 1 — chips: build + attachments + add file -->
      <div class="composer__chips">
        <span v-if="buildChipLabel" class="chip chip--active composer__chip-build">
          <span class="composer__chip-rune">◆</span>
          build · {{ buildChipLabel }}
          <button
            type="button"
            class="composer__chip-x"
            aria-label="Detach build"
            @click="onClearBuild"
          >×</button>
        </span>

        <AttachmentChip
          v-for="(att, idx) in attachments"
          :key="idx"
          :attachment="att"
          @remove="removeAttachment(idx)"
        />

        <button type="button" class="chip chip--ghost composer__attach" @click="triggerAttach">
          + attach file
        </button>

        <input
          ref="fileInputRef"
          type="file"
          class="composer__file-input"
          :accept="ACCEPT"
          multiple
          @change="onFileChange"
        />
      </div>

      <!-- Row 2 : textarea (Garamond plain placeholder) -->
      <textarea
        ref="textareaRef"
        v-model="draft"
        class="composer__textarea"
        :placeholder="
          isStreaming
            ? 'The chronicler is writing… (Esc to interrupt)'
            : 'Ask anything about your build'
        "
        rows="1"
        :disabled="isStreaming"
        @keydown="handleKeydown"
      />

      <!-- Row 3 : footer dashed top + model selector + send button solid ink -->
      <div class="composer__footer">
        <button
          type="button"
          class="composer__model"
          :title="`Active model: ${modelName} — click to switch`"
          @click="ui.openModel()"
        >
          <span class="composer__model-label">model</span>
          <span class="composer__model-name">{{ modelName }}</span>
          <span class="composer__model-caret">▾</span>
        </button>
        <span class="composer__grow" />
        <span class="composer__hint">⌘↵</span>

        <RunicButton
          v-if="isStreaming"
          variant="primary"
          no-runes
          size="sm"
          danger
          icon="close"
          kbd="esc"
          @click="cancel"
        >
          Stop
        </RunicButton>
        <RunicButton
          v-else
          variant="primary"
          no-runes
          size="sm"
          icon="arrow"
          kbd="⏎"
          :disabled="!canSend"
          @click="send"
        >
          Send
        </RunicButton>
      </div>
    </div>
  </div>
</template>

<style scoped>
.composer-wrap {
  padding: 14px 24px 18px;
  border-top: 1px solid var(--paper-line);
  background: var(--paper);
  flex: none;
}

.composer {
  background: var(--paper);
  border: 1px solid var(--ink-soft);
  border-radius: 6px;
  padding: 14px 16px;
  display: flex;
  flex-direction: column;
  gap: 10px;
  transition: border-color 0.15s ease;
}
.composer:focus-within {
  border-color: var(--amber);
}

.composer__chips {
  display: flex;
  gap: 5px;
  align-items: center;
  flex-wrap: wrap;
}

/* Build chip — amber dashed border, amber glow background */
.composer__chip-build {
  cursor: default;
  border: 1px dashed var(--amber);
  background: var(--amber-glow);
  color: var(--ink-soft);
  font-family: var(--hand);
  font-size: var(--fs-caps);
  padding: 4px 10px;
  border-radius: 12px;
  display: inline-flex;
  align-items: center;
  gap: 6px;
}
.composer__chip-rune {
  color: var(--amber);
  margin-right: 2px;
}
.composer__chip-x {
  width: 14px;
  height: 14px;
  margin-left: 4px;
  border: 0;
  background: transparent;
  color: var(--ink-faint);
  font-size: 14px;
  line-height: 1;
  cursor: pointer;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0;
}
.composer__chip-x:hover { color: var(--bad); }

.composer__attach {
  cursor: pointer;
  background: transparent;
  font-family: var(--hand);
  font-size: 12px;
  line-height: 1;
}
.composer__attach:hover { border-color: var(--amber); color: var(--ink); }

.composer__file-input { display: none; }

.composer__textarea {
  flex: 1;
  background: transparent;
  border: 0;
  outline: none;
  resize: none;
  font-family: var(--hand);
  font-size: 17px;
  color: var(--ink);
  line-height: 1.5;
  padding: 0;
  min-height: 30px;
  max-height: 240px;
  overflow-y: auto;
  font-weight: var(--fw-regular);
}
.composer__textarea::placeholder {
  color: var(--ink-faint);
  font-family: var(--hand);
  font-style: normal;
  font-weight: var(--fw-regular);
}
.composer__textarea:disabled { color: var(--ink-faint); cursor: not-allowed; }

.composer__footer {
  display: flex;
  align-items: center;
  gap: 8px;
  padding-top: 8px;
  border-top: 1px dashed var(--paper-line);
}
.composer__model {
  display: inline-flex;
  align-items: baseline;
  gap: 7px;
  padding: 3px 8px 3px 6px;
  border: 1px solid transparent;
  border-radius: 4px;
  background: transparent;
  cursor: pointer;
  font-family: var(--hand);
  color: var(--ink-soft);
  -webkit-app-region: no-drag;
  transition: border-color 0.15s ease, background 0.15s ease, color 0.15s ease;
}
.composer__model:hover {
  border-color: var(--paper-line);
  background: var(--paper-shade);
  color: var(--ink);
}
.composer__model:focus { outline: none; }
.composer__model:focus-visible {
  border-color: var(--ink-faint);
  background: var(--paper-shade);
}
.composer__model-label {
  font-family: var(--label);
  font-size: var(--fs-micro);
  letter-spacing: 0.16em;
  text-transform: uppercase;
  color: var(--ink-faint);
  font-weight: var(--fw-semibold);
}
.composer__model-name {
  font-size: var(--fs-meta);
  font-weight: var(--fw-medium);
  color: inherit;
}
.composer__model-caret {
  font-size: 9px;
  color: var(--ink-faint);
}
.composer__grow { flex: 1; }
.composer__hint {
  font-family: var(--label);
  font-size: var(--fs-micro);
  letter-spacing: 0.08em;
  color: var(--ink-faint);
  padding: 1px 5px;
  border: 1px solid var(--ink-faint);
  border-radius: 3px;
}

/* Send / stop buttons — solid-ink primary, irregular radius. */
.composer__send {
  border: 1.5px solid var(--ink);
  background: var(--ink);
  color: var(--paper);
  padding: 8px 16px;
  border-radius: 4px 6px 5px 7px / 6px 5px 7px 4px;
  font-family: var(--hand);
  font-size: var(--fs-body);
  font-weight: var(--fw-semibold);
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  gap: 8px;
  transition: opacity 0.15s ease, background 0.15s ease, border-color 0.15s ease;
  letter-spacing: 0.02em;
}
.composer__send:disabled {
  opacity: 0.5;
  cursor: not-allowed;
  background: transparent;
  color: var(--ink-faint);
  border-color: var(--ink-ghost);
}
.composer__send:not(:disabled):hover {
  background: var(--amber);
  border-color: var(--amber);
}

.composer__send--stop {
  background: var(--bad);
  border-color: var(--bad);
}
.composer__send--stop:hover { background: #8a3434; border-color: #8a3434; }

.composer__send-kbd {
  font-family: var(--label);
  font-size: var(--fs-micro);
  padding: 1px 5px;
  border: 1px solid rgba(244, 241, 234, 0.32);
  border-radius: 3px;
  background: rgba(244, 241, 234, 0.10);
  opacity: 0.85;
  letter-spacing: 0.04em;
}
.composer__send:disabled .composer__send-kbd {
  opacity: 0.5;
  border-color: var(--ink-ghost);
  background: transparent;
}
</style>
