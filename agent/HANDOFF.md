# HANDOFF — next-session scope (written 2026-10-24, session 2)

New sandbox, new session: bootstrap FIRST (the recipe is proven — ~5 min to a warm build lane),
then the scope below. Durable truth = `agent-meta` branch on github + /home/sync copies.
Feature branch: `feature/right-panel-collapse` (tip b6725404) — 6 commits: WIP snapshot +
extraction + plumbing + popover + labels + peek + kittests.

## Bootstrap (≈2 min) + build-lane restore (≈5 min if /home/z was recycled)

1. `git clone https://<work PAT>@github.com/valeryiakahinapri/photocraft-wacom.git /home/z/photocraft-wacom`
   (PAT in agent/secrets.env on agent-meta, base64; also kit .env)
2. `git clone https://<kit PAT>@github.com/xfnwfpho1/zai-codespaces.git /home/z/zai-codespaces`
3. `cd /home/z/photocraft-wacom && git fetch && git switch agent-meta && bash agent/scripts/zai-bootstrap.sh`
4. Codespace: `ZAI_ROOT=/home/z/zai-codespaces CS_ACCOUNT=photocraft bash kit/cs ensure`
5. If /home/z survived (check `ls /home/z/pc-build/target`): nothing to rebuild. If recycled:
   the toolchain tarball is on /home/sync/photocraft-agent/toolchain/ (extract with
   scripts/zstd-extract.py to /home/z, i.e. .cargo/.rustup/.cache) and the CACHE tarball
   (agent/kb/local-build-fanout.md recipe) must be re-pulled from the codespace — check the
   live agent's current tree state first (`pgrep -af cargo`); the cache is only compatible
   with the env overrides in the recipe.
6. Worktrees: `git worktree add /home/z/pc-edit feature/right-panel-collapse` in the clone.

## Immediate scope

1. **Test results**: check /home/z/logs/test-native.status (native kittest build was launched
   in background at session end). If rc=0: PNG fixtures are in /home/z/logs/fixtures —
   review panel-rail-{icons,popover,labels,peek,peek-history}.png, then run the remaining
   gates (all local, warm now): `cargo check --target wasm32-unknown-unknown` (L0-L6),
   clippy, `cargo xtask layers`, i18n report; fix anything; push.
2. **If tests failed**: the log tail is in /home/z/logs/test-native.log; iterate via
   `cargo test -p photocraft-ui-egui --locked --offline panel_rail` (incremental now) — NOTE:
   ui-egui has no `heif` feature. PNG fixtures regenerate with
   PHOTOCRAFT_PANEL_RAIL_FIXTURES=/home/z/logs/fixtures.
3. **panel_body dedupe** (deferred commit F): extract the scrolls_itself/ScrollArea wrapper
   shared by dock.rs show(), panel_docking::body() and the peek window into one
   panel_rail::panel_body — OWN commit; it FIXES #2726 for customized docks (dock.rs sets
   fade.strength=0, panel_docking lacks it) — add scroll assertions.
4. **Codespace housekeeping** (read-only first): rm /tmp/pull/* (658M+ parts from the cache
   pull) once the transfer is surely not needed again; if the live agent has committed its
   WIP (git log origin/main), rebase feature/right-panel-collapse onto plain origin/main and
   re-run the kittests.
5. **GitLab mirror**: still blocked — the PAT from session 1 is REVOKED; ask the user for a
   fresh token before wiring the second remote.

## Session-2 experience notes (what to not re-learn)

- The browser E2E harness LIES: agent-browser mouse events never trigger egui release-clicks
  (press-gesture menus work; Popup::menu/checkbox items don't); synthetic PointerEvents are
  ignored (untrusted); accesskit DOM empty; VLM "confirms" whatever the prompt describes —
  md5/pixel-diff screenshots before believing anything. kittest + PNG fixtures = the truth
  (kb/web-e2e-tooling.md).
- `trunk` CLI breaks if NO_COLOR=1 (wants true/false). python http.server subclassing of
  functools.partial crashes — subclass the handler class directly.
- MultiEdit on long markdown blocks with arrows/dashes can fail to match invisibly — use a
  python splice for big doc edits.
- Never commit the live agent's workspace Cargo.toml (lto=false) — build parity via
  CARGO_PROFILE_WASM_RELEASE_* env vars (verified identical fingerprints).
- The bridge (`window.PhotoCraft`, {id, params} / {steps} envelope) drives an INDEPENDENT
  headless session, NOT the visible UI.
