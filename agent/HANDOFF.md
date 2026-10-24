# HANDOFF — next-session scope (rewritten 2026-10-24, session 5)

New sandbox, new session: bootstrap FIRST, then the scope below.
Durable truth = `agent-meta` branch on github + /home/sync copies + this file.
Feature branch: `feature/right-panel-collapse` (tip 55bc9d00) — session-5 state: merged
with main's c8105921 (REV5 reconciliation of the TWO parallel rail implementations;
see agent/design/2026-10-24-panel-rail-collapse.md REV5 + the 3-rev5-review audit in
the worklog), rail visuals + dock_expanded fold semantics adopted from the live agent,
all E2E re-proven, native test-compile now verified.

## Parallel lane status — the GenEditor embed (session 4, separate scope)

The embed lane (cloud library + PHASE-B doc↔asset identity) SHIPPED and lives
ELSEWHERE — do not conflate with the panel/PIN scope above: wacom
`web-cloud-library-rebase` @ b3b2e5b (rebased onto main 7d6f9df9; the engine bridge
and the embed host_bridge coexist — see kb/browser-js-bridge.md), GenEditor-ui
`feature/photocraft-embed` @ e925ec2, webhost main @ afed653 (its HANDOFF.md = the
embed lane's own session-8 start point, incl. the warm-cache hardlink build recipe
that rides YOUR finished target dirs — kb/wasm-and-targets.md session-4 additions).
Coordination rule unchanged: read-only on each other's trees, git is the shared bus.

## Bootstrap (≈2 min) + build-lane restore (≈5 min if /home/z was recycled)

1. `bash /home/sync/photocraft-agent/agent/scripts/zai-bootstrap.sh` (clones both repos,
   patches the kit; ~1-2 min from a fresh container)
2. If /home/z recycled: `RESTORE_FORCE_TARGET` NOT needed — run
   `python3 agent/scripts/gnu-extract-cache.py` (the ONLY correct extractor: zst → plain
   tar → GNU tar, sentinel-verified) then `python3 agent/scripts/fix-toolchain-mtimes.py`
   then re-run gnu-extract (it also does the fingerprint-only remap + timestamp touch).
   NEVER use python tarfile filter='data' for these trees (mtime poison — kb).
   After restore: `python3 agent/scripts/noop-build.py` should be near-noop (<2s, 0-2
   crates). If it mass-rebuilds, check the kb `review-hub-and-pins.md` "build-lane
   repairs" section first — every failure mode seen so far is documented there.
3. Codespace: `kit/cs ensure` (it may be stopped; disk was 97% full — the live agent's
   new round is consuming; stay read-only, coordinate via git).
4. Worktrees: `git worktree add /home/z/pc-edit feature/right-panel-collapse` in the clone.
5. Review hub (if /home/z/my-project recycled to the platform template):
   `bash agent/scripts/bake-review-hub.sh` regenerates /photocraft + /storybook +
   the landing page from this branch's agent/review-hub/ mirrors.

## Immediate scope

1. **Kittest EXECUTION is the last open gate on the panel feature.** Compilation is now
   PROVEN (session-5 native `cargo check -p photocraft-ui-egui --tests` rc=0 — after
   fixing E0425: `egui_kittest::kittest::Query` never existed; the helper returns
   `egui_kittest::Node<'a>`). Running needs either an idle codespace window (preferred;
   4-core) or a local window with the dev server STOPPED and CARGO_INCREMENTAL=0 (rustc
   on the ui-egui test unit OOM-kills at 3.9G with the server up).
2. **Route B — element-anchored pins on the wgpu canvas** (plan in
   `agent/kb/pin-webgpu-report.md`): `PhotoCraft.ui({op:"rects"})` + testid overlay +
   pin-mode pointer-events, ~300-450 lines. Region pins still work on the merged build.
3. **Reviewer loop**: hub → /photocraft (merged build, 30.7MB release-mode bake) →
   /storybook story route. If the reviewer reports the wasm as slow to load, remember
   the bake must come from `tools/trunk-bundle.py` (--release, wasm-opt strips the 22MB
   name section) — a raw `trunk build` is 57MB.
4. **Next merge with the live agent**: expect more main-side changes; the reconciliation
   pattern is now established (REV5 in the design doc + the audit checklist). If they
   touched panels.rs/panel_rail.rs again, re-run the lockstep checklist (column_shown
   gates, 3-way write-back, peek guard, menus arm gating).
5. **GitLab mirror**: still needs a fresh token (the old one is revoked).

## This session's load-bearing lessons (kb entries have the detail)

- python tarfile filter='data' mtime poison → GNU tar pipeline everywhere.
- UI-level commands = `menus::invoke`, NEVER `app.run` (the engine only catalogs them).
- The in-canvas mount pattern (web.rs boots by element id — one DOM with annotakit).
- Sub-agents run builds unless told not to; research prompts must forbid it.
- Trunk color-env vars (NO_COLOR/CLICOLOR_FORCE) abort rc=2 in 0.1s.
- Background daemons: the python DOUBLE-FORK is the only pattern that survives bash
  toolcall boundaries (setsid+nohup+disown dies; verified again this session).
- bun.lock integrity: a re-packed local tarball dep fails its recorded sha512 — strip
  the hash from the lock entry rather than fighting bun.
- Trunk dev vs --release = 22MB name section difference; review bakes use --release.
- localStorage `panelLayout.panels` is the browser-visible state probe for E2E.
- Native test checks need the dev server stopped (OOM) — restart via init-fullstack.sh.
