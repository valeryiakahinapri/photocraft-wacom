#!/usr/bin/env python3
"""trunk-bundle.py — build the static web bundle (dist/web) from the warm pc-build tree.
Double-forked; status/log under /home/z/logs/.
"""
import os, sys, subprocess, time

def daemonize():
    if os.fork(): os._exit(0)
    os.setsid()
    if os.fork(): os._exit(0)
    devnull = os.open('/dev/null', os.O_RDWR)
    for fd in (0, 1, 2): os.dup2(devnull, fd)

LOG = "/home/z/logs/trunk-bundle.log"
STATUS = "/home/z/logs/trunk-bundle.status"

def main():
    os.chdir("/home/z/pc-build/apps/photocraft-web")
    env = dict(os.environ)
    env.update({
        "PATH": "/home/z/.cargo/bin:" + env.get("PATH", ""),
        "CARGO_HOME": "/home/z/.cargo",
        "RUSTUP_HOME": "/home/z/.rustup",
    })
    # The NO_COLOR trap: trunk 0.21 passes color env values to --no-color, which only
    # accepts true/false — NO_COLOR=1 AND CLICOLOR_FORCE=0 from the shell both abort it
    # instantly (rc=2). Strip them all.
    for var in ("NO_COLOR", "CLICOLOR", "CLICOLOR_FORCE", "FORCE_COLOR"):
        env.pop(var, None)
    env["NO_COLOR"] = "false"
    t0 = time.time()
    r = subprocess.run(["trunk", "build", "--release"], capture_output=True, text=True, env=env, timeout=1800)
    dt = time.time() - t0
    with open(LOG, "w") as f:
        f.write(f"rc={r.returncode} dt={dt:.1f}s\n" + r.stdout[-15000:] + "\n=== STDERR ===\n" + r.stderr[-15000:])
    with open(STATUS, "w") as f:
        f.write(f"rc={r.returncode}\ndt={dt:.1f}\n")

if __name__ == "__main__":
    if "--foreground" not in sys.argv: daemonize()
    main()
