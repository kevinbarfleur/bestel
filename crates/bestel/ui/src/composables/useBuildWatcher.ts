import { onBeforeUnmount, onMounted } from 'vue';
import type { UnlistenFn } from '@tauri-apps/api/event';

import { onPobBuild, onPobCleared, onProviderStatus } from '../api/tauri';
import { useChatStore } from '../stores/chat';
import { useSettingsStore } from '../stores/settings';
import { useToastsStore } from '../stores/toasts';

/**
 * Routes backend events that touch the active build / provider surface
 * to the chat store (build) and settings store (provider). The Rust
 * `POB_BUILD` event fires from two paths:
 *
 *   1. The user explicitly attached a build via the modal — the chat
 *      store already optimistically wrote `activeBuild`; the event
 *      echoes a freshly-parsed snapshot.
 *   2. The watcher detected the live PoB file change on disk for the
 *      currently-attached build — useful so vitals / resists update
 *      without the user re-attaching.
 *
 * `chat.applyExternalBuild(...)` is idempotent and filters by
 * `source_file` match, so unrelated PoB file changes on disk don't
 * silently swap the chat's build out from under the user.
 */
export function useBuildWatcher() {
  const chat = useChatStore();
  const settings = useSettingsStore();
  const toasts = useToastsStore();
  const unlistens: UnlistenFn[] = [];
  let firstBuild = true;

  onMounted(async () => {
    unlistens.push(
      await onPobBuild((ev) => {
        chat.applyExternalBuild(ev.build);
        if (firstBuild) {
          firstBuild = false;
          return;
        }
        toasts.push({
          variant: 'success',
          title: 'Build loaded',
          body: ev.summary.header + ' · ' + ev.summary.file_name,
        });
      }),
    );

    unlistens.push(
      await onPobCleared(() => {
        chat.applyExternalClear();
      }),
    );

    unlistens.push(
      await onProviderStatus((ev) => {
        settings.applyDetection(ev.detection);
      }),
    );
  });

  onBeforeUnmount(() => {
    for (const u of unlistens) u();
    unlistens.length = 0;
  });
}
