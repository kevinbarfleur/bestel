import { defineStore } from 'pinia';
import { computed, ref } from 'vue';

export type UiPicker = 'build' | 'model' | 'chat' | 'settings' | null;

/** Type of artifact promoted to the right adaptive panel. */
export type PanelArtifactType = 'item-card' | 'gem-detail' | 'mechanic' | 'markdown';

export interface PanelArtifact {
  /** Unique ID — typically the source tool segment id, or a generated key
   *  for click-to-expand promotions from inline artifacts. */
  id: string;
  type: PanelArtifactType;
  title: string;
  /** Type-specific payload. Concrete shapes live in the panel components. */
  payload: unknown;
  /** Where the open call came from — for telemetry / future logic. */
  source: 'agent' | 'click';
}

export const useUiStore = defineStore('ui', () => {
  const picker = ref<UiPicker>(null);

  /** URL of the link currently displayed inside the in-app webview overlay,
   *  or null when no link viewer is mounted. */
  const linkViewerUrl = ref<string | null>(null);

  /** Whether the full-sheet modal (Sprint UX-2.8) is currently open. The
   *  modal renders the entire validated Build Sheet read-only — opened
   *  from `BSLinkedSheetCard`'s `View full` button. */
  const sheetModalOpen = ref(false);

  /** Stack of artifacts promoted to the right adaptive panel. The top of
   *  the stack is the visible one. Pop on `goBack()`. Empty stack = panel
   *  is closed. */
  const panelStack = ref<PanelArtifact[]>([]);

  /** Currently visible panel artifact (top of the stack), or null when
   *  the panel is closed. */
  const panelArtifact = computed<PanelArtifact | null>(() =>
    panelStack.value.length > 0
      ? panelStack.value[panelStack.value.length - 1]
      : null,
  );

  /** True when at least one previous artifact sits below the top — used
   *  to show the back button in the panel header. */
  const panelHasHistory = computed(() => panelStack.value.length > 1);

  const openBuild = () => {
    picker.value = 'build';
  };
  const openModel = () => {
    picker.value = 'model';
  };
  const openChat = () => {
    picker.value = 'chat';
  };
  const openSettings = () => {
    picker.value = 'settings';
  };
  const close = () => {
    picker.value = null;
  };

  const openLinkViewer = (url: string) => {
    linkViewerUrl.value = url;
  };
  const closeLinkViewer = () => {
    linkViewerUrl.value = null;
  };

  const openSheetModal = () => {
    sheetModalOpen.value = true;
  };
  const closeSheetModal = () => {
    sheetModalOpen.value = false;
  };

  /** Push an artifact onto the panel stack — opens the panel if closed,
   *  pushes on top of the current one if already open (history grows). */
  const openPanel = (artifact: PanelArtifact) => {
    // Avoid pushing exact-duplicate refresh (same id at top): just update
    // in-place so we don't grow the back-stack on repeated emits.
    const top = panelStack.value[panelStack.value.length - 1];
    if (top && top.id === artifact.id) {
      panelStack.value[panelStack.value.length - 1] = artifact;
      return;
    }
    panelStack.value.push(artifact);
  };

  /** Pop the current artifact and reveal the one beneath, or close the
   *  panel entirely if the stack becomes empty. */
  const goBackPanel = () => {
    panelStack.value.pop();
  };

  /** Close the panel and forget the entire stack. */
  const closePanel = () => {
    panelStack.value = [];
  };

  /** Sprint v3 — DriftDrawer state. Opens a slide-from-right drawer per
   *  drifted signature axis. `axis === null` means closed. */
  const driftDrawerAxis = ref<'identity' | 'gear' | 'tree' | 'skill' | 'config' | null>(null);
  const openDriftDrawer = (axis: 'identity' | 'gear' | 'tree' | 'skill' | 'config') => {
    driftDrawerAxis.value = axis;
  };
  const closeDriftDrawer = () => {
    driftDrawerAxis.value = null;
  };

  return {
    picker,
    linkViewerUrl,
    sheetModalOpen,
    panelArtifact,
    panelHasHistory,
    panelStack,
    driftDrawerAxis,
    openBuild,
    openModel,
    openChat,
    openSettings,
    close,
    openLinkViewer,
    closeLinkViewer,
    openSheetModal,
    closeSheetModal,
    openPanel,
    goBackPanel,
    closePanel,
    openDriftDrawer,
    closeDriftDrawer,
  };
});
