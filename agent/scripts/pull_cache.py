#!/usr/bin/env python3
"""Double-fork daemon: pull pc.tar.zst parts from codespace, reassemble, verify sha256."""
import os, sys, time, subprocess

LOG = "/home/z/logs/pull-cache.log"
STATUS = "/home/z/logs/pull-cache.status"
SHA = "38d48115e7857c700e3df71974d882ff239d68c730ba2c985a13695464a554d8"

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
set_status("running")
os.makedirs("/home/z/logs", exist_ok=True)
os.makedirs("/home/z/pull", exist_ok=True)
os.chdir("/home/z/zai-codespaces")
env = dict(os.environ, ZAI_ROOT="/home/z/zai-codespaces", CS_ACCOUNT="photocraft",
           CS_EXEC_TIMEOUT="1800")

def run(cmd, timeout=1900):
    log("RUN: " + " ".join(cmd))
    try:
        r = subprocess.run(cmd, env=env, capture_output=True, text=True, timeout=timeout)
        log(f"rc={r.returncode} out_tail={r.stdout[-200:]!r} err_tail={r.stderr[-200:]!r}")
        return r.returncode, r.stdout
    except Exception as e:
        log(f"EXC: {e}")
        return 1, ""

ok = True
for part in ["00", "01"]:
    set_status(f"getting part {part}")
    rc, _ = run(["timeout", "1800", "bash", "kit/cs", "get",
                 f"/tmp/pull/pc.part-{part}", f"/home/z/pull/pc.part-{part}"])
    if rc != 0:
        ok = False
        break
    log(f"part {part} downloaded")

if ok:
    set_status("reassembling")
    rc, out = run(["bash", "-c",
                   "cat /home/z/pull/pc.part-00 /home/z/pull/pc.part-01 > /home/z/pull/pc.tar.zst && sha256sum /home/z/pull/pc.tar.zst"])
    if rc == 0 and SHA in out:
        set_status("OK")
        log("SHA256 verified: " + SHA[:16])
    else:
        set_status("SHA-MISMATCH")
        log(f"sha check failed: {out[:200]}")
else:
    set_status("FAILED")
log("daemon finished")
