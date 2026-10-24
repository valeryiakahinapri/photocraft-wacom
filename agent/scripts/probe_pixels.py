#!/usr/bin/env python3
"""Probe specific pixels of an e2e screenshot against expected REV6 colors."""
from PIL import Image
import sys

img = Image.open(sys.argv[1] if len(sys.argv) > 1 else "/home/z/e2e-r6/04-moderncanvas-boot.png").convert("RGB")
W, H = img.size
print(f"size: {W}x{H}")

PAGE = (0x0B, 0x0C, 0x0E)     # page background
CARD = (0x16, 0x17, 0x19)     # island surface
WORK = (0x06, 0x06, 0x07)     # workspace

probes = [
    ("top-left corner (page band)", (4, 4), PAGE),
    ("top edge mid (page band)", (720, 3), PAGE),
    ("title row area (page)", (720, 22), PAGE),
    ("left edge beside toolbar (page)", (3, 430), PAGE),
    ("toolbar island interior (card)", (30, 430), CARD),
    ("right edge beside dock (page)", (W - 4, 430), PAGE),
    ("right dock interior (card)", (W - 60, 430), CARD),
    ("canvas workspace area", (720, 400), WORK),
    ("bottom edge (page band)", (720, H - 4), PAGE),
    ("gap between title and options (~y49)", (720, 49), PAGE),
]

for name, (x, y), want in probes:
    if not (0 <= x < W and 0 <= y < H):
        print(f"{name:38s} OUT OF RANGE")
        continue
    got = img.getpixel((x, y))
    match = "OK " if all(abs(a - b) <= 4 for a, b in zip(got, want)) else "   "
    print(f"{match}{name:38s} ({x:4d},{y:4d})  got={got}  want≈{want}")
