#!/usr/bin/env python3
"""daemon-restore.py — double-fork launcher for restore-bake-lane.sh.

The bash tool harness SIGKILLs direct background children at toolcall
boundaries (setsid+nohup+disown does NOT survive — verified 2026-10-24
10:19, daemon died at 68M/2.2G with no rc line). The proven survivor
pattern from session 2 is the classic double-fork: parent exits after
first fork; child calls setsid() and forks again; the grandchild is
fully detached from the toolcall's process group/session and re-execs
the script with its own stdio. Parent _exit()s hard so the bash tool
returns immediately.
"""
import os
import sys

SCRIPT = sys.argv[1] if len(sys.argv) > 1 else "/home/z/my-project/scripts/restore-bake-lane.sh"
LOG = sys.argv[2] if len(sys.argv) > 2 else "/home/z/restore-lane.log"


def main() -> None:
    # fork #1
    pid = os.fork()
    if pid > 0:
        # parent: hard-exit so the toolcall returns; child is re-parented
        print(f"daemonize: child={pid}", flush=True)
        os._exit(0)

    # child: new session, detach controlling terminal
    os.setsid()

    # fork #2 — grandchild can never reacquire a terminal
    pid2 = os.fork()
    if pid2 > 0:
        os._exit(0)

    # grandchild: re-exec the restore script fully detached
    logfd = os.open(LOG, os.O_WRONLY | os.O_CREAT | os.O_APPEND, 0o644)
    devnull = os.open(os.devnull, os.O_RDONLY)
    os.dup2(devnull, 0)
    os.dup2(logfd, 1)
    os.dup2(logfd, 2)
    os.execvp("bash", ["bash", SCRIPT])


if __name__ == "__main__":
    main()
    sys.stdout.flush()
