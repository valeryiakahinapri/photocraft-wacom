# Rounded dock UI implementation notes

These details are specific to PhotoCraft's current egui shell and complement
[`docs/ui-design.md`](../../../../docs/ui-design.md).

## State and rendering paths

- `Panels.dock` hides or shows the entire right dock (Photoshop Shift+Tab). Do not reuse it for the
  dock's content fold: the icon rail must remain visible while folded.
- The standard panel stack is drawn by `panels::dock_panels`; workspace-customized panel layouts are
  drawn separately by `panel_docking::show`. Both paths use the same animated width helper and
  remembered user width. A fix made in only one path creates workspace-dependent behavior.
- Rail flyouts should call `panels::dock_body` for content, use the group's existing `dock_tabs`,
  and update the selected tab in place. The temporary flyout group belongs in egui context data,
  not in persisted UI/workspace state.
- `dock_width_id()` is the transient resize request consumed at the outer `right_dock` boundary.
  Pass it through to whichever dock renderer is active before it is consumed.

## Surface and animation details

- Rounded shell edges expose the surrounding panel fill. Match the shell and the intended workspace
  gutter to `t.canvas`; use `t.card` for the contents of individual panel cards. `t.chrome` can be a
  different shade and should not be substituted casually.
- Animate the dock's width without overwriting its last user-resized width. Temporarily disable
  resizing during the tween. Custom docking needs a narrow transition phase that avoids its normal
  pane minimums forcing the content wider than the shell.
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
