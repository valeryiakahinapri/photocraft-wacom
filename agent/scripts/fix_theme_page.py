#!/usr/bin/env python3
"""Fix truncated `page:` lines: re-derive from the nearest preceding `canvas:` line
(its full value up to the line's trailing comma)."""
import re

PATH = "/home/z/pc-edit/crates/ui-egui/src/theme.rs"
src = open(PATH).read().splitlines(keepends=True)

canvas_val = None
fixed = 0
out = []
for line in src:
    m = re.match(r"^(\s*)canvas:\s*(.+?),\s*$", line)
    if m:
        canvas_val = m.group(2)
    pm = re.match(r"^(\s*)page:\s*(.+?),\s*$", line)
    if pm and not pm.group(2).endswith(")"):
        # truncated — replace with the block's canvas value
        out.append(f"{pm.group(1)}page: {canvas_val},\n")
        fixed += 1
        continue
    out.append(line)

open(PATH, "w").write("".join(out))
print(f"fixed {fixed} truncated page lines")
