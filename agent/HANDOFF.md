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

1. **Kittests — BLOCKED, one experiment left (see worklog "the SIGTERM wall")**: four detached
   codespace runs all ended with rustc SIGTERMed ~8 minutes into any single big-crate compile
   (engine, ui-egui) — the codespace's memory-pressure watchdog + the live agent's ~1.5-2G of
   VS Code/copilot overhead on the 8G box. Small crates compile fine; wasm + check gates pass.
   Options (in order): (a) after the live agent is idle/removed, re-run — its overhead gone
   may drop the peak below the wall; (b) ask the user about adding a 4G swapfile in codespace
   /tmp (root, 38G free — helps the live agent too, but it IS a system-level write to its box);
   (c) after codespace handover, run in ITS warm target/debug (fast). The launcher
   (/tmp/run-tests.sh, detached pattern, incremental back ON) and warm /tmp/test-target
   (~5G, engine+ui-egui remain) are already in place — just re-launch and poll /tmp/test-run.rc.
   The three tests themselves: COMPILE-VERIFIED up to ui-egui (E0063/E0596 fixed, pushed); they
   have never EXECUTED. Everything else is green: wasm build, cargo check --target wasm32
   (L0-L6, rc=0, ~5 min local), rustfmt clean, i18n rows in 21 catalogs.
   The kittests were re-launched on the CODESPACE (the right home for
   them — a native test graph does NOT fit on the local 9.9G disk, see worklog) in the
   isolated worktree /tmp/panel-rail-test + CARGO_TARGET_DIR=/tmp/test-target:
   check /home/z/logs/test-codespace.{status,log} (status OK + `TEST_RC=0` marker = pass).
   - When done: `ZAI_ROOT=/home/z/zai-codespaces CS_ACCOUNT=photocraft bash kit/cs getdir
     /tmp/fixtures /home/z/logs/fixtures` for the PNG snapshots (panel-rail-{icons,popover,
     labels,peek,peek-history}.png), review them, then clean up the codespace:
     `git worktree remove --force /tmp/panel-rail-test && rm -rf /tmp/test-target /tmp/fixtures`.
   - If failed: re-run in the same worktree (incremental now): cs exec with
     `export PATH="/home/codespace/.cargo/bin:$PATH"` FIRST (remote bash is NOT login —
     cargo is not on PATH; this bit me) and NO `| tail` after cargo (it masks the rc via
     set -e; that bit me too). Local iteration is cargo BUILD only (fast, cached);
     `cargo check --target wasm32 --profile wasm-release` runs locally in ~10 min
     (check-wasm daemon result in /home/z/logs/check-wasm.{status,log}).
2. **Gates still owed after tests pass**: clippy (locally wasm-target or codespace worktree),
   `cargo xtask layers` + i18n-coverage (codespace — xtask is native), final review of
   panel_body dedupe (deferred commit F; see design REV3).
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
