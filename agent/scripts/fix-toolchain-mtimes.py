#!/usr/bin/env python3
"""fix-toolchain-mtimes.py — GNU-tar re-extract of the toolchain + trunk caches so every
file carries its ARCHIVE mtime (python tarfile filter='data' stream extraction leaves
write-time mtimes, which cargo reads as 'registry source changed' -> full dep rebuild).
Idempotent; merges over the existing trees.
"""
import os, sys, subprocess

def daemonize():
    if os.fork(): os._exit(0)
    os.setsid()
    if os.fork(): os._exit(0)
    devnull = os.open('/dev/null', os.O_RDWR)
    for fd in (0, 1, 2): os.dup2(devnull, fd)

LOG = "/home/z/logs/toolchain-mtime.log"
STATUS = "/home/z/logs/toolchain-mtime.status"
TOOLCHAIN_ZST = "/home/sync/photocraft-agent/toolchain/rust-toolchain.tar.zst"
TOOLCHAIN_PLAIN = "/tmp/my-project/photocraft-cache/toolchain.tar"
TRUNK_TGZ = "/home/sync/photocraft-agent/toolchain/trunk-cache.tgz"

def log(m):
    with open(LOG, "a") as f: f.write(m + "\n")

def sh(cmd, timeout=1800):
    r = subprocess.run(cmd, capture_output=True, text=True, timeout=timeout)
    if r.returncode != 0:
        raise RuntimeError(f"cmd failed ({r.returncode}): {cmd[:3]}...\n{r.stderr[-1500:]}")
    return r

def main():
    with open(LOG, "w") as f: f.write("start\n")
    try:
        if not os.path.exists(TOOLCHAIN_PLAIN):
            log("1. decompress toolchain zst -> plain tar ...")
            sh([sys.executable, "-c", f"""
import zstandard as zstd, shutil
c = zstd.ZstdDecompressor()
with open({TOOLCHAIN_ZST!r},'rb') as fi, open({TOOLCHAIN_PLAIN!r},'wb') as fo, c.stream_reader(fi) as r:
    shutil.copyfileobj(r, fo, 8*1024*1024)
"""])
        log(f"   plain tar: {os.path.getsize(TOOLCHAIN_PLAIN)/1e9:.2f} GB")
        log("2. GNU tar extract toolchain over /home/z ...")
        sh(["tar", "-xf", TOOLCHAIN_PLAIN, "-C", "/home/z", "--no-same-owner"])
        log("3. GNU tar extract trunk cache ...")
        sh(["tar", "-xzf", TRUNK_TGZ, "-C", "/home/z", "--no-same-owner"])
        # verify: registry src file must carry its archive mtime (crates.io epoch ~2006)
        import glob, time
        probe = glob.glob("/home/z/.cargo/registry/src/*/unicode-ident-*/src/lib.rs")[0]
        mt = time.strftime("%Y-%m-%d", time.localtime(os.path.getmtime(probe)))
        log(f"   registry probe mtime: {mt} (expect the codespace/archive date, NOT today)")
        rustc = sh(["/home/z/.cargo/bin/rustc", "--version"])
        log(f"   rustc: {rustc.stdout.strip()}")
        with open(STATUS, "w") as f: f.write("ok\n")
    except Exception as e:
        log(f"FAILED: {e!r}")
        with open(STATUS, "w") as f: f.write(f"failed: {e}\n")

if __name__ == "__main__":
    if "--foreground" not in sys.argv: daemonize()
    main()
