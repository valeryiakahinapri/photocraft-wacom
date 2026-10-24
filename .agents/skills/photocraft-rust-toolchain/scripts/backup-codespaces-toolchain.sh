#!/bin/sh
set -eu

skill_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
backup_dir=${1:-/workspaces/.photocraft-toolchain-backup}
archive="$backup_dir/rust-toolchain.tar.zst"
tmp_archive="$archive.partial"

if ! mountpoint -q /workspaces; then
  echo "error: /workspaces is not a mounted persistent workspace volume" >&2
  exit 1
fi
if [ ! -x "$HOME/.cargo/bin/rustup" ] || [ ! -d "$HOME/.rustup/toolchains" ]; then
  echo "error: rustup installation not found under $HOME" >&2
  exit 1
fi
. "$HOME/.cargo/env"

mkdir -p "$backup_dir"
chmod 700 "$backup_dir"
rm -f "$tmp_archive"
set -- .cargo .rustup
if [ -d "$HOME/.cache/trunk" ]; then
  set -- "$@" .cache/trunk
fi
tar --zstd -cf "$tmp_archive" \
  --exclude='.cargo/credentials' \
  --exclude='.cargo/credentials.toml' \
  -C "$HOME" "$@"
chmod 600 "$tmp_archive"
mv "$tmp_archive" "$archive"

cp "$skill_dir/assets/ubuntu-apt-packages.txt" "$backup_dir/apt-packages.txt"
cp "$skill_dir/scripts/restore-codespaces-toolchain.sh" "$backup_dir/restore-toolchain.sh"
{
  rustc --version
  cargo --version
  "$HOME/.cargo/bin/rustup" show active-toolchain
  "$HOME/.cargo/bin/rustup" component list --installed
  "$HOME/.cargo/bin/rustup" target list --installed
  "$HOME/.cargo/bin/trunk" --version
  for tool in "$HOME"/.cache/trunk/wasm-bindgen-*/wasm-bindgen "$HOME"/.cache/trunk/wasm-opt-*/bin/wasm-opt; do
    if [ -x "$tool" ]; then
      "$tool" --version
    fi
  done
} > "$backup_dir/toolchain-manifest.txt"
chmod 700 "$backup_dir/restore-toolchain.sh"
chmod 600 "$backup_dir/apt-packages.txt" "$backup_dir/toolchain-manifest.txt"
(
  cd "$backup_dir"
  sha256sum "$(basename "$archive")" > rust-toolchain.tar.zst.sha256
)
chmod 600 "$backup_dir/rust-toolchain.tar.zst.sha256"

echo "Backed up Rust/Cargo toolchain to $archive"
du -h "$archive"
echo "Verify with: cd $backup_dir && sha256sum -c rust-toolchain.tar.zst.sha256"
