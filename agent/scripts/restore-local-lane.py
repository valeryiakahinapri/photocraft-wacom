#!/usr/bin/env python3
"""restore-local-lane.py — rebuild the local build lane after a sandbox recycle.
Idempotent, double-forked. Restores:
  1. rust toolchain  /home/sync/photocraft-agent/toolchain/rust-toolchain.tar.zst -> /home/z/.cargo|.rustup
  2. trunk cache     /home/sync/photocraft-agent/toolchain/trunk-cache.tgz        -> /home/z/.cache/trunk
  3. warm build dir  /tmp/my-project/photocraft-cache/pc.tar.zst                  -> /home/z/pc-build (./-relative)
Then sed-remaps cargo fingerprints (codespace paths -> local paths) and verifies rustc.
Status: /home/z/logs/restore-lane.{log,status}
"""
import os, sys, tarfile
import zstandard as zstd

LOG = "/home/z/logs/restore-lane.log"
STATUS = "/home/z/logs/restore-lane.status"
TOOLCHAIN = "/home/sync/photocraft-agent/toolchain/rust-toolchain.tar.zst"
TRUNK = "/home/sync/photocraft-agent/toolchain/trunk-cache.tgz"
CACHE = "/tmp/my-project/photocraft-cache/pc.tar.zst"

def daemonize():
    if os.fork(): os._exit(0)
    os.setsid()
    if os.fork(): os._exit(0)
    devnull = os.open('/dev/null', os.O_RDWR)
    for fd in (0, 1, 2): os.dup2(devnull, fd)

def log(msg):
    with open(LOG, "a") as f:
        f.write(msg + "\n")

def extract_zst(path, dest):
    n = 0
    cctx = zstd.ZstdDecompressor()
    with open(path, 'rb') as f, cctx.stream_reader(f) as r, tarfile.open(fileobj=r, mode='r|') as tf:
        tf.extractall(dest, filter='data')
        n += 1
    return n

def remap_fingerprints():
    """Rewrite cargo FINGERPRINT paths: codespace -> local (idempotent, in-place).

    CRITICAL: only ever touch */.fingerprint/* text files (dep-info + json). Binary
    artifacts (deps/*.rmeta, *.rlib, *.so) also contain the paths as raw bytes, but a
    length-changing replacement shifts their offsets -> corrupted metadata -> rustc ICE
    "bytes[len] == STR_SENTINEL". Binaries must stay byte-identical: rustc does not care
    about the stale absolute paths embedded in metadata (proven by the session-2 cache).
    """
    import subprocess
    subs = [
        ("/workspaces/photocraft-wacom", "/home/z/pc-build"),
        ("/home/codespace/.cargo", "/home/z/.cargo"),
    ]
    count = 0
    touched = []
    for dirpath, dirnames, filenames in os.walk("/home/z/pc-build/target"):
        if ".fingerprint" not in dirpath:
            continue
        for fn in filenames:
            p = os.path.join(dirpath, fn)
            try:
                with open(p, 'rb') as f:
                    data = f.read()
                orig = data
                for a, b in subs:
                    data = data.replace(a.encode(), b.encode())
                if data != orig:
                    with open(p, 'wb') as f:
                        f.write(data)
                    count += 1
            except OSError:
                pass
    # Normalize freshness: remapped dep-info files carry mtime=now; every unit's
    # invoked.timestamp must be >= its dep-info mtime, else cargo marks it dirty.
    for dirpath, dirnames, filenames in os.walk("/home/z/pc-build/target"):
        if ".fingerprint" not in dirpath:
            continue
        for fn in filenames:
            if fn == "invoked.timestamp":
                touched.append(os.path.join(dirpath, fn))
    subprocess.run(["touch"] + touched, check=False)
    return count

def main():
    os.makedirs("/home/z/logs", exist_ok=True)
    with open(LOG, "w") as f:
        f.write("restore-lane start\n")
    ok = True
    try:
        if not os.path.exists("/home/z/.cargo/bin/cargo"):
            log("1. extracting toolchain (569M) ...")
            extract_zst(TOOLCHAIN, "/home/z")
        else:
            log("1. toolchain already present")
        os.makedirs("/home/z/.cache", exist_ok=True)
        if not os.path.exists("/home/z/.cache/trunk"):
            log("2. extracting trunk cache ...")
            with tarfile.open(TRUNK) as tf:
                tf.extractall("/home/z", filter='data')
        else:
            log("2. trunk cache already present")
        if not os.path.isdir("/home/z/pc-build/target"):
            log("3. extracting build cache (628M -> ~2.1G) ...")
            os.makedirs("/home/z/pc-build", exist_ok=True)
            extract_zst(CACHE, "/home/z/pc-build")
        else:
            log("3. pc-build already present")
        if os.environ.get("RESTORE_FORCE_TARGET") == "1" or not os.path.isdir("/home/z/pc-build/target/wasm32-unknown-unknown"):
            if os.path.isdir("/home/z/pc-build/target"):
                log("3b. force re-extract target/ (clean binary state) ...")
                import shutil
                shutil.rmtree("/home/z/pc-build/target", ignore_errors=True)
                extract_zst(CACHE, "/home/z/pc-build")
        log("4. fingerprint remap ...")
        n = remap_fingerprints()
        log(f"   remapped {n} files")
        # verify
        r = os.popen("PATH=/home/z/.cargo/bin:$PATH RUSTUP_HOME=/home/z/.rustup CARGO_HOME=/home/z/.cargo /home/z/.cargo/bin/rustc --version 2>&1").read()
        log(f"5. rustc: {r.strip()}")
        du = os.popen("du -sh /home/z/pc-build/target /home/z/.cargo /home/z/.rustup 2>/dev/null").read()
        log("sizes:\n" + du)
        if "1.99.1" not in r:
            log("WARN: rustc version unexpected")
    except Exception as e:
        log(f"FAILED: {e!r}")
        ok = False
    with open(STATUS, "w") as f:
        f.write(("ok" if ok else "failed") + "\n")

if __name__ == "__main__":
    if "--foreground" not in sys.argv:
        daemonize()
    main()
