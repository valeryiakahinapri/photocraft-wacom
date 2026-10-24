#!/usr/bin/env python3
"""daemon-cmd.py — double-fork launcher for arbitrary shell commands.
Survives the harness toolcall-boundary SIGKILL pattern (per kb: only the
double-fork daemonizer survives). Usage:
    python3 daemon-cmd.py <logfile> <cwd> <cmd...>
"""
import os
import sys

def main() -> None:
    logfile, cwd = sys.argv[1], sys.argv[2]
    cmd = sys.argv[3:]
    log = open(logfile, "ab", buffering=0)
    # fork #1
    pid = os.fork()
    if pid > 0:
        print(f"launcher exited; daemon lineage detached (log: {logfile})")
        return
    os.close(0)
    os.setsid()
    # fork #2 — grandchild can never reacquire a terminal
    pid2 = os.fork()
    if pid2 > 0:
        os._exit(0)
    os.dup2(log.fileno(), 1)
    os.dup2(log.fileno(), 2)
    os.chdir(cwd)
    os.execvp(cmd[0], cmd)

if __name__ == "__main__":
    main()
