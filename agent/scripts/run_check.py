#!/usr/bin/env python3
import os, sys, time, subprocess
LOG = "/home/z/logs/check-wasm.log"
STATUS = "/home/z/logs/check-wasm.status"
def daemonize():
    if os.fork(): sys.exit(0)
    os.setsid()
    if os.fork(): os._exit(0)
    devnull = os.open('/dev/null', os.O_RDWR)
    for fd in (0, 1, 2): os.dup2(devnull, fd)
daemonize()
os.chdir("/home/z/pc-build")
os.environ.update({"PATH": "/home/z/.cargo/bin:" + os.environ.get("PATH", ""), "CARGO_HOME": "/home/z/.cargo", "RUSTUP_HOME": "/home/z/.rustup"})
with open(LOG, "w") as f:
    f.write("starting cargo check --target wasm32\n")
r = subprocess.run(["cargo", "check", "--offline", "--locked", "--target", "wasm32-unknown-unknown", "--profile", "wasm-release", "-p", "photocraft-web", "--features", "heif"], capture_output=True, text=True, timeout=14000)
with open(LOG, "a") as f:
    f.write(r.stdout[-40000:] + "\n=== STDERR ===\n" + r.stderr[-40000:] + f"\nrc={r.returncode}\n")
with open(STATUS, "w") as f:
    f.write(f"rc={r.returncode}\n")
