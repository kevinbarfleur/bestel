<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, ref, watch } from 'vue';
import { storeToRefs } from 'pinia';

import {
  closeLinkModal,
  openExternal,
  openLinkModal,
  updateLinkModalBounds,
} from '../../api/tauri';
import { useUiStore } from '../../stores/ui';
import { useToastsStore } from '../../stores/toasts';
import RunicIcon from '../runic/RunicIcon.vue';

/**
 * Overlay modal that hosts a Tauri child webview inside its content area.
 *
 * The DOM provides the chrome (header bar, close, escape hatch, frame), but
 * the actual page rendering is delegated to a *native* sub-webview created
 * via `open_link_modal`. We measure the content `<div ref="frame">` rect and
 * forward its `(x, y, w, h)` (logical pixels, devicePixelRatio-scaled) to the
 * backend so the native webview slots into the reserved viewport.
 *
 * On mount: spawn webview at the frame's bounds.
 * On resize: forward new bounds.
 * On unmount / esc / close click: close the webview.
 */

const ui = useUiStore();
const toasts = useToastsStore();
const { linkViewerUrl } = storeToRefs(ui);

const frame = ref<HTMLElement | null>(null);
const isMounted = ref(false);
const isLoading = ref(false);

const visible = computed(() => linkViewerUrl.value !== null);

const displayHost = computed(() => {
  if (!linkViewerUrl.value) return '';
  try {
    return new URL(linkViewerUrl.value).host;
  } catch {
    return linkViewerUrl.value;
  }
});

const displayPath = computed(() => {
  if (!linkViewerUrl.value) return '';
  try {
    const u = new URL(linkViewerUrl.value);
    const path = u.pathname + u.search;
    return path.length > 80 ? path.slice(0, 79) + '…' : path;
  } catch {
    return '';
  }
});

function measureFrame(): { x: number; y: number; w: number; h: number } | null {
  const el = frame.value;
  if (!el) return null;
  const rect = el.getBoundingClientRect();
  return {
    x: rect.left,
    y: rect.top,
    w: rect.width,
    h: rect.height,
  };
}

async function mountWebview(url: string) {
  await nextTick();
  const bounds = measureFrame();
  if (!bounds) return;
  isLoading.value = true;
  try {
    await openLinkModal(url, bounds.x, bounds.y, bounds.w, bounds.h);
    isMounted.value = true;
  } catch (e) {
    toasts.push({
      variant: 'error',
      title: 'Could not open link inside Bestel',
      body: e instanceof Error ? e.message : String(e),
    });
    ui.closeLinkViewer();
  } finally {
    isLoading.value = false;
  }
}

async function unmountWebview() {
  if (!isMounted.value) return;
  try {
    await closeLinkModal();
  } catch {
    // best-effort
  }
  isMounted.value = false;
}

async function syncBounds() {
  if (!isMounted.value) return;
  const bounds = measureFrame();
  if (!bounds) return;
  try {
    await updateLinkModalBounds(bounds.x, bounds.y, bounds.w, bounds.h);
  } catch {
    // best-effort
  }
}

let resizeObs: ResizeObserver | null = null;

function startObserving() {
  if (typeof ResizeObserver === 'undefined') return;
  resizeObs = new ResizeObserver(() => {
    void syncBounds();
  });
  if (frame.value) resizeObs.observe(frame.value);
  window.addEventListener('resize', syncBounds);
}

function stopObserving() {
  if (resizeObs) {
    resizeObs.disconnect();
    resizeObs = null;
  }
  window.removeEventListener('resize', syncBounds);
}

watch(linkViewerUrl, async (next, prev) => {
  if (next && !prev) {
    // Opening
    await mountWebview(next);
    startObserving();
  } else if (!next && prev) {
    // Closing
    stopObserving();
    await unmountWebview();
  } else if (next && prev && next !== prev) {
    // URL change while open — re-navigate
    await mountWebview(next);
  }
});

onBeforeUnmount(() => {
  stopObserving();
  void unmountWebview();
});

function close() {
  ui.closeLinkViewer();
}

function escapeHatch() {
  if (!linkViewerUrl.value) return;
  void openExternal(linkViewerUrl.value);
  close();
}
</script>

<template>
  <Teleport to="body">
    <Transition name="link-viewer">
      <div v-if="visible" class="link-viewer">
        <div class="link-viewer__overlay" @click="close" />
        <div class="link-viewer__shell">
          <header class="link-viewer__header">
            <div class="link-viewer__addr">
              <span class="link-viewer__host">{{ displayHost }}</span>
              <span v-if="displayPath" class="link-viewer__path">{{ displayPath }}</span>
            </div>
            <button
              type="button"
              class="link-viewer__icon-btn"
              :title="`Open in default browser — ${linkViewerUrl}`"
              aria-label="Open in default browser"
              @click="escapeHatch"
            >
              <RunicIcon name="open" :size="16" />
            </button>
            <button
              type="button"
              class="link-viewer__icon-btn"
              title="Close"
              aria-label="Close"
              @click="close"
            >
              <RunicIcon name="close" :size="16" />
            </button>
          </header>
          <div ref="frame" class="link-viewer__frame">
            <p v-if="isLoading" class="link-viewer__loading">Loading…</p>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.link-viewer {
  position: fixed;
  inset: 0;
  z-index: 9000; /* below the native sub-webview's effective layer (it sits on top regardless) */
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 1rem;
}

.link-viewer__overlay {
  position: absolute;
  inset: 0;
  background: rgba(40, 35, 30, 0.45);
  backdrop-filter: blur(2px);
}

.link-viewer__shell {
  position: relative;
  width: 92vw;
  max-width: 1280px;
  min-width: 720px;
  height: 88vh;
  max-height: 820px;
  background: var(--paper);
  border-radius: 6px;
  box-shadow: 0 18px 40px rgba(60, 40, 20, 0.22);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.link-viewer__header {
  flex: none;
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 14px;
  border-bottom: 1px solid var(--paper-line);
  background: var(--paper);
}

.link-viewer__addr {
  flex: 1;
  min-width: 0;
  display: flex;
  align-items: baseline;
  gap: 6px;
  font-family: var(--mono);
  font-size: var(--fs-meta);
  color: var(--ink-soft);
  overflow: hidden;
}
.link-viewer__host {
  color: var(--ink);
  font-weight: var(--fw-semibold);
  flex: none;
}
.link-viewer__path {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.link-viewer__icon-btn {
  flex: none;
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: 1px solid transparent;
  border-radius: 4px;
  color: var(--ink-soft);
  cursor: pointer;
  transition: background 0.15s ease, color 0.15s ease;
}
.link-viewer__icon-btn:hover {
  background: var(--paper-shade);
  color: var(--ink);
}

.link-viewer__frame {
  flex: 1;
  min-height: 0;
  position: relative;
  background: var(--paper-shade);
  /* The native sub-webview is positioned over this rect by the backend.
     Anything drawn here (loader, errors) is occluded once the webview boots. */
}

.link-viewer__loading {
  position: absolute;
  inset: 0;
  margin: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  font-family: var(--hand);
  font-size: var(--fs-body);
  color: var(--ink-soft);
}

.link-viewer-enter-active,
.link-viewer-leave-active { transition: opacity 0.2s ease; }
.link-viewer-enter-active .link-viewer__shell,
.link-viewer-leave-active .link-viewer__shell { transition: transform 0.2s ease, opacity 0.2s ease; }
.link-viewer-enter-from,
.link-viewer-leave-to { opacity: 0; }
.link-viewer-enter-from .link-viewer__shell,
.link-viewer-leave-to .link-viewer__shell {
  opacity: 0;
  transform: scale(0.97) translateY(-8px);
}
</style>
