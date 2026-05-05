<script setup lang="ts">
import { computed } from 'vue';

interface GemPayload {
  name: string;
  level: number | null;
  quality: number | null;
  enabled: boolean;
  element: 'fire' | 'cold' | 'lit' | 'chaos' | 'phys';
  is_main: boolean;
  is_aura: boolean;
  attribute: 'str' | 'dex' | 'int' | 'unknown';
  tags: string[];
  description: string | null;
  supports: { name: string; level: number | null; quality: number | null }[];
}

const props = defineProps<{ payload: string }>();

const data = computed<GemPayload>(() => {
  try {
    return JSON.parse(props.payload) as GemPayload;
  } catch {
    return {
      name: '?',
      level: null,
      quality: null,
      enabled: true,
      element: 'phys',
      is_main: false,
      is_aura: false,
      attribute: 'unknown',
      tags: [],
      description: null,
      supports: [],
    };
  }
});

const ATTR_COLOR = {
  str: 'var(--bad)',
  dex: 'var(--good)',
  int: 'var(--el-cold)',
  unknown: 'var(--ink-soft)',
};
const ATTR_LABEL = { str: 'STR', dex: 'DEX', int: 'INT', unknown: 'gem' };

const ELEMENT_COLOR = {
  fire: 'var(--el-fire-deep)',
  cold: 'var(--el-cold-deep)',
  lit: 'var(--el-lit-deep)',
  chaos: 'var(--el-chaos-deep)',
  phys: 'var(--ink)',
};

const attributeColor = computed(() => ATTR_COLOR[data.value.attribute]);
const attributeLabel = computed(() => ATTR_LABEL[data.value.attribute]);
const nameColor = computed(() => ELEMENT_COLOR[data.value.element]);
</script>

<template>
  <div class="gem-tt">
    <div class="gem-tt__band" :style="{ background: attributeColor }">
      <span>{{ attributeLabel }}</span>
      <span v-if="data.is_main" class="gem-tt__band-flag">· main</span>
      <span v-else-if="data.is_aura" class="gem-tt__band-flag">· aura</span>
      <span v-else-if="!data.enabled" class="gem-tt__band-flag">· disabled</span>
    </div>
    <div class="gem-tt__body">
      <div class="gem-tt__name" :style="{ color: nameColor }">{{ data.name }}</div>
      <div v-if="data.tags.length" class="gem-tt__tags">{{ data.tags.join(', ') }}</div>

      <div class="gem-tt__sep" />
      <div class="gem-tt__props">
        <div v-if="data.level != null" class="gem-tt__prop">
          <span class="gem-tt__prop-k">level</span><span class="gem-tt__prop-dots" />
          <span class="gem-tt__prop-v">{{ data.level }}</span>
        </div>
        <div v-if="data.quality != null" class="gem-tt__prop">
          <span class="gem-tt__prop-k">quality</span><span class="gem-tt__prop-dots" />
          <span class="gem-tt__prop-v">+{{ data.quality }}%</span>
        </div>
        <div v-if="data.element !== 'phys'" class="gem-tt__prop">
          <span class="gem-tt__prop-k">element</span><span class="gem-tt__prop-dots" />
          <span class="gem-tt__prop-v" :style="{ color: nameColor }">{{ data.element }}</span>
        </div>
      </div>

      <div v-if="data.description" class="gem-tt__sep" />
      <div v-if="data.description" class="gem-tt__desc">{{ data.description }}</div>

      <div v-if="data.supports.length" class="gem-tt__sep" />
      <div v-if="data.supports.length" class="gem-tt__supports">
        <div class="gem-tt__supports-title">Supports</div>
        <div v-for="(s, i) in data.supports" :key="i" class="gem-tt__support">
          <span class="gem-tt__support-mark">·</span>
          <span class="gem-tt__support-name">{{ s.name }}</span>
          <span v-if="s.level != null || s.quality != null" class="gem-tt__support-meta">
            ({{ [s.level != null ? `lvl ${s.level}` : '', s.quality != null && s.quality > 0 ? `${s.quality}q` : ''].filter(Boolean).join(', ') }})
          </span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.gem-tt {
  font-family: var(--hand);
  margin: -8px -12px;
  padding: 0 0 10px;
  min-width: 240px;
  max-width: 340px;
}
.gem-tt__band {
  padding: 4px 12px;
  font-family: var(--label);
  font-size: 9px;
  letter-spacing: 0.20em;
  text-transform: uppercase;
  font-weight: 700;
  color: var(--paper);
  display: flex;
  align-items: center;
  gap: 6px;
}
.gem-tt__band-flag {
  font-family: var(--hand);
  letter-spacing: 0.05em;
  text-transform: none;
  font-size: 12px;
  opacity: 0.95;
}
.gem-tt__body {
  padding: 8px 12px 0;
}
.gem-tt__name {
  font-family: var(--hand-display);
  font-size: 15px;
  font-weight: 700;
  line-height: 1.2;
}
.gem-tt__tags {
  font-family: var(--label);
  font-size: 9px;
  letter-spacing: 0.10em;
  text-transform: uppercase;
  color: var(--ink-faint);
  margin-top: 4px;
}
.gem-tt__sep {
  height: 1px;
  background: var(--paper-line);
  margin: 6px 0;
}
.gem-tt__props {
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.gem-tt__prop {
  display: flex;
  align-items: baseline;
  gap: 6px;
  font-size: 12px;
}
.gem-tt__prop-k {
  font-family: var(--label);
  font-size: 9px;
  letter-spacing: 0.10em;
  text-transform: uppercase;
  color: var(--ink-faint);
  font-weight: 500;
}
.gem-tt__prop-dots {
  flex: 1;
  border-bottom: 1px dotted var(--paper-line);
  transform: translateY(-3px);
}
.gem-tt__prop-v {
  font-family: var(--hand-display);
  font-weight: 600;
  color: var(--ink);
}
.gem-tt__desc {
  font-family: var(--hand);
  font-size: 12.5px;
  line-height: 1.5;
  color: var(--ink-soft);
  white-space: pre-wrap;
}
.gem-tt__supports {
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.gem-tt__supports-title {
  font-family: var(--label);
  font-size: 9px;
  letter-spacing: 0.18em;
  text-transform: uppercase;
  color: var(--ink-faint);
  font-weight: 600;
  margin-bottom: 2px;
}
.gem-tt__support {
  font-size: 12px;
  color: var(--ink);
  display: flex;
  gap: 4px;
}
.gem-tt__support-mark { color: var(--ink-faint); }
.gem-tt__support-meta { color: var(--ink-faint); }
</style>
