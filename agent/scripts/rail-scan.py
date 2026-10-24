#!/usr/bin/env python3
"""rail-scan.py — find the rail band and icon glyph rows in a screenshot crop.

Scans the right edge band for dark (icon-glyph) pixel rows and clusters them;
prints cluster centers so E2E clicks can target them.
"""
import sys
from PIL import Image

path = sys.argv[1]
img = Image.open(path).convert("RGB")
w, h = img.size
x0 = int(sys.argv[2]) if len(sys.argv) > 2 else max(0, w - 110)
x1 = int(sys.argv[3]) if len(sys.argv) > 3 else w

# 1) Find the rail's vertical separator/fill band: columns whose variance across
#    the band differs from the canvas. Simplest: collect per-column mean brightness
#    in the band; the rail island + dock show as a distinct shade change.
px = img.load()

# 2) Within the rightmost ~100px, find dark-glyph rows (icons) by counting pixels
#    significantly darker than the local background per row.
clusters = []
cur = None
for y in range(40, h - 30):
    row = [px[x, y] for x in range(x0, x1)]
    # median per channel as the row's fill color
    rs = sorted(p[0] for p in row); gs = sorted(p[1] for p in row); bs = sorted(p[2] for p in row)
    med = (rs[len(rs) // 2], gs[len(gs) // 2], bs[len(bs) // 2])
    dev = sum(
        1
        for p in row
        if abs(p[0] - med[0]) + abs(p[1] - med[1]) + abs(p[2] - med[2]) > 90
    )
    if dev >= 2:
        if cur is None:
            cur = [y, y, 0]
        cur[1] = y
        cur[2] += dev
    else:
        if cur is not None and cur[1] - cur[0] >= 4:
            clusters.append(cur)
        cur = None
if cur is not None and cur[1] - cur[0] >= 4:
    clusters.append(cur)

print(f"image {w}x{h}; scanning x in [{x0},{x1})")
for c in clusters[:20]:
    center = (c[0] + c[1]) // 2
    print(f"  glyph rows y {c[0]}..{c[1]} (center ~{center}, {c[2]} dark px)")
