#!/bin/bash
# restore-bake-lane.sh — restore Rust toolchain + warm pc-build cache from durable lanes.
# Session-2 lessons baked in:
#   - GNU tar ONLY. python tarfile with filter='data' corrupted .rmeta binaries
#     (path-remap shifted bytes) and reset mtimes → mass cache invalidation.
#   - These staged PLAIN tars (PolarFS) are the proven-pristine artifacts.
#   - Preserve mtimes: tar extracts with archive mtimes by default. Do NOT touch.
set -u
LOG=/home/z/restore-lane.log
P=/tmp/my-project/photocraft-cache

log() { echo "[$(date '+%F %T')] $*" >> "$LOG"; }

log "=== lane restore start ==="

# 1) Toolchain (.cargo + .rustup) → /home/z
if [ ! -x /home/z/.cargo/bin/rustc ]; then
  log "[1/3] extracting toolchain.tar → /home/z (2.2G, few min)"
  rm -rf /home/z/.cargo /home/z/.rustup
  tar -xf "$P/toolchain.tar" -C /home/z --overwrite >> "$LOG" 2>&1
  log "[1/3] toolchain extract rc=$?"
else
  log "[1/3] toolchain already present, skip"
fi

# 2) Trunk tool cache (wasm-bindgen 0.2.129 + wasm-opt v123) → /home/z/.cache/trunk
if [ ! -d /home/z/.cache/trunk ]; then
  log "[2/3] extracting trunk-cache.tgz → /home/z/.cache"
  mkdir -p /home/z/.cache
  tar -xzf /home/sync/photocraft-agent/toolchain/trunk-cache.tgz -C /home/z/.cache >> "$LOG" 2>&1
  log "[2/3] trunk cache extract rc=$?"
else
  log "[2/3] trunk cache already present, skip"
fi

# 3) pc-build tree: repo source + 2.0G warm target/ → /home/z/pc-build
if [ ! -f /home/z/pc-build/Cargo.toml ]; then
  log "[3/3] extracting pc.tar → /home/z/pc-build (2.2G, few min)"
  mkdir -p /home/z/pc-build
  tar -xf "$P/pc.tar" -C /home/z/pc-build >> "$LOG" 2>&1
  log "[3/3] pc.tar extract rc=$?"
else
  log "[3/3] pc-build already present, skip"
fi

# Verification block
{
  echo "--- verify ---"
  export PATH=/home/z/.cargo/bin:$PATH
  export CARGO_HOME=/home/z/.cargo
  export RUSTUP_HOME=/home/z/.rustup
  rustc --version 2>&1
  cargo --version 2>&1
  trunk --version 2>&1
  ls /home/z/pc-build/target/wasm32-unknown-unknown 2>&1 | head -3
  echo "git in pc-build: $(ls -d /home/z/pc-build/.git 2>/dev/null || echo NONE)"
} >> "$LOG" 2>&1

log "=== lane restore DONE ==="
