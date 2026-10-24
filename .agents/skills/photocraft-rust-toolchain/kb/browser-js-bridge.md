# Browser JavaScript bridge

Use this note when extending the public JavaScript surface of `photocraft-web`.

## Project-specific boundary

- The web shell is the same Rust `PhotocraftApp` running under eframe/WASM. Before this bridge was added, `wasm-bindgen` only connected browser services (dialogs, downloads, IndexedDB, pointer input); those internal bindings were not a host-facing editor API.
- Keep the HTML page free of hand-written scripts. Export the API from Rust/WASM and install the page-global `globalThis.PhotoCraft` from the web shell. The Rust `PhotoCraftBridge` class is also a direct `wasm-bindgen` export.
- Use `photocraft_engine::Session` and `photocraft_engine::automate_cmds::parse_action` so command IDs, action-step shapes, and results stay shared with the CLI. Keep command-list rows and case-insensitive id/label/keyword filtering aligned with `photocraft-cli commands --json --filter`. The CLI's `photocraft_automation::Headless` is not an appropriate browser dependency: it includes server, Tokio, and filesystem-capability code.
- The browser boundary is byte-in/byte-out, never ambient paths. `commands`, `info`, `run`, `convert`, `export`, and in-memory `batch` have browser equivalents; filesystem path protection, droplets over directories, MCP, and local TCP/stdio servers do not. Document those limits rather than inventing browser-local filesystem semantics.
- The host-facing bridge owns an independent headless `Session`; it does not mutate the eframe canvas's visible editor session. Keep this explicit in API docs so a host does not assume it is controlling the open GUI document.
- Pass file contents as `Uint8Array`. Do not round-trip typed arrays through `JSON.stringify`: it turns them into index-keyed objects and creates large per-byte JSON copies. Use wasm-bindgen's `Vec<u8>` conversion or `Uint8Array::to_vec`, and return encoded files as typed arrays too.
- Preserve CLI export option names and validate unknown keys and ranges. CLI parity should be checked on results and encoded bytes, while normalizing expected browser-only metadata such as local paths.

## Oracle testing without a browser

The Node harness at `apps/photocraft-web/tests/js-bridge.mjs` loads a `wasm-bindgen --target nodejs` wrapper, exercises the actual WASM exports/global namespace, and invokes `photocraft-cli` for the oracle. Build the CLI and raw `wasm-release` app first, and pass the cached Trunk tool path explicitly when needed:

```sh
cargo build --locked -p photocraft-cli
cargo build --locked --target wasm32-unknown-unknown --profile wasm-release -p photocraft-web --features heif
WASM_BINDGEN="$HOME/.cache/trunk/wasm-bindgen-<version>/wasm-bindgen" \
  node apps/photocraft-web/tests/js-bridge.mjs
```

This keeps the harness independent of a browser driver and tests the generated `Uint8Array`/JSON boundary. The web app's Rust startup checks for a DOM and returns harmlessly under Node, but its exported bridge remains callable. Use a real browser separately for UI, rendering, file-dialog, and download workflows; Node does not validate those.

Engine-generated layer IDs can differ between otherwise equivalent CLI and WASM sessions because they are process-global allocations. Normalize only those generated identifiers when comparing command-result JSON; still compare command IDs, other result fields, document info, and encoded output bytes exactly.

The WASM bridge runs synchronous engine work on the page thread. Do not describe it as non-blocking merely because JavaScript receives a WASM export; long operations need a deliberate worker/async architecture before being advertised for unattended batches.
