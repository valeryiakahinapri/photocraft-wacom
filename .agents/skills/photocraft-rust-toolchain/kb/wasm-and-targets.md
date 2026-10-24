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

The production Trunk path runs wasm-bindgen and then `wasm-opt -Oz --converge`. The latter is CPU-heavy on a small Codespace and can be terminated by a tool runner's per-process time limit even after Cargo has successfully produced the wasm. Check which stage failed before diagnosing the source as broken. `cargo xtask wasm` is the fast compatibility gate, not a replacement for the web shell build or package-size check.

When only a development build is feasible, build both web apps without Trunk `--release`, using the profile overrides above if needed, then run:

```sh
packaging/web/package.sh --skip-build
```

This validates the package layout and size checks on those development artifacts. Do not claim that it verifies the release optimizer or release output size.

## Toolchain version and editor support

Use the Rust minimum documented by `docs/development.md` and the stable channel used in CI. Install `rustfmt` and `clippy` for formatting and linting, plus `rust-analyzer` and `rust-src` for VS Code analysis. Trunk is versioned in CI; use that version for comparable web-build results. Add only the Rust standard-library targets required by the requested platforms.
