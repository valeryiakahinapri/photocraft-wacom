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

### Phase 1.5 — local build fan-out (DONE + MEASURED, 2026-10-24 session 2)
Local sandbox RUNS inherited-cache incremental wasm builds (not just proven-capable):
- [x] Toolchain extracted locally (rustc 1.99.1 == codespace, trunk 0.21.14, wasm target)
- [x] Storage lanes measured (overlay=build, ossfs/polarfs=transport/backup only)
- [x] kb/local-build-fanout.md: full recipe + ALL measurements (pull 5min, sed 0.5s,
      ~600 registry deps survive the transfer, true noop 0.3s, ui-egui edit to full wasm 70s,
      trunk web bundle 11.3s; one-time refingerprint of workspace crates 8m30s)
- [x] PILOT: RAN — pulled the live agent's own mid-flight tree+cache (its uncommitted
      lto=false Cargo.toml came in the tar; verified env overrides hash identically, so the
      feature branch builds warm WITHOUT committing that hack)
- Working setup: /home/z/pc-edit (git worktree, feature branch) then rsync -rc (content
      checksum, mtime-preserving) into /home/z/pc-build (mtime-pristine build space, no git).
- Decision (unchanged): release/fat-LTO + wasm-opt + packaging stay on codespace; local does
      cargo-check + LTO=false incrementals + kittest + patch authoring.

### Phase 1.6 — panel rail feature (IMPLEMENTED, under test — 2026-10-24 session 2)
The right-panel collapse UX the user asked for, used as the fan-out stress test:
- [x] Design + sub-agent review round (REV2: agent/design/2026-10-24-panel-rail-collapse.md)
- [x] feature/right-panel-collapse (6 commits, tip b6725404): rail-top panel-right toggle with
      popover (Hide/Show Dock + a checkbox row per group via set_group_shown), Character in
      the rail icons; collapsed dock shows 90-degree rotated label chips of shown groups
      (RTL mirrored); clicking a label opens a floating peek window (tab pills + shared dock
      body) closed by Escape / close box / click outside / dock return / dock::apply;
      window.panel.peek command; P0 fixes (panel_docking no longer re-asserts
      panels.dock=true; Character/Brushes rows routed safely); Hide Dock/Show Dock
      localized in all 21 catalogs.
- [ ] kittest result (native build ran in background; status /home/z/logs/test-native.status)
- [ ] Remaining gates: cargo check wasm (L0-L6), clippy, xtask layers, i18n report
- [ ] Deferred: panel_body dedupe (own commit + scroll assertions)
- [ ] After the live agent commits its WIP: rebase feature commits onto plain origin/main

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

- Codespace = build plane; local Z sandbox = control plane; REVISED 2026-10-24: local
  runs the SAME toolchain (extracted from the backup) + inherited wasm cache as a second
  build lane; codespace keeps release/heavy work.
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

### Phase 1.7 — mockup refinement + review hub (DONE, 2026-10-24 session 3)

The owner shipped the modern-canvas-studio mockup = the panel-behavior vision. REV4
(design doc): constant icon rail in both states (rotated labels + popover dropped),
direct dock toggle, re-click closes the peek, egui `Panel::show_collapsible` slide
(the review caught my hand-rolled glide corrupting the persisted dock width), the
customized-docking column keeps its instant toggle (documented deviation).
Live E2E on the baked build caught the `menus::invoke` dispatch bug (app.run →
engine → "unknown command"), fixed in production + tests; the feature is now
browser-proven (toggle/peek/re-click/focus-loss all verified by pixel diff).
Deliverables on the branch: 88875cde (icon rail), 06c6bcb4 (show_collapsible),
bc1a67c3 (dispatch fix), fa187d81 (merge of main's bridge commit).
New capability: the review hub (see kb/review-hub-and-pins.md) — /photocraft bake +
annotakit /storybook with the app mounted IN the canvas doc; region pins E2E-proven
against the wgpu UI; Route B (element pins via the bridge overlay) designed and
scoped as the next slice. AccessKit-on-web verified dead (eframe discards the tree).
