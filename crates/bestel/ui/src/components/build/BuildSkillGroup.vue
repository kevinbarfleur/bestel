<script setup lang="ts">
import { computed } from 'vue';

import type { SkillGemDto, SkillGroupDto } from '../../api/types';
import SlotIcon from './SlotIcon.vue';

const props = defineProps<{ group: SkillGroupDto }>();

const isSupport = (gem: SkillGemDto): boolean => {
  const n = gem.name.toLowerCase();
  return n.endsWith(' support') || n.startsWith('support');
};

interface Buckets {
  active: SkillGemDto[];
  supports: SkillGemDto[];
}

const buckets = computed<Buckets>(() => {
  const active: SkillGemDto[] = [];
  const supports: SkillGemDto[] = [];
  for (const g of props.group.gems) {
    if (isSupport(g)) supports.push(g);
    else active.push(g);
  }
  return { active, supports };
});

const cleanName = (gem: SkillGemDto): string =>
  gem.name.replace(/\s+Support$/i, '').trim();

const tooltipFor = (gem: SkillGemDto): string => {
  const parts: string[] = [];
  if (gem.level != null) parts.push(`lvl ${gem.level}`);
  if (gem.quality != null && gem.quality > 0) parts.push(`qual. ${gem.quality}%`);
  if (!gem.enabled) parts.push('disabled');
  return parts.length ? parts.join(' · ') : 'gem';
};
</script>

<template>
  <article class="skg" :class="{ 'skg--main': group.is_main }">
    <header class="skg__head">
      <SlotIcon v-if="group.slot" :slot="group.slot" />
      <span class="skg__label">{{ group.label }}</span>
      <span v-if="group.is_main" class="skg__main-tag">★ principal</span>
    </header>

    <div class="skg__active">
      <span
        v-for="g in buckets.active"
        :key="g.name"
        class="skg__gem skg__gem--active"
        :class="{ 'skg__gem--off': !g.enabled }"
        :data-tooltip-text="tooltipFor(g)"
        :data-tooltip-title="g.name"
      >
        <span class="skg__rune">◆</span>
        <span class="skg__gem-name">{{ cleanName(g) }}</span>
        <span v-if="g.level != null" class="skg__gem-meta">{{ g.level }}</span>
      </span>
    </div>

    <ul v-if="buckets.supports.length" class="skg__supports">
      <li
        v-for="g in buckets.supports"
        :key="g.name"
        class="skg__gem skg__gem--support"
        :class="{ 'skg__gem--off': !g.enabled }"
        :data-tooltip-text="tooltipFor(g)"
        :data-tooltip-title="g.name + ' (Support)'"
      >
        <span class="skg__link">↳</span>
        <span class="skg__gem-name">{{ cleanName(g) }}</span>
        <span v-if="g.level != null" class="skg__gem-meta">{{ g.level }}</span>
      </li>
    </ul>
  </article>
</template>

<style scoped>
.skg {
  display: flex;
  flex-direction: column;
  gap: 0.4rem;
  padding: 0.5rem 0.55rem 0.45rem;
  background: rgba(15, 14, 12, 0.55);
  border: 1px solid rgba(50, 46, 42, 0.45);
  border-radius: 5px;
}

.skg--main {
  border-color: rgba(175, 96, 37, 0.5);
  background:
    radial-gradient(circle at top left, rgba(175, 96, 37, 0.07) 0%, transparent 50%),
    rgba(20, 18, 15, 0.7);
  box-shadow: inset 0 1px 0 rgba(175, 96, 37, 0.12);
}

.skg__head {
  display: flex;
  align-items: center;
  gap: 0.4rem;
}

.skg__label {
  font-family: 'Cinzel', serif;
  font-size: 0.7rem;
  letter-spacing: 0.1em;
  color: var(--color-accent-light, #c97a3a);
  text-transform: uppercase;
  flex: 1;
}

.skg--main .skg__label { color: var(--color-gold, #c9a227); }

.skg__main-tag {
  font-family: 'Cinzel', serif;
  font-size: 0.6rem;
  letter-spacing: 0.1em;
  color: var(--color-gold, #c9a227);
  padding: 0.05rem 0.35rem;
  border: 1px solid rgba(201, 162, 39, 0.4);
  border-radius: 3px;
  background: rgba(201, 162, 39, 0.08);
}

.skg__active {
  display: flex;
  flex-wrap: wrap;
  gap: 0.3rem;
}

.skg__gem {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  padding: 0.18rem 0.45rem;
  font-family: 'Crimson Text', serif;
  font-size: 0.85rem;
  border-radius: 3px;
  border: 1px solid rgba(60, 55, 50, 0.5);
  background: rgba(20, 18, 16, 0.7);
  color: var(--color-text-primary, #e8e6e3);
}

.skg__gem--active {
  border-color: rgba(175, 96, 37, 0.6);
  background:
    repeating-linear-gradient(
      -45deg,
      transparent,
      transparent 4px,
      rgba(175, 96, 37, 0.08) 4px,
      rgba(175, 96, 37, 0.08) 8px
    ),
    rgba(25, 22, 18, 0.85);
  font-family: 'Cinzel', serif;
  font-size: 0.78rem;
  letter-spacing: 0.05em;
  color: var(--color-gold, #c9a227);
}

.skg__gem--off {
  opacity: 0.4;
  text-decoration: line-through;
}

.skg__rune {
  color: var(--color-accent, #af6025);
  font-size: 0.65rem;
}

.skg__gem-meta {
  margin-left: auto;
  font-family: 'JetBrains Mono', 'Consolas', monospace;
  font-size: 0.7rem;
  color: var(--color-text-dim, #7f7f7f);
  font-variant-numeric: tabular-nums;
}

.skg__supports {
  margin: 0;
  padding-left: 0.85rem;
  list-style: none;
  display: flex;
  flex-direction: column;
  gap: 0.2rem;
  border-left: 1px dashed rgba(175, 96, 37, 0.28);
  margin-left: 0.4rem;
}

.skg__gem--support {
  font-size: 0.82rem;
  background: rgba(15, 13, 11, 0.6);
  border-color: rgba(50, 46, 42, 0.4);
  color: var(--color-text, #c8c8c8);
}

.skg__link {
  color: var(--color-accent, #af6025);
  font-family: 'Crimson Text', serif;
  font-size: 0.85rem;
}
</style>
