#!/bin/bash
# trunk-bundle.sh — the wasm release bundle lane (reconstructed REV8; the prior
# trunk-bundle.py lived untracked in a wiped lane — now committed).
# Traps encoded (kb):
#   - trunk 0.21 passes shell color vars to a bool-only flag: strip NO_COLOR /
#     CLICOLOR / CLICOLOR_FORCE / TERM before invoking.
#   - wasm-bindgen + wasm-opt want ~2-3G RSS on this 4G box: the dev server and
#     any vite must be DOWN; kill stray rustc/wasm-opt orphans before starting.
#   - the raw RELEASE _bg.wasm still carries the ~22MB name section; running
#     wasm-opt (any -O level) drops it — that is the 57MB -> ~31MB step.
#   - dist = ../../dist/web (from apps/photocraft-web/Trunk.toml); the bake
#     step (hub scripts/bake-photocraft.sh) copies from there.
set -euo pipefail
LANE=${1:-/home/z/pc-build}
LOG=/home/z/trunk-bundle.log
{
echo "[$(date '+%F %T')] trunk bundle start (lane=$LANE)"
export PATH=/home/z/.cargo/bin:$PATH
export CARGO_HOME=/home/z/.cargo RUSTUP_HOME=/home/z/.rustup
export CARGO_TARGET_DIR="$LANE/target"
unset NO_COLOR CLICOLOR CLICOLOR_FORCE TERM
pkill -f "wasm-opt" 2>/dev/null || true
cd "$LANE/apps/photocraft-web"
trunk build --release --offline
echo "[$(date '+%F %T')] trunk rc=$?"
BG=$(ls "$LANE/dist/web/"photocraft-web-*_bg.wasm | head -1)
RAW=$(stat -c %s "$BG")
echo "raw _bg.wasm: $RAW bytes"
WOPT=$(ls -d /home/z/.cache/trunk/wasm-opt-* | head -1)/bin/wasm-opt
if [ ! -x "$WOPT" ]; then WOPT=$(find /home/z/.cache/trunk -name wasm-opt -type f | head -1); fi
"$WOPT" -O2 "$BG" "$BG.opt" && mv "$BG.opt" "$BG"
echo "[$(date '+%F %T')] wasm-opt rc=$? final: $(stat -c %s "$BG") bytes"
ls -la "$LANE/dist/web/"
echo "[$(date '+%F %T')] trunk bundle DONE"
} >> "$LOG" 2>&1
