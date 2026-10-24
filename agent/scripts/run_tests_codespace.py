#!/usr/bin/env python3
"""Double-fork daemon: run the panel_rail kittests on the CODESPACE in an isolated
worktree + /tmp target dir (the live agent's tree and caches stay untouched)."""
import os, sys, time, subprocess

LOG = "/home/z/logs/test-codespace.log"
STATUS = "/home/z/logs/test-codespace.status"

SCRIPT = r"""set -e
export PATH="/home/codespace/.cargo/bin:$PATH"
cd /workspaces/photocraft-wacom
git fetch origin feature/right-panel-collapse 2>&1 | tail -1
git worktree remove --force /tmp/panel-rail-test 2>/dev/null || true
rm -rf /tmp/panel-rail-test
git worktree add --detach /tmp/panel-rail-test origin/feature/right-panel-collapse 2>&1 | tail -1
cd /tmp/panel-rail-test
export PHOTOCRAFT_PANEL_RAIL_FIXTURES=/tmp/fixtures
export CARGO_TARGET_DIR=/tmp/test-target
cargo test -p photocraft-ui-egui --locked panel_rail -- --nocapture
echo "TEST_RC=$?"
"""

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
log("launching codespace kittest run")
os.chdir("/home/z/zai-codespaces")
env = dict(os.environ, ZAI_ROOT="/home/z/zai-codespaces", CS_ACCOUNT="photocraft", CS_EXEC_TIMEOUT="3300")
try:
    r = subprocess.run(["bash", "kit/cs", "exec"], input=SCRIPT.encode(), capture_output=True, timeout=3400, env=env)
    out = (r.stdout or b"")[-80000:].decode(errors="replace")
    err = (r.stderr or b"")[-20000:].decode(errors="replace")
    with open(LOG, "a") as f:
        f.write(out + "\n=== STDERR ===\n" + err + "\n")
    rc_marker = "TEST_RC="
    if "TEST_RC=0" in out:
        set_status("OK")
    else:
        set_status(f"rc={r.returncode}")
    log(f"finished rc={r.returncode}, TEST_RC marker: {[l for l in out.splitlines() if 'TEST_RC' in l]}")
except Exception as e:
    log(f"EXC: {e}")
    set_status("exception")
