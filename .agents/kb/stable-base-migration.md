# kb: stable-base migration (v0.16.0 / egui 0.36.2) — recipe + API map

Written session 9 (2026-10-29) after executing the migration end-to-end in one pass
(cargo check green in 18s incremental, wasm build 4m warm, both oracles + fold PASS).
Reference for any future port (forward to 0.37+ when upstream fixes the perf regression,
or another stable tag hop).

## Why (never relitigate)

Upstream's egui 0.36→0.37 migration regresses wasm fps ~3-4x in GPU-less environments
(sibling workstream isolation matrix: official zip ≈ tag v0.16.0 11.1fps; vendored tag 14.2fps;
main (0.37) 3.6fps — our code exonerated by same-profile cross-testing). Owner directive:
tagged, verified release milestones only; never ride upstream main.

## Branch topology (as of session 9)

- `afccf85e7` = our sessions 1-6 work (rail/fold/island/REV6/web-boot) on **stock egui 0.36.2**
  (upstream base 28f3c18c0 + the codespace fork's 2 commits 7d6f9df9b/c8105921d).
- `417f57be8` = the 0.37 main merge (upstream through 3877bb1d — the migration entered here).
- `cb8634b06` = session 7's port of our shell onto 0.37 (merge afccf85e7 + 417f57be8).
- `feature/right-panel-collapse` @ cc3aacd1c = the 0.37 lane incl. REV7+REV8 (PARKED).
- `stable/right-panel-collapse` @ efd46e426 = THE lane: afccf85e7 + merge v0.16.0 + the
  REV7/8 delta ported back. Upstream tags: v0.16.0 = b1794c6e (egui 0.36.2 STOCK, no
  [patch.crates-io]); v0.17.0 = the 0.37 migration release.

## The recipe (5 steps)

1. `git checkout -b <new-stable> afccf85e7 && git merge v0.16.0` — expect near-zero conflicts
   (ours: 1, state.rs panel_peek + their doc line → keep both).
2. `git diff cb8634b06 cc3aacd1c > delta.patch && git apply --3way delta.patch` — conflicts
   only where the patch's context is 0.37-shaped (ours: 5 hunks in canvas.rs/dock.rs/panels.rs).
   Resolution rule: **feature content from theirs, 0.36 API from ours** (map below).
3. `cargo check -p photocraft-ui-egui -p photocraft-web --target wasm32-unknown-unknown` in
   the pc-build lane (rsync source first; NEVER build from pc-edit paths). Clean-context
   hunks can still hide 0.37 APIs — expect 1-2 more fixes from the compiler (ours: a second
   `Role::Button`).
4. Build: `scripts/trunk-bundle.sh` with the env traps (LTO=false, CODEGEN_UNITS=16, no
   --offline, trunk-cache.tgz restored). 0.36-era eframe emits NO snippets/ dir → the bake
   script's snippet copy must be conditional.
5. Gates (all three, no exceptions): `tools/oracle/run.py` 0/48 + `--profile mirror` 0/55 +
   `tools/oracle/e2e_fold.py`. Oracle calibration constants live in run.py (mirror: title-bar
   probe at 80%, L3 tolerance 2.5 — the corner-ink noise floor).

## The 0.36 ↔ 0.37 API map (photocraft-codebase-specific)

| 0.37 (main) | 0.36.2 (tag) | where it bit |
|---|---|---|
| `ui.scope_id().with(..)` | `ui.id().with(..)` | canvas.rs dtabjob, dock.rs splitter |
| `egui::WidgetInfo::labeled(egui::Role::Button, ..)` | `..(egui::WidgetType::Button, ..)` | panels.rs toolbar ×2 |
| `egui::Id::unique(..)` | `egui::Id::new(..)` | panels.rs flyout id |
| `Window::open()` → `Option<InnerResponse>` | `Window::show()` → `InnerResponse` | (peek window: clean here) |
| `snippets/inline0.js` emitted by trunk | no snippets dir | bake-photocraft.sh |

Also: the 0.37 lane pins `storytold/egui@6d518fc` via [patch.crates-io]; the tag + our
pre-port branch use STOCK crates.io egui — the warm cache is keyed to the stock 0.36.2
graph (session-5 pc.tar.zst), which is why deps did not rebuild.

## Cache-warmth facts

- pc.tar.zst (sync: /home/sync/photocraft-agent/toolchain/) = the session-5-era target =
  stock-0.36.2 graph → ~fully warm for the stable lane (only workspace crates rebuild).
- MUST build with `CARGO_PROFILE_WASM_RELEASE_LTO=false CARGO_PROFILE_WASM_RELEASE_CODEGEN_UNITS=16`
  (fingerprint match + avoids the thin-LTO 2G+ cold-build death).
- `~/.cargo/registry/src` (860M) is prunable on demand (re-extracts from the .crate cache);
  `target/debug` (1.3G) prunable when the native lane is parked.
