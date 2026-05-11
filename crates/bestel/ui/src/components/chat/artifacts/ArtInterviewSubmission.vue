<script setup lang="ts">
import { computed } from 'vue';

import ArtShell from './ArtShell.vue';
import ArtHead from './ArtHead.vue';

// Parses the structured `[INTERVIEW SUBMISSION ...]` user message body
// (produced by `useSheetStore().submitInterview()`) into renderable
// pieces. Format reference: `prompts/references/32_build_sheets.md`
// § Phase 3. Renders the result as an always-expanded summary card so
// the submission shows up as a persistent artifact in the chat history
// instead of a raw markdown text block.

interface Section {
  id: string;
  title: string;
  body: string;
}
interface Answer {
  questionId: string;
  sectionId: string;
  title: string;
  value: string;
}

const props = defineProps<{ text: string }>();

const SECTION_LINE_RE = /^- \*\*([a-z_-]+)\*\* \(([^)]+)\): (.*)$/;
const ANSWER_LINE_RE = /^- \*\*([a-z_-]+)\*\* \[([a-z_-]+)\] \(([^)]+)\): (.*)$/;
const SECTION_ORDER: Record<string, number> = {
  identity: 0,
  archetype: 1,
  damage: 2,
  defense: 3,
  items: 4,
  intent: 5,
};

function splitBlocks(text: string): Record<string, string> {
  const blocks: Record<string, string> = {};
  let currentKey: string | null = null;
  const buf: string[] = [];
  const flush = () => {
    if (currentKey) blocks[currentKey] = buf.join('\n').trim();
    buf.length = 0;
  };
  for (const raw of text.split('\n')) {
    const line = raw.trimEnd();
    const m = line.match(/^## (.+)$/);
    if (m) {
      flush();
      currentKey = m[1].trim().toLowerCase();
      continue;
    }
    if (currentKey) buf.push(line);
  }
  flush();
  return blocks;
}

const parsed = computed(() => {
  const blocks = splitBlocks(props.text);
  const sectionLines = (blocks['sections'] ?? '').split('\n');
  const sections: Section[] = [];
  let activeSection: Section | null = null;
  for (const line of sectionLines) {
    const m = line.match(SECTION_LINE_RE);
    if (m) {
      if (activeSection) sections.push(activeSection);
      activeSection = { id: m[1], title: m[2], body: m[3].trim() };
      continue;
    }
    if (activeSection && line.trim()) {
      activeSection.body = (activeSection.body + '\n' + line).trim();
    }
  }
  if (activeSection) sections.push(activeSection);
  sections.sort(
    (a, b) =>
      (SECTION_ORDER[a.id] ?? 99) - (SECTION_ORDER[b.id] ?? 99),
  );

  const answerLines = (blocks['question answers'] ?? '').split('\n');
  const answers: Answer[] = [];
  for (const line of answerLines) {
    const m = line.match(ANSWER_LINE_RE);
    if (m) {
      answers.push({
        questionId: m[1],
        sectionId: m[2],
        title: m[3],
        value: m[4].trim(),
      });
    }
  }

  const notesRaw = (blocks['notes'] ?? '').trim();
  const notes =
    notesRaw && notesRaw !== '(none)' ? notesRaw.replace(/^"|"$/g, '') : null;

  return { sections, answers, notes };
});

const identityHeadline = computed(() => {
  const id = parsed.value.sections.find((s) => s.id === 'identity');
  if (!id) return null;
  return id.body.replace(/\*\*/g, '').split(/\s*\|\s*/)[0].trim();
});

const sectionPreviews = computed(() =>
  parsed.value.sections.map((s) => ({
    id: s.id,
    title: s.title.toUpperCase(),
    preview: stripMarkdown(firstNonEmptyLine(s.body)),
  })),
);

function firstNonEmptyLine(body: string): string {
  for (const line of body.split('\n')) {
    const trimmed = line.replace(/^[-*]\s*/, '').trim();
    if (trimmed) return trimmed;
  }
  return body.trim();
}

function stripMarkdown(s: string): string {
  return s
    .replace(/\*\*([^*]+)\*\*/g, '$1')
    .replace(/`([^`]+)`/g, '$1')
    .replace(/\*([^*]+)\*/g, '$1');
}

const answerRows = computed(() =>
  parsed.value.answers.map((a) => ({
    label: shortLabel(a.title),
    value: stripMarkdown(firstClause(a.value)),
  })),
);

function shortLabel(title: string): string {
  // Pull the first 1-3 words that capture the question's topic.
  return title.replace(/[—–\-:].*$/, '').trim();
}
function firstClause(value: string): string {
  const m = value.match(/^([^|()]+)/);
  return (m ? m[1] : value).trim();
}
</script>

<template>
  <ArtShell accent="amber">
    <ArtHead
      kind="Interview submission"
      title="Build sheet"
      :meta="identityHeadline ? `· ${identityHeadline}` : ''"
      :status="{ label: '✓ confirmed', tone: 'good' }"
    />
    <div class="iv-grid">
      <div v-for="s in sectionPreviews" :key="s.id" class="iv-leader">
        <span class="iv-leader__k">{{ s.title }}</span>
        <span class="iv-leader__dots" />
        <span class="iv-leader__v">{{ s.preview }}</span>
      </div>
    </div>
    <div v-if="answerRows.length" class="iv-answers">
      <div class="iv-answers__head">ANSWERS</div>
      <div v-for="(r, i) in answerRows" :key="i" class="iv-answer">
        <span class="iv-answer__k">{{ r.label }}</span>
        <span class="iv-answer__arr">→</span>
        <span class="iv-answer__v">{{ r.value }}</span>
      </div>
    </div>
    <p v-if="parsed.notes" class="iv-notes">
      <span class="iv-notes__head">NOTES</span>
      <span class="iv-notes__body">"{{ parsed.notes }}"</span>
    </p>
  </ArtShell>
</template>

<style scoped>
.iv-grid {
  display: flex;
  flex-direction: column;
  gap: 3px;
  margin-top: 4px;
}
.iv-leader {
  display: flex;
  align-items: baseline;
  gap: 6px;
}
.iv-leader__k {
  font-family: var(--label);
  font-size: 9px;
  letter-spacing: 0.10em;
  text-transform: uppercase;
  color: var(--ink-faint);
  font-weight: 500;
  flex-shrink: 0;
  width: 78px;
  white-space: nowrap;
}
.iv-leader__dots {
  flex: 1;
  min-width: 12px;
  border-bottom: 1px dotted var(--paper-line);
  transform: translateY(-3px);
}
.iv-leader__v {
  font-family: var(--hand);
  font-size: 12px;
  color: var(--ink-soft);
  flex: 0 1 auto;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 70%;
}
.iv-answers {
  margin-top: 10px;
  padding-top: 8px;
  border-top: 1px dotted var(--paper-line);
}
.iv-answers__head {
  font-family: var(--label);
  font-size: 9px;
  letter-spacing: 0.12em;
  text-transform: uppercase;
  color: var(--ink-faint);
  margin-bottom: 4px;
}
.iv-answer {
  display: flex;
  align-items: baseline;
  gap: 6px;
  font-family: var(--hand);
  font-size: 12px;
  color: var(--ink);
  line-height: 1.4;
}
.iv-answer__k {
  color: var(--ink-soft);
  white-space: nowrap;
  flex-shrink: 0;
}
.iv-answer__arr {
  color: var(--ink-faint);
  flex-shrink: 0;
}
.iv-answer__v {
  flex: 1;
}
.iv-notes {
  margin-top: 10px;
  padding-top: 8px;
  border-top: 1px dotted var(--paper-line);
  font-family: var(--hand);
  font-size: 12px;
  color: var(--ink-soft);
  line-height: 1.4;
  display: flex;
  gap: 8px;
  align-items: baseline;
}
.iv-notes__head {
  font-family: var(--label);
  font-size: 9px;
  letter-spacing: 0.12em;
  text-transform: uppercase;
  color: var(--ink-faint);
  flex-shrink: 0;
}
.iv-notes__body {
  flex: 1;
  font-style: italic;
}
</style>
