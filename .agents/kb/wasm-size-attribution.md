# kb: wasm size attribution + exclusion candidates (sessions 10-11, 2026-10-30)

The size-report deliverable (owner ask 5, executed session 11) applied to the 31.0MB bundle
(`ad49879d`, 31,007,104 bytes — deterministic rebuild of stable tip e461fdf3d).
**gzip transfer = 13.4MB.**

## Tools

- `scripts/wasm-sections.py` — section-size breakdown of the bundle.
- `scripts/wasm-name-attr.py` (NEW, committed session 11) — per-crate/module/function code
  attribution parsed from the name section of the RAW pre-opt wasm.

## The raw-wasm path trick

- wasm-opt STRIPS the ~22MB name section, so the optimized 31MB bundle can't be attributed —
  attribute from the RAW cargo output at
  `target/wasm32-unknown-unknown/wasm-release/<bin>.wasm` (56MB, name section intact).
- Rust v0 mangling: crate roots are `C<hash>_<len>name` chains (`C<base62-hash>_` +
  `<decimal-len><name>`, nested per module path) — `scripts/wasm-name-attr.py` walks them.

## Section breakdown (of the 31.0MB bundle)

| section | size | share |
|---|---|---|
| code | 24.94MB | 80.4% |
| data | 5.93MB | 19.1% |

## Per-crate/module code attribution (name-section parse)

| group | code size | share of bundle |
|---|---|---|
| photocraft_* (our crates) | 11.07MB | 35.9% |
| stdlib | 6.01MB | 19.5% |
| font/text stack | 2.01MB | 6.5% |
| GPU stack (naga/wgpu/vello) | 1.93MB | 6.3% |
| fearless_simd | 1.72MB | 5.6% |
| serde | 1.71MB | 5.5% |
| image codecs | 1.60MB | 5.2% |
| egui family | 1.22MB | 4.0% |
| wasmi plugin runtime | 1.08MB | 3.5% |
| svg stack | 0.71MB | 2.3% |
| hashbrown | 0.60MB | — |
| other | 1.18MB | — |

## Feature findings (do not relitigate)

- **heif is INTENTIONALLY on**: apps/photocraft-web/index.html carries
  `data-cargo-features="heif"` (trunk passes `--features heif`). heic_rs 152K +
  photocraft-heif 4.2K + ruzstd 97K ≈ 250K. iPhone HEIC support = a product decision to KEEP.
- **photocraft-ipp is dead-stripped** (negligible) — not worth gating.
- **photocraft-psd is NOT excludable**: it IS the native document format (vector maps,
  annotations, patterns all use psd descriptors).
- **rayon + rayon_core ≈ 126K of dead threading code IS linked** into wasm because
  crates/engine's rayon dep is not `cfg(not(wasm32))`-gated (crates/compose and all others
  gate it correctly; engine's call sites have unconditional `use rayon::prelude::*` at module
  tops — an upstream-scale refactor, documented as a finding).
- **i18n**: 21 language TSVs, build-time-deflated to ~829KB total inside the data section —
  the biggest single data item. A breadth cut or lazy-load could save ~600KB.

## Exclusion candidates (OWNER decides; do not gate without a pick)

| candidate | size | note |
|---|---|---|
| wasmi plugin runtime | 1.08MB | Filter › Plug-ins feature |
| svg stack | 711KB | vector rendering |
| i18n (breadth cut / lazy-load) | ~600KB savable | 21 TSVs ≈ 829KB in data today |
| heif | 250K | keep? iPhone photos — product decision |
| exr | 226K | — |
| raw | 167K | Camera Raw feature |
| rayon | 126K | needs the upstream call-site cfg refactor |
| affinity | 75K | Affinity import |

Realistic total: **~2.5-3.5MB (~10%)** of the bundle for modest product cost.
