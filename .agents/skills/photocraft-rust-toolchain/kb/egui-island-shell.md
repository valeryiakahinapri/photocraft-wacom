# egui 0.36 island shell — the verified mechanics (REV6, session 6)

Everything below was verified against egui 0.36.2 source (in the cargo registry)
and proven by pixel-matrix E2E. Read BEFORE touching the shell layout or the
right-side island.

## The strip/island pattern (the only correct shape in this egui)

- `Frame::paint` fills **content + inner_margin (+ stroke) ONLY** — `outer_margin`
  paints NOTHING and the root `App::ui` has no background. So a chrome strip is:
  `Panel.frame(Frame::NONE.fill(t.page).inner_margin(gaps))` — the page-colored
  gap bands ARE the frame fill — and the body's first act paints the island
  (shadow → fill → hairline) at `ui.max_rect()` (panels.rs `paint_island*`).
- `exact_size`/`default_size` are OUTER (incl. inner margins): add the margin sums
  (title +12, options +8, status +12 under the 4/8 scheme; dock default 328 outer
  = 316 content; rail `rw + 4 + edge`).
- Two fills are impossible in one Frame → content padding inside the island is a
  transparent `Frame::NONE.inner_margin(pad)` wrapper, or the body-closure pattern
  (`let body = |ui| …; if islands { Frame.show(ui, body) } else { body(ui) }`).
  `Frame::show` returns `InnerResponse<R>` — pass `.show(ui, content).inner` when
  the body's value is consumed (panel_docking's `draw`).
- Island shadow = `egui::Shadow { offset, blur, spread, color }.as_shape(rect,
  radius)` then `rect_filled` + `rect_stroke(Inside)` — `RectShape` has NO shadow
  field in 0.36 (only `blur_width`); `Shadow::as_shape` is the supported path.

## The right side = ONE island, painted as TWO SEGMENTS (never layers)

- egui 0.36 has NO `Order::PanelResizeLine`, NO `Ui::with_layer_id`, and a raw
  non-Area layer is INVISIBLE TO HIT-TESTING (`hit_test` walks `areas().order()`
  only) — content moved to such a layer goes completely dead. Do not revisit.
- The verified scheme: the COLUMN panel (claimed FIRST = far right) paints its
  island segment = content rect expanded 4px over its seam-side margin, with the
  screen-edge corners rounded (CornerRadius nw/se pairs per RTL); the RAIL panel
  (claimed SECOND = seam side) paints its segment = its content rect, rounded on
  the canvas side, seam-side rounding animated 0→r; identical `t.card` fills make
  the seam invisible; a 1px hairline at the seam = the mockup's `border-r`.
- The rail's screen-edge margin animates 0↔8 IN LOCKSTEP with the column's slide
  by reading egui's OWN animation value: `ctx.animate_bool_responsive(
  Id::new("dock").with("animation"), column_shown)` — the exact id
  `show_collapsible` uses (`panel_id.with("animation")`, panel.rs:25-27). Calling
  it twice per frame is idempotent. "shared-panel-dock" for the customized path.
- `.show_separator_line(false)` on the column panel: egui's separator is painted
  then BURIED by later panels AND adds +1px to the resize-side outer margin.
- The slide clip does the width animation for free: `show_collapsible` lays the
  body at full width and clips to the animated slice → the island segment shrinks
  with it; at fraction 0 the panel claims nothing and the rail becomes a complete
  4-corner island (verified pixel-matrix in e2e-r6).
- Panel claim order for RIGHT panels: first `.show()` = rightmost. Column first →
  column against the screen edge, rail to its left = the mockup's order. (The
  pre-REV6 !pro order was rail-first = the mirror; the swap changed the rail's
  resting x — rail position asserts in tests needed revisiting.)

## Rounded gradient / rounded texture fills

`RectShape.brush` (epaint::Brush = texture × fill color, uv can exceed 1 = tiling)
is the ONLY native way to get a rounded rect with a non-solid fill:
- the active tool's gradient: a 1×32 vertical `ColorImage` gradient loaded per
  frame (256B upload, negligible) multiplied into `RectShape::filled(WHITE)`;
- the workspace dot grid under islands: the tiled dots texture as a brush with
  the workspace's corner radius (a plain `painter().image` cannot round).
`ColorImage` in 0.36 has a `source_size: Vec2` field — use `ColorImage::new`, not
the struct literal.

## E2E tooling truths (session 6)

- agent-browser profiles PERSIST on disk across `close`; a named `--session` is a
  fresh profile. Seeding localStorage on a LIVE page races the app's beforeunload
  / change-time saves (prefs_ui::write_prefs 3-way merge): seed → fresh session,
  or expect your seed to be overwritten by the old page's in-memory state.
- The app boots with a DEFAULT prefs file already written by the time you can
  eval; read `interface.darkTheme` + `interface.modernMigrated` to know which
  path ran (default vs migrated vs explicit pick).
- VLMs reliably MISJUDGE fine dark-on-dark geometry (8px page gaps, 5px radii)
  and mix up panel content when comparing app-vs-mockup states: they called a
  pixel-verified island layout "no islands, 25/100". Ground truth = PIL pixel
  probes + corner matrices (scripts/probe_pixels.py + the scan one-liners in the
  worklog session-6 entry). Use VLM only for content identity ("what does this
  window say") and coarse state checks.
