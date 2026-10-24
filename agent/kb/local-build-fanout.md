# Local build fan-out (Z.ai sandbox as second build lane)

Measured + verified 2026-10-24. Answers: "does rustc use more cores/RAM?", "can the
sandbox do the final link / incremental builds on inherited caches?", "what goes where?".

## Rust parallelism facts for THIS repo

- `wasm-release` profile = **fat LTO + codegen-units=1** (+ per-crate opt-level z/s/3).
  Cold build = two phases:
  1. Crate-graph compilation (~24 workspace crates + ~600 deps): cargo runs N rustc
     processes in parallel (N = cores). Near-linear scaling up to the widest serial
     dependency chain (proc-macro chains). **Cores help here.**
  2. Final fat-LTO link + `wasm-opt -Oz --converge`: single process, serial,
     memory-hungry (LLVM merges everything; wasm-opt is mostly single-threaded).
     **Cores don't help; RAM prevents OOM/kill.** (The 10-min wasm-opt kill in devlog
     was a time limit, not memory.)
- 4-core/16G verdict: cold builds ~1.5-1.8x faster wall-clock (phase 1 ~1.8x, phase 2
  ~1x); 16G makes the fat-LTO link comfortable. Codespaces bills **core-hours**, so
  core-hr per BUILD is ~constant — you pay the same quota for half the wall time. Idle/
  keepalive time costs 2x on 4-core. Strategy: 2-core for routine work; bump to 4-core
  (stop → machine edit → start) for cold-build/release windows; the larger disk that
  comes with it also relieves the /workspaces pressure (was 87%).
- Incremental one-crate rebuilds are mostly serial anyway (one rustc + link) — 2 cores
  are fine; wide fanout crates (geom/doc touch → many dependents) benefit from 4.

## Local fan-out — VERIFIED VIABLE (2026-10-24)

Pillars proven on this sandbox:
1. **Same rustc**: extracted the codespace's own toolchain backup
   (569M tar.zst → /home/z/.cargo + .rustup + .cache/trunk, 2.3G uncompressed).
   `rustc 1.99.1 (58cf8f9c3)` — bit-identical version to the codespace. Hello-wasm
   builds locally in 0.14s (wasm std precompiled). trunk 0.21.14 + wasm-bindgen
   0.2.129 + wasm-opt v123 present via trunk cache.
2. **Storage lanes (measured, 128MB seq / 100-file create)**:
   | lane | seq W | seq R | create/file | role |
   |---|---|---|---|---|
   | overlay /home/z | 1021 MB/s | 8369 MB/s | ~0 ms | **ALL build work** |
   | ossfs /home/sync | 89 MB/s | 194 MB/s | 28.7 ms | transport + backup only |
   | polarfs /tmp/my-project | 110 MB/s | 98 MB/s | 21.0 ms | durable staging (tarballs), never build dirs |
   A target dir has tens of thousands of small files → ossfs/polarfs metadata cost
   (21-29 ms/file) makes extract/work there ~1000x slower. Never put target/ on them.
3. **Transfer lane**: `cs get` = base64 over cm-ssh ≈ **2.6 MB/s** observed (569M in
   3m45s). A zstd'd 1.5G wasm target (~500M) ≈ 3-5 min. Fine.

### How to inherit the codespace's build cache (the pilot recipe)

- The live agent builds with env overrides — the inheritable cache is
  `target/wasm32-unknown-unknown` (1.5G at snapshot) built with
  **CARGO_PROFILE_WASM_RELEASE_LTO=false CARGO_PROFILE_WASM_RELEASE_CODEGEN_UNITS=16**.
  Locally MUST use the same env or fingerprints invalidate (profile flags are part of
  the fingerprint hash). These overrides are also what make local linking feasible:
  fat LTO on 4G no-swap would likely OOM — **release-grade (fat-LTO + wasm-opt) stays
  on the codespace**; local links only with LTO=false.
- Ship **source tree + target dir in ONE tar** (mtimes must match what fingerprints
  recorded; a fresh clone re-mtimes everything → full rebuild).
- Fingerprint portability caveat: absolute paths differ (/workspaces/... vs /home/z/...).
  Cargo fingerprints are mostly path-independent, but build-script crates (text has a
  build.rs reading CRAFT_FONTS_DIR) may rerun → rebuild of that crate + dependents.
  Empirical noop-test decides (pilot step 3).
- Pilot steps (next session, after live agent idle + WIP committed):
  1. codespace: `tar --zstd -cf /tmp/pc-src-target.tar.zst -C / workspaces/photocraft-wacom`
     (exclude target/debug! only target/wasm32-unknown-unknown + sources) → cs get.
  2. local: extract to /home/z/photocraft-wacom (overlay!), `PATH=/home/z/.cargo/bin:$PATH`.
  3. `CARGO_PROFILE_WASM_RELEASE_LTO=false CARGO_PROFILE_WASM_RELEASE_CODEGEN_UNITS=16
     cargo build --offline --locked --target wasm32-unknown-unknown --profile wasm-release
     -p photocraft-web --features heif` → expect near-noop (or build.rs-crate rebuild).
     Measure. Then touch one crate → measure incremental time. Record numbers here.
  4. NEVER sync target back (divergent mtimes poison the codespace cache; one-way flow).
     Source changes flow via git, as always.

### Local lane roles (division of labor)

- **local, cheap (primary)**: `cargo check --target wasm32-unknown-unknown` (rmeta only,
  no codegen/link — the fast inner loop), clippy, rustfmt, patch authoring, docs,
  sub-agent grep/review fan-out. 2-core/4G comfortable.
- **local, medium (secondary)**: incremental wasm builds with LTO=false env (dev-grade
  wasm; verify bridge changes compile+link). Disk budget: toolchain 2.3G + target ~2G
  + repo 126M fits in 9.9G overlay with ~4G headroom.
- **codespace only**: release/fat-LTO links, wasm-opt, packaging/web checks, native
  builds/tests, corpus tests (target/debug alone is 9.8G).
- **zstd CLI missing locally** (no sudo): `pip install zstandard` +
  `scripts/zstd-extract.py` (agent-meta) handles .tar.zst; or use plain .tar.gz.

## Open measurements (fill in during pilot)

- [ ] noop-build time after cache pull (fingerprint survival rate)
- [ ] one-crate incremental build + LTO=false link time on 2 cores
- [ ] peak RSS of the LTO=false link (headroom check for 4G)
- [ ] codespace 4-core cold-build wall time vs 2-core (when we bump, if we bump)
