# WIP snapshots (read-only rescue copies)

Point-in-time snapshots of the LIVE copilot agent's UNCOMMITTED work in the codespace
(`/workspaces/photocraft-wacom`), taken read-only via `cs exec`/`cs get`. Purpose: the
codespace is the only place this work exists (upstream .gitignore excludes `log/`, and
the files were not yet committed when snapshotted). If the codespace dies before the
agent commits, this branch is the recovery point.

Re-snapshot whenever significant uncommitted work is observed (and the live agent still
owns the tree); never restore INTO the codespace tree without coordinating with the user.

## 2026-10-24T00:22Z

- `js_bridge.rs` — new file, `apps/photocraft-web/src/js_bridge.rs` (576 lines): host-facing
  `globalThis.PhotoCraft` JS API + `PhotoCraftBridge` wasm-bindgen export.
- `js-bridge.mjs` — new file, `apps/photocraft-web/tests/js-bridge.mjs`: Node oracle harness
  (CLI as oracle; WASM_BINDGEN path from trunk cache).
- `uncommitted.diff` — full `git diff` of the tracked-but-modified files at that moment
  (photocraft-web Cargo.toml/main.rs/web.rs, ui-egui panels.rs, docs/development.md,
  docs/ui-design.md; 202 lines).
- Context: a `cargo build --locked --target wasm32-unknown-unknown --profile wasm-release
  -p photocraft-web --features heif` was running at snapshot time.
- Spec: `.agents/skills/photocraft-rust-toolchain/kb/browser-js-bridge.md`.
