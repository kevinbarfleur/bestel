import { onBeforeUnmount, onMounted } from 'vue';
import type { UnlistenFn } from '@tauri-apps/api/event';

import { onPobBuild, onPobCleared, onProviderStatus } from '../api/tauri';
import { useBuildStore } from '../stores/build';
import { useSettingsStore } from '../stores/settings';
import { useToastsStore } from '../stores/toasts';

export function useBuildWatcher() {
  const buildStore = useBuildStore();
  const settings = useSettingsStore();
  const toasts = useToastsStore();
  const unlistens: UnlistenFn[] = [];
  let firstBuild = true;

  onMounted(async () => {
    unlistens.push(
      await onPobBuild((ev) => {
        buildStore.applyBuildEvent(ev.build);
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
        buildStore.applyClearedEvent();
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
