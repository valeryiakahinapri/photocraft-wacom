# HANDOFF — next-session scope (rewritten 2026-10-24, session 6 = REV6 visual round)

New sandbox, new session: bootstrap FIRST, then the scope below.
Durable truth = `agent-meta` branch on github + /home/sync copies + this file.
Feature branch: `feature/right-panel-collapse` (tip 00c8a0b9) — session-6 state: the
modern-canvas-studio mockup is now THE look — `ModernCanvas` theme (exact mockup
tokens, island shell, segment-scheme right side, gradient active tool, doc-tab flip
+ status dot, Segmented pills, divide-y dock) is the app default, web boot no longer
pins Pro (that pin was THE root cause of the "far off" look), saved proMedium
migrates once via interface.modernMigrated. Design: agent/design/
2026-10-24-modern-canvas-visual-adoption.md (REV6 + REV6-final, two opus audits).
E2E: pixel-proven islands + fold/peek/expand/⇧Tab lifecycle + migration, console
clean (evidence: /home/sync/e2e-r6/ + worklog session-4... entry "rev6-round").

## Session-6 immediate scope (in priority order)

1. **Owner reaction round**: the review hub serves the new look at /, /photocraft,
   /storybook (bundle 94b8b4ce). Expect residual-polish asks — likely candidates
   per the 88/100 VLM gap list: options-bar density (real tool options vs the
   mockup's Move row = content, not styling), island shadow strength (blur 18/
   offset 4 — bump toward the mockup's heavier float if asked), header search pill.
2. **Floating Properties deviation** (Studio grammar floats Properties; the mockup
   keeps it docked as a Segmented card). Decide + implement under `t.islands`:
   dock it in the column (likely 10-20 lines in dock.rs/cards) or keep the float.
   The E2E "card rect" that looked like a fold bug was this window — don't re-chase.
3. **Native test RUN** (parked twice for RAM): migration/token/id tests compile but
   were never executed locally (engine test link OOMs at 3.9G with the dev server
   up). Stop the server, or use the codespace window if it exists again.
4. **Element-anchored PINs Route B** (designed, ~350 lines — kb/pin-on-webgpu.md):
   still awaiting the owner's go.

## Known deviations from the mockup (stated, not bugs)

- Status bar = slim island strip (mockup floats it inside the workspace).
- Left fold = the existing toolbar_double width animation (mockup's 2-col→1-col).
- Brand mark stays monochrome (mockup's gradient "Ps" tile) — one-liner if asked.
- Floating Properties (see scope 2).

## Session-6 hard-won gotchas (do not rediscover)

- egui 0.36: Frame paints content+inner_margin only (outer_margin paints NOTHING);
  strips = `Frame::NONE.fill(page).inner_margin(gaps)` + body paints the island.
  `Order::PanelResizeLine` does NOT exist; `with_layer_id` does NOT exist; raw
  non-Area layers are NOT hit-testable (widgets go dead) — the segment scheme in
  panels.rs/panel_rail.rs is the verified answer; do not "improve" it with layers.
- The rail's edge-margin lockstep reads the SAME animation id egui uses:
  `Id::new("dock").with("animation")` via animate_bool_responsive (idempotent
  per frame; "shared-panel-dock" for the customized path).
- wasm-bindgen/wasm-opt OOM with the dev server up: stop `next dev` for bundles.
- agent-browser profiles PERSIST across `close` (named --session = fresh profile);
  seeding localStorage on a LIVE page races the app's unload/save — seed, then
  use a fresh session, or navigate away first.
- VLMs misread 8px dark-on-dark gaps and zoomed-out islands (called a verified
  pixel-perfect layout "no islands"): trust pixel probes + corner matrices
  (scripts/probe_pixels.py), not VLM verdicts, for fine geometry.
- The app saved prefs with a 3-way merge on every change (prefs_ui::write_prefs);
  a boot-path write (web.rs set_theme) defeats defaults+migrations silently —
  grep the boot path for prefs/theme writes whenever a pref change "doesn't show".

## Parallel lane status — the GenEditor embed (separate scope, unchanged)

wacom `web-cloud-library-rebase` @ b3b2e5b, GenEditor-ui `feature/photocraft-embed`
@ e925ec2, webhost main @ afed653. Read-only on each other's trees; git is the bus.

## Bootstrap (≈2 min) + build-lane restore (≈5 min if /home/z was recycled)

1. `bash /home/sync/photocraft-agent/agent/scripts/zai-bootstrap.sh` (clones both
   repos, patches the kit; ~1-2 min from a fresh container)
2. If /home/z recycled: `python3 agent/scripts/gnu-extract-cache.py` (the ONLY
   correct extractor) then `fix-toolchain-mtimes.py` then re-run gnu-extract.
   NEVER python tarfile filter='data' (mtime poison — kb).
3. Server: `SKIP_INSTALL=true bash /home/z/init-fullstack.sh` (platform way; picks
   up existing project + .zscripts/dev.sh). bun.lock's annotakit tarball path may
   need the re-pack dance (worklog session-5 has the recipe).
4. Build: `sync-build.sh` then `python3 agent/scripts/trunk-bundle.py` (release
   mode — the dev bundle carries a 22MB name section), `bake-photocraft.sh`.
   Budget ~4 min warm; stop the dev server first (see gotchas).
