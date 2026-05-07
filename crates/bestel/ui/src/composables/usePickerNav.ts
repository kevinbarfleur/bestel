import { computed, ref, type Ref, watch } from 'vue';

/**
 * Shared keyboard navigation for picker sidebars.
 *
 * - ArrowUp/Down move the highlighted index, clamped to [0, items.length - 1].
 * - Enter calls onSelect(items[highlighted]) when the list is non-empty.
 * - Highlight resets to 0 whenever the items array changes (e.g. search).
 *
 * Returns:
 *   - highlighted: ref<number>
 *   - selected: computed<T | null>
 *   - onKeydown: handler to attach to a containing element (or the search
 *     input's keydown), so the user can navigate without losing focus on
 *     the input.
 */
export function usePickerNav<T>(
  items: Ref<readonly T[]>,
  onSelect: (item: T) => void,
) {
  const highlighted = ref(0);

  watch(items, () => {
    highlighted.value = 0;
  });

  const selected = computed<T | null>(() => {
    const list = items.value;
    if (!list.length) return null;
    return list[Math.min(highlighted.value, list.length - 1)] ?? null;
  });

  function onKeydown(e: KeyboardEvent) {
    const list = items.value;
    if (!list.length) return;
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      highlighted.value = Math.min(highlighted.value + 1, list.length - 1);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      highlighted.value = Math.max(highlighted.value - 1, 0);
    } else if (e.key === 'Enter') {
      const target = list[highlighted.value];
      if (target !== undefined) {
        e.preventDefault();
        onSelect(target);
      }
    }
  }

  return { highlighted, selected, onKeydown };
}
