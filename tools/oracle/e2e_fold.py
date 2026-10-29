#!/usr/bin/env python3
"""e2e_fold.py — fold ⇧Tab round-trip regression on the live app (profile-independent gate).

Boot the app, sample the dock-side pixel mass, fold (Shift+Tab), verify the strip
empties, expand (Shift+Tab), verify it returns. Also screenshots the three states.
Exit 0 = PASS, 1 = FAIL.
"""
import subprocess, sys, time
from PIL import Image
import numpy as np

SESSION = "e2e-fold-s9"
HUB = "http://localhost:3000"
URL = f"{HUB}/photocraft/?webgl"
OUT = "/home/z/my-project/download/oracle"

def ab(*args):
    r = subprocess.run(["agent-browser", "--session", SESSION, *args], capture_output=True, text=True, timeout=120)
    if r.returncode != 0:
        raise RuntimeError(f"agent-browser {args}: {r.stderr or r.stdout}")
    return r.stdout

def shot(path):
    ab("screenshot", path)
    return np.asarray(Image.open(path).convert("RGB"))

def surface_mass(img, x0, x1, y0=120, y1=820):
    """count pixels near the panel surface tone (#161719) in the right strip"""
    band = img[y0:y1, x0:x1].reshape(-1, 3).astype(int)
    d = np.abs(band - np.array([22, 23, 25])).max(axis=1)
    return int((d <= 6).sum())

def main():
    ab("open", URL)
    ab("set", "viewport", "1440", "900")
    # boot: #photocraft_loading removed + settle
    for _ in range(60):
        gone = "true" in ab("eval", "Boolean(!document.querySelector('#photocraft_loading'))")
        if gone:
            break
        time.sleep(0.5)
    else:
        print("BOOT TIMEOUT"); return 1
    time.sleep(4)  # settle frames
    expanded = shot(f"{OUT}/s9-fold-1-expanded.png")
    ab("press", "Shift+Tab")
    time.sleep(2.5)
    folded = shot(f"{OUT}/s9-fold-2-folded.png")
    ab("press", "Shift+Tab")
    time.sleep(2.5)
    expanded2 = shot(f"{OUT}/s9-fold-3-roundtrip.png")

    m1 = surface_mass(expanded, 1052, 1420)
    m2 = surface_mass(folded, 1052, 1420)
    m3 = surface_mass(expanded2, 1052, 1420)
    print(f"surface mass: expanded={m1} folded={m2} roundtrip={m3}")
    ok = m2 < m1 * 0.15 and abs(m3 - m1) < m1 * 0.20
    print("FOLD ROUNDTRIP:", "PASS" if ok else "FAIL")
    # console errors
    errs = ab("eval", "(window.__errs||[]).length")
    print("window errors:", errs.strip())
    return 0 if ok else 1

if __name__ == "__main__":
    sys.exit(main())
