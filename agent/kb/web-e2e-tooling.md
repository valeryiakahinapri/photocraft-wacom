# Web E2E tooling for the egui web build (what works, what lies)

Written 2026-10-24 (session 2), validating the panel-rail feature in the browser. Verified
first-hand on this sandbox; applies to any egui/winit-web target (photocraft-web).

## Serving

`trunk build` (dev) then serve `dist/web` with a double-forked python http.server on :8765
(`/home/z/tools/serve_dist.py`). Preview URL externally = `?XTransformPort=8765`; locally the
agent-browser CLI hits `http://localhost:8765` directly. Subclassing `functools.partial` for
the request handler crashes the daemon — subclass `SimpleHTTPRequestHandler` directly, and
set `allow_reuse_address` (see the script).

## Headless Chromium + egui web: pointer release-clicks are unreliable

- Presses, moves and hover DO reach egui; menus that open on a PRESS gesture work (the app's
  title-bar menus have their own press-gesture machinery in menus.rs).
- Release-driven `Response::clicked()` was NEVER observed firing from `agent-browser mouse
  down/up` in this harness: plain `Popup::menu` popovers, dock panel menus and File-menu ITEMS
  all failed to open/fire, while press-opened menus rendered fine. Clicks on egui buttons,
  including our rail, cannot be trusted via `mouse down`+`mouse up`.
- `dispatchEvent(new PointerEvent(...))` via `eval` = ignored (winit checks trusted events).
- The egui/accesskit DOM tree stays empty unless a screen reader attaches — `snapshot -i`
  shows only `Canvas` + a textbox; no refs/roles to click semantically.
- VLM screenshot reads are PROMPT-SENSITIVE: it will confidently "read" the popover rows you
  DESCRIBED in the prompt from a baseline-identical image. Always (a) `md5`/pixel-diff
  screenshots against a known baseline before believing any "change", (b) use NEUTRAL prompts
  ("is there a popup? list what you see" — never feed expected contents), (c) prefer objective
  pixel-diff over VLM for state assertions.

## The reliable validation stack (in order)

1. **egui_kittest tests** (`Harness::builder().build_ui_state`) — real app, synthetic events
   INSIDE egui (no browser layer). `h.get_by_label(...).click()` actually triggers `clicked()`;
   `query_by_role_and_label(Role::Button, ...)` for presence/absence. This is the repo's own
   gate and the ONLY trustworthy way to prove UI behavior headlessly.
2. **PNG fixtures from the same harness**: `PHOTOCRAFT_PANEL_RAIL_FIXTURES=dir` style env
   gating (pattern from panel_docking_tests `CRAFT_UI_DOCKING_FIXTURES`) + `h.render()`
   → save = visual evidence WITHOUT the browser.
3. **Browser only for eyeball-level smoke**: the wasm boots, panels render, no console errors
   (`agent-browser errors`). Use neutral-prompt VLM on diff-verified screenshots.
4. The **js bridge** (`window.PhotoCraft`, the live agent's WIP) exposes
   `commands({filter})`, `execute({id, params})`, `run({steps})`, `create`, `info`, `export` —
   ONE-OBJECT envelope args. It drives an INDEPENDENT headless session, NOT the visible egui
   UI state — good for engine/command automation tests, useless for visible-UI E2E.

## Cost note

A cold NATIVE build for kittest (ui-egui test graph) is the expensive one (~30-60 min on this
2-vCPU box; the wasm cache does not transfer). Plan test runs so the first one runs in the
background while doing docs; subsequent runs are incremental. `cargo test -p
photocraft-ui-egui` needs NO `--features heif` (ui-egui has no such feature).
