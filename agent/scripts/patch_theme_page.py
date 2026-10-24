#!/usr/bin/env python3
"""Insert `page` (mirroring the block's canvas fill) + `islands: false` after each
Tokens block's `scrim:` line in theme.rs, skipping blocks that already have them."""
import re

PATH = "/home/z/pc-edit/crates/ui-egui/src/theme.rs"
src = open(PATH).read().splitlines(keepends=True)

out = []
canvas_val = None
scrim_idx_pending = None
patched = 0
for i, line in enumerate(src):
    m = re.match(r"^(\s*)canvas:\s*(Color32::[^,]+),", line)
    if m:
        canvas_val = m.group(2)
    if re.match(r"^\s*scrim:", line):
        # look ahead within the next 3 lines for an existing `page:` (already patched)
        window = "".join(src[i + 1 : i + 4])
        if re.search(r"^\s*page:", window, re.M):
            out.append(line)
            continue
        indent = re.match(r"^(\s*)", line).group(1)
        out.append(line)
        out.append(f"{indent}page: {canvas_val or 'Color32::TRANSPARENT'},\n")
        out.append(f"{indent}islands: false,\n")
        patched += 1
        continue
    out.append(line)

open(PATH, "w").write("".join(out))
print(f"patched {patched} Tokens blocks")
