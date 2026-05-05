<script setup lang="ts">
import { computed } from 'vue';

import type { AttachmentDto } from '../../../api/types';

const props = defineProps<{ attachment: AttachmentDto; removable?: boolean }>();
const emit = defineEmits<{ remove: [] }>();

const isImage = computed(() => props.attachment.mime.startsWith('image/'));
const dataUrl = computed(() => `data:${props.attachment.mime};base64,${props.attachment.data_base64}`);

const docExt = computed(() => {
  const m = props.attachment.mime;
  if (m === 'application/pdf') return 'pdf';
  if (m === 'text/markdown') return 'md';
  if (m === 'text/plain') return 'txt';
  return 'doc';
});
</script>

<template>
  <span class="att-chip" :title="attachment.name">
    <span v-if="isImage" class="att-chip__thumb">
      <img :src="dataUrl" :alt="attachment.name" />
    </span>
    <span v-else class="att-chip__icon">{{ docExt }}</span>
    <span class="att-chip__name">{{ attachment.name }}</span>
    <button
      v-if="removable !== false"
      type="button"
      class="att-chip__remove"
      aria-label="Remove attachment"
      @click="emit('remove')"
    >×</button>
  </span>
</template>

<style scoped>
.att-chip {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 3px 6px 3px 4px;
  border: 1.2px solid var(--ink-soft);
  border-radius: 12px;
  background: var(--paper);
  font-family: var(--hand);
  font-size: 11px;
  color: var(--ink);
  max-width: 200px;
}

.att-chip__thumb {
  width: 18px;
  height: 18px;
  border-radius: 3px;
  overflow: hidden;
  flex: none;
  background: var(--paper-shade);
  display: flex;
  align-items: center;
  justify-content: center;
}
.att-chip__thumb img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.att-chip__icon {
  width: 18px;
  height: 18px;
  border-radius: 3px;
  background: var(--paper-shade);
  border: 1px solid var(--ink-faint);
  font-family: var(--label);
  font-size: 8px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--ink-soft);
  display: flex;
  align-items: center;
  justify-content: center;
  flex: none;
}

.att-chip__name {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex: 1;
  min-width: 0;
}

.att-chip__remove {
  width: 14px;
  height: 14px;
  border: 0;
  background: transparent;
  color: var(--ink-faint);
  font-size: 14px;
  line-height: 1;
  cursor: pointer;
  flex: none;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0;
}
.att-chip__remove:hover { color: var(--bad); background: rgba(164, 72, 72, 0.1); }
</style>
