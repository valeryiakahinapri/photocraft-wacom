# HANDOFF — next-session scope (rewritten 2026-10-24, session 3)

New sandbox, new session: bootstrap FIRST, then the scope below.
Durable truth = `agent-meta` branch on github + /home/sync copies + this file.
Feature branch: `feature/right-panel-collapse` (tip bc1a67c3) — merged with main's
bridge commit + 4 session-3 commits (mockup refinement F/G, menus::invoke dispatch fix).

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

1. **Panel feature — E2E-VERIFIED LIVE, kittests still pending.** Session 3 proved the
   feature in a real browser (bake → CDP clicks → pixel diffs: toggle collapses the
   dock, icon opens the peek flyout, re-click/outside-click close it — screenshots in
   /home/z/logs/e2e2-*.png, mirrored to /home/sync/photocraft-agent/e2e-evidence/).
   The kittests remain COMPILE-UNVERIFIED for the session-3 rewrite (they route through
   `menus::invoke` now — the dispatch bug they encoded is fixed) and never EXECUTED:
   native graphs don't fit the 9.9G local disk; the codespace SIGTERMs big rustc while
   the live agent occupies it. Same retry options as before (idle-agent rerun / swapfile
   with user OK / post-handover warm-target run).
2. **Route B — element-anchored pins on the wgpu canvas** (the designed follow-up,
   full plan in `agent/kb/pin-webgpu-report.md`): `PhotoCraft.ui({op:"rects"})` bridge
   command + testid overlay + pin-mode pointer-events. ~300-450 lines. Region pins
   already work (E2E-verified thread #1); element pins are the upgrade.
3. **The reviewer loop**: the owner tries <preview>/ (hub) → /storybook (story route!) →
   region-pins the panel work. GitHub mirror needs a fine-grained Issues-scoped PAT
   from the owner (never the full work PAT — the baked gh.json is world-readable).
4. **GitLab mirror**: still needs a fresh token (the old one is revoked).

## This session's load-bearing lessons (kb/review-hub-and-pins.md has the detail)

- python tarfile filter='data' mtime poison → GNU tar pipeline everywhere.
- UI-level commands = `menus::invoke`, NEVER `app.run` (the engine only catalogs them).
- The in-canvas mount pattern (web.rs boots by element id — one DOM with annotakit).
- Sub-agents run builds unless told not to; research prompts must forbid it.
- Trunk color-env vars (NO_COLOR/CLICOLOR_FORCE) abort rc=2 in 0.1s.
