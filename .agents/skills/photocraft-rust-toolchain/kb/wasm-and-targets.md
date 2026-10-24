# PhotoCraft WASM and target notes

This note records build-system behavior that is easy to miss when setting up the project from a Linux Codespace. Confirm current commands and target lists in `docs/development.md`, `.github/workflows/ci.yml`, `packaging/*/package.*`, and `Cargo.toml` before changing them.

## Headless Linux is a CLI, not a separate engine executable

`photocraft-engine` is a library. `photocraft-cli` is its supported headless application surface: `run`, `batch`, `mcp`, and `serve` use an in-process engine session. For Linux engine-only workflows build and exercise the CLI; a desktop binary is not a prerequisite. A useful smoke test is `photocraft-cli run --new ... --cmd ... --out ...`, followed by `photocraft-cli info` on the saved file.

## Keep targets task-scoped

The package scripts in this repository define the canonical architecture set:

- Linux package architectures: x86_64, aarch64, and riscv64 (the latter is a cross-build).
- macOS: `x86_64-apple-darwin` and `aarch64-apple-darwin`.
- Windows: x86_64, i686, and aarch64 MSVC; only add these when the task actually calls for Windows.
- Browser: `wasm32-unknown-unknown`.

On an x86_64 Linux Codespace used for headless Linux + macOS source checks + web, retain only the host Linux target, the two macOS targets if cross-checking them, and wasm32. `cargo check --target` can expose cfg/type errors without a target linker, but it does not prove a target-native executable can be linked or packaged.

## Trunk can build the release profile without release mode

`apps/photocraft-web/index.html` has `data-cargo-profile="wasm-release"`. Consequently, even a plain `trunk build` can invoke Cargo with `--profile wasm-release`. Cargo environment overrides are useful for local iteration without editing the checked-in size-optimized profile:

```sh
CARGO_BUILD_JOBS=2 \
CARGO_PROFILE_WASM_RELEASE_LTO=false \
CARGO_PROFILE_WASM_RELEASE_CODEGEN_UNITS=16 \
trunk build
```

For the actual distribution validation, use `packaging/web/package.sh`. It builds both `photocraft-web` and `photocraft-web-boot`, then packages/checks the static site. Run from the repository root; Trunk resolves each app's own config.

The production Trunk path runs wasm-bindgen and one `wasm-opt -Oz` pass. In this Codespace, a 55,877,758-byte no-LTO WASM module optimized in 220 seconds to 31,831,923 bytes (13,120,294 bytes gzip). Asking Binaryen to converge ran for 767 seconds and was terminated before it produced an output file. A single size-focused pass provides a meaningful compressed-size reduction and completes within the runner budget; do not enable `--converge` without a measured reason. The full no-LTO/16-codegen-unit package path completed in about 7m46s on the first build and 3m53s warm, including both Trunk builds and packaging. The web package gzips the module into capped parts, so its raw module size need not fit the per-file limit. Check which stage failed before diagnosing the source as broken. `cargo xtask wasm` is the fast compatibility gate, not a replacement for the web shell build or package-size check.

## Optional web features: measured footprint

These numbers are from the same no-LTO/16-codegen-unit WASM profile, then `wasm-bindgen --target web --weak-refs` and one `wasm-opt -Oz` pass. Gzip is `gzip -9 -n`; source and compiler changes can move the totals.

| Configuration | Optimized raw | Gzip |
|---|---:|---:|
| Official web build, HEIF enabled | 30,696,627 bytes | 13,216,212 bytes |
| HEIF disabled | 30,525,569 bytes | 13,146,289 bytes |
| AVIF encoder enabled, HEIF disabled | 31,463,238 bytes | 13,509,606 bytes |

HEIF adds about 171 KB raw / 70 KB gzip here, so dropping it saves little. AVIF is not enabled by the official web feature set; enabling the codec dependency directly adds about 938 KB raw / 363 KB gzip. The plug-in host is different: `photocraft-engine` requires `photocraft-plugins` (including `wasmi`), and there is no feature to remove it. Do not claim a plugin size saving without a feature-gated build and tests covering command/UI availability. Other built-in codecs are normal dependencies, not independent optional switches.

When only a development build is feasible, build both web apps without Trunk `--release`, using the profile overrides above if needed, then run:

```sh
packaging/web/package.sh --skip-build
```

This validates the package layout and size checks on those development artifacts. Do not claim that it verifies the release optimizer or release output size.

## Toolchain version and editor support

Use the Rust minimum documented by `docs/development.md` and the stable channel used in CI. Install `rustfmt` and `clippy` for formatting and linting, plus `rust-analyzer` and `rust-src` for VS Code analysis. Trunk is versioned in CI; use that version for comparable web-build results. Add only the Rust standard-library targets required by the requested platforms.
