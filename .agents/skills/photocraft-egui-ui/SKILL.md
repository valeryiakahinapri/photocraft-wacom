# PhotoCraft egui UI work

Use this guidance for UI layout, shell, dock, and visual parity changes in PhotoCraft. The desktop
and browser UI share the Rust egui implementation; do not create a separate web UI.

## PhotoCraft-specific gotchas

- Read `docs/ui-design.md` for the tokens and interaction conventions. Theme surfaces are distinct:
  use `t.canvas` for shell/gutter space revealed around rounded chrome, `t.card` for panel bodies,
  and the relevant tokens for controls. A rounded shell with a mismatched fill is visibly wrong.
- `Panels.dock` is the whole dock's visibility state used by Shift+Tab. Folding the dock must use
  separate state so the persistent rail remains available and Shift+Tab behavior does not change.
- The legacy `dock_panels` and customizable `panel_docking` paths are both live. Keep fold width,
  background, resize handling, and remembered width consistent in both. Custom docking has pane
  minimums; do not let them force content outside the dock while its shell animates.
- Reuse `dock_body` for dock and rail flyouts so panel behavior and selected tabs stay consistent.
  Keep transient flyout selection in egui context data, not serialized workspace state.
- Document tabs are painted in `canvas.rs`; the active document uses the canvas shade and inactive
  tabs use a surface shade. Pro tabs and the Studio strip have separate render paths.
- For visual verification, use `crates/ui-egui/examples/snapshot.rs` for deterministic offscreen
  PNGs, and the control protocol for live desktop/browser screenshots. Test both the legacy and
  customizable dock paths, Classic's square bevels, RTL placement, and persisted older UI state.

## On-demand reference

- [Rounded dock implementation notes](./kb/rounded-dock-ui.md): state separation, dual dock paths,
  width tweening, flyout interactions, and visual comparison checklist.
