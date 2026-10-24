# PLAN — long-horizon tracker: adopting PhotoCraft (Rust+egui) for a web project

Mission: use the photocraft-wacom fork as the base of a custom web project. The product is
the same Rust codebase compiled to wasm (trunk + wasm-bindgen) — so custom features mean
modifying Rust and producing our own builds. This file tracks work across sessions.
Update it as items start/finish. Immediate next-session scope lives in HANDOFF.md.

## Mission phases

### Phase 0 — toolchain & environment (DONE, codespace, 2026-10-23)
- [x] Rust 1.99.1 stable + wasm32 target + Trunk 0.21.14 + apt libs in codespace
- [x] Native Linux build (CLI + desktop, HEIF) green; CLI smoke test (create/edit/save/reopen)
- [x] `cargo xtask wasm` green (24 crates); web app built; ZIP 15 MiB; package.sh green
- [x] Cross-target `cargo check` (macOS x86/aarch64, Windows x3, Linux aarch64/riscv64)
- [x] Toolchain backup: /workspaces/.photocraft-toolchain-backup (569M tar.zst + sha256 +
      restore script + apt manifest)
- [x] Z.ai control plane: kit bootstrapped + photocraft account patched; agent-meta branch

### Phase 1 — host integration API (IN PROGRESS, live copilot agent in codespace)
- [ ] js_bridge.rs: `globalThis.PhotoCraft` + `PhotoCraftBridge` wasm-bindgen export
      (commands/info/run/convert/export/batch equivalents; independent headless Session)
- [ ] tests/js-bridge.mjs Node oracle harness (CLI as oracle; normalize generated layer ids)
- [ ] wasm-release build of the bridged app + oracle run green
- [ ] (await user confirmation of agent completion before taking ownership / committing)
- Snapshot of WIP at 2026-10-24T00:22Z: `wip/2026-10-24T00:22Z/` on agent-meta branch

### Phase 1.5 — local build fan-out (PILOT READY, 2026-10-24)
Local sandbox proven capable of inherited-cache incremental wasm builds:
- [x] Toolchain extracted locally (rustc 1.99.1 == codespace, trunk 0.21.14, wasm target)
- [x] Storage lanes measured (overlay=build, ossfs/polarfs=transport/backup only)
- [x] kb/local-build-fanout.md written (profile facts, LTO=false env requirement, recipe)
- [ ] PILOT: pull source+wasm-target tarball from codespace → noop-build test →
      one-crate incremental timing (blocked on: live agent idle + WIP committed)
- Decision: release/fat-LTO + wasm-opt + packaging stay on codespace; local does
  cargo-check + LTO=false incrementals + patch authoring.

### Phase 2 — the actual web project integration (NOT STARTED — needs user input)
- [ ] Define the host project: framework, hosting, embedding model (iframe? same-page canvas?
      headless engine-as-library via the bridge, UI hidden?)
- [ ] Decide fork strategy: custom app as new `apps/<name>` crate? branch model vs upstream
      (recommend: `web-project` branch off main, rebase-onto-upstream rhythm)
- [ ] CI for our fork: reuse `web build and wasm size gate` workflow; keep wasm green
- [ ] Custom branding/build pipeline (our own dist, versioning, release artifacts)

### Phase 3 — productionization (LATER)
- [ ] Hosting with compression + correct wasm MIME; size budget watch (twiggy top -n 40)
- [ ] Long-op async/worker architecture if the bridge must run batches (see kb note: the
      bridge is synchronous on the page thread today)
- [ ] Sync/merge policy with upstream storytold/photocraft (fork tracks it actively)

## Standing decisions (do not relitigate without new facts)

- Codespace = build plane; local Z sandbox = control plane; no local Rust toolchain.
- All agent docs durable on `agent-meta` branch of the work repo (+ /home/sync copies);
  main stays clean for upstream tracking.
- Kit access: `ZAI_ROOT=/home/z/zai-codespaces CS_ACCOUNT=photocraft bash kit/cs <cmd>`.
- Read-only in the codespace while the live agent owns the tree (until user hands over).
- Never force-push; never touch /home/z/my-project with repo work (watchdog).

## Open questions for the user (ask when convenient)

1. What is the target web project (stack, hosting, timeline)? What does "adopt" concretely
   mean — embed the full editor, or use the engine headless behind a custom UI?
2. When do I get full codespace ownership (live copilot agent still active now)?
3. Should the fork keep merging upstream aggressively (current pattern), or freeze on a
   version for stability while we customize?
4. Codespace economics: stay on basicLinux32gb or upgrade to 4-core/16G for faster wasm
   builds? (wasm-release full build ≈ the slow gate; 4-core roughly halves it.)
   → ANSWERED 2026-10-24 (kb/local-build-fanout.md): per-build core-hour cost is ~constant
   (billing is core-hours); 4-core = same quota per build, half wall time, 2x idle burn,
   +RAM/disk headroom. Plan: bump only for cold-build/release windows.

## Risk register

- Codespace 12h hard cap + 30m idle: builds must be resumable; `cs start`/`ensure`
  resurrects; /workspaces persists; /tmp is wiped on stop.
- /workspaces disk 82% (5.5G free): target/debug 9.8G prunable when agent idle; a second
  full target dir does NOT fit — my builds should use /tmp (38G fast scratch) via
  CARGO_TARGET_DIR, accepting stop-wipe.
- Local sandbox recycle wipes /home/z (incl. the kit state, gh profiles): re-run
  agent/scripts/zai-bootstrap.sh (on agent-meta branch) to rebuild everything in ~1 min.
- Upstream moves fast (67 commits backlog pushed in one day): keep fork merges frequent
  and small; rebases of custom work get harder the longer we drift.
- Gitlab mirror: WAF blocks ~1/3 of HK-IP requests (retry pattern, background push).
