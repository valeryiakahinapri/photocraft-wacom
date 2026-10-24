---
name: zai-photocraft-workenv
description: "Operate the Z.ai sandbox + GH Codespaces two-plane dev environment for the photocraft-wacom fork: bootstrap after recycle, codespace access via the cs kit (photocraft account), build/test patterns, durability rules. Use when starting any new session on this project or when sandbox/toolchain/codespace access questions arise."
---

# Z.ai ⇄ Codespaces two-plane work environment (photocraft-wacom)

Load-bearing facts for every session. Project build knowledge lives in
`.agents/skills/photocraft-rust-toolchain/` (mirrored from the codespace); this skill is
about the ENVIRONMENT around it.

## Topology (the one diagram that matters)

```
Z.ai sandbox (control plane)                GH Codespace (build plane)
  /home/z/photocraft-wacom   ←—— git (origin) ——→  /workspaces/photocraft-wacom
  /home/z/zai-codespaces (kit) ←— cs exec/put/get —→  cargo / trunk / wasm builds
  /home/z/my-project  = platform dirs ONLY (watchdog!)
  durable = github (agent-meta branch) + /home/sync
```

- **Local compiles TOO (revised 2026-10-24):** the same toolchain (extracted from the backup
  tarball into /home/z/.cargo + .rustup + .cache/trunk) plus the INHERITED wasm target cache
  (codespace tar + sed path remap) make the sandbox a second build lane: true-noop rebuild
  0.3s, ui-egui edit to full wasm 70s, trunk bundle 11.3s. Native kittest cold builds are
  the slow exception (30-60 min once, incremental after). Recipe + all numbers:
  agent/kb/local-build-fanout.md. Fat-LTO release links + wasm-opt + packaging still belong
  to the codespace. Local = control plane + iteration lane.
- **Codespace `miniature-waddle-gx65q95446v4cv4`** (basicLinux32gb, 2c/8G/32G) compiles
  everything. Whole container root survives stop/start; only /tmp is wiped on start;
  a rebuild resets everything except /workspaces — the 569M toolchain backup tarball at
  /workspaces/.photocraft-toolchain-backup covers rebuild recovery (restore script sits
  next to it).
- 12h hard lifetime cap from creation; 30m idle stop. `cs start` (or `ensure`) resurrects;
  state persists. My execs through the gh relay reset the idle timer.

## Access recipe (after any sandbox recycle)

1. Clone both repos (work repo PAT = `ghp_` + token in kit .env on agent-meta; codespaces
   repo PAT as in zai-codespaces docs). Work clone MUST be `/home/z/photocraft-wacom`
   (outside the watchdog path — /home/z/my-project gets force-switched to main ~every 20s
   and any non-main working tree there is reverted).
2. `bash kit/bootstrap-local.sh` with `ZAI_ROOT=/home/z/zai-codespaces` (installs user-space
   gh + openssh, rebuilds gh profiles).
3. If `kit/lib.sh` lacks the `photocraft` account case (check `grep photocraft kit/lib.sh`),
   re-apply the 3rd-account patch — see `agent/scripts/zai-bootstrap.sh` on agent-meta
   (idempotent, does steps 2–3 + verification in one shot).
4. Every kit call: `ZAI_ROOT=/home/z/zai-codespaces CS_ACCOUNT=photocraft bash kit/cs <cmd>`
   — ZAI_ROOT defaults to /home/z/my-project (wrong) and CS_ACCOUNT defaults to primary
   (the zai fleet, different account entirely).
5. Multi-line remote work: `bash kit/cs exec <<'EOF' … EOF` (QUOTED delimiter, always) —
   byte-exact script transport, rc propagates. `CS_EXEC_TIMEOUT=<secs>` raises the 120s
   default for long builds.

## Durability (GIT IS THE DISK)

- Agent docs + plan + worklog + WIP snapshots: `agent-meta` branch of the work repo.
  Mirror codespace-side `.agents/` + `log/devlog.md` there after changing them (they are
  upstream-gitignored and otherwise exist nowhere).
- Belt-and-braces: copy agent/ + .agents/ + worklog.md to /home/sync/ after milestones.
- Push after EVERY micro-step. Never force-push. On divergence, stop and reconcile.
- Local my-project/worklog.md is the session scratch copy; agent-meta is canonical.

## Working with the live agent in the codespace

- A copilot agent may own the codespace working tree (dirty tree = active or abandoned WIP;
  check `pgrep -af copilot` / build processes before touching anything).
- Read-only is always safe: cat/ls/git-status/git-diff — never write, never cargo (build
  lock contention), until the user hands over ownership.
- Snapshot its uncommitted work to `wip/<timestamp>/` on agent-meta (cs get + git diff) —
  safety net costs nothing and has saved real work before.
- If building there while it's idle: separate worktree (`git worktree add /tmp/wt main`)
  + `CARGO_TARGET_DIR=/tmp/target-<name>` (38G fast scratch, wiped on stop — fine for
  checks; /workspaces has no room for a second target dir).

## Build quick-reference (codespace side; details in photocraft-rust-toolchain kb)

- Fast wasm gate: `cargo xtask wasm` (compat check, not a web build)
- Web app: `cd apps/photocraft-web && trunk build [--release]` (index.html forces
  `wasm-release` profile even without --release! LTO override env vars in kb)
- Full site: `packaging/web/package.sh` (24MiB per-file gate, gzip+parts splitting)
- Cheap iteration: profile overrides + `package.sh --skip-build`
- `wasm-opt -Oz --converge` can exceed 10-min tool limits — a kill there is NOT a Rust
  failure; check which stage died.

## Things that bite (env-specific, learned the hard way)

- `cs exec` with `ZAI_ROOT=/home/z/my-project` → lib.sh not found (kit lives elsewhere) —
  always set ZAI_ROOT explicitly.
- Unquoted heredoc `<<EOF` in cs exec → LOCAL bash expands $vars before transport.
- `pgrep -af cargo` self-matches the exec wrapper's bash — filter `| grep -v bash`.
- Z-container: caddy/filtered commands + rapid probe loops = IRREVERSIBLE session 403 —
  one probe per toolcall, ever; delegate risky probes to sub-agents (separate tool sessions).
- Background processes die at toolcall end unless double-forked (python daemonize recipe).
- Kit state (.state/ gh profiles) is in the zai-codespaces clone — recyclable, rebuilt by
  bootstrap; never store anything precious only there.

## Session-2 additions (2026-10-24)

- **The local build loop** (edit→wasm→bundle in ~80s): pc-edit worktree → `rsync -rc` →
  pc-build (mtime-pristine) → cargo with `CARGO_PROFILE_WASM_RELEASE_LTO=false
  CARGO_PROFILE_WASM_RELEASE_CODEGEN_UNITS=16` (the env vars hash identically to the live
  agent's in-file lto=false — verified, so its Cargo.toml hack stays uncommitted).
- **Trunk CLI**: breaks with `NO_COLOR=1` (wants true/false). `trunk build` dev mode still
  builds the wasm-release profile (data-cargo-profile) but skips wasm-opt — good.
- **Browser E2E lies**: agent-browser mouse events never trigger egui release-clicks;
  synthetic PointerEvents ignored (untrusted); accesskit DOM empty; VLM confirms whatever
  the prompt describes — md5/pixel-diff screenshots first. kittest + PNG fixtures = truth
  (agent/kb/web-e2e-tooling.md).
- **js bridge**: `window.PhotoCraft` ({id,params}/{steps} envelope) drives an INDEPENDENT
  headless session, not the visible UI.
- Python `sys._exit` does not exist in this container — daemonize with `os._exit`.
- functools.partial subclassing crashes http.server — subclass the handler directly.
