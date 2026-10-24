#!/bin/bash
# Sync the git worktree (pc-edit) into the build workspace (pc-build).
# Content-checksummed (-c): unchanged files keep pc-build mtimes so the inherited
# cargo fingerprint cache stays warm. One-way flow: git -> build (never back).
set -e
rsync -rc --delete \
  --exclude '.git' --exclude 'target' --exclude 'dist' --exclude '.agents' \
  --exclude 'agent' --exclude 'wip' --exclude 'log' --exclude 'node_modules' \
  /home/z/pc-edit/ /home/z/pc-build/
echo "synced pc-edit -> pc-build ($(date +%H:%M:%S))"
