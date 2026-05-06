# Brainstorm — In-app passive tree viewer (PoE1)

> **Status:** parked. Plan complete and approved but not implemented. Resume when ready.
> **Original plan source:** `C:\Users\kbarf\.claude\plans\wondrous-puzzling-forest.md` (Phase H)
> **Date parked:** 2026-05-06

## Context

`BuildPanel.vue` exposes today the allocated keystones and notables as leader rows + a single "Open in browser" link that points to the official pathofexile.com viewer. The user wants to stay in-app: a small button in the passive section opens a mini window with a faithful PoE1 passive tree (PoB-style), where they can search a node, click to see details, and see at a glance which nodes the current build allocates.

State at audit time (Phase 1 exploration):
- `BuildPanel.vue` lines 523-538: section "passive tree" with a single button `◆ Open in browser` → natural hook for the new viewer button.
- `crates/bestel-core/src/pob/parser.rs` line 156-162: `allocated_nodes: Vec<u32>` parsed from PoB XML but **not exposed** in `PobBuildDto` → gap to fill (otherwise the viewer cannot highlight allocated nodes of the current build).
- `crates/bestel-core/data/passive_nodes_poe[12].json`: curated seeds of ~46 nodes — feeds the LLM dict, not the viewer. **Not reused** by this feature; the full tree is ingested separately.
- `RunicModal.vue`: max-width xl = 56rem, insufficient for a tree canvas (~1500×1500px). Need a `'full'` variant.
- `PoeVersion::Poe2` wired everywhere on the core side, but the tree.lua of PathOfBuilding-PoE2 has a different structure → out of scope v1.

## Locked choices (validated by user)

1. **v1 scope = PoE1 only.** PoE2 comes in Phase H+1 once the PoB2 format is properly abstracted.
2. **Bake at build time.** A Node script fetches tree.lua at a pinned commit of PathOfBuildingCommunity/PathOfBuilding/dev, parses to compact JSON, and copies skills-3.jpg into `public/`. No runtime fetch, no net required post-install.
3. **Modal in-app full-bleed** rather than a separate Tauri window. Single frontend state, no inter-window IPC, closes with Esc.
4. **Render tech: Pixi.js v8 + pixi-viewport.** De facto standard for 4k+ interactive objects in webview, ~150 KB gzip. WebGL2 = smooth 60fps pan/zoom.
5. **skills-3.jpg as backdrop** rather than redrawing everything from sprites. -90% of rendering complexity, perfect visual fidelity (the PoB image already contains all nodes + connection curves). We only overlay highlights (allocated/search/hover) on top.

## Strategy

### H.1 Data pipeline (build time)

Create `crates/bestel/ui/scripts/fetch-treedata.mjs`:
- Pin a precise commit SHA of `PathOfBuildingCommunity/PathOfBuilding` branch `dev` (deterministic)
- Fetch:
  - `src/TreeData/3_28/tree.lua` (~2.8 MB)
  - `src/TreeData/3_28/skills-3.jpg` (~720 KB)
- Parse `tree.lua` via `luaparse` (npm) → compact JSON
- **Precompute polar `(x, y)` for each node** at build time (avoid per-frame trig on the client):
  ```
  if orbit == 0: (group.x, group.y)
  else:
    angle = (orbitIndex / skillsPerOrbit[orbit]) * 2π
    x = group.x + orbitRadii[orbit] * sin(angle)
    y = group.y - orbitRadii[orbit] * cos(angle)
  ```
- Output schema (`poe1.json`, ~1 MB compact):
  ```json
  {
    "version": "3_28",
    "commit": "<sha>",
    "bounds": { "minX": -13902, "minY": -10689, "maxX": 12430, "maxY": 10900 },
    "nodes": {
      "52349": {
        "n": "The King's Contempt",
        "k": "ascendancy_notable",
        "s": ["Discipline has 50% increased Aura Effect..."],
        "x": -7121.9,
        "y": 12087.57,
        "asc": "Aul",
        "out": ["10696"]
      }
    },
    "jewelSlots": [...]
  }
  ```
- Strip unused-at-runtime fields: `icon`, `classStartIndex`, `isMultipleChoiceOption`, `flavourText`, `orbit`, `orbitIndex`, `group` once `(x, y)` are frozen.
- Normalized `kind`: `keystone | notable | mastery | jewel_socket | small | class_start | ascendancy_notable | ascendancy_small | ascendancy_start`.
- Output:
  - `crates/bestel/ui/src/treedata/poe1.json` (imported as ES module, tree-shaken by Vite)
  - `crates/bestel/ui/public/treedata/poe1/skills-3.jpg`
- Idempotent: skip download if local SHA matches the pin; `--force` to re-download.
- Hooked via `package.json` script `prebuild`. Manual: `npm run fetch-tree`.

### H.2 Rust backend — surface allocated_node_ids

Single change:
- `crates/bestel/src/dto.rs`: add `allocated_node_ids: Vec<u32>` to `PobBuildDto`; copy from `b.allocated_nodes` in the `From<&PobBuild>` impl.
- `crates/bestel/ui/src/api/types.ts`: mirror the field.

No tree-related Rust code. The viewer is purely frontend.

### H.3 Vue components

**New folder** `crates/bestel/ui/src/components/tree/`:
- **`TreeViewerModal.vue`** — RunicModal wrapper (variant `'full'`). 3 zones: top bar (search), main (canvas), right rail (node detail panel). Closes with Esc.
- **`TreeViewer.vue`** — the canvas. Mounts Pixi.Application + pixi-viewport. Emits `node:hover`, `node:click` events.
- **`TreeNodeDetail.vue`** — right panel ~320px. Shows the selected node: name (hand-display 22px), kind (small caps amber), ascendancy (italic note if present), stats (EB Garamond, one per line), neighbors (clickable list from `out` array), "Center on tree" button.
- **`TreeSearch.vue`** — input + match counter + nav buttons (◀ ▶). Match on `name.toLowerCase()` OR `stats.join(' ').toLowerCase()`. Debounced 100ms.

**Composable** `crates/bestel/ui/src/composables/useTreeViewer.ts`:
- Encapsulates Pixi.Application lifecycle (mount, destroy).
- Manages 5 layers (Pixi `Container`):
  1. **bgLayer**: `skills-3.jpg` loaded via `Pixi.Assets.load()`, anchored at `(bounds.minX, bounds.minY)`.
  2. **allocLayer**: amber-filled circles (radius 18) at the positions of `allocated_node_ids`. Redrawn only when the build store changes (watcher).
  3. **searchLayer**: amber rings (stroke only) on nodes matching the search. Redrawn only when the query changes.
  4. **hoverLayer**: ring + tooltip (Pixi.Text) on the hovered node.
  5. **hitLayer**: invisible 32×32 px (in tree coords) `Sprite`s with `eventMode: 'static'` for click/hover. Pixi handles spatial indexing.
- Exposes `centerOn(nodeId)`, `setSearch(query)`, `setSelected(nodeId)`.
- Viewport: drag, wheel, pinch, clamp to bounds, decelerate. Zoom min 0.1, max 4.

### H.4 UI wiring

- **`crates/bestel/ui/src/components/runic/RunicModal.vue`**: add variant `'full'` (95vw × 90vh).
- **`crates/bestel/ui/src/components/build/BuildPanel.vue`**: section "passive tree" (lines 523-538):
  - Replace the single link with two buttons:
    - `◆ Open viewer` (primary) — opens `<TreeViewerModal>` via `useUiStore().openTreeViewer()` or local state.
    - `or open official viewer` (small text link, current behavior preserved).
  - If `current.game === 'poe2'`: "Open viewer" button disabled with label `(PoE1 only)`; click → toast "PoE2 tree viewer coming in a future patch".
- **`crates/bestel/ui/src/stores/build.ts`**: computed `allocatedNodeIds: Set<number>` from `current.allocated_node_ids` (Set for O(1) lookup).
- **`crates/bestel/ui/package.json`**: add deps `pixi.js@^8`, `pixi-viewport@^5`; devDep `luaparse@^0.3`; script `"fetch-tree": "node scripts/fetch-treedata.mjs"`; script `"prebuild": "node scripts/fetch-treedata.mjs"`.

### H.5 Performance strategy

- skills-3.jpg loaded once via `Pixi.Assets.load()` (browser cache after first open).
- Hit zones via `eventMode: 'static'` — Pixi spatial system optimal for 4127 invisible sprites.
- allocLayer redrawn only when the build store changes (not per frame).
- searchLayer redrawn only when the query changes.
- Tooltip Pixi.Text, not DOM (no reflow).
- Viewport decelerate built-in for smooth scroll.
- Target: 60fps on continuous pan/zoom (verify via DevTools Performance).

## Critical files

**To create**
- `crates/bestel/ui/scripts/fetch-treedata.mjs` — Node bake script (download + parse Lua + emit JSON + copy JPG)
- `crates/bestel/ui/src/treedata/poe1.json` — generated
- `crates/bestel/ui/public/treedata/poe1/skills-3.jpg` — generated
- `crates/bestel/ui/src/components/tree/TreeViewerModal.vue`
- `crates/bestel/ui/src/components/tree/TreeViewer.vue`
- `crates/bestel/ui/src/components/tree/TreeNodeDetail.vue`
- `crates/bestel/ui/src/components/tree/TreeSearch.vue`
- `crates/bestel/ui/src/composables/useTreeViewer.ts`

**To modify**
- `crates/bestel/src/dto.rs` — add `allocated_node_ids: Vec<u32>` to `PobBuildDto` + From impl
- `crates/bestel/ui/src/api/types.ts` — add `allocated_node_ids: number[]`
- `crates/bestel/ui/src/components/runic/RunicModal.vue` — add variant `'full'` (95vw × 90vh)
- `crates/bestel/ui/src/components/build/BuildPanel.vue` — "Open viewer" button + PoE2 gating
- `crates/bestel/ui/src/stores/build.ts` — computed `allocatedNodeIds: Set<number>`
- `crates/bestel/ui/package.json` — pixi.js + pixi-viewport + luaparse deps + fetch-tree/prebuild scripts

**Reused as-is**
- `crates/bestel-core/src/pob/parser.rs` — `allocated_nodes` already parsed, just expose it
- `crates/bestel-core/src/llm/dict.rs` — untouched; seeds still serve LLM tools, distinct from the full tree
- `RunicModal.vue` — base reused, just variant extension
- `RunicInput.vue` — for the search bar
- `useToast()` — for the PoE2 fallback message

## Implementation order

1. **Build script**: `fetch-treedata.mjs` + `luaparse` dep. Pin SHA. Generates `poe1.json` + copies `skills-3.jpg`. Hook `prebuild` in package.json.
2. **Visual coord validation**: minimal test page that overlays a simple canvas on skills-3.jpg with dots at computed positions → verify alignment by eye before building the full viewer.
3. **Rust DTO**: add `allocated_node_ids` to `PobBuildDto` + From impl. Compile.
4. **TS types + store**: mirror in `types.ts`; computed `allocatedNodeIds: Set<number>` in `build.ts`.
5. **RunicModal full variant**: add `'full'` (95vw × 90vh).
6. **TreeViewer.vue v0**: Pixi.Application + Viewport + bgLayer (skills-3.jpg) + invisible hit zones + zoom/pan handlers. No search/highlight yet.
7. **TreeViewer + allocLayer**: amber circles on `allocatedNodeIds`. Redraw on store change.
8. **TreeNodeDetail.vue**: right panel with stats + clickable neighbors + center button.
9. **TreeSearch.vue + searchLayer**: index + match + amber rings + cycle ◀ ▶.
10. **TreeViewerModal.vue**: assemble TreeViewer + Search + Detail in the full-bleed modal. Esc close.
11. **BuildPanel.vue**: "Open viewer" button in passive tree section. PoE2 gated with toast.
12. **Build + smoke test**: kill bestel.exe; `npm run build` (triggers prebuild → fetch-tree); `cargo build --release -p bestel -j 1`; launch; checklist.

## Out of scope v1 (deferred)

- PoE2 tree (button cleanly disabled)
- Ascendancy panels in a dedicated sub-view (ascendancy nodes visible in main tree but no separate panel)
- Per-node re-rendering of allocated/unallocated state (just amber circles overlay v1)
- Pathfinding (shortest path between 2 nodes)
- Cluster jewels rendering
- Mastery effect picker (mastery node clickable, but not the effect)
- Tree editing (read-only viewer)

## Verification (when implemented)

- [ ] `npm run fetch-tree` downloads tree.lua + skills-3.jpg, generates `poe1.json` without error, idempotent (run 2× = no-op).
- [ ] `poe1.json` contains ~4127 nodes, each with precomputed `(x, y)` and a `kind` ∈ {keystone, notable, mastery, jewel_socket, small, class_start, ascendancy_*}.
- [ ] Click on "Open viewer" in BuildPanel opens the full-bleed modal (95vw × 90vh).
- [ ] Tree displayed: skills-3.jpg backdrop visible, nodes visually aligned with the image.
- [ ] Pan via mouse drag smooth (60fps DevTools). Zoom via wheel: clamp 0.1× to 4×.
- [ ] Hover on a node → ring + Pixi.Text tooltip with name + kind.
- [ ] Click on a node → right panel shows name + kind + stats + neighbors. Click on a neighbor centers the viewport on it.
- [ ] Allocated nodes of the current build rendered as amber circles overlay. Loading another build via Ctrl+B updates the amber circles instantly.
- [ ] Search "life" → all nodes containing "life" in name OR stats highlighted with amber rings. Counter "12 / 47". ▶ centers viewport on next match.
- [ ] PoE2 build loaded: "Open viewer" button disabled with label "(PoE1 only)" + toast "PoE2 tree viewer coming in a future patch" on click.
- [ ] Esc closes the modal.
- [ ] `cargo build --release -p bestel -j 1` succeeds. Binary ~13-16 MB (delta ~1-2 MB from baseline for pixi + embedded json — JPG served from public/ so not in JS bundle).

## Decisions

- **Pixi.js v8 + pixi-viewport**: ~150 KB gzip bundle, WebGL2, mature, Vue 3 supported.
- **skills-3.jpg as backdrop**: -90% rendering complexity vs redrawing from sprites. Perfect fidelity (PoB ships this image). 720 KB acceptable in `public/`.
- **Polar precomputation at build time**: no runtime trig, stable positions, simpler viewer.
- **Full-bleed modal** rather than separate Tauri window: single state, no inter-window IPC, native Esc-to-close. The separate window could be added in Phase H+ if the user wants a "side-monitor" mode.
- **PoE1 only v1**: tree.lua structure differs significantly between PoB1 and PoB2, better to ship a clean PoE1 v1 than a v1 with two badly-abstracted schemas.
- **No connector redraw**: skills-3.jpg already contains them (curves included).
- **No per-node re-rendering of allocated state**: just amber circles overlay. Less visually faithful than official PoB, but 100× less code and largely sufficient for a "reading + reference" viewer.
- **`luaparse` npm**: Lua grammar is regular but luaparse is tested and tiny (~30 KB). Not worth writing a custom parser.
- **`include via static import` for `poe1.json`** rather than runtime fetch: Vite tree-shakes and compresses, asset shipped in JS bundle, no network roundtrip nor runtime loading state.

## Unresolved questions

None.

## Source data reference (collected during exploration)

- PathOfBuilding TreeData index: https://github.com/PathOfBuildingCommunity/PathOfBuilding/tree/dev/src/TreeData
- 40 patch snapshots from `2_6` to `3_28` (+ `_alternate`, `_ruthless` variants).
- Per-patch payload (e.g. `3_28/`): `tree.lua` (2.8 MB, 88,764 lines, 4127 nodes), `sprites.lua` (0.7 MB), `skills-3.jpg` (724 KB backdrop), various overlay PNGs.
- `tree.lua` top-level keys: `tree`, `classes` (7 + ascendancies), `alternate_ascendancies`, `groups` (spatial clusters with x/y), `nodes` (4127 entries), `jewelSlots` (60 IDs), `min_x/max_x/min_y/max_y` (bounds), `constants` (skillsPerOrbit, orbitRadii), `points` (totalPoints=123).
- Node anatomy: `{ skill, name, icon, stats, group, orbit, orbitIndex, out, in, isKeystone?, isNotable?, isMastery?, isJewelSocket?, ascendancyName? }`.
- Census 3_28: 4127 nodes total, 987 notables, 54 keystones, 351 masteries, 60 jewel sockets.
- Constants 3_28: `skillsPerOrbit = [1, 6, 16, 16, 40, 72, 72]`, `orbitRadii = [0, 82, 162, 335, 493, 662, 846]`.
