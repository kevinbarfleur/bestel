// Bestel Pickers v2 — shared primitives.
//
// These components implement the "Bestel Pickers v2" design handoff. They
// are intentionally a NEW namespace (pickers-v2) rather than an in-place
// rewrite of `components/pickers/*` so the legacy pickers can keep
// working while the v2 modals are migrated screen by screen. Once every
// caller has been ported, the old folder can be retired.
//
// Naming convention: `Picker<X>.vue` — the prefix avoids collision with
// `RunicX` (pre-existing) and `<X>Tab` (dev panel tabs). All components
// are leaf primitives; pickers themselves (ModelPicker v2,
// BuildPicker v2, ChatPicker v2) compose them in `crates/bestel/ui/src/
// components/pickers-v2/<picker>/<Picker>V2.vue`.

export { default as PickerIcon } from './PickerIcon.vue';
export { default as PickerKbd } from './PickerKbd.vue';
export { default as PickerButton } from './PickerButton.vue';
export { default as PickerSectionHead } from './PickerSectionHead.vue';
export { default as PickerSearchInput } from './PickerSearchInput.vue';
export { default as PickerGroupLabel } from './PickerGroupLabel.vue';
export { default as PickerStatusDot } from './PickerStatusDot.vue';
export { default as PickerListItem } from './PickerListItem.vue';
export { default as PickerLeader } from './PickerLeader.vue';
export { default as PickerModal } from './PickerModal.vue';
