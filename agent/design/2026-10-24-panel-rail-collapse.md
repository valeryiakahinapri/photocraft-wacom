# Design — Panel visibility toggle + collapsed-dock inspector rail

Date: 2026-10-24 · Status: REV 2 after sub-agent review (P0s incorporated) · Branch target: `feature/right-panel-collapse`

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

## REV 2 — review round 1 outcomes (sub-agent audit, INCORPORATED)

Verdict was "needs changes" (3 P0s). All incorporated below; supersedes conflicting earlier text.

- **P0-1 (D1 revised):** popover rows must NOT run `window.toggle.<key>` — Character has no slot
  in that arm (menus.rs:644-668, its real id is `window.panel.character`), and unchecking
  Brushes would migrate the whole workspace to customized docking (panel_docking.rs:179-191 +
  change() committing `docking=Some`). New dispatcher `panel_rail::set_group_shown(app, g, show)`:
  legacy → `window.panel.<primary>` with explicit `{"show": bool}` (view_cmds.rs:654-685,
  bypasses #129); customized → show=`window.panel.activate {panel:<first member>}`,
  hide=`window.panel.group {panel:<first member>, operation:"close"}` (group-level, matches
  `sync_visibility` projection). Tests must cover Character + Brushes in both modes.
- **P0-2:** remove the `app.ui.panels.dock = true;` re-asserts in panel_docking.rs:267 and :355 —
  they resurrect a collapsed dock on every panel command and corrupt Tab/`hidden_by_tab` state.
  Rail visibility is a pure render gate. Regression tests: dock stays hidden across every
  popover row + `window.panel.layers` fired from crop_ui.rs:1657 in customized mode.
- **P0-3 (D5 revised):** keep ONE entry point. lib.rs:2444 becomes
  `if (chrome && (self.ui.panels.dock || self.ui.panels.rail)) || tabbed_in { panels::right_dock(self, ui); }`;
  right_dock gates internally: `panel_docking::show` and both `dock_panels` calls behind
  `panels.dock`; rail block behind the derived `rail_visible` rule (P1-1 below); `dock::persist`
  stays at the end of right_dock (else popover hides never persist). Pro order (dock column,
  THEN rail) and Studio order (rail then dock) preserved — panels.rs:1797-1838.
  `panel_rail::rail()` is called from right_dock, NOT lib.rs (keeps kittest harnesses green).
- **P1-1:** `rail_visible = panels.rail && !(hidden_by_tab.is_some() && !panels.dock)` —
  Tab still hides everything; no schema change (do NOT widen hidden_by_tab — it's serialized).
- **P1-2:** label predicate: customized → `panel_docking::rail_on(app, g)`; legacy →
  `g.shown(&p) && !app.ui.dock.visible_tabs(g, pro).is_empty()` (all-Closed groups must not
  produce empty peeks; dock.rs:531-534 skips them).
- **P1-3 (D3 revised):** egui 0.36.2 `Window::open` takes `&mut bool` (no Option variant —
  window.rs:167-176); use `let mut open = true; … if !open { clear }`. First frame after open:
  `ctx.memory area_rect(PEEK_ID)` is None → treat as inside (no insta-close). Clear `panel_peek`
  on: dock re-show transition, `dock::apply` (like `docking_reveal`, dock.rs:856), workspace
  switch. Do NOT close on OS-window blur (`input.focused==false` is only for drag-cancel —
  panel_docking.rs:383).
- **P1-4:** store anchor (click pos) in `PanelPeek`; `.current_pos(clamped)` on the first frame
  of each open (window.rs:325-331); `default_pos` is not re-applied with a fixed id. Coordinates
  in points from canvas/content rect (brush_panel.rs:511-518 precedent), not "screen right".
- **P1-5 (RTL):** chip geometry derived from the allocated rect; RTL mirrors to
  `angle: +FRAC_PI_2` with pivot at chip top-right; popup align LEFT_START (LTR) /
  RIGHT_START (RTL); peek anchored to the rail's actual side. RTL tests via
  `direction_scope(Dir::Rtl)` asserting `TextShape::visual_bounding_rect()` ⊆ chip rect.
- **P2 bundle:** toggle button uses `icons::button` (accent states; rail_button has no accent)
  — icons.rs:185-216; reuse the EXISTING `panel-right` icon (icon_data.rs:105, no new asset, no
  ATTRIBUTION row); `Group::label()` maps to strings ALREADY translated in the 21 catalogs
  ("Color & Swatches" etc.); only "Hide Dock"/"Show Dock" are new → add i18n/*.tsv rows for
  all catalogs + `cargo xtask i18n-coverage` gate; labels elide + ScrollArea fallback for short
  windows; wrap toggle in `ui.push_id("panel-toggle")` for a stable popup id across modes;
  `widgets::style_spectrum_popup_menu`; add `window.panel.peek {panel, show}` shell command
  (automation parity, menus.rs arm pattern); `panel_body` dedupe is its OWN commit (it changes
  customized-dock scroll: dock.rs:545-553 sets fade.strength=0/#2726 which panel_docking::body
  lacks).
- **D8 answers adopted:** (1) group names on the rail; peek window carries the group's TAB PILLS
  (writing dock_tabs.<g>; no activate-command churn while dock hidden). (2) single floating at a
  time — yes. (3) no right-click popover on chips. (4) icon-mode height fine; labels mode gets
  elide+scroll. (5) panels.rail=false hides everything incl. toggle — acceptable, documented.
  (6) focus-loss close is unconditional but in-app only (no OS blur close).
- **egui 0.36.2 mechanics (verified signatures):** rotated chip = `allocate_exact_size` +
  `epaint::TextShape::new(pos, galley, tint).with_angle(-FRAC_PI_2)`; galley via
  `ctx.fonts_mut(|f| f.layout_no_wrap(...))` (fonts() view is read-only in 0.36 — canvas.rs:538
  precedent); popup = `egui::Popup::menu(&resp).align(egui::RectAlign::LEFT_START).show(|ui| …)`;
  focus-loss = `input(|i| i.pointer.any_pressed().then(|| i.pointer.press_origin())` +
  `memory(|m| m.area_rect(id))` idiom from dock.rs:465-477 + popup rect check.
- **Branching (revised):** WIP snapshot commit includes all 9 modified + 2 untracked live-agent
  files EXCEPT the workspace Cargo.toml profile hunk (lto=false is the live agent's pending
  call — not ours to commit; build parity comes from the pc-build workspace + env overrides
  `CARGO_PROFILE_WASM_RELEASE_LTO=false CARGO_PROFILE_WASM_RELEASE_CODEGEN_UNITS=16` locally).
  First feature commit = panel_rail.rs extraction (moves the WIP-touched rail frame block out of
  panels.rs → eliminates the one true collision zone); never touch dock_panels/toolbar after.
  Before PR: rebase feature commits onto plain origin/main.
- **Implementation order (revised):** 1) branch+WIP snapshot → 2) panel_rail extraction with
  right_dock internal gating + rail_visible (no new UI yet; adjust existing harness tests) →
  3) headless plumbing: set_group_shown, remove dock re-asserts, panel_peek field + clear
  points, window.panel.peek command → 4) D1 popover → 5) D2 labels + RTL → 6) D3 peek + focus
  loss → 7) panel_body dedupe (own commit) → 8) tests D7+P0/P1 regressions, gates, wasm E2E,
  final review.

## REV 3 — code-review round outcomes (2026-10-24 session 2, INCORPORATED)

Round 2 (implementation review) verdict: fix-then-ship. Landed in f8d0faae:

- P1 fixed: `peek_window` now owns its lifecycle — `panels.dock || !rail_visible` clears the
  peek whatever path forgot (Tab chrome, ui.set, workspace restore, panels.rail off, a peek
  command while the dock shows). The rule lives in `panel_rail::rail_visible`.
- P2s landed: per-group window ids (resized size no longer leaks across groups); live tab read
  each frame; the visibility popover's rect counts as inside the focus-loss rule; customized
  hide closes every layout member of the group (members can span tab groups); #6760 full-screen
  Tab keeps drawing the dock column (`show_column = p.dock || tabbed_in`); chips use
  `theme::medium(11.5)`; RTL fallback rect mirrored; separator in both rail modes;
  `window.panel.peek` registered in `engine/src/shell_commands.rs`; test order fixed (explicit
  show BEFORE the no-params activate, so the close assert is not vacuous) + Tab regression row.
- Deliberate deviations from REV2 text (code wins, doc amended here):
  - D1 toggle visual: `selected = p.dock` — accent while the dock SHOWS (a standard on/off
    toggle; the labels-mode rail itself advertises the hidden state). REV2's "accent when
    hidden" line is superseded.
  - D3 outside-press rule now includes the popover rect (as REV2 originally specified).
  - Label elide: dropped in favour of the ScrollArea alone (the chip taller than the whole
    rail is pathological; revisit only with real locale data). RTL kittest row + the
    panel_body dedupe stay deferred (see HANDOFF).
- Environment lessons recorded elsewhere: native kittest graphs DO NOT fit on the 9.9G local
  disk (target/debug needed 6-10G, filled /home/z at 100% mid-engine) — tests run on the
  CODESPACE in an isolated worktree + CARGO_TARGET_DIR=/tmp/test-target; close the headless
  browser before local rustc (OOM kill otherwise); the cache tarball now lives at
  /tmp/my-project/photocraft-cache/pc.tar.zst (PolarFS, recycle-durable).

## REV 4 — modern-canvas-studio mockup refinement (2026-10-24 session 3, DRAFT FOR REVIEW)

The owner shipped a mockup (github.com/valeryiakahinapri/modern-canvas-studio —
`src/components/ps/Shell.tsx`, a Lovable/React Photoshop mock) that IS the vision for the
right-panel behavior. Its grammar (extracted from the code, not guesswork):

- The right island = permanent 44px icon rail + a 316px panel column. The rail NEVER changes
  shape: same three group icons (color/adjust/layers) in both states.
- The rail's TOP button is a DIRECT overall toggle (`setRightOpen(!rightOpen)`): the column
  animates `transition-[width,opacity] duration-300 ease-out` to `w-0 opacity-0` while the
  rail stays. No popover anywhere.
- Expanded (`rightOpen`): rail icon click = per-group toggle (`collapsed[key]` flips — the
  card collapses to its header). Icon `active = !collapsed[key]`.
- Collapsed (`!rightOpen`): rail icon click = FLYOUT — a 300px floating panel anchored
  beside the rail at the icon's own y-offset (`right-full`, `top: {color:46,…}`), header =
  name + X, `max-h-[70vh]` scroll, fade+slide-in-right 150ms, ONE flyout at a time
  (same-icon re-click closes), expand-all closes any flyout. Icon `active = flyout === key`.

### Deltas vs our REV3 implementation

R1. **Collapsed rail = the same constant icon list, not rotated labels.** The 90° label
    chips (D2) are replaced by the ICON_ORDER icons — always all of them, like the mockup's
    constant rail. `on` = peeked group only (`active = flyout === key`); shown-state does not
    style the collapsed rail (the column is gone; there is nothing to reflect). Clicking a
    hidden group's icon still opens its peek (the rail is the Window-menu-style affordance).
    The label_chip fn, its RTL rotation, its a11y widget_info and the ScrollArea die; icons
    already carry hover tooltips + rail_button styling. Rotated labels were OUR reading of
    "vertical strip" — the owner's mock supersedes. (The rotated-text mechanism — TextShape
    angle — remains proven for future use; nothing else in-tree needs it.)
R2. **The top toggle becomes direct.** Click = `window.toggle.dock` (the same command the
    popover's row ran; #129-safe, both dock modes, both themes). The D1 popover
    (`visibility_menu`), its popup-id publication, and the peek's popup-rect focus-loss
    exception are REMOVED — per-group visibility is reachable by clicking the rail icons
    themselves (expanded) and in the Window menu. `set_group_shown` loses its only
    production caller — REMOVED with its test (its knowledge lives in dock::rail_click /
    panel_docking::rail_click which the icons use; the Window menu keeps its own ids).
    Keep the "Hide Dock"/"Show Dock" catalog rows (unused keys are harmless; the i18n gate
    is a coverage report) — noted as accepted debt, reusable by a future menu row.
R3. **Dock column animation (the one NEW mechanism).** Target width = the resizable
    column's current width (NOT hardcoded 316): on `p.dock`/`tabbed_in` flip, animate
    0 ↔ current width over ~300ms with an ease-out mapping, content laid out at full width
    inside a clip (the mockup's `overflow-hidden` reveal: the column wipes open from the
    canvas side). Mechanics: `ctx.animate_value_with_time(id, target, 0.3)` for the glide
    + a cubic-out remap of the 0..1 progress; during the glide the panel is presented
    `exact_size(animated)` (resize handle suppressed for the 300ms — imperceptible); below
    ~1px the panel is not drawn at all (zero cost while collapsed, #1313 intact). Opacity
    fade only if `Ui::set_opacity` exists in egui 0.36 and behaves for panel fills — else
    width alone (the reveal reads as the dominant effect). GOTCHAs to verify in review:
    (a) resizable SidePanel width state must not fight exact_size — read the live width,
    animate, restore; (b) the canvas reflows per frame for 300ms — acceptable repaint cost,
    but the animation must TERMINATE (no per-frame wake once at target); (c) kittests must
    still find panels: tests wait or assert post-animation state.
R4. **Kept from REV3 (mockup is silent = our spec wins):** focus-loss close on the peek
    (outside-press + Escape + close + dock-return; the mockup simply doesn't model outside
    clicks — Photoshop's collapsed flyouts close on click-away, and the owner's original
    spec demanded it), per-group window ids, live tab pills, #6760 tabbed_in, RTL (via
    direction::end_panel — icon rail is RTL-shape-free), the `window.panel.peek` command,
    panel_body dedupe deferral.
R5. **Tests:** rewrite panel_rail_tests for the icon rail: toggle click flips panels.dock
    (no popover); collapsed rail shows group ICONS (a11y labels still query by group name
    — rail_button titles); icon click opens the peek (window with the group's label
    exists); re-click/Escape/outside-press closes; expanded icon click still routes
    rail_click (collapse-to-strip, #129). Drop the popover + set_group_shown rows.

## REV 4 — review outcomes (2026-10-24 session 3, INCORPORATED)

Round 4 (sub-agent review, worklog `1-r4-review`) — verdicts landed:

- **R3 replaced wholesale**: the hand-rolled exact_size glide would have corrupted egui's
  persisted dock width (`PanelState` stores every frame outside its own animation paths; the
  collapse glide's last store ≈ 0 → dock resets to DOCK_MIN_WIDTH on every hide/show). The
  robust mechanic is egui 0.36's own `Panel::show_collapsible`: full-width content clipped to
  the animated slice, store skipped while animating, cubic-out ~0.2s, repaint only while
  sliding, RTL automatic, honors the Enable-Animations pref. Both rail-order call sites
  (pro/classic) pass `&mut expanded` through `dock_panels`; `.drag_to_open(false)` (the rail's
  icons and toggle are the way back — no stray grab handle); egui's gesture collapses
  (drag-past-min) write back through `window.toggle.dock` — or `window.togglePanels` when the
  #6760 tabbed-in chrome was the shower. Kittests set animation_time=0, so final states assert
  as before.
- **Deliberate deviation (scoped)**: the CUSTOMIZED-docking column keeps its instant toggle
  this round. `panel_docking::show`'s prologue consumes `docking_reveal` and the
  `dockWidth` request before the panel decides to draw, so calling it while collapsed
  (required to animate its slide-out) would eat those requests — a safe integration needs
  that state moved into the draw closure, a refactor beyond this round. The shared RAIL (the
  mockup's core grammar — icons, peek flyout, focus loss) behaves identically in both modes;
  only the collapse animation is legacy-only for now. Noted for a follow-up.
- **P1s landed**: re-click on the lit (peeked) rail icon closes the peek (no-params
  `window.panel.peek` parity); the STALE pre-existing test assert (`area_rect(peek_id())`
  after f8d0faae moved windows to per-group ids — verified never-registered) fixed to
  `peek_window_id(g)`; hidden-group icons open peeks (all-Closed groups render their stored
  tab, no pill row — asserted); the dock-resurrection regression rows survive in their own
  test now that `set_group_shown` is gone.
- **P2s**: opacity fade dropped (`Ui::set_opacity` does not fade a SidePanel's Frame fill);
  `ui.set dockWidth` mid-animation is consumed-and-lost (accepted; automation + a 0.2s window
  is pathological); module docs updated.
- The unused "Hide Dock"/"Show Dock" catalog rows stay (harmless; coverage is a report).

---

## REV5 — reconciling the parallel implementation (main `c8105921` vs this branch)

2026-10-24, after the live agent landed `c8105921 Refine rounded dock and panel interactions`
on main — an independent implementation of the SAME mockup grammar (fold + flyouts + top
toggle), built in the codespace in parallel with REV4's implementation on this branch.

### Their side (mechanism inventory)

- `panels.dock_expanded` (serde default yes): the fold state, deliberately INDEPENDENT of
  `panels.dock` — "the dock can fold without changing Shift+Tab visibility" (their test).
  Rationale: the fold is a persistent user preference; Tab/⇧Tab visibility is a session
  toggle; a fold driven by `p.dock` would be undone by a Tab round-trip.
- Animation: `animate_bool_with_time(id, expanded, 0.22)` × remembered width
  (`dock_last_width_id` temp) fed as `exact_size` to the same `Panel::right` — on BOTH the
  legacy column and `panel_docking::show` (which now receives `(width, transitioning,
  requested)`; its early `width <= 0.5` return happens BEFORE `docking_reveal.take()`, so
  reveal requests survive a fold).
- Rail: 44/52px sizes, `t.canvas` island fill + `outer_margin(4)`, top chevron toggle
  (`chevrons-right/left`, "Collapse/Expand Panel Dock") + separator, icon flyout state in
  ctx data (`right-dock-flyout` temp Group), flyout = `Area::fixed_pos` anchored per-icon,
  300px viewport-clamped, X + Escape + outside-click close.
- New command `window.toggle.dockExpanded` (registered in shell_commands; handled as an
  early-return arm in `menus::invoke_unguarded`), `checked()` row, `state.rs` field +
  `Default` + deserialization test, two dock_tests additions, toolbar width tween
  (`animate_value_with_time` 0.22 + `t.canvas` + +8 pad) + tween-aware test steps
  (`with_step_dt(1/60)`, `run_steps(16)`), canvas/tab_strip/theme/prefs/widgets visual
  polish (t.canvas tab fills, radius_sm, dropped separators), kb + SKILL + ui-design docs.
- GAPs in their commit: the two new "Collapse/Expand Panel Dock" strings have NO i18n
  catalog rows; the hand-rolled width tween is the P0-1 hazard class REV4 rejected; the
  flyout is inline in panels.rs; no browser E2E verification (kittests on codespace are
  the parked blocker).

### This branch's side (already E2E-verified)

- `Panel::show_collapsible` on the legacy column (cubic-out, store-skip while animating,
  RTL-aware, honors Enable-Animations) with gesture write-back through commands; rail
  extracted to `panel_rail.rs`; peek = per-group resizable `egui::Window`, session-local
  `PanelPeek` (never saved), X/Escape/outside/rail-rect lifecycle; ALL UI-level ids dispatch
  through `menus::invoke` (bc1a67c3 — the E2E-caught dispatch bug); scoped deviation: the
  customized-docking column toggles instantly (reveal/dockWidth consumption ordering, see
  the REV4 P0-2 note).

### Decision (D1–D9)

- **D1 — base structure: this branch.** Keep `show_collapsible` + `panel_rail.rs` + peek +
  `menus::invoke` routing. REJECT the hand-rolled width tween (exact_size glide writes
  garbage into `PanelState` mid-flight — the P0-1 class; `dock_last_width_id` patches the
  symptom, not the class) and the inline flyout. The customized column keeps its instant
  toggle this round (scoped REV4 deviation stands; their early-return-before-reveal trick
  is noted for the follow-up that moves reveal consumption into the draw closure).
- **D2 — ADOPT the `dock_expanded` split.** The fold becomes its own persisted preference,
  independent of Tab/⇧Tab (`expanded = (p.dock && p.dock_expanded) || tabbed_in`); the top
  rail button and gesture write-back dispatch `window.toggle.dockExpanded` when the fold
  flips; View>Dock keeps `p.dock`; the fold survives Tab round-trips and menu checkmarks
  stay honest. Their deserialization + fold-state tests port as-is.
- **D3 — ADOPT their rail visuals**: 44/52 sizes, `t.canvas` fill, `outer_margin(4)`,
  chevrons-right/left top toggle + separator + "Collapse/Expand Panel Dock" labels
  (dispatching `window.toggle.dockExpanded`, clearing any open peek).
- **D4 — ADOPT the toolbar width tween** + its test timing changes.
- **D5 — ADOPT canvas/tab_strip/theme/prefs/widgets + panel_docking VISUAL polish**
  (`t.canvas` fills, radius_sm, dropped separators); NOT their transition-phase/exact_size
  machinery (superseded by D1).
- **D6 — tests**: port their two dock_tests additions (adapted: fold assertions read
  `app.ui.panels.dock_expanded`; the folded-rail flyout test asserts `app.ui.panel_peek`,
  not their ctx temp id) + toolbar timing; keep panel_rail_tests.
- **D7 — docs**: adopt their kb/SKILL/ui-design.md, amended to the reconciled mechanism.
- **D8 — i18n**: add the two new keys to every catalog (their commit shipped none).
- **D9 — shell_commands**: keep their `window.toggle.dockExpanded` registration; implement
  the arm in the `menus::invoke` match (not their early-return special case) + `checked()`.

Not addressed (pre-existing in BOTH implementations): a rail icon click while the column is
⇧Tab-hidden (`p.dock=false`) reveals a group nobody can see — the reveal path predates the
rail; left for a follow-up with its own tests.

Merge mechanics: `git merge c8105921` into `feature/right-panel-collapse`; conflicts resolved
per D1–D9 (panels.rs/panel_docking.rs structurally mine + their visual hunks; state.rs,
shell_commands.rs, canvas.rs, tab_strip.rs, theme.rs, prefs_ui.rs, widgets.rs, docs mostly
theirs); the merge commit message records the reconciliation. After build: full browser E2E
re-run (toggle → flyout → re-click → focus-loss → expand) against the NEW command.

---

## REV5 — implementation outcome (2026-10-24, session 5)

Merged per D1–D9 with the audit's four P0s and both P1 rulings applied (separator
repositioned ±10 under the outer margin; chevrons RTL-swapped like the toolbar's).
Branch tip 55bc9d00. Every audit E2E item ran green on the baked release bundle except
the full-screen/RTL/Studio-classic visual matrix (deferred to the kittest/ codespace
window; the reconciled code paths are shared with what E2E did cover). The
localStorage `panelLayout.panels` probe made the dock/dock_expanded state machine
directly readable from the browser — the decisive evidence for the D2 semantic.
