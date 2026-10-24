#!/bin/bash
# bake-photocraft.sh — copy the fresh trunk dist/web bundle into the Next.js static bakes.
# The /photocraft route serves WITHOUT a trailing slash, so relative wasm refs would resolve
# against / — the <base href="/photocraft/"> injection (session-2 lesson) fixes loading.
# The storybook story parses the hashed filename from the served page at runtime, so no
# storybook rebuild is needed when only the bundle changes.
set -e
D=/home/z/pc-build/dist/web
P=/home/z/my-project/public/photocraft
S=/home/z/my-project/public/storybook/photocraft

for T in "$P" "$S"; do
  mkdir -p "$T"
  rm -f "$T"/photocraft-web-*.js "$T"/photocraft-web-*_bg.wasm
  cp "$D"/index.html "$T"/index.html
  cp "$D"/photocraft-web-*.js "$T"/
  cp "$D"/photocraft-web-*_bg.wasm "$T"/
  grep -q '<base href' "$T"/index.html || sed -i 's|<head>|<head>\n  <base href="/photocraft/">|' "$T"/index.html
  grep -q '<base href' "$T"/index.html || { echo "BASE INJECTION FAILED: $T"; exit 1; }
done
echo "BAKE OK: $(ls "$P"/photocraft-web-*_bg.wasm | xargs -n1 basename)"
ls -la "$P" | grep -E 'wasm|js$' | awk '{print $5, $9}'
