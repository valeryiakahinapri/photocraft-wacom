# HANDOFF — next-session scope (rewritten session 11, 2026-10-30)

New sandbox, new session. State: **everything green** — the owner's five asks (GH retry, stable-base
+ scroll-perf verification, workspace JS bridge, corner menu, wasm size report) are closed and
verified. Stable tip **e461fdf3d**, served bundle **ad49879d** (31,007,104 bytes — deterministic
rebuild of the tip), E2E **10/10** (`scripts/e2e_s10.py`), oracle gates fresh **PASS 0/48 + 0/55 +
fold ⇧Tab round-trip** (byte-stable). **Both remotes in sync — GH pushes work again** (owner
verified the account email in session 10). Re-read `/home/z/my-project/worklog.md` (the sessions
10 + 11 entry) after boot.

## The one known engineering open item: the 2px separator remnant

- Exact repro: fresh boot → click the corner button at (28,28) → click the File row → press
  Escape twice → a 1px (153,154,157) line at y≈98-99 spanning x 18-1196 persists at the
  options-bar bottom (the "2px" remnant band spans the canvas well).
- NEVER happens in bar mode. Appears only after corner-menu cascade + Escape×2.
- The e461fdf3d MenuConfig inheritance fix (the corner popup now builds its config from
  `MenuConfig::find(ui)` — the panel's own close_behavior + style — instead of bare
  `MenuConfig::new()` which carried egui's default style) was the SUSPECTED cause but is
  UNCONFIRMED to remove the remnant.
- The E2E D-check excludes the y 92-110 band meanwhile — if you fix the remnant, re-tighten the
  check.

## Owner-decision queue (do not implement before the owner picks)

- **Wasm size exclusions** (full report: `.agents/kb/wasm-size-attribution.md`): wasmi plugin
  runtime 1.08MB (Filter › Plug-ins feature), svg stack 711KB (vector rendering), exr 226K,
  heif 250K (keep? iPhone photos — product decision), raw 167K (Camera Raw feature), affinity
  75K (Affinity import), i18n ~600KB savable (21 language TSVs ≈ 829KB in the data section
  today), rayon 126K (needs an upstream-scale call-site cfg refactor in crates/engine).
  Realistic total: ~2.5-3.5MB (~10%) for modest product cost. NOT excludable: photocraft-psd
  (it IS the native document format); photocraft-ipp (already dead-stripped — don't gate).
- **Properties A/B/C** (carried from s9): A keep floating / B dock (2-flag: panels.rs ~2106
  gate + properties_window ~4198 early-return) / C hybrid — the owner pins on Review/Photocraft
  ▸ Mirror (PropertiesDeviation story). Threads live in THEIR browser localStorage — ask them
  to export/paste or place a fresh pin. Icon-set / title-bar / options-cluster convergence rides
  on the same pin directives.
- **annotakit gh-mirror fine-grained PAT** (owner) → threads survive browser loss.

## Bootstrap (post-recycle — updated session 11)

1. **Server first** (owner preview): `cp` the tgz from
   `/home/sync/photocraft-agent/wip/annotakit-tgz-backup/` into `/home/z/storybook-annotakit/`
   (mkdir first), then `bash /tmp/init-fullstack.sh`, then IMMEDIATELY the supervisor:
   `python3 /home/z/my-project/scripts/daemon-restore.py /home/z/my-project/scripts/dev-supervisor.sh /home/z/dev-supervisor.log`
   (plain nohup dies at toolcall boundaries — the reaper, re-proven s9).
2. **Python**: fresh container lacks `zstandard` → `python3 -m pip install zstandard` (the
   active python IS the venv; `--user` is rejected inside it).
3. **Rust lane** (only if building wasm): `scripts/lane-restore-s9b.sh` via `daemon-restore.py`;
   re-clone pc-edit (gitlab remote or the PolarFS bundle) then
   `rsync -a --delete --exclude target --exclude .git /home/z/pc-edit/ /home/z/pc-build/`;
   build IN pc-build. **Free RAM must be ~3.4G before trunk builds**: pkill dev-supervisor +
   "bun run dev" + "next dev" + **next-server + postcss** — bun's dev-server children SURVIVE
   the first two pkills and wasm-bindgen (needs ~2-3G RSS) gets OOM-SIGKILLed otherwise.
4. **Mockup/oracle lane**: fresh clones need `bun install` (520 packages from the offline
   cache) before the vite server boots: `PORT=8081 REPO=/home/z/mockup python3
   /home/z/my-project/scripts/boot-mockup.py` (passes `--port=8081 --strictPort` internally).
   agent-browser daemon-EOF → `pkill -f agent-browser` + relaunch.
5. **Hub durability**: /home/z/my-project has **NO git remote** — durability = the container
   snapshot + the PolarFS agent mirror (`/home/sync/photocraft-agent/agent/`). Copy agent/ +
   .agents/ + worklog/PLAN/HANDOFF there after milestones; never git-add photocraft-cache/,
   tool-results/, or .zscripts.
6. **Sanity sweep**: routes `/`, `/photocraft`, `/storybook` 200 (expect bundle `ad49879d`);
   `df -h /`.

## Version policy (standing decision, unchanged)

- **`stable/right-panel-collapse` = THE lane**, now tip **e461fdf3d** (upstream tag v0.16.0,
  stock egui 0.36.2, all REV7/8 visual work + the s10/11 features: workspace modes, corner
  menu, pan-without-glide). PolarFS bundle:
  `photocraft-stable-right-panel-collapse-e461fdf3d.bundle` (82.4MB, Oct 30 10:53) — supersedes
  the efd46e426 one. s10/11 commits: 719cc5612 (pan fix + workspace modes + PhotoCraftUI
  bridge), 1cf22ba31 (corner menu), 644892996 (popup-click fix), 8008beb (perf probes),
  947288b2f (wasm-opt pass removal), e461fdf3d (corner MenuConfig inheritance).
- `feature/right-panel-collapse` @ cc3aacd1c = the 0.37 lane, PARKED (upstream's 0.36→0.37
  migration regresses wasm fps ~3-4x). Port forward only when upstream fixes it; recipe +
  5-hunk API map in the session-9 commit messages + kb/stable-base-migration.md.
- Perf truth (measured s10, commit 8008beb): wasm frame = 3-5ms; the ~12.4fps sandbox ceiling
  is ENV (SwiftShader raster between rAFs; headless rAF throttles to ~13fps under input) — on
  a real GPU the app paints 60fps. The old scroll sluggishness was egui's wheel glide
  (exponential_smooth_factor(0.90, 0.1, dt), ~250ms tail) — fixed (`smooth:false` at both wheel
  read sites). Do not re-chase fps in the sandbox; it's the rasterizer, not the app.

## Immediate scope for next session

1. **The 2px separator remnant investigation** (above) — the only engineering open item.
2. **Owner decisions**: wasm exclusions + Properties A/B/C + mirror pins (ask for exports/pins;
   their calls override everything).
3. Native test run (codespace lane — RAM/disk-limited locally): `cargo test -p photocraft-ui-egui`.
4. Watch upstream for the egui 0.37 perf fix — port forward when it lands.
5. Any visual/UX change must keep ALL FOUR gates green: `tools/oracle/run.py` (0/48) +
   `--profile mirror` (0/55) + `tools/oracle/e2e_fold.py` + `scripts/e2e_s10.py` (10 checks).
   E2E boots must clear localStorage (`photocraft.preferences` persists by design) and use
   SYNTHETIC `WheelEvent` dispatch for pan/zoom (CDP `mouse wheel` never reaches egui-web).

## Where everything lives

- Hub (watched, main): wasm `public/photocraft` = `ad49879d`; `scripts/e2e_s10.py` = the
  s10-feature E2E harness; `scripts/wasm-name-attr.py` = per-crate size attribution;
  `download/oracle/` = the s9 reports (re-verified byte-identical fresh on ad49879d).
- Work repo `stable/right-panel-collapse` tip `e461fdf3d` — on GitHub + gitlab + the PolarFS
  bundle. Mockup REV8 `67867a7` — same three lanes.
- Size report: `.agents/kb/wasm-size-attribution.md` (sections + per-crate groups + exclusion
  candidates with exact bytes + gzip numbers).
- Design docs: `/home/sync/photocraft-agent/agent/design/` (REV8 + the s10 audit amendments);
  kb: stable-base-migration.md, wasm-size-attribution.md.

## Sticky rules (never break)

GIT IS THE DISK — push/bundle every micro-step (BOTH remotes work now; gitlab WAF ~1/3 →
3-5x retry loop with exit-code-branching, not `| tail -1 && break`). Never force-push. Commit
harness/tooling edits BEFORE the wrap-up (the s10 lesson: it died mid-wrapup with no worklog
entry and a stale baked bundle). Watchdog force-checkouts /home/z/my-project to main ~every 20s
(work in pc-edit/pc-build/mockup, not the hub tree). Never build wasm from pc-edit paths (build
IN pc-build). On tool failure: delegate to sub-agents (s11's bash-infra death was the only
failure and was worked around exactly this way). dev-supervisor must run whenever the owner
preview matters. Never git-add photocraft-cache/. Before wasm builds: the full pkill set +
~3.4G free RAM.
