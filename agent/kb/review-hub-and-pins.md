# Review hub: static-baking photocraft + annotakit in the Z container (and the PIN-in-webgpu answer)

Written 2026-10-24 (session 3). Everything here is E2E-verified in THIS container class.

## The hub layout (all on the one :3000 Next.js dev server)

- `/` — the hub landing page (source: `agent/review-hub/page.tsx`, platform app `src/app/page.tsx`)
- `/photocraft` — the trunk wasm bundle baked into `public/photocraft/` (dev-grade
  lto=false build; ~30MB wasm — the 24MiB packaging gate is a release-lane concern)
- `/storybook` — `storybook build` static output + annotakit's bake, under `public/storybook/`
- Regenerate: `agent/scripts/bake-review-hub.sh` (idempotent, from a recycled container)

## The traps (each cost real time today)

1. **Trailing-slash trap on /photocraft**: Next serves the bake at the BARE path, so
   the page's directory is `/` — every relative asset ref (the wasm fetch included)
   resolves at root and 404s; the app hangs on "Loading PhotoCraft…" forever.
   Fix: inject `<base href="/photocraft/">` into the baked index.html (annotakit's
   bake script does exactly this for /storybook with `--base`).
2. **Vite vs the hub's postcss**: `bunx storybook build` loads the hub's
   `postcss.config.mjs` (Tailwind 4) and dies "Invalid PostCSS Plugin". Neutralize in
   `.storybook/main.ts` (`viteFinal: config.css.postcss = {plugins: []}`) — stories use
   inline styles anyway.
3. **SB10 renders NO addon toolbar on the root route** (`?path=/`): the manager looks
   broken, zero toolbar buttons. Navigate to the STORY route
   (`?path=/story/review-photocraft--app`) — pin/region/freeze/threads all appear.
4. **eslint OOM**: `bun run lint` babel-parses `public/` bundles (1.2MB minified) and
   gets SIGKILLed. Add `"public/**", ".storybook/**"` to eslint ignores.
5. **The dev server must restart** after `next.config.ts` edits (rewrites read at boot);
   relaunch is a double-forked `bun run dev` (see the bake script step 6).
6. **Trunk's color-env trap** (rc=2, 0.1s, "invalid value '1' for '--no-color'"): trunk
   0.21 passes the shell's color vars to a bool-only flag — `NO_COLOR=1` AND
   `CLICOLOR_FORCE=0` both abort it. Strip all four color vars in build scripts.
7. **wasm-bindgen SIGKILL during trunk**: it shares the 4G no-swap box with the dev
   server (~550MB) — kill stray rustc/wasm-opt orphans before bundling (a killed
   interactive trunk leaves its wasm-opt child running at ~1G RSS!).

## The PIN-in-webgpu answer (see agent/kb/pin-webgpu-report.md for the full research)

- **AccessKit is a verified dead end on web in this stack** (eframe 0.36 discards the
  tree: `accesskit_update: _, // not currently implemented`; accesskit_winit compiles a
  null adapter on wasm; no web adapter crate exists). No feature flag fixes it.
- **What works TODAY (E2E-verified): the in-canvas mount + region pins.** Mount the wasm
  app DIRECTLY in the story canvas doc (web.rs boots against `#photocraft_canvas` by
  element id — it does NOT need its own page): the story builds the canvas + loading
  div, reads the hashed asset names from the baked index.html, dynamic-imports the
  module, and guards double-boot (a story remount ADOPTS the live canvas instead of
  re-initializing). One DOM = the annotakit overlay coexists with the app canvas:
  ⌥B → drag → composer → "Pin it" → **region thread #1 pinned on the live wgpu UI**,
  persisted to localStorage (the static store). An IFRAME story CANNOT work — pointer
  events never bubble out of a nested iframe, so the drag dies inside the app.
- **Element-anchored pins = Route B (designed, not yet built)**: `PhotoCraft.ui({op:"rects"})`
  bridge command streaming `{key, role, label, rect}` (dock.rs `last_rects` already
  publishes group rects) + an overlay of transparent divs with stable
  `data-testid`s, `pointer-events:none` until annotakit's pin mode arms (its
  `body.annota-cursor` class). PoC ≈ 300-450 lines; full plan in the report.
- **The GitHub mirror is NOT armed** (deliberately): annotakit-gh.json is world-readable
  by design; baking the full-scope work PAT would hand the repo to anyone with the
  preview URL. It needs a fine-grained PAT (Issues read/write, ONE repo) from the owner,
  or the reviewer configures publishing in the panel at runtime.

## Local build-lane repairs (the morning's archaeology, distilled)

- **python tarfile `filter='data'` does NOT preserve mtimes in stream mode** — every
  extracted file gets write-time mtimes, which cargo reads as "every registry source
  changed" → a full ~600-dep rebuild on every restore (session-2's unexplained 8m30s).
  **Always decompress zst → plain tar → GNU `tar -xf`** (sets archive mtimes; verified
  by the registry probe reading 2006 — crates.io's fixed epoch — instead of today).
- **Never byte-replace paths inside target/ BINARY artifacts** (rmeta/rlib embed source
  paths; a length-changing replacement shifts every byte after it → rustc ICE
  `bytes[len] == STR_SENTINEL`). Remap ONLY `*/.fingerprint/*` text files, then touch
  every `invoked.timestamp` (a dep-info file newer than the timestamp = dirty unit).
  Artifacts with stale absolute paths inside decode fine — proven.
- The restore pipeline: `scripts/gnu-extract-cache.py` (verify-gated) +
  `scripts/fix-toolchain-mtimes.py` + `scripts/restore-local-lane.py`.
- **Sub-agents with bash WILL run your build scripts** if their task touches them —
  explicitly forbid builds in research prompts (today a reviewer "verified" the build
  state and raced the toolchain re-extraction: rustc exit 127, libLLVM unreadable).

## UI-command dispatch (the bug compile gates cannot catch)

`window.toggle.dock` and every UI-level id live in the ENGINE REGISTRY as metadata but
only `menus::invoke(app, &ctx, id, params)` EXECUTES them — `app.run()` goes straight
to the engine and returns "unknown command '…'" in the status bar. Session-2's popover
had this bug, my first rail toggle had it, and the kittests (never executed) encoded
it too — the repo's own dock_tests always wrapped `menus::invoke`. Only live E2E
caught it: baked the app, clicked the real rail toggle, read the status bar, fixed all
call sites, re-baked, re-clicked — the dock collapsed. **The E2E loop (bake →
agent-browser CDP clicks → pixel-diff + status-bar reads) is now the only execution
proof that exists for the panel feature; the kittests still need the codespace lane.**
