#!/bin/sh
set -eu

backup_dir=${1:-/workspaces/.photocraft-toolchain-backup}
archive="$backup_dir/rust-toolchain.tar.zst"

if [ ! -f "$archive" ] || [ ! -f "$backup_dir/rust-toolchain.tar.zst.sha256" ]; then
  echo "error: Rust toolchain backup is incomplete in $backup_dir" >&2
  exit 1
fi
(
  cd "$backup_dir"
  sha256sum -c rust-toolchain.tar.zst.sha256
)

for path in "$HOME/.cargo" "$HOME/.rustup" "$HOME/.cache/trunk"; do
  if [ -e "$path" ]; then
    echo "error: $path already exists; inspect or move it aside before restoring" >&2
    exit 1
  fi
done

tar --zstd -xf "$archive" -C "$HOME"

for profile in "$HOME/.profile" "$HOME/.bashrc"; do
  touch "$profile"
  if ! grep -Fq '. "$HOME/.cargo/env"' "$profile"; then
    printf '\n. "$HOME/.cargo/env"\n' >> "$profile"
  fi
done

. "$HOME/.cargo/env"
rustc --version
cargo --version
rustup target list --installed
echo "Toolchain restored. Reinstall Linux libraries from $backup_dir/apt-packages.txt if required."
