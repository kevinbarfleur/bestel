<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue';
import { storeToRefs } from 'pinia';

import { useBuildStore } from '../../stores/build';
import { useToastsStore } from '../../stores/toasts';
import RunicInput from '../runic/RunicInput.vue';
import RunicModal from '../runic/RunicModal.vue';

const props = defineProps<{ modelValue: boolean }>();
const emit = defineEmits<{ 'update:modelValue': [value: boolean] }>();

const buildStore = useBuildStore();
const { list, loadingList } = storeToRefs(buildStore);
const toasts = useToastsStore();

const search = ref('');
const highlighted = ref(0);
const inputRef = ref<InstanceType<typeof RunicInput> | null>(null);

const filtered = computed(() => {
  const q = search.value.trim().toLowerCase();
  if (!q) return list.value;
  return list.value.filter((b) => {
    const haystack = [
      b.header,
      b.file_name,
      b.class,
      b.ascendancy ?? '',
      b.main_skill_hint ?? '',
    ]
      .join(' ')
      .toLowerCase();
    return haystack.includes(q);
  });
});

watch(filtered, () => {
  highlighted.value = 0;
});

const close = () => emit('update:modelValue', false);

const choose = async (path: string, label: string) => {
  const result = await buildStore.setActive(path);
  if (result) {
    toasts.push({ variant: 'success', title: 'Build loaded', body: label });
    close();
  } else {
    toasts.push({ variant: 'error', title: 'Failed to load build', body: label });
  }
};

const handleKey = (e: KeyboardEvent) => {
  if (e.key === 'ArrowDown') {
    e.preventDefault();
    highlighted.value = Math.min(highlighted.value + 1, filtered.value.length - 1);
  } else if (e.key === 'ArrowUp') {
    e.preventDefault();
    highlighted.value = Math.max(highlighted.value - 1, 0);
  } else if (e.key === 'Enter') {
    const target = filtered.value[highlighted.value];
    if (target) {
      e.preventDefault();
      void choose(target.path, target.header);
    }
  }
};

watch(
  () => props.modelValue,
  async (open) => {
    if (!open) return;
    search.value = '';
    highlighted.value = 0;
    await buildStore.refreshList();
    await nextTick();
    inputRef.value?.focus();
  },
);
</script>

<template>
  <RunicModal
    :model-value="modelValue"
    title="Select a PoB build"
    icon="◆"
    max-width="lg"
    @update:model-value="(v) => emit('update:modelValue', v)"
  >
    <div class="build-picker" @keydown="handleKey">
      <RunicInput
        ref="inputRef"
        v-model="search"
        icon="search"
        placeholder="Filter by class, ascendancy, skill, file…"
      />

      <p v-if="loadingList" class="build-picker__hint">Loading…</p>
      <p v-else-if="!filtered.length" class="build-picker__hint">No builds found.</p>

      <ul class="build-picker__list runic-scrollbar" role="listbox">
        <li
          v-for="(b, idx) in filtered"
          :key="b.path"
          class="build-picker__row"
          :class="{ 'build-picker__row--active': idx === highlighted }"
          role="option"
          :aria-selected="idx === highlighted"
          @click="choose(b.path, b.header)"
          @mouseenter="highlighted = idx"
        >
          <span class="build-picker__rune">◆</span>
          <span class="build-picker__title">{{ b.header }}</span>
          <span class="build-picker__file">{{ b.file_name }}</span>
        </li>
      </ul>
    </div>
  </RunicModal>
</template>

<style scoped>
.build-picker {
  display: flex;
  flex-direction: column;
  gap: 0.85rem;
  min-height: 320px;
}

.build-picker__hint {
  margin: 0;
  text-align: center;
  font-family: var(--hand);
  color: var(--ink-faint);
}

.build-picker__list {
  margin: 0;
  padding: 0;
  list-style: none;
  display: flex;
  flex-direction: column;
  gap: 0.3rem;
  max-height: 50vh;
  overflow-y: auto;
}

.build-picker__row {
  display: flex;
  align-items: baseline;
  gap: 0.6rem;
  padding: 0.5rem 0.4rem;
  border: 0;
  border-bottom: 1px solid var(--paper-line);
  background: transparent;
  cursor: pointer;
  font-family: var(--hand);
  font-size: 13px;
  color: var(--ink);
  transition: background 0.15s ease;
}

.build-picker__row:hover,
.build-picker__row--active {
  background: var(--paper-shade);
}
.build-picker__row--active .build-picker__title {
  color: var(--amber);
}

.build-picker__rune {
  color: var(--amber);
  font-size: 0.75rem;
  opacity: 0.55;
}

.build-picker__title {
  flex: 1;
  font-family: var(--hand-display);
  font-size: 14px;
  font-weight: 600;
  color: var(--ink);
}

.build-picker__file {
  font-family: var(--hand);
  font-size: 11px;
  color: var(--ink-faint);
}
</style>
