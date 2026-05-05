<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref } from 'vue';

interface Option {
  value: string;
  label: string;
  description?: string;
  count?: number;
  standardCount?: number;
  foilCount?: number;
  synthesisedCount?: number;
}

interface Props {
  options: Option[];
  modelValue: string | string[];
  placeholder?: string;
  size?: 'sm' | 'md' | 'lg';
  label?: string;
  maxVisibleItems?: number;
  searchable?: boolean;
  multiple?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  placeholder: undefined,
  size: 'md',
  maxVisibleItems: 8,
  searchable: false,
  multiple: false,
});

const displayPlaceholder = computed(() => props.placeholder ?? 'Sélectionner');

const emit = defineEmits<{
  'update:modelValue': [value: string | string[]];
}>();

const isOpen = ref(false);
const selectRef = ref<HTMLElement | null>(null);
const triggerRef = ref<HTMLElement | null>(null);
const dropdownRef = ref<HTMLElement | null>(null);
const searchInputRef = ref<HTMLInputElement | null>(null);
const searchQuery = ref('');

const dropdownPosition = ref({ top: 0, left: 0, width: 0 });

const selectedValues = computed<string[]>(() => {
  if (props.multiple) {
    return Array.isArray(props.modelValue) ? props.modelValue : [];
  }
  return props.modelValue ? [props.modelValue as string] : [];
});

const displayText = computed(() => {
  if (selectedValues.value.length === 0) return displayPlaceholder.value;
  if (props.multiple) {
    if (selectedValues.value.length === 1) {
      const opt = props.options.find((o) => o.value === selectedValues.value[0]);
      return opt?.label || selectedValues.value[0];
    }
    return `${selectedValues.value.length} sélectionnés`;
  }
  const opt = props.options.find((o) => o.value === props.modelValue);
  return opt?.label || displayPlaceholder.value;
});

const processedOptions = computed(() => {
  let opts = [...props.options];
  if (props.searchable && searchQuery.value) {
    const query = searchQuery.value.toLowerCase();
    opts = opts.filter(
      (opt) =>
        opt.label.toLowerCase().includes(query) ||
        opt.value.toLowerCase().includes(query),
    );
  }
  if (props.multiple) {
    opts.sort((a, b) => {
      const aSelected = selectedValues.value.includes(a.value);
      const bSelected = selectedValues.value.includes(b.value);
      if (aSelected && !bSelected) return -1;
      if (!aSelected && bSelected) return 1;
      return 0;
    });
  }
  return opts;
});

const isSelected = (value: string) => selectedValues.value.includes(value);

const itemHeight = computed(() => {
  switch (props.size) {
    case 'sm': return 48;
    case 'lg': return 64;
    default: return 56;
  }
});

const searchInputHeight = computed(() => (props.searchable ? 52 : 0));

const maxDropdownHeight = computed(
  () => props.maxVisibleItems * itemHeight.value + 8 + searchInputHeight.value,
);

const actualDropdownHeight = computed(() => {
  const actualItemCount = processedOptions.value.length;
  const contentHeight = actualItemCount * itemHeight.value + 8 + searchInputHeight.value;
  return Math.min(contentHeight, maxDropdownHeight.value);
});

const updateDropdownPosition = () => {
  if (!triggerRef.value) return;
  const rect = triggerRef.value.getBoundingClientRect();
  const viewportWidth = window.innerWidth;
  const viewportHeight = window.innerHeight;
  const spaceBelow = viewportHeight - rect.bottom;
  const spaceAbove = rect.top;
  const dropdownHeight = actualDropdownHeight.value;
  const openAbove = spaceBelow < dropdownHeight && spaceAbove > spaceBelow;
  const maxWidth = viewportWidth - 16;
  const dropdownWidth = Math.min(rect.width, maxWidth);
  let left = rect.left;
  if (left + dropdownWidth > viewportWidth - 8) left = viewportWidth - dropdownWidth - 8;
  if (left < 8) left = 8;
  dropdownPosition.value = {
    top: openAbove
      ? rect.top - Math.min(dropdownHeight, spaceAbove - 10) - 4
      : rect.bottom + 4,
    left,
    width: dropdownWidth,
  };
};

const toggleDropdown = () => {
  if (!isOpen.value) {
    updateDropdownPosition();
    searchQuery.value = '';
  }
  isOpen.value = !isOpen.value;
  if (isOpen.value && props.searchable) {
    nextTick(() => searchInputRef.value?.focus());
  }
};

const selectOption = (value: string) => {
  if (props.multiple) {
    const newValues = [...selectedValues.value];
    const index = newValues.indexOf(value);
    if (index === -1) newValues.push(value);
    else newValues.splice(index, 1);
    emit('update:modelValue', newValues);
  } else {
    emit('update:modelValue', value);
    isOpen.value = false;
  }
};

const clearAll = () => {
  if (props.multiple) emit('update:modelValue', []);
};

const handleClickOutside = (event: MouseEvent) => {
  const target = event.target as Node;
  if (
    selectRef.value &&
    !selectRef.value.contains(target) &&
    dropdownRef.value &&
    !dropdownRef.value.contains(target)
  ) {
    isOpen.value = false;
  }
};

const handleScrollResize = () => {
  if (isOpen.value) updateDropdownPosition();
};

onMounted(() => {
  document.addEventListener('click', handleClickOutside);
  window.addEventListener('scroll', handleScrollResize, true);
  window.addEventListener('resize', handleScrollResize);
});

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside);
  window.removeEventListener('scroll', handleScrollResize, true);
  window.removeEventListener('resize', handleScrollResize);
});
</script>

<template>
  <div
    ref="selectRef"
    class="runic-select"
    :class="[
      `runic-select--${size}`,
      {
        'runic-select--open': isOpen,
        'runic-select--multiple': multiple,
        'runic-select--has-selection': selectedValues.length > 0,
      },
    ]"
  >
    <label v-if="label" class="runic-select__label">{{ label }}</label>

    <button
      ref="triggerRef"
      type="button"
      class="runic-select__trigger"
      :aria-expanded="isOpen"
      @click="toggleDropdown"
    >
      <span class="runic-select__rune runic-select__rune--tl">◆</span>
      <span class="runic-select__rune runic-select__rune--tr">◆</span>
      <span class="runic-select__rune runic-select__rune--bl">◆</span>
      <span class="runic-select__rune runic-select__rune--br">◆</span>

      <span class="runic-select__value">{{ displayText }}</span>

      <span v-if="multiple && selectedValues.length > 0" class="runic-select__badge">
        {{ selectedValues.length }}
      </span>

      <button
        v-if="multiple && selectedValues.length > 0"
        type="button"
        class="runic-select__clear"
        @click.stop="clearAll"
      >
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
        </svg>
      </button>

      <svg class="runic-select__chevron" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path stroke-linecap="round" stroke-linejoin="round" d="M19 9l-7 7-7-7" />
      </svg>

      <div class="runic-select__glow"></div>
    </button>

    <Teleport to="body">
      <Transition name="dropdown">
        <div
          v-if="isOpen"
          ref="dropdownRef"
          class="runic-select__dropdown"
          :class="[
            `runic-select__dropdown--${size}`,
            { 'runic-select__dropdown--searchable': searchable },
          ]"
          :style="{
            position: 'fixed',
            top: `${dropdownPosition.top}px`,
            left: `${dropdownPosition.left}px`,
            width: `${dropdownPosition.width}px`,
            maxHeight: `${maxDropdownHeight}px`,
          }"
        >
          <div v-if="searchable" class="runic-select__search">
            <svg class="runic-select__search-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path stroke-linecap="round" stroke-linejoin="round" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
            </svg>
            <input
              ref="searchInputRef"
              v-model="searchQuery"
              type="text"
              class="runic-select__search-input"
              placeholder="Rechercher…"
              @click.stop
            />
            <button
              v-if="searchQuery"
              type="button"
              class="runic-select__search-clear"
              @click.stop="searchQuery = ''"
            >
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>

          <div class="runic-select__dropdown-inner">
            <div v-if="processedOptions.length === 0" class="runic-select__empty">
              Aucun résultat
            </div>

            <button
              v-for="option in processedOptions"
              :key="option.value"
              type="button"
              class="runic-select__option"
              :class="{
                'runic-select__option--selected': isSelected(option.value),
                'runic-select__option--pinned': multiple && isSelected(option.value),
              }"
              @click="selectOption(option.value)"
            >
              <span v-if="multiple" class="runic-select__checkbox">
                <svg v-if="isSelected(option.value)" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3">
                  <path stroke-linecap="round" stroke-linejoin="round" d="M5 13l4 4L19 7" />
                </svg>
              </span>

              <span v-if="!multiple" class="runic-select__check-container">
                <svg
                  v-if="isSelected(option.value)"
                  class="runic-select__check"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2.5"
                >
                  <path stroke-linecap="round" stroke-linejoin="round" d="M5 13l4 4L19 7" />
                </svg>
              </span>

              <span class="runic-select__option-content">
                <span class="runic-select__option-label">{{ option.label }}</span>
                <span v-if="option.description" class="runic-select__option-desc">
                  {{ option.description }}
                </span>
              </span>

              <span
                v-if="option.standardCount || option.foilCount || option.synthesisedCount"
                class="runic-select__option-tags"
              >
                <span v-if="option.standardCount" class="runic-select__tag runic-select__tag--standard">
                  {{ option.standardCount }}
                </span>
                <span v-if="option.foilCount" class="runic-select__tag runic-select__tag--foil">
                  {{ option.foilCount }}
                </span>
                <span v-if="option.synthesisedCount" class="runic-select__tag runic-select__tag--synthesised">
                  {{ option.synthesisedCount }}
                </span>
              </span>

              <span
                v-if="option.count !== undefined && option.count > 1"
                class="runic-select__option-count"
              >
                ×{{ option.count }}
              </span>
            </button>
          </div>
        </div>
      </Transition>
    </Teleport>
  </div>
</template>

<style scoped>
.runic-select {
  position: relative;
  display: inline-flex;
  flex-direction: column;
  width: 100%;
  font-family: var(--hand);
}

.runic-select__label {
  display: block;
  margin-bottom: 0.5rem;
  font-family: var(--label);
  font-size: 0.875rem;
  font-weight: 600;
  letter-spacing: 0.05em;
  text-transform: uppercase;
  color: rgba(140, 130, 120, 0.8);
}

.runic-select__trigger {
  position: relative;
  display: flex;
  align-items: center;
  gap: 0.5rem;
  width: 100%;
  cursor: pointer;
  text-align: left;
  background: var(--paper);
  border-radius: 4px 7px 5px 6px / 6px 4px 7px 5px;
  border: 1.4px solid var(--ink-faint);
  overflow: hidden;
  transition: border-color 0.18s ease, background 0.18s ease;
}

.runic-select__trigger:hover,
.runic-select--open .runic-select__trigger {
  border-color: var(--amber);
  background: var(--paper-shade);
}

.runic-select__rune {
  position: absolute;
  font-size: 0.55rem;
  color: rgba(175, 96, 37, 0.25);
  transition: color 0.18s ease;
  pointer-events: none;
  z-index: 2;
}
.runic-select__rune--tl { top: 4px; left: 6px; }
.runic-select__rune--tr { top: 4px; right: 6px; }
.runic-select__rune--bl { bottom: 4px; left: 6px; }
.runic-select__rune--br { bottom: 4px; right: 6px; }

.runic-select__trigger:hover .runic-select__rune,
.runic-select--open .runic-select__rune {
  color: var(--amber);
}

.runic-select__value {
  flex: 1;
  color: var(--ink);
  font-family: var(--hand);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.runic-select__badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 20px;
  height: 20px;
  padding: 0 6px;
  font-family: var(--label);
  font-size: 11px;
  font-weight: 700;
  color: var(--paper);
  background: var(--amber);
  border-radius: 10px;
}

.runic-select__clear {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
  padding: 0;
  background: var(--paper-shade);
  border: 1px solid var(--ink-faint);
  border-radius: 50%;
  color: var(--ink-soft);
  cursor: pointer;
  transition: all 0.15s ease;
}
.runic-select__clear:hover {
  background: rgba(175, 96, 37, 0.12);
  border-color: var(--amber);
  color: var(--amber);
}
.runic-select__clear svg { width: 12px; height: 12px; }

.runic-select__chevron {
  flex-shrink: 0;
  width: 16px;
  height: 16px;
  color: var(--ink-faint);
  transition: transform 0.18s ease, color 0.18s ease;
}
.runic-select--open .runic-select__chevron {
  transform: rotate(180deg);
  color: var(--amber);
}

.runic-select__glow {
  position: absolute;
  bottom: 0;
  left: 50%;
  transform: translateX(-50%);
  width: 0%;
  height: 1px;
  background: linear-gradient(90deg, transparent, var(--amber), transparent);
  transition: width 0.3s ease;
  opacity: 0.7;
}
.runic-select__trigger:hover .runic-select__glow,
.runic-select--open .runic-select__glow { width: 80%; }
.runic-select--has-selection .runic-select__glow { width: 40%; }

.runic-select__dropdown {
  z-index: 10001;
  background: var(--paper);
  border-radius: 4px 7px 5px 6px / 6px 4px 7px 5px;
  border: 1.4px solid var(--ink-soft);
  box-shadow:
    0 12px 32px rgba(40, 35, 30, 0.22),
    0 4px 10px rgba(40, 35, 30, 0.14);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  box-sizing: border-box;
}
.runic-select__dropdown *,
.runic-select__dropdown *::before,
.runic-select__dropdown *::after { box-sizing: border-box; }

.runic-select__search {
  position: relative;
  display: flex;
  align-items: center;
  padding: 8px 12px;
  border-bottom: 1px dashed var(--paper-line);
  background: var(--paper-shade);
  width: 100%;
}

.runic-select__search-icon {
  flex-shrink: 0;
  width: 16px;
  height: 16px;
  color: var(--ink-faint);
  margin-right: 8px;
}

.runic-select__search-input {
  flex: 1 1 0%;
  min-width: 0;
  background: transparent;
  border: none;
  outline: none;
  font-family: var(--hand);
  font-size: 14px;
  color: var(--ink);
}
.runic-select__search-input::placeholder {
  color: var(--ink-faint);
  font-style: italic;
}

.runic-select__search-clear {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 18px;
  height: 18px;
  padding: 0;
  background: var(--paper);
  border: 1px solid var(--ink-faint);
  border-radius: 50%;
  color: var(--ink-soft);
  cursor: pointer;
  transition: all 0.15s ease;
}
.runic-select__search-clear:hover {
  background: rgba(175, 96, 37, 0.12);
  border-color: var(--amber);
  color: var(--amber);
}
.runic-select__search-clear svg { width: 10px; height: 10px; }

.runic-select__dropdown-inner {
  overflow-y: auto;
  overflow-x: hidden;
  flex: 1;
  min-height: 0;
  padding: 4px;
  text-align: left;
}
.runic-select__dropdown-inner::-webkit-scrollbar { width: 6px; }
.runic-select__dropdown-inner::-webkit-scrollbar-track {
  background: var(--paper-shade);
  border-radius: 3px;
}
.runic-select__dropdown-inner::-webkit-scrollbar-thumb {
  background: var(--ink-faint);
  border-radius: 3px;
}
.runic-select__dropdown-inner::-webkit-scrollbar-thumb:hover {
  background: var(--ink-soft);
}

.runic-select__empty {
  padding: 1rem;
  text-align: center;
  font-family: var(--hand);
  font-size: 14px;
  font-style: italic;
  color: var(--ink-faint);
}

.runic-select__option {
  display: flex;
  align-items: center;
  gap: 10px;
  width: 100%;
  padding: 0.75rem 1rem;
  background: transparent;
  border: 1px solid transparent;
  border-radius: 3px;
  cursor: pointer;
  text-align: left;
  position: relative;
  transition: all 0.2s ease;
  overflow: hidden;
  box-sizing: border-box;
}

.runic-select__option:hover {
  border-color: var(--amber);
  background: var(--paper-shade);
}

.runic-select__option:hover::before,
.runic-select__option:hover::after {
  content: "◆";
  position: absolute;
  font-size: 0.45rem;
  color: var(--amber);
  opacity: 0.6;
}
.runic-select__option:hover::before { top: 50%; left: 4px; transform: translateY(-50%); }
.runic-select__option:hover::after { top: 50%; right: 4px; transform: translateY(-50%); }

.runic-select__option--selected {
  background:
    repeating-linear-gradient(
      -45deg,
      transparent 0 4px,
      rgba(175, 96, 37, 0.08) 4px 5px
    ),
    var(--paper);
  border-color: var(--amber);
}

.runic-select__option--selected:hover {
  background:
    repeating-linear-gradient(
      -45deg,
      transparent 0 4px,
      rgba(175, 96, 37, 0.12) 4px 5px
    ),
    var(--paper-shade);
  border-color: var(--amber);
}

.runic-select__option--pinned {
  border-left: 2px solid var(--amber);
  padding-left: calc(1rem - 2px);
}

.runic-select__checkbox {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 18px;
  height: 18px;
  flex-shrink: 0;
  background: var(--paper);
  border: 1.4px solid var(--ink-soft);
  border-radius: 3px;
  transition: all 0.15s ease;
}
.runic-select__option--selected .runic-select__checkbox {
  background: var(--amber);
  border-color: var(--amber);
}
.runic-select__checkbox svg { width: 12px; height: 12px; color: var(--paper); }

.runic-select__option-content {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  flex: 1 1 0%;
  min-width: 0;
  overflow: hidden;
}

.runic-select__option-label {
  display: block;
  font-family: var(--hand);
  font-size: 14px;
  color: var(--ink);
  transition: color 0.15s ease;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  width: 100%;
  text-align: left;
}
.runic-select__option:hover .runic-select__option-label { color: var(--ink); }
.runic-select__option--selected .runic-select__option-label { color: var(--amber); font-weight: 600; }

.runic-select__option-desc {
  font-family: var(--hand);
  font-size: 12px;
  color: var(--ink-faint);
  margin-top: 2px;
  font-style: italic;
}

.runic-select__option-count {
  flex-shrink: 0;
  font-family: var(--label);
  font-size: 11px;
  font-weight: 700;
  color: var(--amber);
  background: rgba(175, 96, 37, 0.1);
  padding: 2px 7px;
  border-radius: 4px;
  border: 1px solid var(--amber);
  letter-spacing: 0.02em;
}
.runic-select__option--selected .runic-select__option-count {
  background: var(--amber);
  color: var(--paper);
}

.runic-select__check-container {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 16px;
  height: 16px;
  flex-shrink: 0;
}
.runic-select__check { flex-shrink: 0; width: 16px; height: 16px; color: var(--amber); }

.runic-select__option-tags { display: flex; gap: 4px; flex-shrink: 0; }

.runic-select__tag {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 18px;
  height: 18px;
  padding: 0 5px;
  font-family: var(--label);
  font-size: 0.6875rem;
  font-weight: 600;
  border-radius: 3px;
  letter-spacing: 0.02em;
}

.runic-select__tag--standard {
  color: var(--ink-soft);
  background: var(--paper-shade);
  border: 1px solid var(--ink-faint);
}

.runic-select__tag--foil {
  color: #c9a227;
  background: rgba(201, 162, 39, 0.1);
  border: 1px solid #c9a227;
}

.runic-select__tag--synthesised {
  color: #4f8e8a;
  background: rgba(79, 142, 138, 0.1);
  border: 1px solid #4f8e8a;
}

.runic-select__option--selected .runic-select__tag--standard {
  color: var(--ink);
  border-color: var(--ink-soft);
}
.runic-select__option--selected .runic-select__tag--foil {
  color: var(--paper);
  background: #c9a227;
  border-color: #c9a227;
}
.runic-select__option--selected .runic-select__tag--synthesised {
  color: var(--paper);
  background: #4f8e8a;
  border-color: #4f8e8a;
}

.dropdown-enter-active,
.dropdown-leave-active { transition: all 0.25s ease; }
.dropdown-enter-from,
.dropdown-leave-to { opacity: 0; transform: translateY(-8px); }

.runic-select--sm .runic-select__trigger { min-height: 38px; padding: 0.5rem 1.25rem; }
.runic-select--sm .runic-select__value { font-size: 0.875rem; }
.runic-select--sm .runic-select__rune { font-size: 0.4rem; }
.runic-select--sm .runic-select__option { padding: 0.5rem 0.75rem; gap: 6px; }
.runic-select--sm .runic-select__option-label { font-size: 0.875rem; }
.runic-select__dropdown--sm .runic-select__search { padding: 6px 10px; }
.runic-select__dropdown--sm .runic-select__search-input { font-size: 0.875rem; }

.runic-select--md .runic-select__trigger { min-height: 46px; padding: 0.625rem 1.5rem; }
.runic-select--md .runic-select__value { font-size: 1rem; }

.runic-select--lg .runic-select__trigger { min-height: 54px; padding: 0.75rem 1.75rem; }
.runic-select--lg .runic-select__value { font-size: 1.125rem; }
.runic-select--lg .runic-select__rune { font-size: 0.5625rem; }
.runic-select--lg .runic-select__chevron { width: 20px; height: 20px; }

.runic-select__dropdown--sm .runic-select__option { padding: 0.5rem 0.875rem; }
.runic-select__dropdown--sm .runic-select__option-label { font-size: 0.875rem; }
.runic-select__dropdown--sm .runic-select__option-desc { font-size: 0.9375rem; }
.runic-select__dropdown--lg .runic-select__option { padding: 1rem 1.25rem; }
.runic-select__dropdown--lg .runic-select__option-label { font-size: 1.0625rem; }

@media (max-width: 640px) {
  .runic-select--sm .runic-select__trigger { min-height: 34px; padding: 0.375rem 1rem; }
  .runic-select--md .runic-select__trigger { min-height: 40px; padding: 0.5rem 1rem; }
  .runic-select--lg .runic-select__trigger { min-height: 48px; padding: 0.625rem 1.25rem; }
  .runic-select__rune { display: none; }
  .runic-select__dropdown { max-height: 300px !important; }
}
</style>
