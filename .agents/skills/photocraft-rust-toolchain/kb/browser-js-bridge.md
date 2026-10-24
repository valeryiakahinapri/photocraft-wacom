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

## Coexistence with the embed host bridge (2026-10-24, session 4)

Upstream main now carries TWO bridges, and they coexist by design:

- **The engine bridge (upstream, `7d6f9df9`)**: `globalThis.PhotoCraft` +
  the direct `wasm-bindgen` class export. Headless `Session`, CLI-parity
  commands (`commands`/`info`/`run`/`convert`/`export`/`batch`),
  byte-in/byte-out `Uint8Array` boundary. It never touches the visible
  eframe editor session — hosts drive it for automation/headless work.
- **The host bridge (embed lane, `web-cloud-library-rebase` branch —
  `apps/photocraft-web/src/host_bridge.rs`)**: the `postMessage` channel
  for embedding the CANVAS app in a host page. It routes the platform
  seams: `open`→inbox ingest, `library.list`/`library.open` (in-app cloud
  browser), the save-request queue + `saved-from-write` (real Blob,
  guard-exact shapes), docs/dirty truth, esc, `save-all-done`, prefs
  pre-seed, and (PHASE-B) the doc↔asset identity echo — `open` captures
  the host asset id into a pending slot, `write` consumes it and echoes
  the id back so the host UPDATEs the minted asset instead of minting a
  sibling; `save-rename` re-keys, doc-close prunes, and flat-format
  saves (identity formats only: psd/psb/pcraft…) park nothing.

Rules: extend the ENGINE bridge for anything a script/automation host
needs (keep CLI parity, keep the page script-free law); extend the HOST
bridge for anything the embedded MDI experience needs (menu arming,
identity, library). Never route canvas-app UX through the headless
Session. The rebase that merged the two lives on
`web-cloud-library-rebase` (tip carries the identity test fix); the OLD
`web-cloud-library` branch is history. The GenEditor side of the
protocol (`protocol.ts`, the modal, the store) is documented in the
webhost repo's `EMBED-DESIGN-PHASEB.md` — keep both sides in sync when
touching message shapes.
