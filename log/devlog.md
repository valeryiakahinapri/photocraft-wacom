# Development log

## 2026-10-23

- Installed stable Rust 1.99.1, rustfmt, Clippy, rust-analyzer, rust-src, Trunk 0.21.14, WASM and packaged Linux/macOS/Windows Rust targets; installed the documented Ubuntu desktop development libraries. Rustup PATH integration is present in the login and interactive shell profiles. Removed Windows and non-host Linux Rust targets afterward, retaining only macOS x86_64/aarch64, host Linux x86_64, and WASM.
- Native Linux `photocraft` and `photocraft-cli` builds passed with HEIF. Headless CLI smoke test created a document, added a layer, saved `.pcraft`, and reopened it with `info`.
- `cargo xtask wasm` passed (24 crates plus the HEIF check). Both browser apps built with Trunk and `packaging/web/package.sh --skip-build` passed; ZIP is 15 MiB and the largest site file is 15,494,706 bytes.
- Cross-target `cargo check` passed for macOS x86_64/aarch64, Windows x86_64/i686/aarch64 MSVC, and Linux aarch64/riscv64gc.
- The production Trunk path compiled the WASM app with local LTO/codegen overrides, but `wasm-opt -Oz --converge` was terminated at the runner's 10-minute subprocess limit. Full macOS/Windows linking and packaging still require their native SDK/host runners. No tracked source changes.
