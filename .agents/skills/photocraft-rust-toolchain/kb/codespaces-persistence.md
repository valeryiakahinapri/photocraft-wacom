# Codespaces toolchain persistence

## What persists

In this environment, `/workspaces` is a separate ext4 mount (`/dev/loop4`) and is the appropriate place for backups intended to survive a Codespace container rebuild. `$HOME` is on the container's overlay filesystem; do not assume `.cargo` or `.rustup` there will survive a rebuild. A stop/start and a rebuild are different lifecycle events, so persistent storage should be verified rather than inferred.

The current backup is `/workspaces/.photocraft-toolchain-backup/rust-toolchain.tar.zst` (SHA-256 sidecar, restore script, apt package list, and toolchain manifest are alongside it). It contains the current host Rust stable toolchain, rust-analyzer, rustfmt, Clippy, Trunk, downloaded Cargo sources, and only the requested Linux/macOS/WASM targets.

Keep a toolchain archive and restore instructions outside the Git repository, for example:

```text
/workspaces/.photocraft-toolchain-backup/
  rust-toolchain.tar.zst
  rust-toolchain.tar.zst.sha256
  apt-packages.txt
  toolchain-manifest.txt
  restore-toolchain.sh
```

The archive should contain `$HOME/.cargo` and `$HOME/.rustup` so it preserves rustup's host toolchain, installed Rust targets/components, `cargo` and Trunk executables, downloaded crate sources, and pinned git dependency sources. Also include `$HOME/.cache/trunk` when present: it holds Trunk's versioned `wasm-bindgen` and `wasm-opt` binaries, which are not on `PATH` and otherwise must be re-downloaded. Exclude credential files (for example `.cargo/credentials` and `.cargo/credentials.toml`) and never include signing keys, tokens, or project secrets. Set archive permissions so only the Codespace user can read it.

Create the archive from `$HOME` with relative paths so it restores under a different home directory:

```sh
set -- .cargo .rustup
[ ! -d "$HOME/.cache/trunk" ] || set -- "$@" .cache/trunk
tar --zstd -cf /workspaces/.photocraft-toolchain-backup/rust-toolchain.tar.zst \
  --exclude='.cargo/credentials' \
  --exclude='.cargo/credentials.toml' \
  -C "$HOME" "$@"
sha256sum /workspaces/.photocraft-toolchain-backup/rust-toolchain.tar.zst \
  > /workspaces/.photocraft-toolchain-backup/rust-toolchain.tar.zst.sha256
```

After a rebuild, verify the hash, extract to the new `$HOME`, then load the Cargo environment:

```sh
cd /workspaces/.photocraft-toolchain-backup
sha256sum -c rust-toolchain.tar.zst.sha256
tar --zstd -xf rust-toolchain.tar.zst -C "$HOME"
. "$HOME/.cargo/env"
rustc --version
cargo --version
rustup target list --installed
```

Do not overwrite an existing non-empty `.cargo` blindly; inspect first and merge or move aside intentionally. Shell profile hooks that source `$HOME/.cargo/env` may need re-adding after a full container rebuild.

## System packages are separate from Rust

Rust/Cargo caches do not include Linux native development libraries installed by apt. Keep a short package manifest, not a whole OS image. For the documented Ubuntu desktop build, the project guide lists `libxkbcommon-dev`, `libwayland-dev`, `libx11-dev`, `libxrandr-dev`, `libxi-dev`, `libgl1-mesa-dev`, and `libgtk-3-dev`; retain the package list actually installed for this machine in `apt-packages.txt`. Reinstall with `sudo apt-get update && xargs -r sudo apt-get install -y < apt-packages.txt` when needed. A headless CLI-only build may not need the desktop packages.

Check free space before backing up or restoring. Rust stdlibs/caches, `target/`, and unbundled `dist/web` outputs can grow substantially; keep only targets and artifacts needed for the task. Rustup can remove unneeded target components, while individual generated target directories can be removed after confirming they contain only disposable build outputs.
