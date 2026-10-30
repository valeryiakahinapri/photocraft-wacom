#!/bin/bash
# trunk-bundle.sh — the wasm release bundle lane (reconstructed REV8; the prior
# trunk-bundle.py lived untracked in a wiped lane — now committed).
# Traps encoded (kb):
#   - trunk 0.21 passes shell color vars to a bool-only flag: strip NO_COLOR /
#     CLICOLOR / CLICOLOR_FORCE / TERM before invoking.
#   - wasm-bindgen + wasm-opt want ~2-3G RSS on this 4G box: the dev server and
#     any vite must be DOWN; kill stray rustc/wasm-opt orphans before starting.
#   - index.html's data-wasm-opt="z" makes trunk itself run wasm-opt (the ~22MB name
#     section of the raw cargo output never reaches dist) — S11: do NOT add a second
#     pass; it grew 685069da by +39K and version_123 broke the old CLI syntax.
#   - dist = ../../dist/web (from apps/photocraft-web/Trunk.toml); the bake
#     step (hub scripts/bake-photocraft.sh) copies from there.
set -euo pipefail
LANE=${1:-/home/z/pc-build}
LOG=/home/z/trunk-bundle.log
{
echo "[$(date '+%F %T')] trunk bundle start (lane=$LANE)"
export PATH=/home/z/.cargo/bin:$PATH
export CARGO_HOME=/home/z/.cargo RUSTUP_HOME=/home/z/.rustup
# REV8 trap: the warm cache was built with these — without them a thin-LTO cold build eats 2G+ and dies.
export CARGO_PROFILE_WASM_RELEASE_LTO=false CARGO_PROFILE_WASM_RELEASE_CODEGEN_UNITS=16
export CARGO_TARGET_DIR="$LANE/target"
unset NO_COLOR CLICOLOR CLICOLOR_FORCE TERM
pkill -f "wasm-opt" 2>/dev/null || true
cd "$LANE/apps/photocraft-web"
trunk build --release
echo "[$(date '+%F %T')] trunk rc=$?"
BG=$(ls "$LANE/dist/web/"photocraft-web-*_bg.wasm | head -1)
RAW=$(stat -c %s "$BG")
echo "final _bg.wasm: $RAW bytes"
# S11: NO post-trunk wasm-opt pass. index.html's data-wasm-opt="z" already runs it inside
# trunk (the raw 57MB cargo output never lands in dist). Measured on 685069da: the extra
# -O2 GREW the bundle by +39,187 bytes, and wasm-opt version_123 also rejects the old
# two-positional-args CLI ("Unexpected second positional argument ... for INFILE").
ls -la "$LANE/dist/web/"
echo "[$(date '+%F %T')] trunk bundle DONE"
} >> "$LOG" 2>&1
