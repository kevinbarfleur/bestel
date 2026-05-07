<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, watch } from 'vue';

import { EditorState, Compartment } from '@codemirror/state';
import { EditorView, keymap, lineNumbers, highlightActiveLine, highlightActiveLineGutter } from '@codemirror/view';
import { defaultKeymap, history, historyKeymap } from '@codemirror/commands';
import { searchKeymap, search, openSearchPanel } from '@codemirror/search';
import { markdown } from '@codemirror/lang-markdown';
import { HighlightStyle, syntaxHighlighting } from '@codemirror/language';
import { tags as t } from '@lezer/highlight';

import { usePromptsStore } from '../../stores/prompts';

const store = usePromptsStore();

const emit = defineEmits<{
  cursor: [{ line: number; col: number }];
}>();

const editorEl = ref<HTMLDivElement | null>(null);
let view: EditorView | null = null;
const themeCompartment = new Compartment();

defineExpose({
  jumpToLine(line: number) {
    if (view == null) return;
    const lineObj = view.state.doc.line(Math.max(1, Math.min(line, view.state.doc.lines)));
    view.dispatch({
      selection: { anchor: lineObj.from },
      effects: EditorView.scrollIntoView(lineObj.from, { y: 'center' }),
    });
    view.focus();
  },
  focusEditor() {
    view?.focus();
  },
  openSearch() {
    if (view) openSearchPanel(view);
  },
});

const markdownHighlight = HighlightStyle.define([
  { tag: t.heading1, fontFamily: 'var(--hand-display)', fontSize: '22px', fontWeight: '700', color: 'var(--ink)' },
  { tag: t.heading2, fontFamily: 'var(--hand-display)', fontSize: '17px', fontWeight: '600', color: 'var(--ink)' },
  { tag: t.heading3, fontFamily: 'var(--hand-display)', fontSize: '15px', fontWeight: '600', color: 'var(--ink)' },
  { tag: t.strong, fontWeight: '700', color: 'var(--ink)' },
  { tag: t.emphasis, fontStyle: 'italic', color: 'var(--ink-soft)' },
  { tag: t.link, textDecoration: 'underline', color: 'var(--amber)' },
  { tag: t.url, color: 'var(--amber-soft)' },
  { tag: t.monospace, color: 'var(--rune)', backgroundColor: 'rgba(175,96,37,0.10)' },
  { tag: t.processingInstruction, color: 'var(--ink-faint)' },
  { tag: t.contentSeparator, color: 'var(--ink-faint)' },
  { tag: t.list, color: 'var(--ink)' },
  { tag: t.quote, color: 'var(--ink-soft)', fontStyle: 'italic' },
]);

const baseTheme = EditorView.theme({
  '&': {
    height: '100%',
    fontFamily: 'var(--mono)',
    fontSize: '13.5px',
    color: 'var(--ink)',
    backgroundColor: 'var(--paper)',
  },
  '.cm-scroller': {
    fontFamily: 'var(--mono)',
    lineHeight: '22px',
    overflow: 'auto',
  },
  '.cm-content': {
    padding: '14px 22px 14px 16px',
    caretColor: 'var(--amber)',
  },
  '.cm-gutters': {
    backgroundColor: 'var(--paper)',
    borderRight: '1px solid var(--paper-line)',
    color: 'var(--ink-faint)',
    fontFamily: 'var(--mono)',
    fontSize: '12px',
    minWidth: '56px',
  },
  '.cm-lineNumbers .cm-gutterElement': {
    padding: '0 12px 0 14px',
    textAlign: 'right',
  },
  '.cm-activeLineGutter': {
    backgroundColor: 'transparent',
    color: 'var(--amber)',
    fontWeight: '600',
  },
  '.cm-activeLine': {
    backgroundColor: 'var(--amber-glow)',
  },
  '.cm-cursor, .cm-dropCursor': {
    borderLeft: '1.5px solid var(--amber)',
  },
  '&.cm-focused .cm-selectionBackground, ::selection': {
    backgroundColor: 'rgba(45, 43, 40, 0.18)',
  },
  '.cm-panels': {
    backgroundColor: 'var(--paper-shade)',
    color: 'var(--ink)',
    borderTop: '1px solid var(--paper-line)',
  },
  '.cm-panel input, .cm-panel button': {
    fontFamily: 'var(--hand)',
  },
  '.cm-searchMatch': {
    backgroundColor: 'var(--amber-glow)',
    outline: '1px solid var(--amber)',
  },
});

function buildState(initial: string) {
  return EditorState.create({
    doc: initial,
    extensions: [
      lineNumbers(),
      history(),
      highlightActiveLine(),
      highlightActiveLineGutter(),
      search({ top: true }),
      keymap.of([...defaultKeymap, ...historyKeymap, ...searchKeymap]),
      markdown(),
      syntaxHighlighting(markdownHighlight),
      themeCompartment.of(baseTheme),
      EditorView.lineWrapping,
      EditorView.updateListener.of((u) => {
        if (u.docChanged) {
          const path = store.activeFile;
          if (path != null) {
            store.setContent(path, u.state.doc.toString());
          }
        }
        if (u.selectionSet || u.docChanged) {
          const sel = u.state.selection.main;
          const lineObj = u.state.doc.lineAt(sel.head);
          emit('cursor', { line: lineObj.number, col: sel.head - lineObj.from + 1 });
        }
      }),
    ],
  });
}

function reflectActiveFile() {
  if (view == null) return;
  const wantedPath = store.activeFile;
  const next = wantedPath != null ? store.contents[wantedPath] ?? '' : '';
  if (next === view.state.doc.toString()) return;
  // Preserve cursor position if it remains within the new doc length.
  const sel = view.state.selection.main.head;
  const safe = Math.min(sel, next.length);
  view.dispatch({
    changes: { from: 0, to: view.state.doc.length, insert: next },
    selection: { anchor: safe },
  });
}

onMounted(() => {
  if (editorEl.value == null) return;
  view = new EditorView({
    state: buildState(store.activeContent),
    parent: editorEl.value,
  });
  view.focus();
});

onBeforeUnmount(() => {
  view?.destroy();
  view = null;
});

watch(
  () => store.activeFile,
  () => reflectActiveFile(),
);

watch(
  () => (store.activeFile ? store.contents[store.activeFile] : null),
  (next) => {
    if (view == null || next == null) return;
    if (next === view.state.doc.toString()) return;
    reflectActiveFile();
  },
);

function closeTab(path: string, ev: MouseEvent) {
  ev.stopPropagation();
  store.closeTab(path);
}

function selectTab(path: string) {
  void store.openFile(path);
}

function shortName(rel: string) {
  return rel.split('/').pop() ?? rel;
}
</script>

<template>
  <div class="pe-body">
    <div class="pe-tabs">
      <div
        v-for="path in store.openTabs"
        :key="path"
        class="pe-tab"
        :class="{ 'pe-tab--active': path === store.activeFile }"
        @click="selectTab(path)"
      >
        <span>{{ shortName(path) }}</span>
        <span v-if="store.dirty.has(path)" class="pe-tab__dirty" />
        <button
          type="button"
          class="pe-tab__close"
          aria-label="Close tab"
          @click.stop="closeTab(path, $event)"
        >×</button>
      </div>
      <span class="pe-tabs__spacer" />
    </div>
    <div ref="editorEl" class="pe-editor" />
  </div>
</template>

<style scoped>
.pe-body {
  display: flex;
  flex-direction: column;
  background: var(--paper);
  min-height: 0;
  min-width: 0;
}

.pe-tabs {
  display: flex;
  align-items: stretch;
  background: var(--paper-shade);
  border-bottom: 1px solid var(--paper-line);
  overflow-x: auto;
}

.pe-tab {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 14px;
  background: transparent;
  border-right: 1px solid var(--paper-line);
  border-top: 2px solid transparent;
  margin-top: -1px;
  font-family: var(--mono);
  font-size: 12.5px;
  color: var(--ink-soft);
  font-weight: 400;
  cursor: pointer;
  position: relative;
  white-space: nowrap;
}

.pe-tab--active {
  background: var(--paper);
  color: var(--ink);
  font-weight: 600;
  border-top: 2px solid var(--amber);
}

.pe-tab__dirty {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--amber);
  flex: none;
}

.pe-tab__close {
  width: 14px;
  height: 14px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  font-size: 14px;
  line-height: 1;
  color: var(--ink-faint);
  background: transparent;
  border: none;
  cursor: pointer;
  margin-left: 2px;
  padding: 0;
  border-radius: 2px;
}

.pe-tab__close:hover {
  background: var(--paper-line);
  color: var(--ink);
}

.pe-tabs__spacer {
  flex: 1;
}

.pe-editor {
  flex: 1;
  min-height: 0;
  min-width: 0;
  display: flex;
}

.pe-editor :deep(.cm-editor) {
  flex: 1;
  height: 100%;
}
</style>
