#!/usr/bin/env python3
"""noop-build test — verify the restored pc-build cache is warm (cargo build ≈ no-op)."""
import os, subprocess, time

def daemonize():
    if os.fork(): os._exit(0)
    os.setsid()
    if os.fork(): os._exit(0)
    devnull = os.open('/dev/null', os.O_RDWR)
    for fd in (0, 1, 2): os.dup2(devnull, fd)

LOG = "/home/z/logs/noop-build.log"
STATUS = "/home/z/logs/noop-build.status"

def run():
    os.chdir("/home/z/pc-build")
    env = dict(os.environ)
    env.update({
        "PATH": "/home/z/.cargo/bin:" + env.get("PATH", ""),
        "CARGO_HOME": "/home/z/.cargo",
        "RUSTUP_HOME": "/home/z/.rustup",
    })
    t0 = time.time()
    r = subprocess.run(
        ["cargo", "build", "--offline", "--locked", "--target", "wasm32-unknown-unknown",
         "--profile", "wasm-release", "-p", "photocraft-web", "--features", "heif"],
        capture_output=True, text=True, env=env, timeout=3600)
    dt = time.time() - t0
    with open(LOG, "w") as f:
        f.write(f"rc={r.returncode} dt={dt:.1f}s\n")
        f.write(r.stdout[-20000:] + "\n=== STDERR ===\n" + r.stderr[-20000:])
    with open(STATUS, "w") as f:
        f.write(f"rc={r.returncode}\ndt={dt:.1f}\n")

if __name__ == "__main__":
    import sys
    if "--foreground" not in sys.argv:
        daemonize()
    run()
