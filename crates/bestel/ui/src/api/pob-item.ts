// Parses raw Path of Building item text into a structured form
// suitable for in-game-style tooltip rendering.
//
// PoB stores items in a textual format that mirrors the in-game item text,
// plus a few metadata lines (rarity, sockets, item level…) and a series of
// `{tag}` annotations on mod lines. This parser strips internal noise
// (`{range:N}`, `Unique ID:`, `Crafted: true`, base percentile flags…) and
// classifies each mod line by its source (implicit / explicit / crafted /
// enchant / fractured / eater / exarch / corrupted).

export type Rarity = 'normal' | 'magic' | 'rare' | 'unique' | 'gem' | 'currency' | 'relic';

export type ModSource =
  | 'implicit'
  | 'explicit'
  | 'crafted'
  | 'enchant'
  | 'fractured'
  | 'eater'
  | 'exarch'
  | 'unveiled'
  | 'corrupted';

export interface ParsedMod {
  text: string;
  source: ModSource;
}

export interface ParsedItem {
  rarity: Rarity;
  name: string | null;
  base: string | null;
  itemLevel: number | null;
  quality: number | null;
  sockets: string | null;
  levelReq: number | null;
  uniqueId: string | null;
  /** "Evasion: 2200", "Energy Shield: 482", etc. — defensive / offensive stats. */
  properties: { label: string; value: string }[];
  implicits: ParsedMod[];
  explicits: ParsedMod[];
  /** Standalone flag lines like `Corrupted`, `Searing Exarch Item`. */
  flags: string[];
}

const KEY_VALUE_KEEP = new Set([
  'Quality', 'Sockets', 'LevelReq', 'Item Level', 'Implicits',
  'Armour', 'Evasion', 'Energy Shield', 'Block', 'Ward', 'Spirit',
  'Radius', 'Selected Variant', 'Variant', 'Requires Class',
  'Stack Size',
]);

const KEY_VALUE_HIDE = new Set([
  'Crafted', 'Source', 'StatOrder', 'Catalyst', 'CatalystQuality',
  'Prefix', 'Suffix', 'AlternateQuality', 'Cluster',
  'Shaper', 'Elder', 'Synthesised',
  'EvasionBasePercentile', 'EnergyShieldBasePercentile',
  'ArmourBasePercentile', 'WardBasePercentile',
  'BlockBasePercentile', 'StatRoll',
  'Searing Exarch', 'Eater of Worlds',
]);

const FLAG_LINES = new Set([
  'Corrupted', 'Mirrored', 'Split', 'Searing Exarch Item', 'Eater of Worlds Item',
  'Synthesised Item', 'Shaper Item', 'Elder Item', 'Fractured Item', 'Influenced Item',
  'Hunter Item', 'Crusader Item', 'Redeemer Item', 'Warlord Item', 'Veiled Item',
  'Replica',
]);

function parseModSource(line: string): ParsedMod {
  let cleaned = line;
  // Strip {range:N.NN}
  cleaned = cleaned.replace(/^\s*\{range:[\d.]+\}\s*/, '');
  // Strip {tags:list_of,tags}
  cleaned = cleaned.replace(/\{tags:[^}]+\}/g, '');
  // Strip {variant:N}
  cleaned = cleaned.replace(/\{variant:[^}]+\}/g, '');
  // Strip {custom}
  cleaned = cleaned.replace(/\{custom\}/g, '');
  let source: ModSource = 'explicit';
  const sourceMatch = cleaned.match(/^\{(crafted|eater|exarch|fractured|enchant|enchant_[a-z0-9_]+|unveiled|corrupted)\}\s*/);
  if (sourceMatch) {
    const tag = sourceMatch[1];
    cleaned = cleaned.slice(sourceMatch[0].length);
    if (tag.startsWith('enchant')) source = 'enchant';
    else source = tag as ModSource;
  }
  // Catch any remaining {...} prefix
  cleaned = cleaned.replace(/^\s*\{[^}]+\}\s*/, '');
  return { text: cleaned.trim(), source };
}

const RARITY_SET: Set<Rarity> = new Set(['normal', 'magic', 'rare', 'unique', 'gem', 'currency', 'relic']);

export function parsePobItem(raw: string): ParsedItem {
  const lines = raw.split(/\r?\n/).map((l) => l.trimEnd());
  let rarity: Rarity = 'normal';
  let name: string | null = null;
  let base: string | null = null;
  let itemLevel: number | null = null;
  let quality: number | null = null;
  let sockets: string | null = null;
  let levelReq: number | null = null;
  let uniqueId: string | null = null;
  const properties: { label: string; value: string }[] = [];
  const implicitLines: ParsedMod[] = [];
  const explicitLines: ParsedMod[] = [];
  const flags = new Set<string>();
  let implicitCount = 0;
  let i = 0;

  // Skip leading blank lines
  while (i < lines.length && !lines[i].trim()) i++;

  // Line 1: Rarity: <RARE|UNIQUE|MAGIC|NORMAL|GEM|RELIC>
  if (lines[i]?.startsWith('Rarity:')) {
    const v = lines[i].slice('Rarity:'.length).trim().toLowerCase();
    if (RARITY_SET.has(v as Rarity)) rarity = v as Rarity;
    i++;
  }

  // Line 2 (optional name) + Line 3 (base type).
  // For unique/rare/gem/relic items, the next two non-empty lines are name + base.
  // For magic items there's a single combined line.
  // For normal items there's just a base.
  const nextLine = (): string | null => {
    while (i < lines.length && !lines[i].trim()) i++;
    if (i >= lines.length) return null;
    return lines[i++];
  };

  if (rarity === 'unique' || rarity === 'rare' || rarity === 'gem' || rarity === 'relic') {
    name = nextLine();
    base = nextLine();
  } else if (rarity === 'magic') {
    name = nextLine();
    base = name;
  } else {
    base = nextLine();
  }

  // Now parse remaining lines
  const remaining: string[] = [];
  for (; i < lines.length; i++) {
    const line = lines[i];
    const trimmed = line.trim();
    if (!trimmed) continue;

    if (FLAG_LINES.has(trimmed)) {
      flags.add(trimmed);
      continue;
    }

    const colon = trimmed.indexOf(':');
    if (colon > 0 && colon < 32 && !trimmed.startsWith('{') && !trimmed.startsWith('+')) {
      const key = trimmed.slice(0, colon).trim();
      const val = trimmed.slice(colon + 1).trim();
      if (KEY_VALUE_HIDE.has(key)) continue;
      if (key === 'Unique ID') {
        uniqueId = val;
        continue;
      }
      if (key === 'Quality') {
        quality = parseInt(val, 10);
        continue;
      }
      if (key === 'Sockets') {
        sockets = val;
        continue;
      }
      if (key === 'LevelReq') {
        levelReq = parseInt(val, 10);
        continue;
      }
      if (key === 'Item Level') {
        itemLevel = parseInt(val, 10);
        continue;
      }
      if (key === 'Implicits') {
        implicitCount = parseInt(val, 10) || 0;
        continue;
      }
      if (KEY_VALUE_KEEP.has(key)) {
        properties.push({ label: key, value: val });
        continue;
      }
      // Unknown key:value — likely PoB internal metadata. Hide it.
      continue;
    }

    // Otherwise it's a mod line
    remaining.push(line);
  }

  for (let j = 0; j < remaining.length; j++) {
    const parsed = parseModSource(remaining[j]);
    if (j < implicitCount && parsed.source === 'explicit') {
      parsed.source = 'implicit';
    }
    if (parsed.source === 'implicit' || j < implicitCount) {
      implicitLines.push({ ...parsed, source: parsed.source === 'implicit' ? 'implicit' : 'implicit' });
    } else {
      explicitLines.push(parsed);
    }
  }

  return {
    rarity,
    name,
    base,
    itemLevel,
    quality,
    sockets,
    levelReq,
    uniqueId,
    properties,
    implicits: implicitLines,
    explicits: explicitLines,
    flags: Array.from(flags),
  };
}
