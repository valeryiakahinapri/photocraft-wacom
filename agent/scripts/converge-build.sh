#!/bin/bash
# converge-build.sh — one-time cache-convergence build of the CURRENT pc-build tree.
# Warms the ~600 shared dep crates (their fingerprints survive the source sync);
# workspace crates rebuild after the merged source lands anyway.
# Log: /home/z/converge.log  (rc + last lines on completion)
set -u
L=/home/z/converge.log
{
echo "[$(date '+%F %T')] convergence build start"
PATH=/home/z/.cargo/bin:$PATH
export CARGO_HOME=/home/z/.cargo RUSTUP_HOME=/home/z/.rustup
export CARGO_PROFILE_WASM_RELEASE_LTO=false CARGO_PROFILE_WASM_RELEASE_CODEGEN_UNITS=16
cd /home/z/pc-build
cargo build --offline --locked --target wasm32-unknown-unknown --profile wasm-release -p photocraft-web --features heif
RC=$?
echo "[$(date '+%F %T')] convergence build rc=$RC"
exit $RC
} >> "$L" 2>&1
