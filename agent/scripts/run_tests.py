#!/usr/bin/env python3
"""Double-fork daemon: run the panel_rail kittests natively (cold build, long)."""
import os, sys, time, subprocess

LOG = "/home/z/logs/test-native.log"
STATUS = "/home/z/logs/test-native.status"

def log(msg):
    with open(LOG, "a") as f:
        f.write(f"[{time.strftime('%H:%M:%S')}] {msg}\n")

def daemonize():
    if os.fork(): sys.exit(0)
    os.setsid()
    if os.fork(): os._exit(0)
    devnull = os.open('/dev/null', os.O_RDWR)
    for fd in (0, 1, 2):
        os.dup2(devnull, fd)

def set_status(s):
    with open(STATUS, "w") as f:
        f.write(s + "\n")

daemonize()
set_status("compiling")
log("starting cargo test panel_rail")
os.chdir("/home/z/pc-build")
os.environ.update({
    "PATH": "/home/z/.cargo/bin:" + os.environ.get("PATH", ""),
    "CARGO_HOME": "/home/z/.cargo",
    "RUSTUP_HOME": "/home/z/.rustup",
    "PHOTOCRAFT_PANEL_RAIL_FIXTURES": "/home/z/logs/fixtures",
})
try:
    r = subprocess.run(
        ["cargo", "test", "-p", "photocraft-ui-egui", "--locked", "--offline", "panel_rail", "--", "--nocapture"],
        capture_output=True, text=True, timeout=15000,
    )
    with open(LOG, "a") as f:
        f.write(r.stdout[-60000:] + "\n=== STDERR ===\n" + r.stderr[-60000:] + "\n")
    set_status(f"rc={r.returncode}")
    log(f"finished rc={r.returncode}")
except Exception as e:
    log(f"EXC: {e}")
    set_status("exception")
