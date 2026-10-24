# PIN-in-WebGPU: element-anchored design-review pins against the photocraft-web canvas

**Task ID:** 2-pin-webgpu-research
**Date:** 2026-10-24 (Z.ai sandbox)
**Scope:** deep source investigation, no builds, no production code.
**Sources read:** eframe 0.36.2, egui 0.36.2, egui-winit 0.36.2, egui_kittest 0.36.2, winit 0.30.13, accesskit 0.24.1, accesskit_winit 0.32.2 (all from `/home/z/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/`), the photocraft worktree `/home/z/pc-edit` (branch `feature/right-panel-collapse`), and the annotakit clone `/home/z/storybook-annotakit`.

---

## 0. Executive summary

Photocraft-web renders its whole UI into one `<canvas>`; annotakit pins anchor to DOM elements via `@medv/finder` selectors + text/fingerprint/bbox evidence. The two worlds are bridged by **mirroring named egui widget rects as absolutely-positioned transparent DOM divs** (created and positioned from Rust via the existing js-bridge) that annotakit can pin like any other element.

Three candidate architectures were evaluated:

| # | Route | Verdict |
|---|-------|---------|
| A | **AccessKit DOM** — let egui's AccessKit integration materialize DOM nodes on web | **Dead end today.** eframe 0.36.2's web runner *discards* the AccessKit tree update (`app_runner.rs:394 "not currently implemented"`), and AccessKit has **no web backend at all** in this stack (`accesskit_winit` compiles a no-op `null` adapter on wasm; eframe README: "AccessKit lacks a Web backend"). No DOM nodes ever exist. Would additionally require an eframe fork, and its node ids (call-order hashes for buttons) are unstable anchors anyway. |
| B | **Bridge-overlay** — Rust publishes `{key, role, label, rect}` for named widgets through the existing `globalThis.PhotoCraft` bridge; a review-page overlay of transparent divs mirrors those rects; annotakit pins the divs (stable `data-testid` anchors) | **RECOMMENDED.** All in-repo, uses two patterns photocraft already has (per-frame rect registration via `ctx.data` — `dock.rs last_rects`; the js-bridge), gives annotakit exactly the identity fields it wants (`data-testid`, `role`, `aria-label`), and survives UI refactors by construction because keys are semantic (`panel-layers`, `tool-brush`). ~300–450 lines for a working PoC. |
| C | **Region-only** — mount annotakit as-is, use its region pins (pure geometric fragments, no element anchor) | **Zero-code stopgap.** Region threads store only a rect (`layer.tsx:1239-1257`), so they work against the canvas today. But no element identity for agents, and they silently drift on relayout (no orphan honesty). Keep as fallback; it also composes with B (regions for canvas-art comments, pins for UI elements). |

**Minimal proof-of-concept** (section 6): expose the already-published dock panel rects + a few button rects as `PhotoCraft.ui({op:"rects"})`, build a dev-only review page that wraps the canvas in `<div id="storybook-root">` + mounts annotakit + mirrors the rects as `data-testid` divs. That proves element-anchored pinning on the canvas with zero egui/eframe surgery.

---

## 1. Q1 — The AccessKit path (platform-native answer): NOT AVAILABLE on web

### (a) Does the winit-web AccessKit adapter materialize real DOM nodes?

**No — there is no web AccessKit adapter at all.** Verified at every layer:

- **winit 0.30.13** (`src/platform_impl/web/event_loop/{mod,proxy,runner,state,window_target}.rs`): a ripgrep for `accesskit|AccessKit` over the whole winit 0.30.13 source tree returns **zero matches**. winit has no AccessKit code; AccessKit integrates via the separate `accesskit_winit` crate.
- **accesskit_winit 0.32.2** `src/platform_impl/mod.rs:34-50`: the platform adapter is selected per-OS; anything that is not Windows/macOS/Linux-Unix/Android compiles `null.rs`, which is a complete no-op:
  ```rust
  // null.rs:21-23
  pub fn update_if_active(&mut self, _updater: impl FnOnce() -> TreeUpdate) {}
  pub fn process_event(&mut self, _window: &Window, _event: &WindowEvent) {}
  ```
  Its `Cargo.toml:96-120` declares accesskit_unix/macos/windows/android dependencies only — there is no wasm target. No `accesskit_js` / `accesskit_web` crate exists anywhere in the local registry index.
- **eframe 0.36.2 `src/web/app_runner.rs:394`**: the web runner receives `egui::PlatformOutput` from `ctx.run_ui` (app_runner.rs:284-294) and then *discards* the AccessKit field:
  ```rust
  let egui::PlatformOutput {
      ...
      accesskit_update: _,        // not currently implemented
      ...
  } = platform_output;
  ```
- **eframe 0.36.2 `README.md:45`** states it explicitly: *"egui supports AccessKit, but as of early 2024, AccessKit lacks a Web backend."*

So: no DOM nodes, no JS API, nothing. The only web accessibility path in eframe is the separate `web_screen_reader` feature (`Cargo.toml.orig:79-80`: `["web-sys/SpeechSynthesis", ...]`), which speaks `platform_output.events_description()` through the browser's speech synthesis (app_runner.rs:382-385) — event narration only, no tree, no rects.

### (b) What egui sends (the tree is real, complete, and cross-platform — it is simply thrown away on web)

egui 0.36.2 has a full, platform-independent AccessKit tree builder that **photocraft already ships in its wasm binary** (see (c)):

- Gated by `ContextImpl::is_accesskit_enabled` (`egui-0.36.2/src/context.rs:412`), turned on by the **public** `Context::enable_accesskit()` (context.rs:3701-3707). **Nobody calls it on web** — the only production caller is eframe *native*: `src/native/winit_integration.rs:180`, inside `accesskit_winit::WindowEvent::InitialTreeRequested` handling (winit_integration.rs:172-189; the same file routes action requests at run.rs:305, 346-348). egui_kittest's harness also calls it (`egui_kittest-0.36.2/src/lib.rs:144`).
- Per pass, when enabled: a root `Window` node is created with `set_transform(Affine::scale(pixels_per_point))` (context.rs:520-529); every widget that gets a `Response` may add a node via `Context::accesskit_node_builder` (public, write-only — context.rs:3684-3689, node-map insert + ancestor parent-chain at 600-628). Two write paths guarantee coverage:
  - all focusable widgets: `if allow_focus && w.sense.is_focusable() { self.accesskit_node_builder(w.id, |b| res.fill_accesskit_node_common(b)) }` (context.rs:1292-1297);
  - every widget that calls `Response::widget_info(...)` (response.rs:869-895) — and every egui built-in does: button.rs, label.rs, checkbox.rs, slider.rs, drag_value.rs, radio_button.rs, image.rs, progress_bar.rs, spinner.rs, hyperlink.rs, color_picker.rs, text_edit/builder.rs, containers window.rs (×4), combo_box.rs, collapsing_header.rs, scroll_area.rs, area.rs (verified by `rg -c widget_info` over `egui-0.36.2/src/widgets/` and `containers/`).
- **Every node carries a RECT**: `Response::fill_accesskit_node_common` does `builder.set_bounds(accesskit::Rect { x0: self.rect.min.x, … })` (response.rs:908-917); roles are mapped from `WidgetType` (`Button/CollapsingHeader/SelectableLabel → Role::Button`, `Panel → Role::Pane`, `Slider`, `ComboBox`, `CheckBox`, `TextInput`, `Splitter`, etc. — response.rs:935-958); labels/values/placeholders/toggled are set (response.rs:962-987). All in **egui points** (the root transform carries `pixels_per_point`, context.rs:526).
- At `end_pass` the whole map is packed into `platform_output.accesskit_update = Some(TreeUpdate { nodes: <ALL nodes>, tree: Some(Tree::new(root_id)), tree_id, focus })` (context.rs:2694-2717) and the accumulated `viewport.output` is `mem::take`n into the returned `FullOutput` (context.rs:2686). After the pass the stored output is empty — `ctx.output()` (public, context.rs:1058-1060) cannot retrieve the tree retroactively, and `App` gets no `PlatformOutput` hook on web (epi.rs App trait: only `logic` :167, `ui` :182, `raw_input_hook` :279). **⇒ The tree is only visible to whoever calls `ctx.run*` — i.e. eframe — and eframe-web drops it.**

**Proof the tree is complete and already used against PhotocraftApp:** photocraft's own kittest suite runs `PhotocraftApp` under `Harness::builder()…wgpu().build_eframe(...)`, and egui_kittest enables accesskit and consumes `output.platform_output.accesskit_update.take().expect("AccessKit was disabled")` to power `kittest::Queryable` role+label queries (egui_kittest lib.rs:144-177; ui-egui tests e.g. `tests/it/full_res_preview.rs:16,23`, `ui-egui/Cargo.toml:60`). The same data a bridge would export is what the tests already query.

**Node id stability (critical for anchors):** `Id::accesskit_id()` is the raw u64 hash (id.rs:99-105). Buttons get ids from `ui.next_auto_id()` (button.rs:325) — a per-`Ui` call-order counter seeded by `id.with("auto")` and incremented per widget (ui.rs:58, 154, 262, 889-900). **Adding/removing any widget above shifts every subsequent auto id** (e.g. the conditional preset-storage banner in `WebShell::ui`, web.rs:370-390, shifts the whole column below it). Widgets with explicit ids (panels: `Panel::right("browser_preset_storage")` web.rs:373, `"shared-panel-dock"` panel_docking.rs:452-460; photocraft's `Id::new("dock-group-rects")` etc.) are stable. ⇒ Raw AccessKit NodeIds are *not* a stable anchor currency for design review (whose whole point is that the UI then changes); anchors must be role+label+rect or explicit keys.

### (c) Feature flags — what photocraft-web compiles today

- `apps/photocraft-web/Cargo.toml:33`: `eframe = { workspace = true, features = ["wgpu", "accesskit"] }` (wasm32 target block). Also `apps/photocraft/Cargo.toml:32` (desktop) and `crates/ui-egui/Cargo.toml:34`. Workspace dep: root `Cargo.toml:55` `eframe = { version = "0.36", default-features = false }`.
- eframe's `accesskit` feature = `["egui-winit/accesskit"]` (eframe Cargo.toml.orig:39-40); egui-winit's = `["dep:accesskit_winit"]` (egui-winit Cargo.toml:54-55); on wasm that dep resolves to the null adapter. Note **egui itself depends on `accesskit` unconditionally** (egui Cargo.toml.orig:75 `accesskit.workspace = true`) — so the Node/TreeUpdate types are already in the wasm binary and the feature's only extra payload on web is the no-op adapter. Enabling the tree at runtime costs nothing until `enable_accesskit()` is called.

### (d) Perf characteristics (would-be, for a patched build)

- Tree building is per-frame and full-tree: every pass allocates one `accesskit::Node` per widget (profiling scopes `"accesskit"` at context.rs:521, 2695) and `end_pass` resends the **entire** node map (context.rs:2699-2715) — there is no diffing on egui's side. On native this is mitigated by lazy activation: the adapter's `update_if_active` only runs the updater when an assistive technology has connected (`accesskit_winit/src/lib.rs:262`; eframe native reacts to `InitialTreeRequested`, winit_integration.rs:178-188). A web bridge would pay the full-tree cost every frame it streams; hence the bridge should stream **on demand** (only while pin mode is armed) or diff in JS.
- `register_widget_info` — the separate `WidgetRects.infos` map (widget_rect.rs:104-106, "Only filled in if the widget is interacted with, or if this is a debug build") — is debug-gated (context.rs:1572-1583); release builds can't read labels from WidgetRects, another reason the accesskit tree (labels ride in the nodes themselves) is the right source *if* it were reachable.

### (e) Would the DOM nodes be clickable by annotakit? What would the pin capture?

Moot today — no nodes exist. For completeness, had a web adapter existed: DOM a11y trees are typically mirror-`div`s with `role`/`aria-*` attributes, usually `pointer-events: none` and frequently outside normal layout; annotakit's hit-testing uses `document.elementsFromPoint` (which **skips `pointer-events:none` elements** — its own `hittest.ts:5-8` documents this, with a manual rect-walk fallback `deepestGreyedAt` that only refines *descendants* of the top hit — hittest.ts:88-151). Canvas-mirror nodes as siblings of the canvas would not be hittable without making them interactive, and their identity (`div` + `role` + `aria-label`) would still be fuzzier than explicit `data-testid`s. This is why even a future real web adapter would want the same overlay shim recommended here.

---

## 2. Q2 — The JS-bridge path (what exists, and the natural extension)

### (a) What `window.PhotoCraft` exposes today

`apps/photocraft-web/src/js_bridge.rs` (576 lines), installed by `web::start()` before the eframe runner starts (web.rs:47-51, `install_global()` js_bridge.rs:354-408 sets `globalThis.PhotoCraft`):

- **Methods** (js_bridge.rs:356-403): `commands(filter)`, `open({name,bytes})`, `create(params)`, `execute({id,params})`, `run({steps})`, `info()`, `export({format,options})`, `convert(...)`, `batch({inputs,steps,format,options})`, `close()`. Also a direct `#[wasm_bindgen]` class `PhotoCraftBridge` (js_bridge.rs:275-345).
- **Architecture fact (kb `browser-js-bridge.md:11`)**: "The host-facing bridge owns an independent headless `Session`; it does not mutate the eframe canvas's visible editor session" — `BrowserBridge { session: Session, … }` (js_bridge.rs:11-15), parked in a `thread_local! GLOBAL_BRIDGE` (js_bridge.rs:349-352), methods stored as `Closure`s in `GLOBAL_METHODS` (js_bridge.rs:351, 410-423). Values cross as JSON via `JSON.stringify/parse` (js_bridge.rs:467-478), file bytes as `Uint8Array` (never JSON — kb:12; js_bridge.rs:445-451).
- **No events, no UI introspection**: the surface is purely engine commands + document info. Nothing today can answer "what does the live UI look like".

### (b) Does the egui side already publish widget rects anywhere reachable?

**Yes — three distinct layers:**

1. **Photocraft's own per-frame rect registries in `ctx.data`** (egui `IdTypeMap` temp storage, publicly readable via `ctx.data(...)`):
   - `crates/ui-egui/src/dock.rs:465-467` — `pub fn last_rects(ctx) -> Vec<(Group, Rect)>`, stored under `Id::new("dock-group-rects")`, written each frame at dock.rs:727. `Group` is a stable panel identity enum (dock.rs:28, `ALL: [Group; 7]` :47). This is *exactly* the panel-rect stream the design review needs, already used by ~15 test files (`dock_tests.rs:130` etc.).
   - Nine more ad-hoc rect stores, same pattern: `panel_rail.rs:96` (rail rect), `panels.rs:233` (tool flyout), `panels.rs:2096` (navigator preview), `panels.rs:2218` (navigator controls), `titlebar.rs:822` (`pc-dialog-rects`), `dialogs.rs:1112`, `jobs_ui.rs:850` (open-card rects), `mask_thumbs_ui.rs:170` (per-layer thumb rects), `pressure_curve_ui.rs:68`.
2. **egui 0.36.2 public API, limited**: `Context::interactive_rects_last_pass()` (context.rs:1389-1421) — all interactive widget rects of the previous pass, in global points with layer transforms, but **no ids/labels** (designed for platform input regions). The full `WidgetRects` (id+rect+sense, widget_rect.rs:89-233) is stored per-pass but is `pub(crate)` — no public getter.
3. **The AccessKit tree** — richest data (id/role/label/bounds) but unreachable on web without an eframe patch (Q1b).

### (c) The natural extension: `PhotoCraft.ui({op:"rects"})`

**Rust side** (all in-repo, ~150–250 lines):

1. **A named-target registry in ui-egui** (new module, e.g. `crates/ui-egui/src/annota.rs`, ~80–150 lines): one canonical `Vec<AnnotaTarget{key, role, label, rect}>` under a single `ctx.data` temp id (consolidating the scattered stores; panels get keys from `Group`, buttons from their command ids). Published at end of each panel draw (one line at the existing call sites; `last_rects` already covers the dock). This is the same pattern the codebase uses nine times over, so it needs no new egui APIs and stays wasm-safe (AGENTS.md rule 7) and testable with the existing kittest harness (assert keys+rects stable across collapse/relayout — the dock tests already assert rect stability, `dock_tests.rs:587`).
2. **A bridge extension** (`apps/photocraft-web/src/js_bridge.rs` or sibling `annota_bridge.rs`, ~80–120 lines): a second `thread_local` holding `Option<egui::Context>` + canvas element, set inside the app-creator closure in `web.rs:85-108` (the `cc.egui_ctx` clone is right there); one new method `ui` with op `rects` that reads the registry via `ctx.data`, converts egui points → CSS px (canvas `getBoundingClientRect().width() / ctx.screen_rect().width()` — the inverse of the mapping `drag_pos_service` already does at web.rs:174-181), and returns the JSON array. Follow the bridge's existing conventions: JSON round-trip for plain values (js_bridge.rs:467-478), unknown-key validation (js_bridge.rs:197-199 precedent), and a Node-oracle test appended to `apps/photocraft-web/tests/js-bridge.mjs` (kb "Oracle testing without a browser", browser-js-bridge.md:15-28).

**Frontend consumption — pull, not push.** The bridge's method surface is request/response; rect streaming is only needed *while annotakit's pin mode is armed*. The review harness runs a rAF loop calling `PhotoCraft.ui({op:"rects"})` only while pin mode is active (detectable without touching annotakit: `enterMode('pin')` adds `document.body.classList 'annota-cursor'` — layer.tsx:747-748; a `MutationObserver` on the body class gates the loop and the overlay's pointer-events). Cost when idle: zero. A push API (`PhotoCraft.ui.onRects(cb)` via a stored `Closure` — the storage pattern already exists at js_bridge.rs:347-352, 410-423) is a possible later optimization; polling is simpler and frame-accurate enough (≤1 frame staleness).

---

## 3. Q3 — Overlay-shim design (the DOM annotakit pins against)

**Structure** (review page only; created by Rust via web-sys — `Element`, `HtmlElement`, `CssStyleDeclaration`, `DomRect` features are already in `photocraft-web/Cargo.toml:38` — or by ~80 lines of harness JS; both keep the *product* JS-free per AGENTS.md):

```
<body>
  <div id="storybook-root">            ← annotakit's storyRoot(): layer.tsx:96-97
    <canvas id="photocraft_canvas"/>
    <div id="photocraft_annota" style="position:absolute;inset:0;pointer-events:none;z-index:5">
      <div data-testid="panel-layers" role="region" aria-label="Layers"
           style="position:absolute;left:..px;top:..px;width:..px;height:..px"></div>
      <div data-testid="tool-brush" role="button" aria-label="Brush" …></div>
      <div data-testid="canvas-viewport" role="region" aria-label="Document canvas" …></div>
    </div>
  </div>
</body>
```

- **Annotakit root contract**: `storyRoot()` = `document.getElementById('storybook-root') ?? document.body` (layer.tsx:96-97). Wrapping canvas+overlay in `#storybook-root` makes (i) `bboxOf(el, root)` root-relative fragments (anchor.ts:94-103) equal to canvas-relative CSS px, (ii) `isPinnableTarget(target, scope)` pass — pins must be *strict descendants* of the scope (hittest.ts:164-166, enforced at layer.tsx:1130), and (iii) the region fragment math (layer.tsx:1248-1254) come out right. Either the review page carries that wrapper (index.html variant for review builds, mirroring the `?webgl`/`?cpu` query-flag pattern at main.rs:14-15 / web.rs:201-202), or annotakit's one-line `storyRoot()` gets parameterized (it's the owner's own project).
- **Identity keys**: `data-testid` = semantic, dash-separated (`panel-layers`, `tool-brush`, `dialog-export`). Dash-separation matters: @medv/finder v4 only accepts attribute seeds whose *name and value are "word-like"* (dist bundle `chunk-54KKMA3X.mjs:2-8, 96-110, 129-137` — `data-*` names are accepted, values must match `/^[a-z\-]{3,}$/` + vowel heuristics), so `[data-testid="panel-layers"]` is an emittable low-penalty selector while `panel:layers` would never be. `role` + `aria-label` ride along for humans (`aria-label` is also in finder's accepted attrs and in annotakit's `FINGERPRINT_ATTRS`, anchor.ts:18-27).
- **Pointer-events lifecycle**: default `pointer-events:none` on the overlay root ⇒ `elementsFromPoint` skips the divs entirely (hittest.ts:5-8), no layout impact (absolute positioning), **zero interference with canvas input**: eframe-web installs `pointerdown`/`wheel` on the *canvas* (eframe `web/events.rs:94,104`) and only `pointermove`/`pointerup` on the document (events.rs:92-93) — so with pe:none the app behaves identically to today. When pin mode arms (`body.annota-cursor` appears — layer.tsx:747), the harness flips the overlay root to `pointer-events:auto`: now `elementsFromPoint` returns the div, `hitTest` (hittest.ts:173-188) picks it, `actOnHit` captures the anchor (layer.tsx:1118-1146). While pe:auto, clicks over a div do not reach the canvas's `pointerdown` listener (divs are siblings, not canvas descendants) so egui won't start interactions under the pin cursor — desirable during capture; wheel-zoom over a div is likewise blocked, transiently. On `annota-cursor` removal (exitMode, layer.tsx:751-757) → back to pe:none.
- **Z-order**: annotakit's own chrome is `position:fixed; z-index:2147483000; pointer-events:none` with interactive children (styles.ts:9-12, 40-42) — anything below that coexists fine; `z-index:5` for the mirror keeps it visually nothing (transparent) and hit-order-independent (pe governs).
- **Sync**: rAF pull loop (Q2c) only while pin mode is armed; ≤1 frame latency (16ms) between an egui relayout and the mirrored rects — imperceptible for pinning; annotakit captures whatever rect the div has at click time. Optional refinement: annotate the harness to re-pull rects inside the same rAF tick as the class-flip.
- **Cleanup / lifecycle**: overlay created only when the review flag is on (e.g. `?annota` query flag handled in `web::start()` next to `?webgl`/`?cpu`, web.rs:61-69); divs are diffed by key (reuse nodes, `hidden` removed missing ones — a removed widget ⇒ its div disappears ⇒ the pin resolves to *orphan* and honestly renders at its stored fragment bbox, anchor.ts:10, layer.tsx:1526-1532 — do **not** keep stale divs alive, they would "resolve" to wrong rects via the attr rung).
- **What the captured pin stores** (what annotakit gets from a div): `context {tag:'div', role, ariaLabel, testid, outerHTML-clip}`, `fingerprint {tag:'div', attrs:[data-testid, aria-label], ownText}`, `selector {cssSelector: 'div[data-testid="panel-layers"]' (finder attr seed) + fragment}` — resolvable even after big moves because the **attr rung** (`findByFingerprint`, anchor.ts:400-415) queries `[data-testid=…]` with no bbox gate; the css rung additionally demands text-Jaccard/attr + IoU≥0.3 at same viewport width (anchor.ts:307-320). `elementSummary` renders `<div [testid=panel-layers label="Layers"]>` into agent digests (describe.ts:26-49).

---

## 4. Q4 — Annotakit anchor mechanics (what a pin IS, orphans, and the region escape hatch)

### What a pin stores

`ThreadTarget` (`src/shared/types.ts:214-221`): `{kind:'pin', selector:{cssSelector?, textQuote?, fragment?}, fingerprint?, context, bbox, captureViewportWidth}`.

- `captureAnchor(el, root)` (anchor.ts:250-282) fills it at click time:
  - `cssSelector` from **@medv/finder v4** rooted at the story root, ids excluded, CSS-module-ish classes excluded (anchor.ts:110-121); finder emits tag/class/attr/`:nth-child` seeds (chunk-54KKMA3X.mjs:19-150), including `[data-testid="…"]` (its default `attr` filter is *not* overridden by annotakit).
  - `textQuote` = own text-node quote + prefix/suffix + occurrence index (anchor.ts:125-162).
  - `fragment` = root-relative bbox snapshot (anchor.ts:94-103, 275).
  - `fingerprint` = tag + `FINGERPRINT_ATTRS` (`aria-label, title, placeholder, alt, name, type, href, data-testid`, anchor.ts:18-27) + neighbor text + own text (anchor.ts:164-187).
  - `context` = tag/role/ariaLabel/text/id/classes/**testid**/nth/name/value/placeholder/alt/label/outerHTML (anchor.ts:254-269; types.ts:154-179) — "the fields an agent needs to identify the EXACT node".
- Threads add `storyId`, optional React `component` fiber meta (absent for non-React hosts — `component?: ComponentRef | null`, types.ts:289 — photocraft threads simply carry null), comments, status lifecycle (types.ts:255-300).

### Orphan semantics + re-bake (seed merge)

- `resolveAnchor` cascade (anchor.ts:467-489): cssSelector (+score gate) → textQuote (bbox-gated, anchor.ts:326-341) → fingerprint attrs → **orphan**. Orphan pins render at the stored fragment bbox (layer.tsx:1526-1532) and count into the orphan badge (layer.tsx:693-694). Resolution re-runs on mutations, debounced 350ms/1200ms (layer.tsx:616-617, 646).
- Re-baked statics: `bake-static-threads.mjs` writes `annotakit-threads.json` next to the build; `staticStore` merges seed with per-deployment localStorage via the server's pure `logicalMerge` — per-thread union, **local edits win per-thread, tombstones delete-wins, resolved-wins** (staticStore.ts:8-25, 206-254). Anchors are not touched by the merge — they re-resolve against the *deployed* DOM. ⇒ **A canvas-widget anchor survives re-bakes iff its identity signals survive the UI revision**: explicit `data-testid` (semantic key) ≫ text ≫ position. This is precisely what the overlay mints, and why keys should be derived from stable concepts (panel `Group`, command ids) rather than labels or call-order.

### Region pins + freeze tool (the no-element coverage)

- **Region threads store NO element anchor**: `completeRegion` mints `{kind:'region', selector:{fragment:{x,y,w,h}}, context:{tag:'region'}, bbox, captureViewportWidth}` — pure viewport/fragment geometry (layer.tsx:1239-1257). They render as `rootRect + fragment`, always `status:'resolved'` (layer.tsx:1511-1520, 599) — they **work against the raw canvas today** (annotakit mounted on a wrapper page, region drag over "this panel's spacing", done). Caveats: no agent-usable identity ("what is this rect?"), and they never orphan — a relayout silently leaves them pointing at the wrong pixels (honesty gap, acceptable for spacing commentary, poor for "this button is wrong").
- **Freeze/frozen view**: clones the live DOM (`cloneNode(true)`, snapshot.ts:235-315) — DOM overlay divs clone fine (inline styles survive), but the *canvas bitmap* is copied best-effort via `clone.getContext('2d').drawImage(live, 0, 0)` ("works for 2D sources directly and best-effort for webgl", snapshot.ts:399-414; a wgpu/WebGL2 canvas without `preserveDrawingBuffer` may snapshot blank). So freeze on photocraft is *best-effort*: the cloned UI overlay geometry is right, the cloned canvas pixels may be empty on some backends. Treat freeze as experimental for photocraft; a future upgrade could feed it a real screenshot (eframe-web supports `ViewportCommand::Screenshot`, app_runner.rs:330-363 — photocraft already has screenshot plumbing on desktop).

---

## 5. Q5 — Cross-check: eframe 0.36 web + accesskit known issues / gating

- The discard is explicit and long-standing: `app_runner.rs:394 // not currently implemented` (and the general web-backend gaps logged right above it: unhandled `ViewportCommand`s "not implemented in web backend", app_runner.rs:335-340).
- eframe README.md:45: "egui supports AccessKit, but as of early 2024, AccessKit lacks a Web backend." (AccessKit's web work has not shipped into this registry — no `accesskit_js`/`accesskit_web` crate present.)
- There is **no `cfg(not(target_family="wasm"))` gate hiding a working path**: egui's tree builder itself compiles and runs on wasm (feature-independent; `accesskit` is an unconditional egui dependency, egui Cargo.toml.orig:75) — the break is purely at the eframe-web consumption point (`accesskit_update: _`) plus the missing platform adapter (accesskit_winit null.rs). The native path is fully wired for contrast: winit_integration.rs:172-189 (`InitialTreeRequested` → `enable_accesskit` + `RepaintNow`), run.rs:305/346-348 (`UserEvent::AccessKitActionRequest` → `on_accesskit_event`), egui-winit lib.rs:196-205 (`init_accesskit` with event-loop proxy) and 1192-1197 (`handle_platform_output` → `accesskit.update_if_active`).
- Consequence for photocraft-web today: the `accesskit` feature (Cargo.toml:33) compiles in the null adapter + already-present egui tree code and does nothing at runtime. Harmless, and conveniently means **all egui-side a11y data plumbing is already in the binary** — a future upstream web adapter (or a tiny fork) could light it up without re-architecting.

---

## 6. Ranked architectures + recommendation

### Ranking

1. **B — Bridge-overlay (RECOMMENDED)**: lowest risk, all in-repo, stable semantic anchors, composes with annotakit's existing capture/resolve machinery with **zero annotakit changes** (its attr/`data-testid` rungs were built for exactly this), incremental coverage, testable with existing harnesses (kittest rect asserts + Node bridge oracle). Costs: a bounded one-time registration sweep over the panels/buttons the review targets; a dev-only review host page.
2. **C — Region-only (fallback/stopgap)**: zero photocraft code; ships "canvas design commentary" today; but no element identity for implementer agents and silent drift. Use it *in addition* to B (regions for canvas-art, pins for UI chrome), or as the escape hatch if the bridge slice slips.
3. **A — AccessKit DOM (upstream-blocked)**: richest data (every widget, roles, labels) and zero per-widget instrumentation, but requires forking/patching eframe (2 edits: call `ctx.enable_accesskit()` on web; forward `accesskit_update` at app_runner.rs:394 instead of `_`) *and* a JS-side anchor synthesizer because NodeIds for buttons are call-order hashes (ui.rs auto-id salts; button.rs:325) — you'd re-implement annotakit's fuzzy matching on *worse* signals. Becomes the better route only when (i) AccessKit ships a real web adapter and eframe wires it (watch upstream; eframe README:45), and (ii) egui grows stable per-widget ids. The overlay+annotakit half of B survives that switch untouched (the overlay consumes `{key,role,label,rect}` regardless of the source).

### Recommended concrete steps (PoC first, then coverage)

**PoC — "pin a panel, prove identity + movement tracking" (~300–450 lines total):**

1. `crates/ui-egui/src/annota.rs` (~100 lines): `AnnotaTarget {key, role, label, rect}`; `publish(ctx, targets)` → `ctx.data_mut insert_temp(Id::new("annota-targets"), …)` (clone dock.rs:465-471 mechanics); `targets(ctx)` getter. Seed it from `dock::last_rects` + `Group` display names (both already exist — dock.rs:28/47/465/727), i.e. **the right-panel feature under review is covered by data that already exists**. Kittest test: keys+rects stable across collapse/expand (mirror dock_tests.rs:587).
2. `apps/photocraft-web/src/annota_bridge.rs` (~120–180 lines): `thread_local CTX: Option<(egui::Context, HtmlCanvasElement)>` set in the `WebRunner::start` creator (web.rs:85-108); method `PhotoCraft.ui({op:"rects"})` added in `install_global` (js_bridge.rs:354-408 pattern) returning `[{key,role,label,x,y,w,h}]` in CSS px (points × `canvas_rect.width / ctx.screen_rect().width()`; precedent web.rs:174-181). Unknown-op validation like js_bridge.rs:197-199. Node-oracle rows in `tests/js-bridge.mjs` per kb browser-js-bridge.md:15-28.
3. Dev-only review host (`apps/photocraft-web/review.html` + ~80–120 lines of harness JS, or Rust-side div management via web-sys — recommended for the "no product JS" rule, AGENTS.md:3): wraps canvas in `#storybook-root`, mounts the annotakit preview layer (it's a React component with a plain `storyId` prop — preview/index.ts:23-45), rAF-mirrors `rects` into `data-testid` divs, `MutationObserver` on `body.annota-cursor` flips overlay pe (Q3). Annotakit server/static-store runs beside it as for Storybook (mode.ts:51-55 probes).
4. Acceptance: with `?annota`, pin the Layers panel → thread captures `testid=panel-layers`; collapse/expand the dock → pin follows (attr-rung re-resolution); rename/remove the panel in a UI revision → pin shows orphan at last rect. Region pin over the canvas works out of the box.

**Build/test commands (codespace plane — do NOT run locally, 3GB box):** `cargo check --locked --target wasm32-unknown-unknown -p photocraft-web` (wasm gate), `cargo test -p photocraft-ui-egui` (registry asserts), `cargo build --locked --target wasm32-unknown-unknown --profile wasm-release -p photocraft-web --features heif` + `trunk serve` (Trunk.toml port 8765), `node apps/photocraft-web/tests/js-bridge.mjs` oracle, `cargo xtask layers` for layering.

**Then incremental coverage:** toolbar buttons (key = command id from menu_catalog.rs — command ids are the app's canonical stable identity, AGENTS.md:63), rail icons (panel_rail.rs:96 already stores the rail rect), dialogs (titlebar.rs:822 `pc-dialog-rects` already there), menus/popovers (ephemeral: publish-or-skip; pins on closed popovers orphan honestly).

### Risks / unknowns

- **Coordinate mapping** under browser zoom × DPR: derive from live `getBoundingClientRect` ratio every pull (never cache); verified against eframe's own inverse mapping (web.rs:174-181, eframe web/input.rs:62,87).
- **Locale drift**: `aria-label`s come from `i18n::t` — anchors must ride `data-testid` (stable), with labels as human garnish only (the attr rung matches testid first).
- **Annotakit host assumptions**: `#storybook-root` element (layer.tsx:96-97) and same-origin/`/annotakit/api` or static seed (mode.ts:38-55) — the review page must provide both, or annotakit gets a parameterized root (owner's own project, 1-line change).
- **Freeze view on wgpu canvas**: bitmap copy is best-effort (snapshot.ts:399-414); may render blank on WebGL2-without-preserveDrawingBuffer — don't advertise freeze for photocraft until verified on WebGPU.
- **Pe:none subtlety**: annotakit's `deepestGreyedAt` greyed-refinement only walks descendants of the top hit (hittest.ts:88-151) — our sibling overlay divs are invisible to it while pe:none, which is exactly why the pe-flip on `annota-cursor` is required; region mode adds no body class (layer.tsx:743-748) but needs no overlay (pure geometry).
- **Upstream movement**: an eventual AccessKit web backend + eframe wiring (eframe README:45) would let the same overlay consume the full tree — design the bridge payload (`{key,role,label,rect}`) so the source can swap without touching the harness or annotakit.
- **Wasm binary size**: bridge additions are behind `#[cfg(target_arch = "wasm32")]` like the rest of js_bridge.rs; the 24MiB per-file gate (docs/development.md) is untouched.

---

## Appendix — key citation index

| Fact | Source |
|---|---|
| eframe web discards accesskit_update | `eframe-0.36.2/src/web/app_runner.rs:394` |
| "AccessKit lacks a Web backend" | `eframe-0.36.2/README.md:45` |
| No web adapter (null on wasm) | `accesskit_winit-0.32.2/src/platform_impl/mod.rs:34-50`, `null.rs:8-24`, `Cargo.toml:96-120` |
| winit has zero accesskit code | grep over `winit-0.30.13/src` (no matches); adapter is external |
| Native enable path | `eframe-0.36.2/src/native/winit_integration.rs:172-189`, `run.rs:305,346-348`; `egui-winit-0.36.2/src/lib.rs:196-205,1192-1197` |
| egui tree build (bounds/roles/labels) | `egui-0.36.2/src/context.rs:412,520-529,600-628,1292-1297,2694-2717,3701-3707`; `response.rs:869-895,908-917,935-987` |
| Id = u64 hash; buttons order-derived | `egui-0.36.2/src/id.rs:99-105`; `widgets/button.rs:325`; `ui.rs:58,154,262,889-900` |
| WidgetRects not publicly iterable | `egui-0.36.2/src/widget_rect.rs:89-233`; public rect-only getter `context.rs:1389-1421` |
| kittest proves tree usable on PhotocraftApp | `egui_kittest-0.36.2/src/lib.rs:144-177`; `pc-edit/crates/ui-egui/Cargo.toml:60`, `tests/it/full_res_preview.rs:16-23` |
| photocraft-web accesskit feature | `pc-edit/apps/photocraft-web/Cargo.toml:33` (also apps/photocraft:32, crates/ui-egui:34) |
| Bridge surface + independent session | `pc-edit/apps/photocraft-web/src/js_bridge.rs:11-15,265-408,410-423,467-478`; kb `browser-js-bridge.md:11-12` |
| WebShell per-frame hooks | `pc-edit/apps/photocraft-web/src/web.rs:47-51,81-111,329-393,201-202` |
| Existing rect registries | `pc-edit/crates/ui-egui/src/dock.rs:28,47,465-471,727`; panel_rail.rs:96; panels.rs:233,2096,2218; titlebar.rs:822; dialogs.rs:1112; jobs_ui.rs:850; mask_thumbs_ui.rs:170; pressure_curve_ui.rs:68 |
| annotakit capture model | `storybook-annotakit/src/preview/anchor.ts:18-27,94-121,250-282,307-320,400-415,467-489`; finder v4 grammar `dist/chunk-54KKMA3X.mjs:2-8,96-137` |
| thread/target types | `storybook-annotakit/src/shared/types.ts:124-179,214-221,255-300` |
| pin/region flows, placement, orphan | `src/preview/layer.tsx:96-97,141,589-624,743-757,1099-1281,1239-1258,1495-1551`; `src/preview/hittest.ts:5-8,88-151,164-166,173-188`; `src/preview/styles.ts:9-12` |
| seed merge / re-bake | `src/shared/staticStore.ts:8-25,92-152,206-254`; `src/server/merge.ts` (logicalMerge) |
| freeze = DOM clone + best-effort canvas copy | `src/preview/snapshot.ts:235-315,399-414` |
| eframe web input targets (collision analysis) | `eframe-0.36.2/src/web/events.rs:63-109` (pointerdown/wheel on canvas; move/up on document) |
