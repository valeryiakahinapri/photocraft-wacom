---
name: photocraft-rust-toolchain
description: "Set up or recover the PhotoCraft Rust development environment, build Linux headless CLI/engine or browser WASM, and check macOS targets from Linux. Use when working on Rust build prerequisites, Codespaces persistence, Cargo target setup, or Trunk web builds."
---

# PhotoCraft Rust Toolchain

Use this workflow when setting up or recovering the PhotoCraft development environment, especially in GitHub Codespaces.

## Fast orientation

- Read `docs/development.md` for current prerequisites and commands; `AGENTS.md` is authoritative if this guidance differs.
- The Linux headless entry point is `photocraft-cli`, not the desktop shell: it can create/open documents, dispatch engine commands, save/export, batch, and serve MCP/JSON-lines.
- On Linux, only install Rust targets needed on the host. macOS Rust targets can be installed for cross-target `cargo check`, but that does not replace compiling/linking and packaging on macOS.
- Before installing toolchains or caches in Codespaces, check storage. `/workspaces` is the persistent workspace filesystem; `$HOME` toolchains may not survive a Codespace rebuild.

## Build and verify

1. Check `rustc --version`, `cargo --version`, `rustfmt --version`, `cargo clippy --version`, `rust-analyzer --version`, `trunk --version`, and `rustup target list --installed`. Install versions/targets from the checked-in docs and workflows rather than assuming a newer Trunk is interchangeable.
2. For Linux headless-only work, build `cargo build --locked -p photocraft-cli`; do not build the `photocraft` desktop package unless the task needs the GUI. Verify the CLI itself with a small create/edit/save/reopen flow, not only `cargo check`.
3. For the wasm-safe crate set, run `cargo xtask wasm`. For the browser app and distributable site, use `packaging/web/package.sh`; that script builds both web shells and checks relative URLs and the site file-size budget.
4. For a source-level macOS compatibility check from Linux, install only the desired Apple targets and run `cargo check --locked --target <triple> -p photocraft-cli` (or include `-p photocraft` when needed). Treat this as a type-check only: native SDKs, linking, signing, and packaging require macOS.
5. After validation, remove unrequested target stdlibs with `rustup target remove <triple>` and remove only identified generated target caches. Preserve the CLI binary or web ZIP only when the user wants those deliverables; do not keep duplicate unbundled build trees by default.

## Important build and persistence details

- Trunk can select the `wasm-release` Cargo profile from HTML metadata even when `trunk build` is run without `--release`. When LTO/codegen is too expensive for the runner, local environment overrides can lower build cost without changing checked-in settings; see [WASM build and target notes](./kb/wasm-and-targets.md).
- The release web path also runs `wasm-opt -Oz --converge`, which can outlast hosted tool subprocess limits. A terminated optimizer is not evidence that Rust/WASM compilation failed. Use the exact CI/release path when its size gate is required; otherwise separate `cargo xtask wasm` and package validation, and clearly label any non-release build as such.
- Back up `$HOME/.cargo`, `$HOME/.rustup`, and (when present) `$HOME/.cache/trunk` to persistent `/workspaces` storage before relying on them across a Codespace rebuild. The Trunk cache includes versioned `wasm-bindgen`/`wasm-opt` tools that aren't on `PATH`. Never archive Cargo credentials or signing secrets. See [Codespaces persistence and restore](./kb/codespaces-persistence.md).

## On-demand references

- [WASM build and target notes](./kb/wasm-and-targets.md): target selection, Trunk profile selection, release optimizer, and useful validation commands.
- [Browser JavaScript bridge](./kb/browser-js-bridge.md): the in-memory API boundary, wasm-bindgen/typed-array gotchas, and Node-versus-CLI oracle harness.
- [Codespaces persistence and restore](./kb/codespaces-persistence.md): persistent mount caveats, backup contents, restore steps, and system package recovery.
- [Backup script](./scripts/backup-codespaces-toolchain.sh) and [restore script](./scripts/restore-codespaces-toolchain.sh): preserve and rehydrate the Rust/Cargo installation on the persistent workspace mount.
