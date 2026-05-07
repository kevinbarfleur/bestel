<script setup lang="ts">
import { computed } from 'vue';
import { storeToRefs } from 'pinia';

import { useBuildStore } from '../../stores/build';
import type {
  ResistanceDto,
  SkillGemDto,
  SkillGroupDto,
  ItemSummaryDto,
  PassiveNodeDto,
} from '../../api/types';
import { openLink } from '../../api/tauri';

const emit = defineEmits<{ openPicker: [] }>();

const buildStore = useBuildStore();
const { current } = storeToRefs(buildStore);

const fmtInt = (v: number | null | undefined): string => {
  if (v == null) return '—';
  if (Math.abs(v) >= 1000) return Math.round(v).toLocaleString('en-US').replace(/,/g, ' ');
  return v.toFixed(0);
};

const fmtBig = (v: number | null | undefined): string => {
  if (v == null) return '—';
  if (v >= 1_000_000) return `${(v / 1_000_000).toFixed(2)}M`;
  if (v >= 1000) return `${(v / 1000).toFixed(1)}k`;
  return Math.round(v).toString();
};

const buildLevelLine = computed(() => {
  const c = current.value;
  if (!c) return '';
  const cls = c.class ?? '';
  const asc = c.ascendancy ?? '';
  const lvl = c.level ? `lvl ${c.level}` : '';
  const game = (c.game ?? '').toUpperCase();
  return [cls && asc ? `${cls} · ${asc}` : cls, lvl, game].filter(Boolean).join(' · ');
});

const ehpLabel = computed(() => {
  const e = current.value?.ehp;
  if (e == null) return null;
  if (e >= 1000) return `ehp ${(e / 1000).toFixed(1)}k`;
  return `ehp ${Math.round(e)}`;
});

interface ResVm {
  name: ResistanceDto['name'];
  short: 'fire' | 'cold' | 'lit' | 'chaos';
  value: number | null;
  cap: number;
  status: 'good' | 'ok' | 'bad' | 'neg';
  display: string;
  /** 0..1 — fill width of the mini bar. */
  fillPct: number;
  /** 0..1 — position of the cap marker on the mini bar. */
  capMarkerPct: number;
}
const RES_LABELS: Record<ResistanceDto['name'], ResVm['short']> = {
  fire: 'fire',
  cold: 'cold',
  lightning: 'lit',
  chaos: 'chaos',
};

const resistances = computed<ResVm[]>(() => {
  const list = current.value?.resistances ?? [];
  return list.map((r) => {
    const v = r.value;
    const cap = r.cap ?? 75;
    let status: ResVm['status'] = 'ok';
    if (v == null) status = 'ok';
    else if (v < 0) status = 'neg';
    else if (v >= cap) status = 'good';
    else status = 'bad';
    const display = v == null ? '—' : `${v.toFixed(0)}`;
    const fillPct = v == null ? 0 : Math.max(0, Math.min(1, v / 100));
    const capMarkerPct = Math.max(0, Math.min(1, cap / 100));
    return {
      name: r.name,
      short: RES_LABELS[r.name],
      value: v,
      cap,
      status,
      display,
      fillPct,
      capMarkerPct,
    };
  });
});

const isSupport = (gem: SkillGemDto) => /\bsupport\b/i.test(gem.name);
const cleanGemName = (gem: SkillGemDto) => gem.name.replace(/\s+Support$/i, '').trim();

interface SkillGroupVm {
  label: string;
  active: SkillGemDto[];
  supports: SkillGemDto[];
  is_main: boolean;
  isAura: boolean;
  element: 'fire' | 'cold' | 'lit' | 'chaos' | 'phys';
}

const detectElement = (name: string): SkillGroupVm['element'] => {
  const n = name.toLowerCase();
  if (/\b(fireball|incinerate|flame|burn|ignite|infernal|combust|pyre)\b/.test(n)) return 'fire';
  if (/\b(frost|cold|ice|hypothermia|chill|freezing)\b/.test(n)) return 'cold';
  if (/\b(spark|lightning|shock|arc|tempest|storm|thunder|wrath|polaric)\b/.test(n)) return 'lit';
  if (/\b(chaos|poison|toxic|wither|despair|withering|essence drain|contagion|caustic)\b/.test(n))
    return 'chaos';
  return 'phys';
};

function buildGroupVm(g: SkillGroupDto): SkillGroupVm {
  const active: SkillGemDto[] = [];
  const supports: SkillGemDto[] = [];
  for (const gem of g.gems) {
    if (isSupport(gem)) supports.push(gem);
    else active.push(gem);
  }
  const headName = active[0]?.name?.toLowerCase() ?? '';
  const isAura =
    headName.includes('aura') ||
    headName.includes('herald') ||
    headName.includes('determination') ||
    headName.includes('grace') ||
    headName.includes('hatred') ||
    headName.includes('discipline') ||
    headName.includes('purity') ||
    headName.includes('wrath') ||
    headName.includes('anger');
  return {
    label: g.label || active[0]?.name || 'Group',
    active,
    supports,
    is_main: g.is_main,
    isAura,
    element: detectElement(active[0]?.name ?? g.label ?? ''),
  };
}

const skillGroups = computed<SkillGroupVm[]>(() => {
  const groups = current.value?.skill_groups ?? [];
  return [...groups]
    .sort((a, b) => Number(b.is_main) - Number(a.is_main))
    .map((g) => buildGroupVm(g));
});

const mainGroup = computed(() => skillGroups.value[0] ?? null);

const equippedItems = computed<ItemSummaryDto[]>(() => {
  const items = current.value?.items ?? [];
  return items.filter((i) => i.slot);
});

const tierColor = (rarity: string | null | undefined): string => {
  switch ((rarity ?? '').toLowerCase()) {
    case 'unique':
      return 'var(--amber)';
    case 'rare':
      return '#c9a227';
    case 'magic':
      return '#5a7080';
    case 'relic':
      return 'var(--good)';
    default:
      return 'var(--ink)';
  }
};

const slotShort = (slot: string | null | undefined): string => {
  const s = (slot ?? '').toLowerCase();
  if (s.includes('helmet')) return 'helm';
  if (s.includes('body')) return 'body';
  if (s.includes('glove')) return 'gloves';
  if (s.includes('boot')) return 'boots';
  if (s.includes('belt')) return 'belt';
  if (s.includes('weapon 1')) return 'weapon';
  if (s.includes('weapon 2')) return 'offhand';
  if (s.includes('amulet')) return 'amulet';
  if (s.includes('ring 1')) return 'ring 1';
  if (s.includes('ring 2')) return 'ring 2';
  if (s.includes('flask')) return 'flask';
  if (s.includes('charm')) return 'charm';
  return s.replace(/\s+/g, ' ').trim();
};

const onPickBuild = () => emit('openPicker');

// ─── Tooltip helpers ────────────────────────────────────────────
const tipVital = (label: string, raw: number | null | undefined): string => {
  if (raw == null) return `${label}: not parsed in this build.`;
  const n = Math.round(raw).toLocaleString('en-US');
  return `${label}: ${n}`;
};

const tipResistance = (r: ResVm): string => {
  if (r.value == null) return `${r.short.toUpperCase()} resistance: not parsed.`;
  if (r.status === 'good') {
    const over = r.value - r.cap;
    return `${r.short.toUpperCase()} ${r.value.toFixed(0)}% / ${r.cap}% cap${over > 0 ? ` · overcapped +${over.toFixed(0)}` : ''}`;
  }
  if (r.status === 'neg') {
    return `${r.short.toUpperCase()} ${r.value.toFixed(0)}% / ${r.cap}% cap · negative resistance — chaos damage will hurt badly.`;
  }
  if (r.status === 'bad') {
    const under = r.cap - r.value;
    return `${r.short.toUpperCase()} ${r.value.toFixed(0)}% / ${r.cap}% cap · ${under.toFixed(0)} under cap. Map mods reducing max res will hurt.`;
  }
  return `${r.short.toUpperCase()} ${r.value.toFixed(0)}%`;
};

const tipSkillGroup = (g: SkillGroupVm): string => {
  const head = g.active[0];
  const lines: string[] = [];
  if (head) {
    lines.push(head.name);
    const meta: string[] = [];
    if (head.level != null) meta.push(`lvl ${head.level}`);
    if (head.quality != null && head.quality > 0) meta.push(`${head.quality}% quality`);
    if (!head.enabled) meta.push('disabled');
    if (meta.length) lines.push(meta.join(' · '));
    lines.push('──────────');
  }
  if (g.is_main) lines.push('Main skill of this build.');
  if (g.isAura) lines.push('Reservation skill (aura/herald).');
  if (g.element !== 'phys') lines.push(`Element: ${g.element}`);
  if (g.supports.length) {
    lines.push('');
    lines.push('Supports:');
    for (const s of g.supports) {
      const meta: string[] = [];
      if (s.level != null) meta.push(`lvl ${s.level}`);
      if (s.quality != null && s.quality > 0) meta.push(`${s.quality}q`);
      lines.push(`  · ${cleanGemName(s)}${meta.length ? ` (${meta.join(', ')})` : ''}`);
    }
  }
  return lines.join('\n') || 'No additional info.';
};

// Tooltip payload helpers — return JSON for kind=node/gem, raw text for kind=item.
const tipNode = (n: PassiveNodeDto): string =>
  JSON.stringify({
    kind: n.kind,
    description: n.description ?? '',
    ascendancy: n.ascendancy ?? null,
  });

// ─── Skill-gem attribute heuristic ───────────────────────────────
// PoB doesn't expose gem requirement attribute. We tag based on the gem's name
// using a list of popular PoE skills. Falls back to 'unknown'.
const STR_GEMS = new Set([
  'Heavy Strike', 'Sweep', 'Ground Slam', 'Cleave', 'Glacial Hammer',
  'Earthquake', 'Sunder', 'Tectonic Slam', 'Vaal Ground Slam',
  'Leap Slam', 'Shield Charge', 'Berserk', 'Blood Rage', 'Warlord\'s Mark',
  'Determination', 'Pride', 'Vitality', 'Purity of Fire', 'Purity of Ice',
  'Purity of Lightning', 'Purity of Elements', 'Vengeance',
  'Molten Strike', 'Vaal Molten Strike', 'Infernal Blow',
  'Rolling Magma', 'Searing Bond', 'Flame Surge', 'Combustion',
  'Anger', 'Vaal Heavy Strike', 'Ancestral Warchief', 'Ancestral Protector',
  'Vaal Ancestral Warchief', 'Devouring Totem', 'Decoy Totem',
]);
const DEX_GEMS = new Set([
  'Tornado Shot', 'Spark', 'Burning Arrow', 'Lightning Arrow', 'Ice Shot',
  'Caustic Arrow', 'Toxic Rain', 'Rain of Arrows', 'Frenzy', 'Puncture',
  'Split Arrow', 'Barrage', 'Shrapnel Ballista', 'Storm Rain', 'Artillery Ballista',
  'Galvanic Arrow', 'Blast Rain', 'Mirror Arrow', 'Blink Arrow',
  'Whirling Blades', 'Cyclone', 'Reave', 'Spectral Throw', 'Lacerate',
  'Double Strike', 'Viper Strike', 'Flicker Strike', 'Frost Blades',
  'Lightning Strike', 'Wild Strike', 'Bladestorm', 'Blade Flurry',
  'Blade Vortex', 'Vaal Lightning Strike', 'Vaal Frost Blades',
  'Grace', 'Hatred', 'Haste', 'Wrath', 'Herald of Ice', 'Herald of Thunder',
  'Herald of Ash', 'Herald of Agony', 'Herald of Purity', 'Wind Dancer',
  'Smoke Mine',
]);
const INT_GEMS = new Set([
  'Spark', 'Fireball', 'Frostbolt', 'Ice Spear', 'Lightning Bolt',
  'Glacial Cascade', 'Cold Snap', 'Vortex', 'Frost Bomb',
  'Freezing Pulse', 'Arc', 'Storm Burst', 'Storm Brand', 'Armageddon Brand',
  'Penance Brand', 'Crackling Lance', 'Galvanic Field',
  'Flameblast', 'Flame Wall', 'Firestorm', 'Incinerate', 'Wave of Conviction',
  'Soulrend', 'Bane', 'Essence Drain', 'Contagion', 'Despair',
  'Wither', 'Withering Step', 'Plague Bearer', 'Blight', 'Pestilent Strike',
  'Poisonous Concoction', 'Forbidden Rite',
  'Discipline', 'Clarity', 'Aspect of the Spider', 'Aspect of the Crab',
  'Aspect of the Cat', 'Aspect of the Avian', 'Skitterbots',
  'Power Siphon', 'Kinetic Bolt', 'Kinetic Blast',
  'Ball Lightning', 'Lightning Tendrils', 'Lightning Trap', 'Bear Trap',
  'Stormbind', 'Ice Crash',
]);

const detectAttribute = (name: string): 'str' | 'dex' | 'int' | 'unknown' => {
  // try direct lookup, then strip "Vaal "/"Awakened " prefix
  const norm = name.replace(/^(Vaal|Awakened|Divergent|Anomalous|Phantasmal)\s+/, '').trim();
  if (STR_GEMS.has(norm)) return 'str';
  if (DEX_GEMS.has(norm)) return 'dex';
  if (INT_GEMS.has(norm)) return 'int';
  return 'unknown';
};

const tipGem = (g: SkillGroupVm): string => {
  const head = g.active[0];
  if (!head) return JSON.stringify({ name: g.label, level: null, quality: null, enabled: true, element: g.element, is_main: g.is_main, is_aura: g.isAura, attribute: 'unknown', tags: [], description: null, supports: [] });
  const tags: string[] = [];
  if (g.element !== 'phys') tags.push(g.element.toUpperCase());
  if (g.isAura) tags.push('Aura');
  if (g.is_main) tags.push('Main skill');
  return JSON.stringify({
    name: cleanGemName(head),
    level: head.level,
    quality: head.quality,
    enabled: head.enabled,
    element: g.element,
    is_main: g.is_main,
    is_aura: g.isAura,
    attribute: detectAttribute(cleanGemName(head)),
    tags,
    description: null,
    supports: g.supports.map((s) => ({ name: cleanGemName(s), level: s.level, quality: s.quality })),
  });
};

const keystones = computed<PassiveNodeDto[]>(() => current.value?.allocated_keystones ?? []);
const notables = computed<PassiveNodeDto[]>(() => current.value?.allocated_notables ?? []);

const tipMainDps = (): string => {
  const c = current.value;
  if (!c?.dps) return 'Combined DPS not parsed.';
  const lines: string[] = [`Combined DPS: ${Math.round(c.dps).toLocaleString('en-US')}`];
  if (c.main_skill) lines.push(`Main skill: ${c.main_skill}`);
  return lines.join('\n');
};
</script>

<template>
  <aside class="bp" data-density="compact">
    <template v-if="current">
      <!-- Header — class / ascendancy in display size + meta line -->
      <header
        class="bp-head"
        :data-tooltip-title="current.ascendancy ?? current.class"
        :data-tooltip-text="`Class: ${current.class}\nAscendancy: ${current.ascendancy ?? '—'}\nLevel: ${current.level ?? '—'}\nGame: ${(current.game ?? '').toUpperCase()}\nFile: ${current.file_name}`"
      >
        <div class="bp-head__title">{{ current.ascendancy ?? current.class }}</div>
        <div class="bp-head__meta">{{ buildLevelLine }}</div>
      </header>

      <!-- Scrollable body — every section in leader-dot grammar -->
      <div class="bp-body runic-scrollbar">

        <!-- Vitals — 2×2 BIG numbers (life, es, mana, spirit/evasion) -->
        <section class="bp-section">
          <div class="bp-h">
            <span class="bp-h__title">vitals</span>
            <span class="bp-h__grow" />
            <span v-if="ehpLabel" class="bp-h__meta">{{ ehpLabel }}</span>
          </div>
          <div class="bp-big-grid">
            <div class="bp-big-tile" :data-tooltip-title="'Life'" :data-tooltip-text="tipVital('Life', current.life)">
              <div class="bp-big-label">life</div>
              <div class="bp-big-value">{{ fmtBig(current.life) }}</div>
            </div>
            <div
              class="bp-big-tile"
              :class="{ 'bp-big-tile--muted': !current.energy_shield }"
              :data-tooltip-title="'Energy Shield'"
              :data-tooltip-text="tipVital('Energy Shield', current.energy_shield)"
            >
              <div class="bp-big-label">energy shield</div>
              <div class="bp-big-value">
                {{ current.energy_shield ? fmtBig(current.energy_shield) : '—' }}
              </div>
            </div>
            <div class="bp-big-tile" :data-tooltip-title="'Mana'" :data-tooltip-text="tipVital('Mana', current.mana)">
              <div class="bp-big-label">mana</div>
              <div class="bp-big-value">{{ fmtBig(current.mana) }}</div>
            </div>
            <div
              v-if="current.game === 'poe2'"
              class="bp-big-tile"
              :class="{ 'bp-big-tile--muted': current.spirit == null }"
              :data-tooltip-title="'Spirit'"
              :data-tooltip-text="tipVital('Spirit', current.spirit)"
            >
              <div class="bp-big-label">spirit</div>
              <div class="bp-big-value">{{ fmtBig(current.spirit) }}</div>
            </div>
            <div
              v-else
              class="bp-big-tile"
              :class="{ 'bp-big-tile--muted': current.evasion == null }"
              :data-tooltip-title="'Evasion'"
              :data-tooltip-text="tipVital('Evasion', current.evasion)"
            >
              <div class="bp-big-label">evasion</div>
              <div class="bp-big-value">{{ fmtBig(current.evasion) }}</div>
            </div>
          </div>
        </section>

        <!-- Resistances — 2×2 BIG numbers + mini bar with dynamic cap marker -->
        <section v-if="resistances.length" class="bp-section">
          <div class="bp-h">
            <span class="bp-h__title">resistances</span>
            <span class="bp-h__grow" />
          </div>
          <div class="bp-big-grid">
            <div
              v-for="r in resistances"
              :key="r.name"
              class="bp-resist-tile"
              :data-tooltip-title="r.short.toUpperCase() + ' resistance'"
              :data-tooltip-text="tipResistance(r)"
            >
              <div class="bp-big-label" :style="{ color: `var(--el-${r.short})` }">{{ r.short }}</div>
              <div class="bp-resist-value-row">
                <span
                  class="bp-big-value"
                  :style="{
                    color:
                      r.status === 'neg' ? 'var(--bad)' :
                      r.status === 'bad' ? 'var(--note)' :
                      `var(--el-${r.short}-deep)`,
                  }"
                >
                  {{ r.value == null ? '—' : `${r.display}%` }}
                </span>
                <span class="bp-resist-cap">/ {{ Math.round(r.cap) }}</span>
              </div>
              <div class="bp-bar">
                <div
                  class="bp-bar-fill"
                  :style="{
                    width: `${r.fillPct * 100}%`,
                    background:
                      r.status === 'neg' ? 'var(--bad)' :
                      r.status === 'bad' ? 'var(--note)' :
                      `var(--el-${r.short})`,
                  }"
                />
                <div class="bp-bar-cap" :style="{ left: `${r.capMarkerPct * 100}%` }" />
              </div>
            </div>
          </div>
        </section>

        <!-- Main skill — name + DPS, no big DPS hero -->
        <section v-if="mainGroup" class="bp-section" :data-tooltip-title="'Main skill'" :data-tooltip-text="tipMainDps()">
          <div class="bp-h">
            <span class="bp-h__title">main skill</span>
            <span class="bp-h__grow" />
            <span v-if="current.dps != null" class="bp-h__meta">{{ fmtBig(current.dps) }} dps</span>
          </div>
          <div class="bp-mainskill__name">
            {{ mainGroup.active[0]?.name ?? mainGroup.label }}
          </div>
          <div class="bp-mainskill__sub">
            <span v-if="mainGroup.supports.length">{{ mainGroup.supports.length + 1 }}-link</span>
            <span v-if="mainGroup.element !== 'phys'"
              :style="`color: var(--el-${mainGroup.element}-deep); margin-left: 6px;`">
              · {{ mainGroup.element }}
            </span>
          </div>
        </section>

        <!-- Skills as leader rows -->
        <section v-if="skillGroups.length" class="bp-section">
          <div class="bp-h">
            <span class="bp-h__title">skills</span>
            <span class="bp-h__grow" />
            <span class="bp-h__meta">{{ skillGroups.length }} groups</span>
          </div>
          <div class="bp-rows">
            <div
              v-for="g in skillGroups"
              :key="g.label"
              class="leader"
              data-tooltip-kind="gem"
              :data-tooltip-title="cleanGemName(g.active[0] ?? { name: g.label } as SkillGemDto)"
              :data-tooltip-text="tipGem(g)"
            >
              <span class="leader__k leader__k--gem">
                <span class="gem-mark" :style="`color: var(--el-${g.element})`">◆</span>
                {{ cleanGemName(g.active[0] ?? { name: g.label } as SkillGemDto) }}<span v-if="g.isAura" class="leader__k-aura"> · aura</span>
              </span>
              <span class="leader__dots" />
              <span
                class="leader__v"
                :style="g.is_main ? `color: var(--el-${g.element}-deep)` : ''"
                :class="{ 'leader__v--main': g.is_main }"
              >
                <template v-if="g.is_main">{{ fmtBig(current.dps) }}</template>
                <template v-else-if="g.isAura">—</template>
                <template v-else>+{{ g.supports.length }}</template>
              </span>
            </div>
          </div>
        </section>

        <!-- Equipment as leader rows with item names as .link -->
        <section v-if="equippedItems.length" class="bp-section">
          <div class="bp-h">
            <span class="bp-h__title">equipment</span>
            <span class="bp-h__grow" />
            <span class="bp-h__meta">{{ equippedItems.length }} / 12</span>
          </div>
          <div class="bp-rows">
            <div
              v-for="it in equippedItems"
              :key="it.id"
              class="leader"
              data-tooltip-kind="item"
              :data-tooltip-title="it.slot ?? ''"
              :data-tooltip-text="it.raw_text"
            >
              <span class="leader__k">{{ slotShort(it.slot) }}</span>
              <span class="leader__dots" />
              <a class="leader__v link" :style="{ color: tierColor(it.rarity), textDecorationColor: tierColor(it.rarity) }">
                {{ it.name ?? it.base ?? '—' }}
              </a>
            </div>
          </div>
        </section>

        <!-- Keystones with rich tooltips -->
        <section v-if="keystones.length" class="bp-section">
          <div class="bp-h">
            <span class="bp-h__title">keystones</span>
            <span class="bp-h__grow" />
            <span class="bp-h__meta">{{ keystones.length }}</span>
          </div>
          <div class="bp-rows">
            <div
              v-for="k in keystones"
              :key="k.id"
              class="leader"
              data-tooltip-kind="node"
              :data-tooltip-title="k.name"
              :data-tooltip-text="tipNode(k)"
            >
              <span class="leader__k leader__k--node">
                <span class="gem-mark" style="color: var(--amber)">◆</span>
                {{ k.name }}
              </span>
              <span class="leader__dots" />
              <span class="leader__v leader__v--node">keystone</span>
            </div>
          </div>
        </section>

        <!-- Notables (top 8) -->
        <section v-if="notables.length" class="bp-section">
          <div class="bp-h">
            <span class="bp-h__title">notables</span>
            <span class="bp-h__grow" />
            <span class="bp-h__meta">{{ notables.length }}</span>
          </div>
          <div class="bp-rows">
            <div
              v-for="n in notables.slice(0, 8)"
              :key="n.id"
              class="leader"
              data-tooltip-kind="node"
              :data-tooltip-title="n.name"
              :data-tooltip-text="tipNode(n)"
            >
              <span class="leader__k leader__k--node">{{ n.name }}</span>
              <span class="leader__dots" />
              <span class="leader__v leader__v--node">notable</span>
            </div>
            <div v-if="notables.length > 8" class="leader leader--dim">
              <span class="leader__k">…</span>
              <span class="leader__dots" />
              <span class="leader__v">+{{ notables.length - 8 }} more</span>
            </div>
          </div>
        </section>

        <!-- Passive tree link -->
        <section v-if="current.passive_tree_url" class="bp-section">
          <div class="bp-h">
            <span class="bp-h__title">passive tree</span>
            <span class="bp-h__grow" />
            <span class="bp-h__meta">official viewer</span>
          </div>
          <div class="bp-tree-actions">
            <a class="link bp-tree-link" @click.prevent="openLink(current.passive_tree_url!)">
              ◆ Open in browser
            </a>
            <p class="bp-tree-hint">
              The official Path of Building tree viewer renders this build's
              allocation node-by-node. Opens in your default browser.
            </p>
          </div>
        </section>

        <!-- Charges (placeholder) -->
        <section class="bp-section">
          <div class="bp-h">
            <span class="bp-h__title">charges</span>
            <span class="bp-h__grow" />
          </div>
          <div class="bp-rows">
            <div class="leader">
              <span class="leader__k">endurance</span><span class="leader__dots" />
              <span class="leader__v leader__v--charges">
                <span v-for="i in 4" :key="i" class="charge-pip" />
              </span>
            </div>
            <div class="leader">
              <span class="leader__k">frenzy</span><span class="leader__dots" />
              <span class="leader__v leader__v--charges">
                <span v-for="i in 4" :key="i" class="charge-pip" />
              </span>
            </div>
            <div class="leader">
              <span class="leader__k">power</span><span class="leader__dots" />
              <span class="leader__v leader__v--charges">
                <span v-for="i in 5" :key="i" class="charge-pip" />
              </span>
            </div>
          </div>
        </section>
      </div>
    </template>

    <!-- Empty state — almanach -->
    <div v-else class="bp-empty">
      <span class="bp-empty__rune">◆</span>
      <p class="bp-empty__title">No build loaded</p>
      <p class="bp-empty__hint">Save a build in Path of Building, or load one from the top bar.</p>
      <a class="link bp-empty__cta" @click="onPickBuild">Drop your PoB code here</a>
    </div>

    <!-- Watcher footer -->
    <footer class="bp-watcher">
      <span class="bp-watcher__status">
        <span class="bp-watcher__dot" />
        <span class="bp-watcher__label">Watcher · live</span>
      </span>
      <span class="bp-watcher__hint">auto-import on save</span>
    </footer>
  </aside>
</template>

<style scoped>
.bp {
  width: 360px;
  flex: 0 0 360px;
  display: flex;
  flex-direction: column;
  border-right: 1px solid var(--paper-line);
  background: var(--paper);
  min-height: 0;
  height: 100%;
  position: relative;
  overflow: hidden;
}

/* Header — class / ascendancy in display size + meta line */
.bp-head {
  flex: none;
  padding: 18px 22px 14px;
}
.bp-head__title {
  font-family: var(--hand);
  font-size: 28px;
  font-weight: var(--fw-bold);
  line-height: 1.1;
  color: var(--ink);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.bp-head__meta {
  margin-top: 6px;
  font-family: var(--hand);
  font-size: 16px;
  color: var(--ink-soft);
  font-weight: var(--fw-regular);
}

/* Scrollable body */
.bp-body {
  flex: 1 1 auto;
  min-height: 0;
  overflow-y: auto;
  padding: 10px 22px 18px;
  display: flex;
  flex-direction: column;
  gap: 22px;
}

.bp-section { display: flex; flex-direction: column; }

/* Section header — small caps + grow + right meta regular */
.bp-h {
  display: flex;
  align-items: baseline;
  gap: 8px;
  padding-bottom: 6px;
  margin-bottom: 10px;
  border-bottom: 1px solid var(--paper-line);
  white-space: nowrap;
}
.bp-h__title {
  font-family: var(--label);
  font-size: var(--fs-caps);
  letter-spacing: 0.18em;
  text-transform: uppercase;
  color: var(--ink-soft);
  font-weight: var(--fw-semibold);
  white-space: nowrap;
}
.bp-h__grow { flex: 1; }
.bp-h__meta {
  font-family: var(--hand);
  font-size: var(--fs-meta);
  color: var(--ink-soft);
  font-weight: var(--fw-regular);
  white-space: nowrap;
}

/* BIG-numbers grid — 2×2, used for vitals + resists */
.bp-big-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px 18px;
}
.bp-big-tile,
.bp-resist-tile {
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.bp-big-label {
  font-family: var(--label);
  font-size: var(--fs-caps);
  letter-spacing: 0.16em;
  text-transform: uppercase;
  color: var(--ink-soft);
  font-weight: var(--fw-semibold);
}
.bp-big-value {
  font-family: var(--hand);
  font-size: 26px;
  font-weight: var(--fw-bold);
  color: var(--ink);
  line-height: 1;
}
.bp-big-tile--muted .bp-big-value {
  color: var(--ink-faint);
}
.bp-resist-value-row {
  display: flex;
  align-items: baseline;
  gap: 6px;
}
.bp-resist-cap {
  font-family: var(--hand);
  font-size: var(--fs-body);
  color: var(--ink-soft);
}
.bp-bar {
  margin-top: 4px;
  position: relative;
  height: 3px;
  background: var(--paper-shade);
  border: 1px solid var(--paper-line);
  border-radius: 2px;
  overflow: hidden;
}
.bp-bar-fill {
  height: 100%;
}
.bp-bar-cap {
  position: absolute;
  top: -2px;
  bottom: -2px;
  width: 1px;
  background: var(--ink-soft);
  opacity: 0.5;
}

/* Main skill — name 22px + meta 15px ink-soft */
.bp-mainskill__name {
  font-family: var(--hand);
  font-size: 22px;
  font-weight: var(--fw-semibold);
  color: var(--ink);
  line-height: 1.2;
}
.bp-mainskill__sub {
  margin-top: 4px;
  font-family: var(--hand);
  font-size: 15px;
  color: var(--ink-soft);
}

/* Leader-dot rows */
.bp-rows {
  display: flex;
  flex-direction: column;
  gap: 3px;
}
.leader {
  display: flex;
  align-items: baseline;
  gap: 6px;
}
.leader--dim { opacity: 0.55; }
.leader__k {
  font-family: var(--label);
  font-size: 10px;
  letter-spacing: 0.10em;
  text-transform: uppercase;
  color: var(--ink-faint);
  font-weight: 500;
  flex-shrink: 0;
  white-space: nowrap;
}
.leader__k--gem,
.leader__k--node {
  text-transform: none;
  letter-spacing: 0;
  font-family: var(--hand);
  font-size: 13px;
  color: var(--ink);
  font-weight: 500;
  display: inline-flex;
  align-items: baseline;
  gap: 4px;
  overflow: hidden;
  text-overflow: ellipsis;
  cursor: help;
}
.leader__v--node {
  font-family: var(--label);
  font-size: 9px;
  letter-spacing: 0.10em;
  text-transform: uppercase;
  color: var(--ink-faint);
  font-weight: 500;
}
.gem-mark {
  font-size: 11px;
  font-weight: 700;
  margin-right: 2px;
}
.leader__k-aura {
  color: var(--ink-faint);
  font-size: 11px;
}
.leader__dots {
  flex: 1;
  min-width: 12px;
  border-bottom: 1px dotted var(--paper-line);
  transform: translateY(-3px);
}
.leader__v {
  font-family: var(--hand-display);
  font-size: 14px;
  font-weight: 600;
  color: var(--ink);
  flex: 0 0 auto;
  white-space: nowrap;
}
.leader__v--main { font-weight: 700; }
.leader__v--charges {
  display: inline-flex;
  gap: 3px;
  align-items: center;
}
.charge-pip {
  display: inline-block;
  width: 7px;
  height: 7px;
  border-radius: 50%;
  border: 1px solid var(--ink-faint);
  background: transparent;
}

/* Main skill block */
.bp-mainskill__row {
  display: flex;
  align-items: baseline;
  gap: 8px;
}
.bp-mainskill__dps {
  font-family: var(--hand-display);
  font-size: 32px;
  font-weight: 700;
  line-height: 1;
  letter-spacing: -0.01em;
}
.bp-mainskill__sub {
  font-family: var(--hand);
  font-size: 12px;
  color: var(--ink-faint);
  margin-top: 2px;
}

/* Resistance pills — 2x2 dashed elemental */
.bp-resists {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 6px;
}
.resist {
  padding: 5px 12px;
  border-radius: 999px;
  display: flex;
  align-items: baseline;
  gap: 6px;
  border-style: dashed;
  border-width: 1px;
}
.resist--fire  { border-color: var(--el-fire);  background: var(--el-fire-bg); }
.resist--cold  { border-color: var(--el-cold);  background: var(--el-cold-bg); }
.resist--lit   { border-color: var(--el-lit);   background: var(--el-lit-bg); }
.resist--chaos { border-color: var(--el-chaos); background: var(--el-chaos-bg); }
.resist__el {
  font-family: var(--label);
  font-size: 9px;
  letter-spacing: 0.10em;
  text-transform: uppercase;
  font-weight: 600;
}
.resist--fire .resist__el  { color: var(--el-fire); }
.resist--cold .resist__el  { color: var(--el-cold); }
.resist--lit .resist__el   { color: var(--el-lit); }
.resist--chaos .resist__el { color: var(--el-chaos); }
.resist__grow { flex: 1; }
.resist__v {
  font-family: var(--hand-display);
  font-size: 16px;
  font-weight: 700;
  line-height: 1;
}
.resist--fire .resist__v  { color: var(--el-fire-deep); }
.resist--cold .resist__v  { color: var(--el-cold-deep); }
.resist--lit .resist__v   { color: var(--el-lit-deep); }
.resist--chaos .resist__v { color: var(--el-chaos-deep); }
.resist__v--neg {
  text-decoration: underline wavy var(--bad);
  text-decoration-thickness: 1px;
  text-underline-offset: 3px;
}
.resist__cap {
  font-family: var(--hand);
  font-size: 10px;
  color: var(--ink-faint);
}

/* Passive tree note */
.bp-tree-note {
  font-family: var(--hand);
  font-size: 12px;
  color: var(--ink-faint);
  line-height: 1.5;
}
.bp-tree-actions {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.bp-tree-link {
  font-family: var(--hand-display);
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
}
.bp-tree-hint {
  margin: 0;
  font-family: var(--hand);
  font-size: 11px;
  color: var(--ink-faint);
  line-height: 1.5;
}

/* Empty state */
.bp-empty {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 10px;
  padding: 60px 22px;
  text-align: center;
  color: var(--ink-soft);
}
.bp-empty__rune {
  font-size: 28px;
  color: var(--amber);
  opacity: 0.7;
}
.bp-empty__title {
  margin: 0;
  font-family: var(--hand-display);
  font-size: 22px;
  font-weight: 600;
  color: var(--ink);
}
.bp-empty__hint {
  margin: 0;
  max-width: 22rem;
  line-height: 1.5;
  font-family: var(--hand);
  font-size: 13px;
  color: var(--ink-faint);
}
.bp-empty__cta {
  margin-top: 8px;
  font-family: var(--hand);
  font-size: 13px;
  font-weight: 600;
}

/* Watcher footer */
.bp-watcher {
  padding: 10px 14px;
  border-top: 1px dashed var(--paper-line);
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  font-family: var(--hand);
  font-size: 12px;
  color: var(--ink-soft);
  flex: none;
  background: var(--paper);
}
.bp-watcher__status {
  display: flex;
  align-items: center;
  gap: 6px;
}
.bp-watcher__dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: var(--good);
  box-shadow: 0 0 0 2px rgba(84, 124, 74, 0.18);
}
.bp-watcher__label { font-family: var(--hand); }
.bp-watcher__hint {
  font-family: var(--label);
  font-size: 9px;
  letter-spacing: 0.10em;
  text-transform: lowercase;
  color: var(--ink-faint);
}
</style>
