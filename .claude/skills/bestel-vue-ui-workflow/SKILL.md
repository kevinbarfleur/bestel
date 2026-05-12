---
name: bestel-vue-ui-workflow
description: Workflow for Vue 3, TypeScript, Pinia, Tauri IPC, markdown rendering, chat streaming, and Build Sheet UI changes in Bestel. Use for frontend implementation, debugging, or review.
---

# Vue UI workflow

## Before code

Determine the owner of state:

- `chat.ts` for streaming messages and transcript state.
- `build.ts` for active PoB build state.
- `sheet.ts` for Build Sheet FSM.
- `settings.ts` for model, theme, keys, and feature toggles.
- component-local state only for temporary UI details.

## Patch rules

- Keep backend DTO handling exhaustive.
- Keep components small and store-driven.
- Sanitize markdown and avoid raw HTML assumptions.
- Do not duplicate server state in multiple stores.
- For Build Sheet changes, update the FSM and renderers together.

## Checks

```bash
cd crates/bestel/ui
npm run build
```

If Rust DTOs or event variants changed, run relevant Rust checks too.
