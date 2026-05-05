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

const wikiBase = (game: Game): string =>
  game === 'poe2' ? 'https://www.poe2wiki.net/wiki/' : 'https://www.poewiki.net/wiki/';

const makeWikiUrl = (name: string, game: Game): string =>
  wikiBase(game) + encodeURIComponent(name.trim().replace(/\s+/g, '_'));

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
    const safe = escapeHtml(raw);
    if (!looksLikePoeEntity(raw)) {
      // Emphasis without linking — Bestel uses backticks to highlight values
      // (numbers, percentages) in addition to entities. Render as a small
      // amber emphasis chip instead of a broken link.
      return `<span class="md-emph">${safe}</span>`;
    }
    const url = makeWikiUrl(raw, game);
    const safeUrl = escapeHtml(url);
    return `<a class="link wiki-link" data-wiki-url="${safeUrl}" href="${safeUrl}">${safe}</a>`;
  };
  return DOMPurify.sanitize(md.render(text), {
    ADD_ATTR: ['target', 'rel', 'data-wiki-url', 'class'],
  });
};
