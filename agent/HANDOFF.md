# HANDOFF — next-session scope (rewritten 2026-10-26, session 7 = REV7 + oracle round)

New sandbox, new session: bootstrap FIRST, then the scope below.
Durable truth = `agent-meta` branch on github + /home/sync copies + this file.
Feature branch: `feature/right-panel-collapse` (tip f1bfc593) — session-7 state: the
mockup's SECOND revision (bezel-free, 12px radius ladder, one-surface right side,
shade-separated dark workspace, floating status pill) is fully implemented and
**PROVEN pixel-exact by the new A/B oracle** (tools/oracle/run.py — 0/48 failing;
VLM 88/100, remaining = content). egui is now 0.37 (upstream merge cb8634b06 — the
owner's "still the same" run had been upstream-only, without our branch).
Design: agent/design/2026-10-26-rev7-bezel-free-and-oracle.md (2 design audit
rounds + 1 implementation audit round + the oracle-driven fix loop, all recorded).
Build served: bundle d2b83eef (storybook serves the same hash).

## THE ORACLE (this round's infra investment — use it for every visual change)

One command: `python3 tools/oracle/run.py` (from the photocraft checkout; needs the
hub dev server on :3000 + the mockup vite on :8081). It renders the mockup's
`/oracle` route and the app's `?oracle=1` boot (oracle_seed: toolbar expanded,
floating windows closed, centered 45% doc, no scrollbars), then compares 4 layers
(L1 token probes / L2 structural boundaries + no-solid-fills-in-gaps / L3 corner
areas / L4 masked ΔE per region) and writes download/oracle/oracle-report.json +
side-by-side + heatmap. PASS rule enforced; failing checks name their region.
**Law: tokens are MEASURED from the browser (agent-browser canvas probes), never
computed — matrix converters drift (R1-0). The mockup repo (modern-canvas-studio,
main @ 4dfc88b) carries the /oracle route + `oracle` prop = the mirror list.**

## Session-8 immediate scope (priority order)

1. **Owner reaction round** — the hub serves REV7 at /, /photocraft, /storybook.
   Likely asks: island shadow strength (a dial in paint_island_rect), options-bar
   density (content), title-bar content (menus vs search — content), the floating
   Properties deviation (oracle seed closes it; decide dock-vs-float for real users).
2. **Oracle states 2-4**: run.py currently compares the default state only — extend
   to leftfolded / rightfolded / flyout (the spec's states table exists; the actions
   are clicks at rail/toolbar toggles — see e2e-final.png probes for coordinates).
3. **Native test RUN** (still parked): the full suite compiles; token/migration/
   toggle tests were re-verified by compile + the oracle's runtime evidence, but
   `cargo test` was never executed locally (RAM). Stop the dev server first.
4. **Element-anchored PINs Route B** (~350 lines, kb/pin-on-webgpu.md): awaiting go.
5. **Radius ladder stragglers** (P2s from the impl audit, low priority): the ~15
   extra `t.radius*` sites in notices/type_panels/channels/move_distance/
   brush_sections/doc_props/filter_dialog/camera_raw/filter_preview.

## Known deviations from the mockup (deliberate, stated)

- Title bar carries the app's real menus + brand (mockup: search island + Ps tile);
  the strip itself is transparent in both — geometry matches, content differs.
- Canvas scrollbars now appear only when an axis actually scrolls (mockup: never).
- The dock's card content is the app's real panels (mockup: fake color presets).
- The oracle masks content (icons/text/images); chrome is compared exactly.
