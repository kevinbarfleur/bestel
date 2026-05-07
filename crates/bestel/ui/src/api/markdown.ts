import MarkdownIt from 'markdown-it';
import hljs from 'highlight.js';
import DOMPurify from 'dompurify';

import type { Game } from './types';
import type { PanelArtifact, PanelArtifactType } from '../stores/ui';

const escapeHtml = (s: string): string =>
  s
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;');

const md: MarkdownIt = new MarkdownIt({
  html: false,
  linkify: true,
  breaks: false,
  highlight: (str: string, lang: string): string => {
    if (lang && hljs.getLanguage(lang)) {
      try {
        const out = hljs.highlight(str, { language: lang, ignoreIllegals: true }).value;
        return `<pre class="hljs"><code>${out}</code></pre>`;
      } catch {
        // fall through
      }
    }
    return `<pre class="hljs"><code>${escapeHtml(str)}</code></pre>`;
  },
});

/**
 * Leading icon prefixed to every web-bound link in the chat body (wiki-link
 * pills + plain markdown [text](url) in prose). Two interlinked rings —
 * the universal "link / chain" affordance. Inline SVG so it survives
 * DOMPurify and renders at the same em-size as the surrounding text.
 */
const EXTERNAL_LINK_SVG =
  '<svg class="md-link-ico" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">' +
  '<path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"/>' +
  '<path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"/>' +
  '</svg>';

/**
 * Loupe glyph prefixed to side-panel buttons (the ⟦panel:…⟧ markers Bestel
 * embeds in the final answer). Visually distinct from EXTERNAL_LINK_SVG —
 * chain icon = open in webview, magnifier = open in side panel.
 */
const LOUPE_SVG =
  '<svg class="md-panel-ico" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">' +
  '<circle cx="11" cy="11" r="6"/>' +
  '<path d="M20 20l-4.5-4.5"/>' +
  '</svg>';

const ALLOWED_PANEL_TYPES: ReadonlySet<PanelArtifactType> = new Set([
  'item-card',
  'gem-detail',
  'mechanic',
  'markdown',
]);

const PANEL_SIDECAR_RE = /⟦panel-data⟧\s*([\s\S]*?)\s*⟦\/panel-data⟧/;
/**
 * Unclosed sidecar — opens with ⟦panel-data⟧ but never closes. Happens
 * when the model truncates mid-JSON (max_tokens hit, network drop). We
 * hide everything from the opening tag to end-of-message so the user
 * doesn't see a JSON dump leak into the prose; the panelMap stays empty
 * (the truncated JSON isn't parseable).
 */
const PANEL_SIDECAR_UNCLOSED_RE = /⟦panel-data⟧[\s\S]*$/;
const PANEL_MARKER_RE = /⟦panel(\*?):([a-z-]+):([^⟧]+?)⟧/g;

/**
 * Pre-pass over an assistant message text. Locates the optional
 * ⟦panel-data⟧{json}⟦/panel-data⟧ sidecar, parses the JSON, validates each
 * entry against the allowed panel-artifact shape, and returns:
 *   - `stripped`: the text with the sidecar block removed, ready to feed
 *     `renderMarkdown`. Markers (⟦panel:…⟧) are kept inline.
 *   - `map`: { name → PanelArtifact } indexed by marker name. Buttons in
 *     prose look up their payload here on click.
 *
 * Tolerant: missing sidecar → empty map + same text. Malformed JSON → warn,
 * strip the sidecar, empty map. Invalid entries are silently skipped.
 */
/**
 * Streaming-friendly variant. Returns null when either the opening OR
 * closing sidecar tag is missing — i.e. the sidecar block hasn't fully
 * arrived yet. Once both tags are seen, delegates to `extractPanelSidecar`
 * (which handles malformed JSON and validation). Used by the chat store's
 * RAF drain to populate `panelMap` as soon as the sidecar block lands,
 * without waiting for end-of-message finalization.
 */
export function tryExtractPanelSidecarPartial(
  text: string,
): { stripped: string; map: Record<string, PanelArtifact> } | null {
  if (!text.includes('⟦panel-data⟧') || !text.includes('⟦/panel-data⟧')) {
    return null;
  }
  return extractPanelSidecar(text);
}

export function extractPanelSidecar(
  text: string,
): { stripped: string; map: Record<string, PanelArtifact> } {
  const match = PANEL_SIDECAR_RE.exec(text);
  if (!match) {
    // No closed sidecar — but the message may still carry a truncated
    // opening tag (model cut off mid-JSON). Strip it so the chat doesn't
    // leak a raw JSON dump into the prose.
    if (PANEL_SIDECAR_UNCLOSED_RE.test(text)) {
      return { stripped: text.replace(PANEL_SIDECAR_UNCLOSED_RE, '').trimEnd(), map: {} };
    }
    return { stripped: text, map: {} };
  }
  const stripped = text.replace(PANEL_SIDECAR_RE, '').trimEnd();
  let parsed: unknown;
  try {
    parsed = JSON.parse(match[1]);
  } catch (err) {
    // eslint-disable-next-line no-console
    console.warn('[panel-data] JSON parse failed', err);
    return { stripped, map: {} };
  }
  if (!parsed || typeof parsed !== 'object' || Array.isArray(parsed)) {
    return { stripped, map: {} };
  }
  const map: Record<string, PanelArtifact> = {};
  for (const [name, raw] of Object.entries(parsed as Record<string, unknown>)) {
    if (!raw || typeof raw !== 'object') continue;
    const obj = raw as Record<string, unknown>;
    const type = obj.type;
    const title = obj.title;
    if (
      typeof type !== 'string' ||
      !ALLOWED_PANEL_TYPES.has(type as PanelArtifactType) ||
      typeof title !== 'string' ||
      !('payload' in obj)
    ) {
      continue;
    }
    map[name] = {
      id: name,
      type: type as PanelArtifactType,
      title,
      payload: obj.payload,
      source: 'agent',
    };
  }
  return { stripped, map };
}

// Override the default `link_open` rule to inject the external-link icon
// right after the opening tag. Plain markdown links `[label](url)` in prose
// then render as `<a href>...{icon}{label}</a>`. CSS hides the icon when the
// link sits inside a list (Sources section).
const defaultLinkOpen =
  md.renderer.rules.link_open ||
  function (tokens, idx, options, _env, self) {
    return self.renderToken(tokens, idx, options);
  };
md.renderer.rules.link_open = function (tokens, idx, options, env, self) {
  return defaultLinkOpen(tokens, idx, options, env, self) + EXTERNAL_LINK_SVG;
};

const wikiBase = (game: Game): string =>
  game === 'poe2' ? 'https://www.poe2wiki.net/wiki/' : 'https://www.poewiki.net/wiki/';

export const makeWikiUrl = (name: string, game: Game): string =>
  wikiBase(game) + encodeURIComponent(name.trim().replace(/\s+/g, '_'));

/**
 * Element-prefixed entity tag — rendered as a non-clickable colored chip
 * inline. The LLM is taught (via SYSTEM_PROMPT) to wrap elemental and status
 * values like this:
 *   `fire:75%` → red-tinted chip
 *   `cold:-12%` → blue-tinted chip
 *   `lit:71/75` or `lightning:71` → yellow-tinted chip
 *   `chaos:-40%` → purple-tinted chip
 *   `phys:35%` → neutral-tinted chip
 *   `good:capped` → green chip (status)
 *   `bad:vulnerable` → red chip (status)
 *   `note:stale` → amber chip (status)
 *
 * Returns the rendered HTML if the raw matches an entity-tag pattern, or
 * null if the raw should fall through to wiki-link/emphasis logic.
 */
const ENTITY_TAG_RE =
  /^\s*(fire|cold|lit|lightning|chaos|phys|good|bad|note)\s*:\s*(.+?)\s*$/i;

const ENTITY_CLASS: Record<string, string> = {
  fire: 'entity-tag--fire',
  cold: 'entity-tag--cold',
  lit: 'entity-tag--lit',
  lightning: 'entity-tag--lit',
  chaos: 'entity-tag--chaos',
  phys: 'entity-tag--phys',
  good: 'entity-tag--good',
  bad: 'entity-tag--bad',
  note: 'entity-tag--note',
};

function tryRenderEntityTag(raw: string): string | null {
  const m = ENTITY_TAG_RE.exec(raw);
  if (!m) return null;
  const kind = m[1].toLowerCase();
  const cls = ENTITY_CLASS[kind];
  if (!cls) return null;
  const value = escapeHtml(m[2]);
  return `<span class="entity-tag ${cls}">${value}</span>`;
}

/**
 * Decide whether the raw content of a backtick-inline-code looks like a
 * PoE entity worth linking to the wiki. Numbers, percentages, ratios,
 * generic lowercase words and label-with-colon (`Synergies:`) should NOT
 * become wiki links — they only render as emphasised text.
 */
function looksLikePoeEntity(raw: string): boolean {
  const s = raw.trim();
  if (s.length === 0 || s.length > 60) return false;
  // Pure numbers / percentages / ranges / decimals / dimensions
  if (/^[+\-±]?\d+(\.\d+)?\s*[%xMKkm]?$/.test(s)) return false;
  if (/^\(?[+\-±]?\d+([.,]\d+)?[-–]\d+([.,]\d+)?\)?\s*%?$/.test(s)) return false;
  if (/^\d+\s*\/\s*\d+/.test(s)) return false;
  if (/^lvl\s*\d+/i.test(s)) return false;
  // Label-with-colon (`Synergies:`, `Path:`, `Note:`)
  if (/:$/.test(s)) return false;
  // Trailing punctuation that's clearly not part of an entity name
  if (/[.,;!?]$/.test(s)) return false;
  // Must start with an uppercase letter (Title Case is the PoE convention)
  if (!/^[A-Z]/.test(s)) return false;
  // Allow only letters, spaces, apostrophes, hyphens, ampersands, "of"/"the"
  // -style connectives, trailing roman numeral, single digit suffix.
  if (!/^[A-Z][A-Za-z0-9'’\-\s&]*$/.test(s)) return false;
  // Single short word that's all caps and < 4 chars is likely an acronym
  // we don't want to link (e.g. "STR", "DPS", "T16").
  if (/^[A-Z0-9]{1,4}$/.test(s)) return false;
  return true;
}

/**
 * Render a marker token into a <button> tag the chat layer can dispatch on.
 * The button has no payload of its own — the chat store keeps the parsed
 * sidecar map and ChatMessage.vue resolves the key on click.
 */
function renderPanelMarker(_full: string, star: string, _type: string, name: string): string {
  const safeName = escapeHtml(name);
  const isPrimary = star === '*';
  const cls = isPrimary ? 'panel-btn panel-btn--primary' : 'panel-btn';
  const primaryAttr = isPrimary ? ' data-panel-primary="1"' : '';
  return (
    `<button type="button" class="${cls}" data-panel-key="${safeName}"${primaryAttr}>` +
    `${LOUPE_SVG}<span class="md-link-label">${safeName}</span></button>`
  );
}

export const renderMarkdown = (
  text: string,
  game: Game,
  panelKeys?: ReadonlySet<string>,
): string => {
  // 1. Strip the optional ⟦panel-data⟧{...}⟦/panel-data⟧ sidecar — its JSON
  //    is the chat store's responsibility (extractPanelSidecar). Inline
  //    ⟦panel:…⟧ markers stay; they're converted to buttons in step 3.
  //    Also strip an unclosed opening tag in case the model truncated mid
  //    JSON (max_tokens hit) — would otherwise leak the raw JSON dump.
  const stripped = text
    .replace(PANEL_SIDECAR_RE, '')
    .replace(PANEL_SIDECAR_UNCLOSED_RE, '')
    .trimEnd();
  md.renderer.rules.code_inline = (tokens: ReadonlyArray<{ content: string }>, idx: number) => {
    const raw = tokens[idx].content;
    // 1a. Element / status entity tag (`fire:75%`, `good:capped`, …)
    const tag = tryRenderEntityTag(raw);
    if (tag) return tag;
    // 1b. PoE entity → wiki link pill (MonoChipLink style)
    const safe = escapeHtml(raw);
    if (!looksLikePoeEntity(raw)) {
      // Emphasis without linking — Bestel uses backticks to highlight values
      // (numbers, percentages) in addition to entities. Render as an
      // emphasis chip in monochrome.
      return `<span class="md-emph">${safe}</span>`;
    }
    // 1c. Dedup: if this entity has a side-panel marker in the same
    //     message, the panel button is the canonical affordance. Drop
    //     the wiki pill (chain icon would duplicate the panel's
    //     external-link button) and render the entity as plain
    //     emphasis instead. Defensive against the model emitting both
    //     a backtick and a marker for the same name.
    if (panelKeys && panelKeys.has(raw.trim())) {
      return `<span class="md-emph">${safe}</span>`;
    }
    const url = makeWikiUrl(raw, game);
    const safeUrl = escapeHtml(url);
    return `<a class="link wiki-link" data-wiki-url="${safeUrl}" href="${safeUrl}">${EXTERNAL_LINK_SVG}<span class="md-link-label">${safe}</span></a>`;
  };
  // 2. Markdown render (linkify, code blocks, lists, wiki/entity tags).
  let html = md.render(stripped);
  // 3. Post-pass: replace ⟦panel:type:name⟧ markers with <button> HTML.
  //    Reset the global regex's lastIndex implicitly via .replace().
  html = html.replace(PANEL_MARKER_RE, renderPanelMarker);
  return DOMPurify.sanitize(html, {
    ADD_ATTR: [
      'target',
      'rel',
      'data-wiki-url',
      'data-panel-key',
      'data-panel-primary',
      'class',
      'type',
    ],
  });
};
