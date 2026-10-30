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

### Phase R6-R8 — the mockup adoption arc (REV8 COMPLETE, 2026-10-28)
- [x] REV6: island shell + ModernCanvas theme (oracle infra born in REV7)
- [x] REV7: bezel-free restyle to the updated mockup + the pixel-exact oracle (48 checks, PASS)
- [x] REV8: contrast fixed at the ROOT (edge grammar restored: 3-part island stack + lit top
      edges + mockup-scale doc shadow + UNLIT workspace; ladder widened +44% panel-vs-canvas
      ΔL, all measured from Chrome); the app mirrored as DOM into the SB (annotakit pins
      PROVEN e2e); oracle grew a mirror profile (55 checks) — BOTH PASS on bundle 24f97aca

### Phase S9 — STABLE-BASE MIGRATION (COMPLETE, 2026-10-29)
- [x] Push forensics: work PAT correct (API-verified identity+scopes); the 403 is the
      account's unverified primary email (`/user/emails` `"verified": false`) — owner fix.
- [x] Gitlab recovered as the working remote (glpat → api/v4/user = valeryiakahinapri;
      mirrors: photocraft-wacom + modern-canvas-studio created/filled; WAF 1/3 → retry loop)
- [x] Version forensics: our lane was egui 0.37 (session-7 port onto the migrated main) = the
      3.6fps regression lane; upstream v0.16.0 = stock egui 0.36.2
- [x] Migration: `stable/right-panel-collapse` = afccf85e7 (pre-port work, stock 0.36.2) +
      merge v0.16.0 (1 conflict) + 3-way REV7/8 delta (5 hunks, API map in commit messages)
      → cargo check green → wasm df463fe4 (30.98MB, 4m warm build) → baked
- [x] Gates on the stable build: oracle default 0/48 PASS + mirror 0/55 PASS + fold
      ⇧Tab round-trip PASS + console clean; session-8's lost harness calibrations recovered
      + committed; e2e_fold.py added as the permanent fold gate
- [x] GH push unblock — DONE session 10: owner verified the account email; everything
      (stable + feature + agent-meta + mockup) pushed to GitHub; both remotes in sync since
- [ ] OWNER DECISION: floating Properties — A keep floating / B dock (2-flag) / C hybrid
      (pin the PropertiesDeviation story); implement the pick + lockstep oracle updates
- [ ] Icon-set/title-bar/options-cluster convergence — driven by the owner's mirror pins
- [ ] Native test run (codespace or RAM window) + kittest run
- [ ] PINs Route B (canvas-anchored) — RETIRED in favor of the DOM mirror (pins work today)
- [ ] annotakit gh-mirror fine-grained PAT (owner) → threads survive browser loss
- [ ] Watch upstream for the egui 0.37 perf fix → then port forward (recipe in the s9
      commits; the 0.37 lane feature/right-panel-collapse@cc3aacd1c stays parked meanwhile)

### Phase S10-S11 — owner round: perf verify + workspace bridge + corner menu + size report (COMPLETE, 2026-10-30)
- [x] GH unblock + sync (ask 1): email verified (owner, s10); all branches pushed to BOTH
      remotes — work repo (stable + feature + agent-meta) + mockup REV8 67867a7
- [x] Stable-base + canvas scroll perf verified (ask 2, s10 commit 8008beb, scripts
      fps_scroll_probe.py + gl_fps_probe.py): wasm frame = 3-5ms; the ~12.4fps sandbox number
      is an ENV ceiling (SwiftShader raster between rAFs; headless rAF throttles to ~13fps
      under input) — on a real GPU the app paints 60fps. Scroll sluggishness root cause =
      egui's wheel glide (exponential_smooth_factor(0.90, 0.1, dt) spreading a web notch over
      ~250ms) → fixed `smooth:false` at both wheel read sites (719cc5612)
- [x] Workspace-type JS bridge (ask 3, 719cc5612): PhotoCraftUI JS global with state mirror;
      ?workspace=paint/photo/etc presets shape the toolbar (shape_toolbar) + pick their tool
      and boot into the mode; canonical() alias resolver; post-prefs-load latch (prefs_loaded
      gate so stored prefs don't undo boot params on frame 1)
- [x] Figma-style corner menu (ask 4): Panels.menu_corner with cfg(wasm32) default so web
      shows the corner button (replaces the horizontal strip; click → vertical popup of all
      menus); ?menu=bar|corner boot override; corner_menu() reuses the bar's
      submenu_row/render_level machinery + item cache + press-drag gesture + #319 anchor
      (1cf22ba31); popup-click survival fixed via PopupCloseBehavior::IgnoreClicks for the
      release that opened the popup (644892996); MenuConfig::find(ui) inheritance fix so the
      popup carries the panel's close_behavior + style (e461fdf3d)
- [x] Wasm size report delivered (ask 5): 31.0MB bundle = code 24.94MB (80.4%) + data 5.93MB
      (19.1%); per-crate/module/function code attribution via NEW scripts/wasm-name-attr.py
      (raw pre-opt name section, v0 mangling); gzip transfer 13.4MB; full tables + exclusion
      candidates in .agents/kb/wasm-size-attribution.md
- [x] wasm-opt pass de-duplicated: trunk's data-wasm-opt="z" (index.html) is the ONLY pass
      needed — the second pass GREW the bundle +39,187 bytes (measured on 685069da) and
      trunk's cached wasm-opt version_123 rejects the old two-positional-args CLI;
      scripts/trunk-bundle.sh trimmed (947288b2f, pushed GH+gitlab)
- [x] Verified end-state: stable tip e461fdf3d; bundle ad49879d (31,007,104 bytes,
      deterministic rebuild of the tip) baked + served, all hub routes 200; E2E 10/10
      (scripts/e2e_s10.py — corner posture, popup click survival, File cascade, Escape close,
      workspace=paint boot, JS bridge live switch, full-notch first-frame pan, zero glide
      tail); oracle gates fresh PASS 0/48 + 0/55 + fold round-trip (byte-identical to the s9
      reports — the oracle boots pin ?menu=bar by design); durable bundle refreshed
      (photocraft-stable-right-panel-collapse-e461fdf3d.bundle, 82.4MB, supersedes efd46e426)

### Phase S12 — pending owner decisions + open investigations (queue, 2026-10-30)
- [ ] OWNER DECISION: wasm size exclusions (full report: .agents/kb/wasm-size-attribution.md)
      — wasmi plugin runtime 1.08MB (Filter › Plug-ins feature), svg stack 711KB (vector
      rendering), exr 226K, heif 250K (keep? iPhone photos — product decision), raw 167K
      (Camera Raw feature), affinity 75K (Affinity import), i18n ~600KB savable (21 language
      TSVs ≈ 829KB in the data section today), rayon 126K (needs an upstream-scale call-site
      cfg refactor in crates/engine); realistic total ~2.5-3.5MB (~10%) for modest product
      cost. NOT excludable: photocraft-psd (the native document format); photocraft-ipp
      (already dead-stripped)
- [ ] INVESTIGATE: the 2px light-gray separator remnant at the options-bar bottom — exact
      repro: fresh boot → click the corner button at (28,28) → click the File row → press
      Escape twice → a 1px (153,154,157) line at y≈98-99 spanning x 18-1196 persists; NEVER
      happens in bar mode; the e461fdf3d MenuConfig inheritance fix was the suspected cause
      but is UNCONFIRMED to remove it; the E2E D-check excludes the y 92-110 band meanwhile
- [ ] Carried from S9 (unchanged, still pending): OWNER DECISION Properties A/B/C (keep
      floating / dock 2-flag / hybrid — pin the PropertiesDeviation story) + the owner's
      mirror pin directives (icon-set/title-bar/options-cluster convergence); annotakit
      gh-mirror fine-grained PAT (owner); native test run + kittest; watch upstream for the
      egui 0.37 perf fix

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

- **Version policy (S9): ride `stable/right-panel-collapse` (upstream tag v0.16.0, stock egui
  0.36.2 + all our visual work). The 0.37 lane is parked until upstream fixes the perf
  regression. Never adopt upstream main blindly — tags only, verified (fps + oracle).**
- Codespace = build plane; local Z sandbox = control plane; no local Rust toolchain.
- All agent docs durable on `agent-meta` branch of the work repo (+ /home/sync copies);
  main stays clean for upstream tracking.
- Kit access: `ZAI_ROOT=/home/z/zai-codespaces CS_ACCOUNT=photocraft bash kit/cs <cmd>`.
- Read-only in the codespace while the live agent owns the tree (until user hands over).
- Never force-push; never touch /home/z/my-project with repo work (watchdog).
- **Gitlab = the second durable remote** (glpat → account valeryiakahinapri; WAF 1/3 → retry
  loop). GH pushes work again since session 10 (owner verified the email) — push BOTH.

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
- GH account: the primary-email-unverified push block was REAL and owner-fixed in session 10
  (2026-10-29) — pushes now work, both remotes in sync. Keep the gitlab WAF retry habit.
- Upstream's egui 0.36→0.37 migration regresses wasm fps ~3-4x (sibling matrix: tag 11.1 /
  vendored 14.2 / main 3.6) — the reason we pin to v0.16.0. Re-verify before any port forward.
- Platform-snapshot file rm in /home/z/.venv, .bun, .cache = whiteout (image layer) — only
  upper-layer deletes free disk (this session's target/, registry/src, git-blob purge).
