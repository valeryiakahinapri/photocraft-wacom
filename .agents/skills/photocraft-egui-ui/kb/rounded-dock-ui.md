# Rounded dock UI implementation notes

These details are specific to PhotoCraft's current egui shell and complement
[`docs/ui-design.md`](../../../../docs/ui-design.md). Updated by the REV5
reconciliation (feature/right-panel-collapse merge of c8105921): the fold,
flyout and animation facts below describe the merged implementation.

## State and rendering paths

- `Panels.dock` hides or shows the entire right dock (Photoshop Shift+Tab). Do not reuse it for the
  dock's content fold: `Panels.dock_expanded` (serde default yes) is the fold, its own persisted
  preference, so Tab round-trips restore visibility without ever unfolding it. The icon rail
  remains visible in every one of those states while `Panels.rail` allows.
- The standard panel stack is drawn by `panels::dock_panels`; workspace-customized panel layouts are
  drawn separately by `panel_docking::show`. Both gates consult `column_shown = (dock &&
  dock_expanded) || tabbed_in` in `panels::right_dock`; a fix made in only one path creates
  workspace-dependent behavior.
- Rail peeks (folded flyouts) call `panels::dock_body` for content, use the group's existing
  `dock_tabs`, and update the selected tab in place. The temporary peek lives in
  `app.ui.panel_peek` (`PanelPeek`, deliberately NOT serde — never saved), so it survives frame
  boundaries but never a session.
- `dock_width_id()` is the transient resize request (`ui.set {dockWidth}`), consumed inside
  `dock_panels` / `panel_docking::show`. A request landing while the column is hidden or folded is
  consumed-and-dropped — accepted debt (automation racing a 0.2s animation is pathological).

## Surface and animation details

- Rounded shell edges expose the surrounding panel fill. Match the shell and the intended workspace
  gutter to `t.canvas`; use `t.card` for the contents of individual panel cards. `t.chrome` can be a
  different shade and should not be substituted casually.
- Animate the fold with egui's own `Panel::show_collapsible` (cubic-out ~0.2s, honors the
  Enable-Animations pref): it lays content out at full width, clips to the animated slice, and —
  the load-bearing part — SKIPS `PanelState` stores while animating, so a quit mid-glide can never
  persist a garbage dock width. Hand-rolled `exact_size` glides do not have that guard and are the
  rejected alternative (REV5 audit). Gesture collapses write back through commands
  (`window.toggle.dockExpanded`, or `window.togglePanels` when the #6760 tabbed-in chrome was the
  shower). The customized-docking column still toggles instantly this round: `panel_docking::show`
  consumes `docking_reveal` and the width request in its prologue, so wrapping it in
  `show_collapsible` needs that state moved into the draw closure first.
- Keep Classic's bevel and square-corner style. For other themes, derive small radii from tokens,
  capped to the visual reference's compact 4–5 px shell radius.
- The toolbar's single/double-column width tween and the right dock's fold tween are separate UI
  states. The rail's flyout is anchored to the clicked icon and must remain inside the viewport,
  including in RTL layouts.
- The active document tab's fill is `t.canvas`; inactive tabs remain on the strip/surface shade.
  Pro and Studio document strips are separate code paths and must be checked independently.

## Verification

1. Unit-test fold state independently from whole-dock visibility and deserialize older UI state.
2. Exercise rail open, same-icon close, Escape/outside dismissal, expansion, and content-tab choice.
3. Render the standard and customizable docks in Pro and Studio, folded and expanded, plus Classic
   and RTL. Verify rounded edges reveal a consistent canvas shade.
4. Capture settled offscreen snapshots with `crates/ui-egui/examples/snapshot.rs`; use live
   screenshots for interactions that depend on pointer-driven animation.
