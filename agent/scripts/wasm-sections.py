#!/usr/bin/env python3
"""wasm-sections.py — list section sizes of a .wasm file (top-level sections only)."""
import struct
import sys

PATH = sys.argv[1]
NAMES = {
    0: "custom", 1: "type", 2: "import", 3: "function", 4: "table", 5: "memory",
    6: "global", 7: "export", 8: "start", 9: "elem", 10: "code", 11: "data", 12: "datacount",
}


def read_leb(buf, off):
    result = 0
    shift = 0
    while True:
        b = buf[off]
        off += 1
        result |= (b & 0x7F) << shift
        if not (b & 0x80):
            return result, off
        shift += 7


with open(PATH, "rb") as fh:
    data = fh.read()
assert data[:4] == b"\x00asm", "not a wasm file"
off = 8
total = len(data)
rows = []
while off < total:
    sec_id = data[off]
    off += 1
    size, off = read_leb(data, off)
    name = NAMES.get(sec_id, f"?{sec_id}")
    if sec_id == 0:
        nlen, p = read_leb(data, off)
        cname = data[p : p + nlen].decode("utf-8", "replace")
        name = f'custom "{cname}"'
    rows.append((size, name))
    off += size
rows.sort(reverse=True)
print(f"file: {PATH}  total: {total:,} bytes")
for size, name in rows[:12]:
    print(f"  {size:>12,}  {name}")
