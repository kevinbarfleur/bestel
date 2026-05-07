import MarkdownIt from 'markdown-it';
import hljs from 'highlight.js';
import DOMPurify from 'dompurify';

import type { Game } from './types';

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

const makeWikiUrl = (name: string, game: Game): string =>
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

export const renderMarkdown = (text: string, game: Game): string => {
  md.renderer.rules.code_inline = (tokens: ReadonlyArray<{ content: string }>, idx: number) => {
    const raw = tokens[idx].content;
    // 1. Element / status entity tag (`fire:75%`, `good:capped`, …)
    const tag = tryRenderEntityTag(raw);
    if (tag) return tag;
    // 2. PoE entity → wiki link pill (MonoChipLink style)
    const safe = escapeHtml(raw);
    if (!looksLikePoeEntity(raw)) {
      // Emphasis without linking — Bestel uses backticks to highlight values
      // (numbers, percentages) in addition to entities. Render as an
      // emphasis chip in monochrome.
      return `<span class="md-emph">${safe}</span>`;
    }
    const url = makeWikiUrl(raw, game);
    const safeUrl = escapeHtml(url);
    return `<a class="link wiki-link" data-wiki-url="${safeUrl}" href="${safeUrl}">${EXTERNAL_LINK_SVG}<span class="md-link-label">${safe}</span></a>`;
  };
  return DOMPurify.sanitize(md.render(text), {
    ADD_ATTR: ['target', 'rel', 'data-wiki-url', 'class'],
  });
};
