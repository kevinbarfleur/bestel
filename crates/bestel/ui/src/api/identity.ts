// Build Identity card parser. The canonical grammar is defined in
// `prompts/references/27_response_contracts.md` (§ "Build identity card —
// extended grammar") and emitted by `crates/bestel-core/src/llm/tools.rs::
// format_identity_line()`. Keep the regex aligned with that source — drift
// here means the linter's BUILD_IDENTITY_REQUIRED rule disagrees with what
// the UI styles, which would surface as a card that lints as missing.
//
// Format:
//   Identity: defense=<list>, hit_model=<single|with-suffix>,
//   mechanic=<list>. Defining uniques: <Name> (<category>), ... .
//   Conversion: <chain or "none">.
//
// The strict trailing period is intentional: while the line is streaming
// char-by-char, partial matches must fall through to plain markdown so
// the card only "lands" once fully assembled.

export type IdentityCategory = 'engine' | 'defining' | 'amplifier' | 'enabler';

export interface IdentityUnique {
  name: string;
  category: IdentityCategory | null;
}

export interface ParsedIdentity {
  defense: string[];
  hitModel: string;
  mechanic: string[];
  uniques: IdentityUnique[];
  conversion: string | null;
}

const IDENTITY_RE =
  /^Identity:\s+defense=([^,]+),\s+hit_model=([^,]+(?:\s*\+\s*[^,]+)?),\s+mechanic=([^.]+)\.(?:\s+Defining uniques:\s+(.+?)\.)?(?:\s+Conversion:\s+(.+?)\.)?\s*$/;

const VALID_CATEGORIES: ReadonlySet<IdentityCategory> = new Set([
  'engine',
  'defining',
  'amplifier',
  'enabler',
]);

const splitTokens = (input: string): string[] =>
  input
    .split(',')
    .map((tok) => tok.trim())
    .filter((tok) => tok.length > 0);

const parseUnique = (raw: string): IdentityUnique => {
  const match = raw.match(/^(.+?)\s*\(([^)]+)\)\s*$/);
  if (!match) return { name: raw.trim(), category: null };
  const cat = match[2].trim().toLowerCase();
  const category = VALID_CATEGORIES.has(cat as IdentityCategory)
    ? (cat as IdentityCategory)
    : null;
  return { name: match[1].trim(), category };
};

/**
 * Split a Defining-uniques string on top-level commas, preserving
 * commas that appear inside the (category) parens. Naive split-on-`, `
 * would over-split on a category like "Watcher's Eye, amplifier".
 */
const splitUniques = (input: string): string[] => {
  const parts: string[] = [];
  let depth = 0;
  let buf = '';
  for (const ch of input) {
    if (ch === '(') depth += 1;
    else if (ch === ')') depth = Math.max(0, depth - 1);
    if (ch === ',' && depth === 0) {
      const trimmed = buf.trim();
      if (trimmed.length > 0) parts.push(trimmed);
      buf = '';
      continue;
    }
    buf += ch;
  }
  const tail = buf.trim();
  if (tail.length > 0) parts.push(tail);
  return parts;
};

export function parseIdentityLine(line: string): ParsedIdentity | null {
  const match = IDENTITY_RE.exec(line.trim());
  if (!match) return null;
  const [, defense, hitModel, mechanic, uniquesRaw, conversionRaw] = match;
  const uniques = uniquesRaw
    ? splitUniques(uniquesRaw).map(parseUnique)
    : [];
  return {
    defense: splitTokens(defense),
    hitModel: hitModel.trim(),
    mechanic: splitTokens(mechanic),
    uniques,
    conversion: conversionRaw ? conversionRaw.trim() : null,
  };
}

const escapeHtml = (s: string): string =>
  s
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');

/**
 * Build the styled card HTML from a parsed Identity. The rendered HTML
 * uses tokens already defined in `markdown.css` (`--paper`, `--ink`,
 * `--ink-soft`, `--amber`, `--good`, `--bad`, `--note`, `--paper-line`);
 * any drift in axis/category names should land here so DOMPurify keeps
 * allowing the resulting `class` attributes. Layout mirrors the
 * Claude-Design Build Identity Card mockup (`PbumU6vt…`):
 *   ◆ identity (margin tag)
 *   3-column axis grid: defense | hit | mechanic (separated by 1px vertical rules)
 *   uniques row: per-role chip styling (engine/defining/amplifier/enabler)
 *   conversion row (optional): mono-font caption when conversion chain exists
 */
export function buildIdentityCardHtml(parsed: ParsedIdentity): string {
  const axisCell = (label: string, tokens: string[]): string => {
    if (tokens.length === 0) return '';
    const parts = tokens
      .map((t) => `<span class="identity-axis__t">${escapeHtml(t)}</span>`)
      .join('<span class="identity-axis__sep-dot">·</span>');
    return `<div class="identity-axis"><div class="identity-axis__k">${label}</div><div class="identity-axis__tokens">${parts}</div></div>`;
  };

  const axisRow = `<div class="identity-card__axes">${axisCell(
    'defense',
    parsed.defense,
  )}<span class="identity-axis__sep" aria-hidden="true"></span>${axisCell(
    'hit',
    [parsed.hitModel],
  )}<span class="identity-axis__sep" aria-hidden="true"></span>${axisCell(
    'mechanic',
    parsed.mechanic,
  )}</div>`;

  const uniqueChips = parsed.uniques
    .map((u) => {
      const cat = u.category ?? 'enabler';
      return `<span class="identity-unique identity-unique--${cat}"><span class="identity-unique__role">${cat}</span><span class="identity-unique__name">${escapeHtml(
        u.name,
      )}</span></span>`;
    })
    .join('');

  const uniquesRow =
    parsed.uniques.length === 0
      ? ''
      : `<div class="identity-card__uniques"><span class="identity-card__uniques-label">uniques</span>${uniqueChips}</div>`;

  const conversionRow =
    parsed.conversion && parsed.conversion.toLowerCase() !== 'none'
      ? `<div class="identity-card__conversion"><span class="identity-card__conv-label">conversion</span><span class="identity-card__conv-chain">${escapeHtml(
          parsed.conversion,
        )}</span></div>`
      : '';

  return `<div class="identity-card"><span class="identity-card__tag">◆ identity</span>${axisRow}${uniquesRow}${conversionRow}</div>`;
}

/**
 * Strip leading markdown emphasis wrappers (`**Identity:**`, `__Identity:__`,
 * `*Identity:*`) and any closing wrapper in the matching position. Models
 * occasionally bold-wrap the `Identity:` label even though the response
 * contract forbids it; the parser stays tolerant so the card still renders.
 * Returns the line with the wrappers removed; passes the input through
 * unchanged when no wrapper is present.
 */
function unwrapEmphasis(line: string): string {
  // Strip a leading `**` / `__` / `*` and the closest matching closer that
  // sits before the first whitespace (covers `**Identity:**` and the rarer
  // `*Identity:*`). We don't try to balance arbitrary spans — we only care
  // that the line `startsWith('Identity:')` after the strip.
  let s = line;
  for (const marker of ['**', '__', '*']) {
    if (s.startsWith(marker)) {
      const after = s.slice(marker.length);
      const close = after.indexOf(marker);
      if (close >= 0) {
        s = after.slice(0, close) + after.slice(close + marker.length);
      } else {
        s = after;
      }
      break;
    }
  }
  return s;
}

/**
 * Scan the text for an Identity line and return both the parsed card and
 * the text with that line removed (plus a single trailing blank line if
 * present so the prose below flows without an extra gap). The grammar
 * is contract-bound to be the FIRST line of the answer per
 * `27_response_contracts.md`, but in practice models occasionally insert
 * a hook sentence above it; scanning every line keeps the card visible
 * regardless. Only the first match is consumed — additional Identity
 * mentions in mid-prose stay as-is (rare; never observed).
 */
export function extractIdentity(text: string): {
  parsed: ParsedIdentity;
  textWithoutLine: string;
} | null {
  const lines = text.split('\n');
  for (let i = 0; i < lines.length; i += 1) {
    const stripped = unwrapEmphasis(lines[i].trim());
    if (!stripped.startsWith('Identity:')) continue;
    const parsed = parseIdentityLine(stripped);
    if (!parsed) continue;
    const before = lines.slice(0, i);
    let after = lines.slice(i + 1);
    // Eat one blank separator if present so removing the Identity line
    // doesn't leave a double-newline stub.
    if (after.length > 0 && after[0].trim() === '') {
      after = after.slice(1);
    }
    // Same on the leading side: if the line above was blank, drop it
    // too (otherwise the card prepends to a stub paragraph break).
    let beforeTrimmed = before;
    if (beforeTrimmed.length > 0 && beforeTrimmed[beforeTrimmed.length - 1].trim() === '') {
      beforeTrimmed = beforeTrimmed.slice(0, -1);
    }
    const textWithoutLine = [...beforeTrimmed, ...after].join('\n');
    return { parsed, textWithoutLine };
  }
  return null;
}

/**
 * Backward-compatible wrapper: returns the input text without the
 * Identity line, or the original text if no line was found.
 */
export function stripIdentityLine(text: string): string {
  const found = extractIdentity(text);
  return found ? found.textWithoutLine : text;
}
