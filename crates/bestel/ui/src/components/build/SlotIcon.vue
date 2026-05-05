<script setup lang="ts">
import { computed } from 'vue';

const props = defineProps<{ slot: string | null | undefined }>();

const kind = computed<string>(() => {
  const s = (props.slot ?? '').toLowerCase().trim();
  if (s.includes('helmet')) return 'helmet';
  if (s.includes('body')) return 'body';
  if (s.includes('glove')) return 'gloves';
  if (s.includes('boot')) return 'boots';
  if (s.includes('belt')) return 'belt';
  if (s.includes('amulet')) return 'amulet';
  if (s.includes('ring')) return 'ring';
  if (s.includes('weapon 1') || s.includes('weapon 1 swap')) return 'weapon';
  if (s.includes('weapon 2') || s.includes('weapon 2 swap')) return 'shield';
  if (s.includes('flask')) return 'flask';
  if (s.includes('charm')) return 'charm';
  if (s.includes('jewel')) return 'jewel';
  return 'generic';
});
</script>

<template>
  <span class="slot-icon" :title="slot ?? ''" aria-hidden="true">
    <!-- Helmet : arc + plumet -->
    <svg v-if="kind === 'helmet'" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="1.6">
      <path d="M4 14c0-4.4 3.6-8 8-8s8 3.6 8 8v3H4v-3z" stroke-linejoin="round" />
      <path d="M12 6V3M9 7l-1-2M15 7l1-2" stroke-linecap="round" />
    </svg>
    <!-- Body : torse -->
    <svg v-else-if="kind === 'body'" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="1.6">
      <path d="M6 5l3-2h6l3 2v15a1 1 0 01-1 1H7a1 1 0 01-1-1V5z" stroke-linejoin="round" />
      <path d="M9 3v3l3 2 3-2V3" stroke-linejoin="round" />
    </svg>
    <!-- Gloves -->
    <svg v-else-if="kind === 'gloves'" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="1.6">
      <path d="M7 5h2v6m0 0V8a1 1 0 112 0v3m0 0V7a1 1 0 112 0v4m0 0V8a1 1 0 112 0v6c0 3-2 6-5 6h-1c-3 0-5-3-5-6V5h2" stroke-linejoin="round" stroke-linecap="round" />
    </svg>
    <!-- Boots -->
    <svg v-else-if="kind === 'boots'" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="1.6">
      <path d="M7 4h4v8c0 0 6 0 7 4v3H4v-3l3-1V4z" stroke-linejoin="round" />
    </svg>
    <!-- Belt -->
    <svg v-else-if="kind === 'belt'" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="1.6">
      <rect x="3" y="9" width="18" height="6" rx="1" />
      <rect x="10" y="11" width="4" height="2" />
    </svg>
    <!-- Amulet -->
    <svg v-else-if="kind === 'amulet'" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="1.6">
      <path d="M6 4l6 4 6-4" stroke-linecap="round" />
      <circle cx="12" cy="15" r="5" />
      <path d="M11 13l1 2 1-2" stroke-linecap="round" stroke-linejoin="round" />
    </svg>
    <!-- Ring -->
    <svg v-else-if="kind === 'ring'" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="1.6">
      <circle cx="12" cy="14" r="6" />
      <path d="M9 8l3-4 3 4" stroke-linejoin="round" />
    </svg>
    <!-- Weapon (épée) -->
    <svg v-else-if="kind === 'weapon'" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="1.6">
      <path d="M14 3l5 5-9 9-2-2 1-3-3 1-2-2 9-9z" stroke-linejoin="round" stroke-linecap="round" />
      <path d="M5 19l3-3" stroke-linecap="round" />
    </svg>
    <!-- Shield -->
    <svg v-else-if="kind === 'shield'" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="1.6">
      <path d="M5 4l7-2 7 2v8c0 5-3.5 8-7 10-3.5-2-7-5-7-10V4z" stroke-linejoin="round" />
    </svg>
    <!-- Flask -->
    <svg v-else-if="kind === 'flask'" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="1.6">
      <path d="M9 3h6v3l4 11a3 3 0 01-3 4H8a3 3 0 01-3-4l4-11V3z" stroke-linejoin="round" />
      <path d="M8 3h8" stroke-linecap="round" />
    </svg>
    <!-- Charm -->
    <svg v-else-if="kind === 'charm'" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="1.6">
      <path d="M12 3l3 6 6 1-4.5 4 1 6L12 17l-5.5 3 1-6L3 10l6-1z" stroke-linejoin="round" />
    </svg>
    <!-- Jewel -->
    <svg v-else-if="kind === 'jewel'" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="1.6">
      <path d="M12 3l8 8-8 10-8-10z" stroke-linejoin="round" />
      <path d="M4 11h16M12 3v18" />
    </svg>
    <!-- Generic -->
    <svg v-else viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="1.6">
      <circle cx="12" cy="12" r="8" />
    </svg>
  </span>
</template>

<style scoped>
.slot-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  color: var(--color-accent-light, #c97a3a);
  flex-shrink: 0;
}
</style>
