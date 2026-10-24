#!/usr/bin/env python3
"""gnu-extract-cache.py — bulletproof cache restore: zstd-decompress to a plain tar on
PolarFS, extract with GNU tar (deterministic mtimes), verify sentinel bytes, then run the
fixed fingerprint-only remap + invoked.timestamp normalization. Any verification failure
aborts with status=failed and the exact reason.
"""
import os, sys, subprocess

def daemonize():
    if os.fork(): os._exit(0)
    os.setsid()
    if os.fork(): os._exit(0)
    devnull = os.open('/dev/null', os.O_RDWR)
    for fd in (0, 1, 2): os.dup2(devnull, fd)

LOG = "/home/z/logs/gnu-extract.log"
STATUS = "/home/z/logs/gnu-extract.status"
ZST = "/tmp/my-project/photocraft-cache/pc.tar.zst"
PLAIN = "/tmp/my-project/photocraft-cache/pc.tar"
BUILD = "/home/z/pc-build"
SENTINEL = "target/wasm32-unknown-unknown/wasm-release/deps/libwasm_bindgen-b84b6fc318c7cddd.rmeta"
SENTINEL_SIZE = 2772405

def log(m):
    with open(LOG, "a") as f: f.write(m + "\n")

def sh(cmd, **kw):
    r = subprocess.run(cmd, capture_output=True, text=True, **kw)
    if r.returncode != 0:
        log(f"CMD FAILED ({r.returncode}): {' '.join(cmd)}\n{r.stderr[-2000:]}")
        raise RuntimeError(f"cmd failed: {cmd[0]}")
    return r

def main():
    with open(LOG, "w") as f: f.write("gnu-extract start\n")
    try:
        # 1. decompress to plain tar (PolarFS) — skip if already done and newer than zst
        if not (os.path.exists(PLAIN) and os.path.getmtime(PLAIN) > os.path.getmtime(ZST)):
            log("1. decompressing zst -> plain tar (PolarFS) ...")
            sh([sys.executable, "-c", f"""
import zstandard as zstd, shutil
c = zstd.ZstdDecompressor()
with open({ZST!r},'rb') as fi, open({PLAIN!r},'wb') as fo, c.stream_reader(fi) as r:
    shutil.copyfileobj(r, fo, 8*1024*1024)
"""], timeout=1200)
        log(f"   plain tar: {os.path.getsize(PLAIN)/1e9:.2f} GB")
        # 2. nuke target
        log("2. rm -rf pc-build/target ...")
        subprocess.run(["rm", "-rf", f"{BUILD}/target"], check=True)
        # 3. GNU tar extract (sets archive mtimes; --no-same-owner for user-z)
        log("3. GNU tar extract ...")
        sh(["tar", "-xf", PLAIN, "-C", BUILD, "--no-same-owner"], timeout=1200)
        # 4. sentinel verification
        log("4. verifying sentinel bytes ...")
        p = f"{BUILD}/{SENTINEL}"
        data = open(p, "rb").read()
        assert len(data) == SENTINEL_SIZE, f"sentinel size {len(data)} != {SENTINEL_SIZE}"
        assert b"/home/codespace/.cargo" in data, "sentinel missing codespace path"
        assert b"/home/z/.cargo" not in data, "sentinel STILL contains remap signature!"
        import time
        mtime = time.strftime("%m-%d %H:%M", time.localtime(os.path.getmtime(p)))
        log(f"   sentinel OK: size={len(data)}, mtime={mtime} (archive value)")
        # 5. fingerprint-only remap
        log("5. fingerprint remap (text files only) ...")
        n = 0; touched = []
        subs = [(b"/workspaces/photocraft-wacom", b"/home/z/pc-build"),
                (b"/home/codespace/.cargo", b"/home/z/.cargo")]
        for dirpath, dirnames, filenames in os.walk(f"{BUILD}/target"):
            if ".fingerprint" not in dirpath: continue
            for fn in filenames:
                fp = os.path.join(dirpath, fn)
                if fn == "invoked.timestamp":
                    touched.append(fp); continue
                try:
                    d = open(fp, "rb").read()
                except OSError: continue
                o = d
                for a, b in subs: d = d.replace(a, b)
                if d != o:
                    open(fp, "wb").write(d); n += 1
        subprocess.run(["touch"] + touched, check=False)
        log(f"   remapped {n} fingerprint files, touched {len(touched)} timestamps")
        with open(STATUS, "w") as f: f.write("ok\n")
    except Exception as e:
        log(f"FAILED: {e!r}")
        with open(STATUS, "w") as f: f.write(f"failed: {e}\n")

if __name__ == "__main__":
    if "--foreground" not in sys.argv: daemonize()
    main()
