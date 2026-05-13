<script setup lang="ts">
import { ref } from 'vue';
import {
  RunicBox,
  RunicButton,
  RunicCheckbox,
  RunicDivider,
  RunicHeader,
  RunicInput,
  RunicModal,
  RunicNumber,
  RunicProgressBar,
  RunicRadio,
  RunicSelect,
  RunicSlider,
  RunicTabs,
  RunicToast,
  RunicToggle,
  RunicTooltip,
} from '../components/runic';

const inputDefault = ref('');
const inputFilled = ref('Lioneye Watch');
const selectSingle = ref('');
const selectFilled = ref('shaman');
const selectMulti = ref<string[]>(['shaman']);
const radioValue = ref('build');
const tabsValue = ref('chat');
const sliderValue = ref(45);
const toggleA = ref(false);
const toggleB = ref(true);
const checkA = ref(false);
const checkB = ref(true);
const modalOpen = ref(false);

const sampleOptions = [
  { value: 'shaman', label: 'Shaman', description: 'Druid ascendancy — totems & weather' },
  { value: 'plague', label: 'Plague Bringer', description: 'Witch ascendancy — chaos clouds' },
  { value: 'invoker', label: 'Invoker', description: 'Monk ascendancy — elemental fists' },
  { value: 'pathfinder', label: 'Pathfinder', description: 'Ranger ascendancy — flask master', count: 3 },
];

const sampleTabs = [
  { value: 'chat', label: 'Chat', icon: '◆' },
  { value: 'build', label: 'Build', icon: '◆' },
  { value: 'settings', label: 'Settings', icon: '◆' },
];

const tooltipTargetRef = ref<HTMLElement | null>(null);
const tooltipVisible = ref(false);
const tooltipRect = ref<{ x: number; y: number; width: number; height: number } | null>(null);

const showTooltip = () => {
  if (!tooltipTargetRef.value) return;
  const r = tooltipTargetRef.value.getBoundingClientRect();
  tooltipRect.value = { x: r.x, y: r.y, width: r.width, height: r.height };
  tooltipVisible.value = true;
};
const hideTooltip = () => {
  tooltipVisible.value = false;
};

const dismissedToasts = ref(new Set<string>());
const dismissToast = (id: string) => dismissedToasts.value.add(id);

// ─── Design tokens overview ─────────────────────────────────────────
// Reads values live from CSS variables defined in `src/styles/tokens.css`.
// The single source of truth is the .css file — update there, this
// page updates with it.

interface Token { name: string; cssVar: string; hint?: string }

const paperNeutrals: Token[] = [
  { name: 'paper', cssVar: '--paper', hint: 'Main background' },
  { name: 'paper-shade', cssVar: '--paper-shade', hint: 'Subtle off-tone' },
  { name: 'paper-line', cssVar: '--paper-line', hint: 'Hairline dividers' },
  { name: 'ink', cssVar: '--ink', hint: 'Primary text' },
  { name: 'ink-soft', cssVar: '--ink-soft', hint: 'Secondary text' },
  { name: 'ink-faint', cssVar: '--ink-faint', hint: 'Meta text' },
  { name: 'ink-ghost', cssVar: '--ink-ghost', hint: 'Borders, very dim' },
];

const chromeAccent: Token[] = [
  { name: 'amber', cssVar: '--amber', hint: 'Chrome accent (graphite light / cream dark)' },
  { name: 'amber-soft', cssVar: '--amber-soft' },
  { name: 'amber-glow', cssVar: '--amber-glow', hint: 'Glow overlay' },
  { name: 'amber-bg', cssVar: '--amber-bg', hint: 'Subtle bg tint' },
];

const semanticTokens: Token[] = [
  { name: 'good', cssVar: '--good', hint: 'API key set, daemon online' },
  { name: 'bad', cssVar: '--bad', hint: 'Invalid, offline, missing' },
  { name: 'note', cssVar: '--note', hint: 'Warning, stale' },
];

const elementalTokens: Token[] = [
  { name: 'fire', cssVar: '--el-fire' },
  { name: 'fire-deep', cssVar: '--el-fire-deep' },
  { name: 'cold', cssVar: '--el-cold' },
  { name: 'cold-deep', cssVar: '--el-cold-deep' },
  { name: 'lit', cssVar: '--el-lit' },
  { name: 'lit-deep', cssVar: '--el-lit-deep' },
  { name: 'chaos', cssVar: '--el-chaos' },
  { name: 'chaos-deep', cssVar: '--el-chaos-deep' },
  { name: 'phys', cssVar: '--el-phys' },
  { name: 'phys-deep', cssVar: '--el-phys-deep' },
];

const typeScale = [
  { name: 'display', cssVar: '--fs-display', sample: 'Bestel awaits your question.' },
  { name: 'h1', cssVar: '--fs-h1', sample: 'Section heading' },
  { name: 'h2', cssVar: '--fs-h2', sample: 'Sub-section heading' },
  { name: 'h3', cssVar: '--fs-h3', sample: 'Card title' },
  { name: 'body', cssVar: '--fs-body', sample: 'Body copy paragraph.' },
  { name: 'meta', cssVar: '--fs-meta', sample: 'Meta caption line' },
  { name: 'caps', cssVar: '--fs-caps', sample: 'Caps label' },
  { name: 'micro', cssVar: '--fs-micro', sample: 'Tiny annotation' },
];

const typeFamilies = [
  { name: 'serif', cssVar: '--serif', sample: 'EB Garamond — narrative voice and body copy.' },
  { name: 'hand', cssVar: '--hand', sample: 'Used for thread titles and meta.' },
  { name: 'script', cssVar: '--script', sample: 'Kalam — handwritten annotations.' },
  { name: 'script-display', cssVar: '--script-display', sample: 'Caveat — display headings.' },
  { name: 'mono', cssVar: '--mono', sample: 'JetBrains Mono — code, paths, masked keys.' },
];

const motionTokens: Token[] = [
  { name: 'transition-fast', cssVar: '--transition-fast' },
  { name: 'transition-base', cssVar: '--transition-base' },
  { name: 'transition-smooth', cssVar: '--transition-smooth' },
  { name: 'transition-bounce', cssVar: '--transition-bounce' },
];
</script>

<template>
  <div class="lab runic-scrollbar">
    <RunicHeader title="Composants Runiques" subtitle="Storybook — port 1:1 du design system" />

    <section class="lab__section">
      <h2 class="lab__heading">Design tokens</h2>
      <p class="lab__hint">
        Source of truth : <code>src/styles/tokens.css</code>. Every swatch and
        sample below reads its value live from the same CSS variable the
        components use — edit the token, this page updates with it.
      </p>

      <h3 class="lab__sub">Paper neutrals (light theme)</h3>
      <div class="lab__swatch-grid">
        <div v-for="t in paperNeutrals" :key="t.cssVar" class="lab__swatch">
          <div class="lab__swatch-chip" :style="{ background: `var(${t.cssVar})` }" />
          <div class="lab__swatch-meta">
            <strong>{{ t.name }}</strong>
            <code>{{ t.cssVar }}</code>
            <span v-if="t.hint" class="lab__swatch-hint">{{ t.hint }}</span>
          </div>
        </div>
      </div>

      <h3 class="lab__sub">Chrome accent</h3>
      <div class="lab__swatch-grid">
        <div v-for="t in chromeAccent" :key="t.cssVar" class="lab__swatch">
          <div class="lab__swatch-chip" :style="{ background: `var(${t.cssVar})` }" />
          <div class="lab__swatch-meta">
            <strong>{{ t.name }}</strong>
            <code>{{ t.cssVar }}</code>
            <span v-if="t.hint" class="lab__swatch-hint">{{ t.hint }}</span>
          </div>
        </div>
      </div>

      <h3 class="lab__sub">Functional / semantic</h3>
      <div class="lab__swatch-grid">
        <div v-for="t in semanticTokens" :key="t.cssVar" class="lab__swatch">
          <div class="lab__swatch-chip" :style="{ background: `var(${t.cssVar})` }" />
          <div class="lab__swatch-meta">
            <strong>{{ t.name }}</strong>
            <code>{{ t.cssVar }}</code>
            <span v-if="t.hint" class="lab__swatch-hint">{{ t.hint }}</span>
          </div>
        </div>
      </div>

      <h3 class="lab__sub">Elemental palette</h3>
      <div class="lab__swatch-grid">
        <div v-for="t in elementalTokens" :key="t.cssVar" class="lab__swatch">
          <div class="lab__swatch-chip" :style="{ background: `var(${t.cssVar})` }" />
          <div class="lab__swatch-meta">
            <strong>{{ t.name }}</strong>
            <code>{{ t.cssVar }}</code>
          </div>
        </div>
      </div>

      <h3 class="lab__sub">Type scale</h3>
      <ul class="lab__type">
        <li v-for="t in typeScale" :key="t.cssVar" :style="{ fontSize: `var(${t.cssVar})` }">
          <span class="lab__type-sample">{{ t.sample }}</span>
          <span class="lab__type-meta"><code>{{ t.cssVar }}</code> · <strong>{{ t.name }}</strong></span>
        </li>
      </ul>

      <h3 class="lab__sub">Type families</h3>
      <ul class="lab__type">
        <li v-for="t in typeFamilies" :key="t.cssVar" :style="{ fontFamily: `var(${t.cssVar})` }">
          <span class="lab__type-sample">{{ t.sample }}</span>
          <span class="lab__type-meta"><code>{{ t.cssVar }}</code> · <strong>{{ t.name }}</strong></span>
        </li>
      </ul>

      <h3 class="lab__sub">Motion</h3>
      <div class="lab__meta-grid">
        <div v-for="t in motionTokens" :key="t.cssVar" class="lab__meta-cell">
          <strong>{{ t.name }}</strong>
          <code>{{ t.cssVar }}</code>
        </div>
      </div>
    </section>

    <RunicDivider variant="accent" />

    <section class="lab__section">
      <h2 class="lab__heading">Boutons</h2>
      <div class="lab__row">
        <RunicButton variant="primary">Primary</RunicButton>
        <RunicButton variant="secondary">Secondary</RunicButton>
        <RunicButton variant="ghost">Ghost</RunicButton>
        <RunicButton variant="danger">Danger</RunicButton>
        <RunicButton variant="primary" disabled>Disabled</RunicButton>
      </div>
      <div class="lab__row">
        <RunicButton size="xs">XS</RunicButton>
        <RunicButton size="sm">SM</RunicButton>
        <RunicButton size="md">MD</RunicButton>
        <RunicButton size="lg">LG</RunicButton>
      </div>
      <div class="lab__row">
        <RunicButton icon="settings">Settings</RunicButton>
        <RunicButton icon="external" variant="secondary">External</RunicButton>
        <RunicButton icon="play">Play</RunicButton>
        <RunicButton icon="close" variant="ghost" iconOnly />
      </div>
    </section>

    <RunicDivider variant="accent" />

    <section class="lab__section">
      <h2 class="lab__heading">Inputs</h2>
      <div class="lab__grid">
        <RunicInput v-model="inputDefault" placeholder="Type something…" icon="search" />
        <RunicInput v-model="inputFilled" placeholder="Filled" icon="search" />
        <RunicInput v-model="inputDefault" placeholder="Small" size="sm" />
        <RunicInput v-model="inputDefault" placeholder="Large" size="lg" />
      </div>
    </section>

    <RunicDivider />

    <section class="lab__section">
      <h2 class="lab__heading">Selects</h2>
      <div class="lab__grid">
        <RunicSelect
          v-model="selectSingle"
          :options="sampleOptions"
          placeholder="Pick an ascendancy"
          label="Single (empty)"
        />
        <RunicSelect
          v-model="selectFilled"
          :options="sampleOptions"
          label="Single (selected)"
        />
        <RunicSelect
          v-model="selectMulti"
          :options="sampleOptions"
          multiple
          searchable
          label="Multiple + search"
          placeholder="Pick several"
        />
      </div>
    </section>

    <RunicDivider />

    <section class="lab__section">
      <h2 class="lab__heading">Radio · Tabs · Toggle · Checkbox</h2>
      <div class="lab__grid">
        <div class="lab__cell">
          <span class="lab__label">RunicRadio (3 options)</span>
          <RunicRadio
            v-model="radioValue"
            :options="[
              { value: 'chat', label: 'Chat' },
              { value: 'build', label: 'Build' },
              { value: 'settings', label: 'Settings' },
            ]"
          />
        </div>

        <div class="lab__cell">
          <span class="lab__label">RunicTabs (sliding)</span>
          <RunicTabs v-model="tabsValue" :tabs="sampleTabs" />
        </div>

        <div class="lab__cell">
          <span class="lab__label">RunicToggle (off / on)</span>
          <div class="lab__row">
            <RunicToggle v-model="toggleA" />
            <RunicToggle v-model="toggleB" labelOn="Auto" labelOff="Manual" />
          </div>
        </div>

        <div class="lab__cell">
          <span class="lab__label">RunicCheckbox</span>
          <div class="lab__row">
            <RunicCheckbox v-model="checkA" label="Stream tokens en direct" />
            <RunicCheckbox v-model="checkB" label="Auto-link wiki" />
          </div>
        </div>
      </div>
    </section>

    <RunicDivider />

    <section class="lab__section">
      <h2 class="lab__heading">Slider · Progress</h2>
      <div class="lab__grid">
        <div class="lab__cell">
          <span class="lab__label">RunicSlider (md)</span>
          <RunicSlider v-model="sliderValue" />
        </div>
        <div class="lab__cell">
          <span class="lab__label">RunicProgressBar — tier T0/T1/T2/T3/Vaal/default</span>
          <div class="lab__stack">
            <RunicProgressBar :value="20" color="default" />
            <RunicProgressBar :value="42" color="t0" />
            <RunicProgressBar :value="58" color="t1" />
            <RunicProgressBar :value="71" color="t2" />
            <RunicProgressBar :value="88" color="t3" />
            <RunicProgressBar :value="100" color="vaal" />
          </div>
        </div>
      </div>
    </section>

    <RunicDivider />

    <section class="lab__section">
      <h2 class="lab__heading">Number — tier colors</h2>
      <div class="lab__row">
        <RunicNumber :value="1242" label="ES" color="default" />
        <RunicNumber :value="2.5" label="DPS k" color="t0" />
        <RunicNumber :value="92" label="Suppress" color="t1" />
        <RunicNumber :value="75" label="Block" color="t2" />
        <RunicNumber :value="34" label="Phys DR" color="t3" />
        <RunicNumber :value="666" label="Chaos" color="vaal" />
      </div>
    </section>

    <RunicDivider />

    <section class="lab__section">
      <h2 class="lab__heading">Box · Header · Divider</h2>
      <div class="lab__grid">
        <RunicBox padding="md">
          <p>RunicBox — bordure stone-carved, 4 corner ornaments, fond gradient + radial overlay subtil.</p>
        </RunicBox>
        <div>
          <RunicHeader title="Section Header" subtitle="With italic subtitle" />
          <RunicBox padding="md" attached>
            <p>Header attaché à une RunicBox via `attached`.</p>
          </RunicBox>
        </div>
      </div>
      <RunicDivider variant="accent" />
      <RunicDivider variant="default" rune="✦" />
      <RunicDivider variant="subtle" :show-rune="false" />
    </section>

    <RunicDivider />

    <section class="lab__section">
      <h2 class="lab__heading">Modal · Tooltip · Toast</h2>
      <div class="lab__row">
        <RunicButton @click="modalOpen = true">Open Modal</RunicButton>
        <button
          ref="tooltipTargetRef"
          type="button"
          class="lab__tooltip-trigger"
          @mouseenter="showTooltip"
          @mouseleave="hideTooltip"
          @focus="showTooltip"
          @blur="hideTooltip"
        >
          Hover ou focus pour la tooltip
        </button>
      </div>

      <RunicModal v-model="modalOpen" title="Resolute Technique" icon="◆">
        <p>
          Your hits can't be Evaded. Never deal Critical Strikes. <br />
          Une keystone classique pour les builds attack-oriented sans crit.
        </p>
        <RunicDivider />
        <p>Bestel rappelle que cette keystone tue tout multiplicateur de chance critique.</p>
        <template #footer>
          <RunicButton variant="ghost" size="sm" @click="modalOpen = false">Fermer</RunicButton>
        </template>
      </RunicModal>

      <RunicTooltip
        :visible="tooltipVisible"
        :target="tooltipRect"
        title="Resolute Technique"
        preferredSide="right"
      >
        <p style="margin: 0">
          Your hits can't be Evaded. Never deal Critical Strikes.
        </p>
      </RunicTooltip>

      <RunicDivider />
      <span class="lab__label">Toasts (variants)</span>
      <div class="lab__toasts">
        <RunicToast
          v-if="!dismissedToasts.has('info')"
          variant="info"
          title="Build switched"
          @dismiss="dismissToast('info')"
        >
          TornadoShot.xml chargée — 1242 ES, 92% Suppress.
        </RunicToast>
        <RunicToast
          v-if="!dismissedToasts.has('success')"
          variant="success"
          title="Model changed"
          @dismiss="dismissToast('success')"
        >
          Provider passé à Anthropic Sonnet 4.5.
        </RunicToast>
        <RunicToast
          v-if="!dismissedToasts.has('warning')"
          variant="warning"
          title="ANTHROPIC_API_KEY missing"
          @dismiss="dismissToast('warning')"
        >
          Auto-fallback sur Claude Code CLI.
        </RunicToast>
        <RunicToast
          v-if="!dismissedToasts.has('error')"
          variant="error"
          title="Provider unavailable"
          @dismiss="dismissToast('error')"
        >
          codex CLI returned exit code 1.
        </RunicToast>
      </div>
    </section>

    <div style="height: 4rem"></div>
  </div>
</template>

<style scoped>
.lab {
  flex: 1;
  overflow-y: auto;
  padding: 1.5rem 2rem;
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.lab__section {
  padding: 1rem 0;
}

.lab__heading {
  margin: 0 0 1rem;
  font-family: 'Cinzel', serif;
  font-size: 0.95rem;
  font-weight: 600;
  letter-spacing: 0.15em;
  text-transform: uppercase;
  color: rgba(180, 165, 140, 0.75);
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.5);
}

.lab__row {
  display: flex;
  flex-wrap: wrap;
  gap: 1rem;
  align-items: center;
  margin-bottom: 1rem;
}

.lab__grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
  gap: 1.25rem;
  margin-bottom: 1rem;
}

.lab__cell {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.lab__stack {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.lab__label {
  font-family: 'Cinzel', serif;
  font-size: 0.7rem;
  font-weight: 600;
  letter-spacing: 0.12em;
  text-transform: uppercase;
  color: rgba(140, 130, 115, 0.7);
}

.lab__tooltip-trigger {
  display: inline-flex;
  align-items: center;
  padding: 0.625rem 1rem;
  background: linear-gradient(180deg, rgba(20, 18, 15, 0.9) 0%, rgba(12, 10, 8, 0.95) 100%);
  border: 1px dashed rgba(175, 96, 37, 0.4);
  border-radius: 4px;
  color: rgba(220, 200, 175, 0.9);
  font-family: 'Crimson Text', serif;
  font-size: 0.95rem;
  cursor: help;
  outline: none;
}
.lab__tooltip-trigger:focus-visible {
  border-color: rgba(175, 96, 37, 0.7);
  box-shadow: 0 0 12px rgba(175, 96, 37, 0.2);
}

.lab__toasts {
  display: flex;
  flex-direction: column;
  gap: 0.625rem;
  align-items: flex-start;
}

/* ─── Design tokens section ──────────────────────────────────── */
.lab__hint {
  color: var(--ink-soft);
  font-size: var(--fs-meta);
  max-width: 70ch;
  margin: 0 0 1rem;
}
.lab__hint code {
  font-family: var(--mono);
  font-size: var(--fs-micro);
  background: var(--paper-shade);
  padding: 1px 4px;
  border-radius: 2px;
}
.lab__sub {
  font-family: var(--hand);
  font-size: var(--fs-h3);
  font-weight: var(--fw-medium);
  color: var(--ink-soft);
  margin: 1.25rem 0 0.5rem;
}
.lab__swatch-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
  gap: 0.625rem;
}
.lab__swatch {
  display: flex;
  gap: 0.625rem;
  padding: 0.5rem;
  background: var(--paper-shade);
  border-radius: var(--radius-runic);
  align-items: center;
}
.lab__swatch-chip {
  width: 40px;
  height: 40px;
  border-radius: var(--radius-runic);
  border: 1px solid var(--paper-line);
  flex: none;
}
.lab__swatch-meta {
  display: flex;
  flex-direction: column;
  gap: 0.125rem;
  min-width: 0;
}
.lab__swatch-meta strong {
  font-size: var(--fs-meta);
  font-weight: var(--fw-semibold);
  color: var(--ink);
}
.lab__swatch-meta code {
  font-family: var(--mono);
  font-size: var(--fs-micro);
  color: var(--ink-soft);
}
.lab__swatch-hint {
  font-size: var(--fs-micro);
  color: var(--ink-faint);
}
.lab__type {
  list-style: none;
  padding: 0;
  margin: 0;
  display: flex;
  flex-direction: column;
  gap: 0.625rem;
}
.lab__type li {
  display: flex;
  align-items: baseline;
  gap: 1rem;
  padding-bottom: 0.5rem;
  border-bottom: 1px solid var(--paper-line);
}
.lab__type-sample { flex: 1; }
.lab__type-meta {
  font-family: var(--mono);
  font-size: var(--fs-micro) !important;
  color: var(--ink-faint);
  flex: none;
}
.lab__type-meta strong { color: var(--ink-soft); }
.lab__meta-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
  gap: 0.625rem;
}
.lab__meta-cell {
  display: flex;
  flex-direction: column;
  gap: 0.125rem;
  padding: 0.5rem 0.75rem;
  background: var(--paper-shade);
  border-radius: var(--radius-runic);
}
.lab__meta-cell strong {
  font-size: var(--fs-meta);
  color: var(--ink);
}
.lab__meta-cell code {
  font-family: var(--mono);
  font-size: var(--fs-micro);
  color: var(--ink-soft);
}
</style>
