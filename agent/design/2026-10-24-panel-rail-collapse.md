# Design — Panel visibility toggle + collapsed-dock inspector rail

Date: 2026-10-24 · Status: draft for peer review · Branch target: `feature/right-panel-collapse`

## User ask (verbatim intent)

"find way to optimize the right panel, to add a way to hide them all, perhaps a toggle ON TOP of
the current tool bar that toggle individual panel visibility and i want to have an overall
toggle, and when it is collapsed it should in vertical (90d orientation) show all the current
active panels, and clicking on them will expand them as floating, while losing focus will
collapse / hide those again, typical ux for inspector panels that support collapsing."

Interpretation: the right panel's own vertical icon rail IS the "tool bar" for panels. Add a
toggle control at its top; when the dock is collapsed (hidden), the rail shows ACTIVE panels as
90°-rotated text labels; clicking one opens it as a floating window; losing focus closes it.

## Current-state facts (all verified in code)

- 7 groups (`dock::Group`): Color, Properties, Character, Navigator, History, Layers, Brushes.
  Legacy dock = `dock.rs` (`right_dock` → `dock_panels` → `dock::show`); customized mode =
  `panel_docking.rs` (craft_ui::docking) once the user drags a panel out; both keep
  `state::Panels` bools in sync via `panel_docking::sync_visibility`.
- Rail: inline in `panels.rs right_dock()` — 6 icon buttons (Character MISSING from entries),
  gated `panels.rail`; the whole `right_dock()` is gated by `panels.dock` at `lib.rs:2444`, so
  when the dock is hidden the rail vanishes with it.
- `panels.dock` = whole right-dock visibility; `window.toggle.dock` flips it; Tab/togglePanels
  (menus.rs:607) is the wider chrome toggle. Everything persists via `dock::snapshot` →
  `prefs.panelLayout`.
- Per-panel legacy toggles: `menus.rs` arm `t if t.starts_with("window.toggle.")` flips the
  `Panels` bool + `dock::reveal` (#129: shown-but-collapsed → expand instead of hide).
  `panel_docking::command` intercepts the same ids in customized mode. → ONE command path works
  in both modes, no new commands needed for popover rows.
- Peek precedent: `dock.rs auto_collapse_peek` — click outside a peeked collapsed group closes
  it (pref `auto_collapse_icon_panels`).
- Floating-window precedent: `panels::properties_window`, `brush_panel::window`,
  `preset_panels::windows` — plain `egui::Window::show(ctx, …)` called after CentralPanel in
  `lib.rs` (lines ~2452-2455).
- Panel content: `panels::dock_body(app, ui, group, tab)`; the scrolls_itself/ScrollArea wrapper
  is duplicated in `dock.rs show()` and `panel_docking::body()` (DRY opportunity).
- Rotated text: `epaint 0.36.2 TextShape.angle` (radians clockwise, pivot = `pos`) — native
  90° labels, no painter-transform hack.
- Icons: `icon_data.rs` (assets/icons/*.svg); rail currently uses sliders-horizontal,
  navigation, palette, layers, clock, brush. A new "panel toggle" SVG will be added.
- Localization: `tl!` macro (lib.rs:11); all new user-visible strings go through it.
- Test harness: `egui_kittest` headless (dock_tests.rs, panel_docking_tests.rs pattern);
  AGENTS.md gates: `cargo check --target wasm32-unknown-unknown` (L0–L6), xtask layers, tests.

## Design decisions

### D1 — Rail top toggle + popover (individual + overall)
- New icon button at the TOP of the rail (separator below), visible in both dock states.
- Click → `egui::Popup` beside it:
  - Row "Hide Dock" / "Show Dock" (overall toggle) → `app.run("window.toggle.dock")`.
  - Separator; one checkbox row per group in `DockLayout::order()` (ALL 7 incl. Character):
    label `tl!(group title)`, checked = `g.shown(&app.ui.panels)`; change →
    `app.run("window.toggle.<key>")` (same path as Window menu + automation; #129 semantics
    free). Checkbox rows also show the group's icon (same icon as the rail).
- The button shows an "on/off" visual state: dock visible → normal; dock hidden → accent
  (reuses `icons::rail_button` styling, which already supports `on`).

### D2 — Collapsed dock → rotated labels rail
- `lib.rs` gate change: rail renders whenever `panels.rail` (independent of `panels.dock`);
  the dock column renders only when `panels.dock`.
- When `panels.dock == false`: below the toggle button, one rotated label per group with
  `shown() == true`, in `DockLayout::order()`. Vertical (−90°, bottom-to-top reading), rail
  width unchanged (pro 36 / studio 44 pts).
- Rendering: `ctx.fonts(|f| f.layout_job(...))` per label (7 max, trivial cost; no caching v1) →
  `TextShape { pos: bottom-left of the chip, angle: -FRAC_PI_2, fallback_color, … }` added via
  `painter.add`; chip = background rect + optional accent when hovered/active; hit rect =
  pre-rotation galley rect rotated about `pos` (w/h swap), `Sense::click()`.
- Click → floating peek (D3). The rail keeps the same separator/chrome painting as today.
- When `panels.dock == true`: rail shows today's icon buttons + the new top toggle (D4).

### D3 — Floating peek windows (inspector mode, focus-loss close)
- New transient state (NOT serialized, `#[serde(skip)]`-safe — lives in `UiState`):
  `pub panel_peek: Option<PanelPeek>` with `struct PanelPeek { group: Group, tab: usize }`.
  One at a time (replace-on-click). Selected tab = the group's current `dock_tabs` selection.
- Renderer: `panel_peek_window(app, ctx)` called in `lib.rs` beside `properties_window`:
  `egui::Window::new(tl!(panel label))` — `.id(Id::new("panel-peek"))`, `.collapsible(false)`,
  `.resizable(true)`, `.min_size([280, group.min_height()])`,
  `.default_size([dock default_width, group.default_height()])`, `.default_pos()` anchored
  left of the rail (screen right − rail width − window width; y = click position clamped).
- Content: shared `panel_body(app, ui, group, tab)` (D5 extraction) inside the window.
- Close conditions (all set `panel_peek = None`):
  1. Escape (`ctx.input Escape`) — same convention as `panel_docking::show`.
  2. Primary pointer press outside (window ∪ rail ∪ popover rects) — pattern of
     `dock.rs auto_collapse_peek`, extended with rail+popup rects.
  3. Window close (X) button via `.open(Option<bool>)`.
- Works in both modes: content path is mode-agnostic (`dock_body`); opening a peek never
  touches `panels.dock` or `docking` layout (read-only on them).

### D4 — Rail in dock-visible state
- Keep icon buttons (existing `rail_click` behavior untouched); add Character as 7th icon;
  add the top toggle button (D1) + separator. `rail_on`/click routing unchanged.

### D5 — Code structure ("optimize the right panel")
- New module `crates/ui-egui/src/panel_rail.rs`: `pub(crate) fn rail(app, ui)` (icons mode +
  labels mode + toggle popover), `pub(crate) fn peek_window(app, ctx)`, the shared
  `pub(crate) fn panel_body(app, ui, group, tab)` extracted from the two duplicated wrappers.
- `right_dock()` slims to: panel_docking::show → (dock column if shown) → rail() → persist.
- `lib.rs`: `if panels.rail { panel_rail::rail(self, ui) }` (always) +
  `if (chrome && panels.dock) || tabbed_in { right_dock(...) }` for the column part;
  `panel_peek_window` next to the other floating windows.

### D6 — Persistence
- Only `panels.dock` + per-group bools change — both already inside `dock::snapshot` →
  `prefs.panelLayout`. `panel_peek` transient (like `docking_reveal`). No schema changes.

### D7 — Tests (`panel_rail_tests.rs`, egui_kittest)
- T1 popover lists 7 groups; checkbox toggles `panels.<g>` (both legacy and customized mode).
- T2 `window.toggle.dock` → rail switches to labels mode; labels = shown groups in order.
- T3 label click → `panel_peek == Some((group, tab))`; window draws group content.
- T4 click outside / Escape / X → peek closes; re-click reopens.
- T5 dock re-shown → icon mode returns; persistence snapshot round-trips.
- Standard gates: `cargo check --target wasm32-unknown-unknown`, clippy, fmt, xtask layers,
  i18n coverage (tl! strings), dock_tests/panel_docking_tests still green.

### D8 — Open questions for review
1. Rotated labels show GROUP names only (not each tab) — right call for v1?
2. Single floating panel at a time — acceptable, or multi-window expected from "panels"?
3. Should the toggle popover ALSO appear on right-click of any rail chip?
4. In dock-visible state, is the extra top toggle + 7th icon too much rail height?
5. `panels.rail == false` (embedding) hides everything incl. toggle — Window menu + commands
   remain the alternative. Confirm acceptable.
6. Peek window and `auto_collapse_icon_panels` pref interplay: peek windows close on focus
   loss ALWAYS (not gated by the pref) since their whole point is transient focus — right?

## Implementation plan (doubles as local-build-loop stress test)

1. Branch off origin/main + commit live-agent WIP snapshot (content parity with the codespace
   build cache we inherit).
2. panel_rail.rs: toggle button + popover (D1) → cargo check loop (local, cached deps).
3. Labels mode (D2) → check + visual pass via local wasm dev build + browser.
4. Peek windows + focus-loss (D3) + wiring (D5) → check.
5. panel_body dedupe; tests D7; gates.
6. Local dev-wasm build (wasm-release profile, lto=false in live agent's Cargo.toml WIP state)
   → trunk serve → agent-browser E2E + screenshots.
7. Peer code review sub-agent → fixes → push branch.
