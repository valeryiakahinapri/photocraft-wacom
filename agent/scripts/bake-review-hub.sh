#!/bin/bash
# bake-review-hub.sh — regenerate the photocraft review hub in the Z container's
# /home/z/my-project (the platform Next.js app on :3000) after a recycle.
# Sources mirror: agent/review-hub/ on the agent-meta branch (this repo).
# Prereqs: (1) pc-build warm + trunk bundle (scripts/trunk-bundle.py, then
# dist/web exists), (2) the annotakit clone at /home/z/storybook-annotakit,
# (3) fullstack env initialized (init-fullstack.sh; dev server runs on :3000).
set -euo pipefail

HUB=/home/z/my-project
KIT=/home/z/storybook-annotakit
DIST=/home/z/pc-build/dist/web

echo "== 1. hub source files (from this repo's agent/review-hub) =="
HERE="$(cd "$(dirname "$0")/.." && pwd)"   # agent/
mkdir -p "$HUB/src/stories" "$HUB/.storybook"
cp "$HERE/review-hub/page.tsx"            "$HUB/src/app/page.tsx"
cp "$HERE/review-hub/next.config.ts"      "$HUB/next.config.ts"
cp "$HERE/review-hub/review.stories.tsx"  "$HUB/src/stories/review.stories.tsx"
cp "$HERE/review-hub/storybook.main.ts"   "$HUB/.storybook/main.ts"

echo "== 2. photocraft static bake (trunk dist -> public/photocraft) =="
rm -rf "$HUB/public/photocraft" && mkdir -p "$HUB/public/photocraft"
cp "$DIST"/* "$HUB/public/photocraft/"
# The trailing-slash trap: the page serves at /photocraft (no slash), so relative
# asset refs resolve against /. A <base href> fixes them (same trick annotakit's
# bake uses). WITHOUT it the wasm 404s from / and the app hangs on "Loading".
python3 - <<'EOF'
p = "/home/z/my-project/public/photocraft/index.html"
s = open(p).read()
if '<base href' not in s:
    s = s.replace("<head>", '<head>\n  <base href="/photocraft/">', 1)
    open(p, "w").write(s)
    print("  base href injected")
EOF

echo "== 3. annotakit + storybook deps (idempotent; ~100MB once) =="
cd "$HUB"
bun pm ls 2>/dev/null | grep -q "storybook-annotakit" || {
  (cd "$KIT" && bun pm pack >/dev/null)
  Tgz=$(ls "$KIT"/storybook-annotakit-*.tgz | sort -V | tail -1)
  bun add -d "$Tgz" storybook@10.6 @storybook/react-vite@10.6 vite@7
}

echo "== 4. storybook static bake (vite needs the hub's postcss NEUTRALIZED — =="
echo "    Tailwind 4's plugin is not a valid vite chain; see .storybook/main.ts)"
bunx storybook build -o public/storybook --config-dir .storybook
node node_modules/storybook-annotakit/scripts/bake-static-threads.mjs \
  public/storybook --base /storybook/ --config-dir .storybook

echo "== 5. eslint ignores for public/ (lint OOMs on the built bundles) =="
grep -q 'public/\*\*' eslint.config.mjs || \
  sed -i 's|"skills"\]|"skills", "public/**", ".storybook/**"]|' eslint.config.mjs

echo "== 6. restart the dev server (next.config.ts rewrites read at boot) =="
pkill -f "next dev" 2>/dev/null || true; pkill -f "next-server" 2>/dev/null || true; sleep 2
python3 -c "
import os
def daemonize():
    if os.fork(): os._exit(0)
    os.setsid()
    if os.fork(): os._exit(0)
    devnull = os.open('/dev/null', os.O_RDWR)
    for fd in (0,1,2): os.dup2(devnull, fd)
daemonize()
os.chdir('$HUB')
os.execvp('bun', ['bun', 'run', 'dev'])
"
sleep 25
for p in "/" "/photocraft" "/storybook" "/storybook/annotakit-threads.json"; do
  echo -n "$p -> "; curl -s -o /dev/null -w "%{http_code}\n" "http://localhost:3000$p"
done

echo "DONE. Reviewer surface: <preview>/ (hub), /photocraft (app), /storybook (pins)."
echo "Notes: navigate to the STORY route (/storybook?path=/story/review-photocraft--app) —"
echo "SB10 renders no addon toolbar on the root route. The GitHub mirror needs a scoped"
echo "fine-grained PAT in .env + a re-bake (step 4); never bake the full work PAT."
